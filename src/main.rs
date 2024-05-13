use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use itertools::Itertools;

struct Request {
    ty: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Request {
    pub fn new(mut stream: &TcpStream) -> Request {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut status_line: String = String::new();
        buf_reader.read_line(&mut status_line).unwrap();

        let (ty, path, _protocol) = status_line.split(" ").map(|fragments| fragments.to_string()).collect_tuple().unwrap();

        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            let mut raw_header: String = String::new();
            buf_reader.read_line(&mut raw_header).unwrap();

            // trim out \r\n from end
            raw_header = raw_header.trim().to_string();

            if raw_header.is_empty() {
                break;
            }

            let (header_key, header_value) = raw_header
                .split(": ")
                .map(|fragment| fragment.trim().to_string())
                .collect_tuple()
                .unwrap();

            headers.insert(header_key, header_value);
        }

        let mut body: Option<String> = None;
        if headers.contains_key("Content-Length") {
            let content_length = headers
                .get("Content-Length")
                .unwrap()
                .parse::<usize>()
                .unwrap_or(0);

            if content_length > 0 {
                let mut buffer = vec![0; content_length];
                buf_reader.read_exact(&mut buffer).unwrap();

                body = Some(std::str::from_utf8(&buffer).unwrap().to_string());
            } else {
                body = Some("".to_string())
            }
        }

        Request {
            ty,
            path,
            headers,
            body
        }
    }
}

fn respond_stream(response: &str, mut stream: &TcpStream) {
    stream.write_all(response.as_bytes()).unwrap()
}

fn process_request(request: &Request, stream: &TcpStream) {
    let path_fragments: Vec<&str> = request.path.split("/").collect();

    if request.ty == "GET" && path_fragments[1] == "echo" && path_fragments.len() >= 3 {
        let string = path_fragments[2];
        let length = string.len();
        return respond_stream(
            format!("HTTP/1.1 200 OK\r\nContent-type: text/plain\r\nContent-length: {length}\r\n\r\n{string}").as_str(),
            stream
        );
    } else if request.ty == "GET" && request.path == "/user-agent" {
        let string = request.headers.get("User-Agent").unwrap();
        let length = string.len();
        return respond_stream(
            format!("HTTP/1.1 200 OK\r\nContent-type: text/plain\r\nContent-length: {length}\r\n\r\n{string}").as_str(),
            stream
        );
    } else if request.ty == "GET" && request.path == "/" {
        return respond_stream("HTTP/1.1 200 OK\r\n\r\n", stream);
    }

    respond_stream("HTTP/1.1 404 Not Found\r\n\r\n", stream)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                let request = Request::new(&_stream);
                process_request(&request, &_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
