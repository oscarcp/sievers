use std::process::Command;

fn main() {
    // Build date (UTC, date only)
    let date = chrono_free_date();
    println!("cargo:rustc-env=BUILD_DATE={date}");

    // Git commit hash (short)
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_COMMIT={commit}");
}

/// Get current UTC date without pulling in the chrono crate.
fn chrono_free_date() -> String {
    // Try the `date` command (available on Linux/macOS)
    Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}
