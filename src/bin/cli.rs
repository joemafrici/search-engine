use search_engine::index::Index;
use search_engine::search::SearchResult;
use std::env;
use text_io::read;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let mut index = Index::new("../../../books").expect("failed to build index");
    println!("found {} unique tokens", index.tokens.len());
    let mut num_tokens: usize = 0;
    for doc in &index.documents {
        num_tokens += doc.total_tokens_in_file;
    }
    println!("found {} total tokens", num_tokens);

    loop {
        // read until a newline (but not including it)
        println!("------------------- Enter search query -------------------");
        let query: String = read!("{}\n");
        let results: Vec<SearchResult> = index.search(&query);
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
