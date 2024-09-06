use std::{
    env,
    io::{self, Read},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread,
};
mod http;

use http::request::HttpRequest;
use http::response::{serve_file, list_directory, send_404};

fn handle_client(mut stream: TcpStream, root: PathBuf) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request = HttpRequest::from_buffer(&buffer, root.clone());

    match request {
        Some(req) => {
            if req.path.exists() {
                if req.path.is_dir() {
                    list_directory(&mut stream, &req.path, &req.request_line)
                } else {
                    serve_file(&mut stream, &req.path)
                }
            } else {
                send_404(&mut stream)
            }
        }
        None => send_404(&mut stream),
    }
}

fn serve(socket_addr: &str, root: PathBuf) -> io::Result<()> {
    let listener = TcpListener::bind(socket_addr)?;
    println!("Serving files from {} on {}", root.display(), socket_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let root = root.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, root) {
                        eprintln!("Error handling client: {:?}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {:?}", e),
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let root = env::current_dir()?;
    let socket_addr = "localhost:5500";

    serve(socket_addr, root)?;
    Ok(())
}
