//use rusqlite::{Connection, Result};
use search_engine::index::Index;
use std::env;
use text_io::read;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    //println!("connecting to database");
    //let conn = Connection::open_in_memory()?;
    //conn.execute(
    //    "CREATE TABLE IF NOT EXISTS documents (
    //        id INTEGER PRIMARY KEY AUTOINCREMENT,
    //        filename TEXT NOT NULL,
    //        raw_contents BLOB NOT NULL,
    //        tf TEXT NOT NULL,
    //        tfidf TEXT NOT NULL,
    //        total_tokens_in_file INTEGER NOT NULL
    //    )",
    //    (),
    //)?;

    //let mut stmt = conn
    //    .prepare("SELECT filename, raw_contents, tf, tfidf, total_tokens_in_file FROM documents");
    //let documents_iter = stmt.query_map([], |row| {
    //    Ok(search_engine::document::Document {
    //        filename: row.get(0)?,
    //        raw_contents: row.get(1)?,
    //        tf: row.get(2)?,
    //        tfidf: row.get(3)?,
    //        total_tokens_in_file: row.get(4)?,
    //    })
    //})?;

    let mut index = {
        println!("building index from files...");
        let index = Index::new("../../../books").expect("failed to build index");
        println!("found {} unique tokens", index.tokens.len());
        index
    };
    let mut num_tokens: usize = 0;
    for doc in &index.documents {
        num_tokens += doc.total_tokens_in_file;
    }
    println!("found {} total tokens", num_tokens);

    loop {
        // read until a newline (but not including it)
        println!("------------------- Enter search query -------------------");
        let query: String = read!("{}\n");
        let results: Vec<search_engine::search::SearchResult> = index.search(&query);
        for result in results {
            println!(
                "filename: {} with similarity: {}",
                result.filename, result.similarity
            );
            for snip in result.snippets {
                println!("{}", snip);
            }
        }
    }
}
