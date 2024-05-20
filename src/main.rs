mod request;
mod thread_pool;

use request::Request;
use thread_pool::ThreadPool;

use std::{
    env, fs::read_to_string, io::Write, net::{TcpListener, TcpStream}, path::Path
};

fn respond_stream(response: &str, mut stream: &TcpStream) {
    stream.write_all(response.as_bytes()).unwrap()
}

fn handle_connection(stream: &TcpStream) {
    let request = Request::new(&stream);
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
        return respond_stream("HTTP/1.1 200 OK\r\nContent-length: 0\r\n\r\n", stream);
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
                )
            }
        }
    }

    respond_stream("HTTP/1.1 404 Not Found\r\nContent-length: 0\r\n\r\n", stream)
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
