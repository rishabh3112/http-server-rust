use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn respond_stream(response: &str, mut stream: &TcpStream) {
    stream.write_all(response.as_bytes()).unwrap()
}

fn read_request(mut stream: &TcpStream) -> Vec<String> {
    let buf_reader = BufReader::new(&mut stream);
    let request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    request
}

fn process_request(stream: &TcpStream) {
    let response = read_request(stream);
    let status_line = response[0].clone();

    let status_parts: Vec<&str> = status_line.split(" ").collect();
    let path_fragments: Vec<&str> = status_parts[1].split("/").collect();

    if status_parts[0] == "GET" && path_fragments[1] == "echo" && path_fragments.len() >= 3 {
        let string = path_fragments[2];
        let length = string.len();
        return respond_stream(
            format!("HTTP/1.1 200 OK\r\nContent-type: text/plain\r\nContent-length: {length}\r\n\r\n{string}").as_str(),
            stream
        );
    } else if status_parts[0] == "GET" && status_parts[1] == "/" {
        return respond_stream("HTTP/1.1 200 OK\r\n\r\n", stream);
    }

    respond_stream("HTTP/1.1 404 Not Found\r\n\r\n", stream)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                process_request(&_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
