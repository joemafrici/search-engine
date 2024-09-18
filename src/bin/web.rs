use axum::extract::{Query, State};
use axum::{http::Method, http::StatusCode, routing::get, Router};
use search_engine::index::Index;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    println!("In directory {file_path}");
    let index = Arc::new(Mutex::new(
        search_engine::index::Index::new(file_path).expect("should be able to build index"),
    ));
    let _results = {
        let index = index.lock().unwrap();
        println!("found {} unique tokens", index.tokens.len());
        let mut num_tokens: usize = 0;
        for doc in &index.documents {
            num_tokens += doc.total_tokens_in_file;
        }
        println!("found {} total tokens", num_tokens);
    };

    let app = Router::new()
        .route("/search", get(handle_client))
        .with_state(index)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        );
    let addr = SocketAddr::from(([0, 0, 0, 0], 7878));
    println!("Server listening on {}...", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
async fn handle_client(
    Query(params): Query<HashMap<String, String>>,
    State(index): State<Arc<Mutex<Index>>>,
) -> Result<String, StatusCode> {
    println!("connected to a client");
    // parse request
    let search_query = params.get("query").unwrap();
    println!("got search query {}", search_query);
    // send response
    let results = {
        let mut index = index.lock().unwrap();
        println!("performing search...");
        let mut thing = index.search(search_query);
        thing.reverse();
        thing
    };

    println!("search complete...");
    let contents = serde_json::to_string(&results)
        .expect("should have been able to convert search results to json");

    Ok(contents)
    //Ok("hello there".to_string())
}
