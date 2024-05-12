use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn respond_with_header(header: &str, mut stream: &TcpStream) {
    let response = format!("{header}\r\n\r\n");
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
    let request_line = response[0].clone();

    if request_line == "GET / HTTP/1.1" {
        return respond_with_header("HTTP/1.1 200 OK", stream);
    }

    respond_with_header("HTTP/1.1 404 Not Found", stream)
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

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
