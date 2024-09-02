use axum::{http::Method, http::StatusCode, routing::get, Router};
use std::env;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    println!("In directory {file_path}");
    let mut _index =
        Arc::new(search_engine::Index::new(file_path).expect("should be able to build index"));
    // /etc/letsencrypt/live/gojoe.dev/fullchain.pem
    // /etc/letsencrypt/live/gojoe.dev/privkey.pem

    let app = Router::new().route("/", get(handle_client)).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET]),
    );

    let addr = "0.0.0.0:7878";
    println!("server listening on {} ...", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
async fn handle_client() -> Result<String, StatusCode> {
    // parse request
    // handle cors
    // send response
    println!("connected to a client");
    //let results = index.search("plato nietzsche");

    //let contents = serde_json::to_string(&results)
    //    .expect("should have been able to convert search results to json");

    //Ok(response)
    Ok("hello there".to_string())
}
//fn create_cors_header() -> String {
//    "Access-Control-Allow-Origin: *\r\n\
//     Access-Control-Allow-Methods: GET, OPTIONS\r\n\
//     Access-Control-Allow-Headers: Content-Type\r\n"
//        .to_string()
//}
