//use rusqlite::{Connection, Result};
use search_engine::{
    db::{get_all_documents, init_db},
    index::Index,
};
use std::env;
use text_io::read;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let mut index = {
        println!("building index from files...");
        let index =
            Index::new("/Users/deepwater/code/search/processed_transcripts/")
                .expect("failed to build index");
        index
    };

    let mut num_tokens: usize = 0;
    for doc in &index.documents {
        num_tokens += doc.total_tokens_in_file;
    }
    println!("found {} total tokens", num_tokens);
    println!("found {} unique tokens", index.tokens.len());

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
