use axum::extract::{Query, Request, State};
use axum::{http::Method, http::StatusCode, routing::get, Router};
use hyper::body::Incoming;
use hyper_util::rt::TokioExecutor;
use rustls_pemfile::{certs, pkcs8_private_keys};
use search_engine::Index;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_rustls::{rustls::ServerConfig, TlsAcceptor};
use tower_http::cors::{Any, CorsLayer};
use tower_service::Service;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    println!("In directory {file_path}");
    let index = Arc::new(Mutex::new(
        search_engine::Index::new(file_path).expect("should be able to build index"),
    ));
    let rustls_config = rustls_server_config(
        PathBuf::from("/etc/letsencrypt/live/gojoe.dev/privkey.pem"),
        PathBuf::from("/etc/letsencrypt/live/gojoe.dev/fullchain.pem"),
    );
    let tls_acceptor = TlsAcceptor::from(rustls_config);
    let bind = "[::]:7878";

    let tcp_listener = TcpListener::bind(bind).await.unwrap();
    println!("server listening on {} ...", bind);

    let app = Router::new()
        .route("/search", get(handle_client))
        .with_state(index)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        );

    loop {
        let tower_service = app.clone();
        let tls_acceptor = tls_acceptor.clone();

        let (cnx, addr) = tcp_listener.accept().await.unwrap();

        tokio::spawn(async move {
            let Ok(stream) = tls_acceptor.accept(cnx).await else {
                println!("error during tls handshake connection from {}", addr);
                return;
            };

            let stream = hyper_util::rt::tokio::TokioIo::new(stream);

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let ret = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await;

            if let Err(err) = ret {
                println!("error serving connection from {}: {}", addr, err);
            }
        });
    }
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
        index.search(search_query)
    };

    let contents = serde_json::to_string(&results)
        .expect("should have been able to convert search results to json");

    Ok(contents)
    //Ok("hello there".to_string())
}
fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = pkcs8_private_keys(&mut key_reader)
        .collect::<Vec<_>>()
        .remove(0)
        .unwrap();
    let certs = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .expect("should have worked");
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            certs,
            tokio_rustls::rustls::pki_types::PrivateKeyDer::Pkcs8(key),
        )
        .expect("bad certificate");

    config.alpn_protocols = vec![b"http/2".to_vec(), b"http/1.1".to_vec()];
    Arc::new(config)
}
