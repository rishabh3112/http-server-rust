use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use itertools::Itertools;

pub struct Request {
    pub ty: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    pub fn new(mut stream: &TcpStream) -> Request {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut status_line: String = String::new();
        buf_reader.read_line(&mut status_line).unwrap();

        let (ty, path, _protocol) = status_line
            .split(" ")
            .map(|fragments| fragments.to_string())
            .collect_tuple()
            .unwrap();

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

            headers.insert(header_key.to_lowercase(), header_value);
        }

        let mut body: Option<String> = None;
        if headers.contains_key("content-length") {
            let content_length = headers
                .get("content-length")
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
            body,
        }
    }
}
