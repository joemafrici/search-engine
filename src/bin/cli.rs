use search_engine::index::Index;

fn main() {
    let index = Index::new("../processed_transcripts").expect("failed to build index");
    println!("found {} unique tokens", index.tokens.len());
    let mut num_tokens: usize = 0;
    for doc in &index.documents {
        num_tokens += doc.total_tokens_in_file;
    }
    println!("found {} total tokens", num_tokens);
}
