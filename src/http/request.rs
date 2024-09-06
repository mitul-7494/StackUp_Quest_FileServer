use std::path::PathBuf;
use url_escape::decode;

pub struct HttpRequest {
    pub path: PathBuf,
    pub request_line: String,
}

impl HttpRequest {
    pub fn from_buffer(buffer: &[u8], root: PathBuf) -> Option<Self> {
        let request = String::from_utf8_lossy(buffer);
        let request_line = request.lines().next()?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 || parts[0] != "GET" {
            return None;
        }

        let decoded_path = decode(parts[1]);
        let requested_path = root.join(decoded_path.trim_start_matches('/'));
        Some(HttpRequest { path: requested_path, request_line: request_line.to_string() })
    }
}
