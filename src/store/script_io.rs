use std::path::Path;

pub fn load_script(path: &Path) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

pub fn save_script(path: &Path, text: &str) -> Result<(), std::io::Error> {
    std::fs::write(path, text)
}
