/// Async ManageSieve client (RFC 5804).
///
/// Supports STARTTLS, SASL PLAIN authentication, and all standard commands:
/// LISTSCRIPTS, GETSCRIPT, PUTSCRIPT, SETACTIVE, DELETESCRIPT, CHECKSCRIPT, LOGOUT.
use base64::Engine;
use rustls::ClientConfig;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use crate::model::profile::ConnectionProfile;

#[derive(Debug, Clone)]
pub struct ScriptInfo {
    pub name: String,
    pub active: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Not connected")]
    NotConnected,
}

// We use a dynamic stream type to handle both plain and TLS connections
enum Stream {
    Plain(BufReader<TcpStream>),
    Tls(Box<BufReader<tokio_rustls::client::TlsStream<TcpStream>>>),
}

impl Stream {
    async fn read_line(&mut self, buf: &mut String) -> Result<usize, std::io::Error> {
        match self {
            Self::Plain(r) => r.read_line(buf).await,
            Self::Tls(r) => r.read_line(buf).await,
        }
    }

    async fn write_all(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        match self {
            Self::Plain(r) => r.get_mut().write_all(data).await,
            Self::Tls(r) => r.get_mut().write_all(data).await,
        }
    }

    async fn flush(&mut self) -> Result<(), std::io::Error> {
        match self {
            Self::Plain(r) => r.get_mut().flush().await,
            Self::Tls(r) => r.get_mut().flush().await,
        }
    }
}

pub struct ManageSieveClient {
    stream: Option<Stream>,
}

impl ManageSieveClient {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Connect to a ManageSieve server, optionally upgrading to TLS via STARTTLS,
    /// then authenticate using SASL PLAIN.
    pub async fn connect(
        &mut self,
        profile: &ConnectionProfile,
        password: &str,
    ) -> Result<(), Error> {
        let tcp = TcpStream::connect((&*profile.host, profile.port)).await?;
        let mut stream = Stream::Plain(BufReader::new(tcp));

        // Read server greeting/capabilities
        read_response(&mut stream).await?;

        // STARTTLS if requested
        if profile.use_starttls {
            send_command(&mut stream, "STARTTLS").await?;
            let resp = read_response(&mut stream).await?;
            if !resp.ok {
                return Err(Error::Server("STARTTLS rejected".to_string()));
            }

            // Upgrade to TLS
            let mut tls_config = ClientConfig::builder()
                .with_root_certificates(root_store())
                .with_no_client_auth();
            tls_config.alpn_protocols = vec![];

            let connector = TlsConnector::from(Arc::new(tls_config));
            let server_name = rustls::pki_types::ServerName::try_from(profile.host.clone())
                .map_err(|e| Error::Protocol(format!("Invalid server name: {e}")))?;

            // Extract the TcpStream from the BufReader
            let tcp = match stream {
                Stream::Plain(r) => r.into_inner(),
                _ => unreachable!(),
            };

            let tls_stream = connector.connect(server_name, tcp).await?;
            stream = Stream::Tls(Box::new(BufReader::new(tls_stream)));

            // Re-read capabilities after TLS
            read_response(&mut stream).await?;
        }

        // Authenticate with SASL PLAIN
        // SASL PLAIN: \0username\0password
        let auth_data = format!("\0{}\0{}", profile.username, password);
        let b64 = base64::engine::general_purpose::STANDARD.encode(auth_data.as_bytes());
        let auth_cmd = format!("AUTHENTICATE \"PLAIN\" \"{}\"", b64);

        send_command(&mut stream, &auth_cmd).await?;
        let resp = read_response(&mut stream).await?;
        if !resp.ok {
            return Err(Error::AuthFailed);
        }

        self.stream = Some(stream);
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(stream) = &mut self.stream {
            let _ = send_command(stream, "LOGOUT").await;
            let _ = read_response(stream).await;
        }
        self.stream = None;
    }

    pub async fn list_scripts(&mut self) -> Result<Vec<ScriptInfo>, Error> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        send_command(stream, "LISTSCRIPTS").await?;

        let mut scripts = Vec::new();
        loop {
            let mut line = String::new();
            stream.read_line(&mut line).await?;
            let trimmed = line.trim();

            if trimmed.starts_with("OK") {
                break;
            }
            if trimmed.starts_with("NO") || trimmed.starts_with("BYE") {
                return Err(Error::Server(trimmed.to_string()));
            }

            // Parse script line: "scriptname" [ACTIVE]
            if let Some(name) = extract_quoted_string(trimmed) {
                let active = trimmed.contains("ACTIVE");
                scripts.push(ScriptInfo { name, active });
            }
        }

