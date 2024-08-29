use std::env;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    println!("In directory {file_path}");
    let query: Vec<String> = args[2..].to_vec();
    let results = search_engine::build_idx(file_path, &query).expect("failed to build index");

    for result in &results {
        if result.similarity > 0.0 {
            println!("{} has similarity {}", result.filename, result.similarity);
            for snippet in &result.snippets {
                println!("      => {}", snippet);
            }
        }
    }

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("server listening on port 7878...");
    for stream in listener.incoming() {
        handle_client(stream?);
    }

    Ok(())
}
fn handle_client(mut stream: TcpStream) {
    println!("connected to a client");
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
}
