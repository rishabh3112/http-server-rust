mod request;
mod thread_pool;

use flate2::{write::GzEncoder, Compression};
use request::Request;
use thread_pool::ThreadPool;

use std::{
    env,
    fs::{read_to_string, write},
    io::Write,
    net::{TcpListener, TcpStream},
    path::Path,
};

fn respond_stream(response: &str, mut stream: &TcpStream) {
    stream.write_all(response.as_bytes()).unwrap()
}

fn respond_stream_with_buffer(buffer: &Vec<u8>, mut stream: &TcpStream) {
    stream.write_all(buffer).unwrap()
}

fn handle_connection(stream: &TcpStream) {
    let request = Request::new(&stream);
    let path_fragments: Vec<&str> = request.path.split("/").collect();

    let mut encoding_header: String = String::new();
    let mut gzip_requested = false;
    if let Some(encodings_str) = request.headers.get("accept-encoding") {
        let encodings: Vec<String> = encodings_str
            .split(", ")
            .map(|encoding| encoding.to_string())
            .collect();

        gzip_requested = encodings.contains(&String::from("gzip"));

        if gzip_requested {
            encoding_header = format!("Content-Encoding: gzip\r\n")
        }
    }

    if request.ty == "GET" && path_fragments[1] == "echo" && path_fragments.len() >= 3 {
        let string = path_fragments[2].to_string();
        let mut buffer: Option<Vec<u8>> = None;

        if gzip_requested {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

            match encoder.write_all(&path_fragments[2].as_bytes()) {
                Ok(_) => {
                    buffer = Some(encoder.finish().unwrap());
                }
                Err(_) => {}
            }
        }

        if buffer.is_none() {
            let length = string.len();
            return respond_stream(
                format!("HTTP/1.1 200 OK\r\n{encoding_header}Content-type: text/plain\r\nContent-length: {length}\r\n\r\n{string}").as_str(),
                stream
            )
        } else {
            let body = buffer.unwrap();
            let length = body.len();

            let mut response = format!("HTTP/1.1 200 OK\r\n{encoding_header}Content-type: text/plain\r\nContent-length: {length}\r\n\r\n").as_bytes().to_vec();
            response.extend(body);
    
            return respond_stream_with_buffer(&response, stream)
            
        };
    } else if request.ty == "GET" && request.path == "/user-agent" {
        let string = request.headers.get("user-agent").unwrap();
        let length = string.len();
        return respond_stream(
            format!("HTTP/1.1 200 OK\r\nContent-type: text/plain\r\nContent-length: {length}\r\n\r\n{string}").as_str(),
            stream
        );
    } else if request.ty == "GET" && request.path == "/" {
        return respond_stream("HTTP/1.1 200 OK\r\n\r\n\r\n", stream);
    } else if request.ty == "GET" && path_fragments[1] == "files" && path_fragments.len() >= 3 {
        let filename = path_fragments[2];
        let args: Vec<String> = env::args().collect();

        if args.len() >= 3 && args[1] == "--directory" {
            let file_path = Path::new(args[2].trim()).join(filename);
            if let Ok(file_content) = read_to_string(&file_path) {
                let length = file_content.len();

                return respond_stream(
                    format!("HTTP/1.1 200 OK\r\nContent-type: application/octet-stream\r\nContent-length: {length}\r\n\r\n{file_content}").as_str(),
                    stream
                );
            }
        }
    } else if request.ty == "POST" && path_fragments[1] == "files" && path_fragments.len() >= 3 {
        let filename = path_fragments[2];
        let file_content = request.body;
        let args: Vec<String> = env::args().collect();

        if args.len() >= 3 && args[1] == "--directory" {
            let file_path = Path::new(args[2].trim()).join(filename);

            if let Ok(_) = write(&file_path, file_content.unwrap()) {
                return respond_stream(
                    format!("HTTP/1.1 201 Created\r\n\r\n\r\n").as_str(),
                    stream,
                );
            }
        }
    }

    respond_stream(
        "HTTP/1.1 404 Not Found\r\nContent-length: 0\r\n\r\n",
        stream,
    )
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let thread_pool = ThreadPool::new(10);

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => thread_pool.execute(move || handle_connection(&stream)),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