        Ok(scripts)
    }

    pub async fn get_script(&mut self, name: &str) -> Result<String, Error> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        let cmd = format!("GETSCRIPT \"{}\"", escape_sieve(name));
        send_command(stream, &cmd).await?;

        let mut content = String::new();
        let mut in_literal = false;
        let mut remaining = 0usize;

        loop {
            let mut line = String::new();
            stream.read_line(&mut line).await?;

            if in_literal {
                if remaining > 0 {
                    let take = line.len().min(remaining);
                    content.push_str(&line[..take]);
                    remaining -= take;
                }
                if remaining == 0 {
                    in_literal = false;
                }
                continue;
            }

            let trimmed = line.trim();
            if trimmed.starts_with("OK") {
                break;
            }
            if trimmed.starts_with("NO") || trimmed.starts_with("BYE") {
                return Err(Error::Server(trimmed.to_string()));
            }

            // Check for literal: {size+}
            if let Some(size) = extract_literal_size(trimmed) {
                remaining = size;
                in_literal = true;
            } else if let Some(s) = extract_quoted_string(trimmed) {
                content.push_str(&s);
            } else {
                content.push_str(&line);
            }
        }

        // Remove trailing \r\n from content
        while content.ends_with('\n') || content.ends_with('\r') {
            content.pop();
        }

        Ok(content)
    }

    pub async fn put_script(&mut self, name: &str, content: &str) -> Result<(), Error> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        let size = content.len();
        let cmd = format!(
            "PUTSCRIPT \"{}\" {{{size}+}}\r\n{content}",
            escape_sieve(name)
        );
        send_command(stream, &cmd).await?;
        let resp = read_response(stream).await?;
        if !resp.ok {
            return Err(Error::Server(resp.message));
        }
        Ok(())
    }

    pub async fn set_active(&mut self, name: &str) -> Result<(), Error> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        let cmd = format!("SETACTIVE \"{}\"", escape_sieve(name));
        send_command(stream, &cmd).await?;
        let resp = read_response(stream).await?;
        if !resp.ok {
            return Err(Error::Server(resp.message));
        }
        Ok(())
    }

    pub async fn delete_script(&mut self, name: &str) -> Result<(), Error> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        let cmd = format!("DELETESCRIPT \"{}\"", escape_sieve(name));
        send_command(stream, &cmd).await?;
        let resp = read_response(stream).await?;
        if !resp.ok {
            return Err(Error::Server(resp.message));
        }
        Ok(())
    }

    pub async fn check_script(&mut self, content: &str) -> Result<bool, Error> {
        let stream = self.stream.as_mut().ok_or(Error::NotConnected)?;
        let size = content.len();
        let cmd = format!("CHECKSCRIPT {{{size}+}}\r\n{content}");
        send_command(stream, &cmd).await?;
        let resp = read_response(stream).await?;
        Ok(resp.ok)
    }
}

// --- Protocol helpers ---

struct Response {
    ok: bool,
    message: String,
}

async fn send_command(stream: &mut Stream, cmd: &str) -> Result<(), Error> {
    let data = format!("{cmd}\r\n");
    stream.write_all(data.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

async fn read_response(stream: &mut Stream) -> Result<Response, Error> {
    // Read lines until we get OK, NO, or BYE
    let mut full_response = String::new();
    loop {
        let mut line = String::new();
        let n = stream.read_line(&mut line).await?;
        if n == 0 {
            return Err(Error::Protocol("Connection closed".to_string()));
        }

        let trimmed = line.trim();
        if trimmed.starts_with("OK") {
            return Ok(Response {
                ok: true,
                message: trimmed.to_string(),
            });
        }
        if trimmed.starts_with("NO") {
            return Ok(Response {
                ok: false,
                message: trimmed.to_string(),
            });
        }
        if trimmed.starts_with("BYE") {
            return Ok(Response {
                ok: false,
                message: trimmed.to_string(),
            });
        }

        full_response.push_str(&line);
    }
}

fn extract_quoted_string(s: &str) -> Option<String> {
    let s = s.trim().strip_prefix('"')?;
    let mut result = String::new();
    let mut chars = s.chars();
    loop {
        match chars.next() {
            Some('\\') => {
                if let Some(c) = chars.next() {
                    result.push(c);
                }
            }
            Some('"') => return Some(result),
            Some(c) => result.push(c),
            None => return None,
        }
    }
}

fn extract_literal_size(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.starts_with('{') {
        let end = s.find('}')?;
        let num_str = &s[1..end];
        let num_str = num_str.trim_end_matches('+');
        num_str.parse().ok()
    } else {
        None
    }
}

fn escape_sieve(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn root_store() -> rustls::RootCertStore {
    let mut store = rustls::RootCertStore::empty();
    store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    store
}
