use std::{
    fs,
    io::{self, Write},
    net::TcpStream,
    path::Path,
};
use walkdir::WalkDir;
use infer;

pub fn serve_file(stream: &mut TcpStream, path: &Path) -> io::Result<()> {
    if !path.exists() {
        return send_404(stream);
    }

    let file_content = fs::read(path)?;

    let mime_type = infer::get_from_path(path)
        .ok()
        .flatten()
        .map(|kind| kind.mime_type())
        .unwrap_or_else(|| match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("mp4") => "video/mp4",
            Some("gif") => "image/gif",
            Some("pdf") => "application/pdf",
            Some("txt") => "text/plain",
            _ => "text/plain",
        });

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
        file_content.len(),
        mime_type
    );

    stream.write(response.as_bytes())?;
    stream.write(&file_content)?;
    stream.flush()
}

pub fn list_directory(stream: &mut TcpStream, path: &Path, request_line: &str) -> io::Result<()> {
    let current_dir_name = path.file_name().unwrap_or_default().to_string_lossy();
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let base_url = parts[1];
    let mut begin_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="utf-8">
        <style>
            .highlight { color: red; }
        </style>
    </head>
    <body>"#.to_string();

    begin_html.push_str(&format!("<h1>Directory listing for {}</h1>", path.display()));

    if base_url.contains("/") {
        let parent_dir;
        if base_url == "/" {
            begin_html.push_str(r#"<a href="/">Parent directory</a><br>"#);
        } else if let Some(pos) = base_url.rfind('/') {
            parent_dir = &base_url[..pos];
            begin_html.push_str(&format!(
                r#"<a href="/{}">Parent directory</a><br>"#,
                parent_dir.trim_start_matches('/')
            ));
        }
    }

    for entry in WalkDir::new(path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let file_name = entry.file_name().to_string_lossy();
        let file_url = format!("{}/{}", base_url.trim_end_matches('/'), file_name);

        // Check if the file/folder name matches the current directory name
        if file_name != current_dir_name {
            begin_html.push_str(&format!(
                r#"<a href="{}">{}</a><br>"#,
                file_url, file_name
            ));
        }
    }

    let end_html = r#"
    </body>
    </html>
    "#;

    let response_body = begin_html + &end_html;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    stream.write(response.as_bytes())?;
    stream.flush()
}

pub fn send_404(stream: &mut TcpStream) -> io::Result<()> {
    let body = "<html><body><h1>404 - Not Found</h1></body></html>";
    let response = format!(
        "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write(response.as_bytes())?;
    stream.flush()
}
