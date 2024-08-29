use search_engine::SearchResult;
use std::env;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    println!("In directory {file_path}");
    let query: Vec<String> = args[2..].to_vec();
    let results = search_engine::build_idx(file_path, &query).expect("failed to build index");

    //for result in &results {
    //    if result.similarity > 0.0 {
    //        println!("{} has similarity {}", result.filename, result.similarity);
    //        for snippet in &result.snippets {
    //            println!("      => {}", snippet);
    //        }
    //    }
    //}

    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("server listening on port 7878...");
    for stream in listener.incoming() {
        handle_client(stream?, &results);
    }

    Ok(())
}
fn handle_client(mut stream: TcpStream, results: &Vec<SearchResult>) {
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

    let status_line = "HTTP/1.1 200 OK";
    let contents = serde_json::to_string(&results)
        .expect("should have been able to convert search results to json");
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
