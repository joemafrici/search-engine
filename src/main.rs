use std::env;
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
    Ok(())
}
