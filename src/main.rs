use std::env;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    println!("In directory {file_path}");
    let mut index = search_engine::Index::new(file_path).expect("should be able to build index");

    let listener = TcpListener::bind("0.0.0.0:7878")?;
    println!("server listening on port 7878...");
    for stream in listener.incoming() {
        handle_client(stream?, &mut index);
    }

    Ok(())
}
fn handle_client(mut stream: TcpStream, index: &mut search_engine::Index) {
    println!("connected to a client");
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("{:#?}", http_request);
    let request_line = http_request
        .first()
        .expect("should have been able to get first line of http request");
    if request_line.starts_with("OPTIONS") {
        let response = format!(
            "HTTP/1.1 204 No Content\r\n{}Cache-Control: max-age=86400\r\n\r\n",
            create_cors_header()
        );
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }
    let query_start = request_line
        .find("?query=")
        .expect("should have been able to find query")
        + 7;
    let query = &request_line[query_start..];
    let end_pos = query
        .find(' ')
        .expect("should have been able to find end of query");
    let query = query[..end_pos].to_string().replace("+", " ");
    println!("query is {}", query);
    let results = index.search(&query);

    let status_line = "HTTP/1.1 200 OK";
    let contents = serde_json::to_string(&results)
        .expect("should have been able to convert search results to json");
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
fn create_cors_header() -> String {
    "Access-Control-Allow-Origin: *\r\n\
     Access-Control-Allow-Methods: GET, OPTIONS\r\n\
     Access-Control-Allow-Headers: Content-Type\r\n"
        .to_string()
}
