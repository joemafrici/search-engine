mod lexer;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::env;
use std::f32;
use std::fs;
use std::io;
use std::path::Path;

use crate::lexer::tokenize;

#[derive(Debug)]
struct Document {
    filename: String,
    raw_contents: Vec<u8>,
    tf: HashMap<String, f32>,
    tfidf: HashMap<String, f32>,
    total_tokens_in_file: usize,
}
struct Index {
    documents: Vec<Document>,
    tokens: HashMap<String, i32>,
    idf: HashMap<String, f32>,
}

struct SearchResult {
    filename: String,
    similarity: f32,
    snippets: Vec<String>,
}

fn init(file_path: &str) -> Result<Vec<Document>, io::Error> {
    let mut all_documents = Vec::<Document>::new();
    let dir = Path::new(file_path);
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "path is not directory",
        ));
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let filename = path
                .file_name()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        "Should have been able to extract file name",
                    )
                })?
                .to_string_lossy()
                .into_owned();
            let raw_contents = fs::read(&path)?;
            all_documents.push(Document {
                filename,
                raw_contents,
                tf: HashMap::new(),
                tfidf: HashMap::new(),
                total_tokens_in_file: 0,
            });
        }
    }
    Ok(all_documents)
}
fn tf(idx: &mut Index) {
    for document in &mut idx.documents {
        let contents = String::from_utf8_lossy(&document.raw_contents);
        let tokens = tokenize(&contents);
        let num_tokens = tokens.len();
        document.total_tokens_in_file = num_tokens;
        for token in tokens {
            *document.tf.entry(token.clone()).or_insert(0.0) += 1.0;
            *idx.tokens.entry(token).or_insert(0) += 1;
        }
        for count in document.tf.values_mut() {
            *count /= num_tokens as f32;
        }
    }
}
fn idf(idx: &mut Index) {
    for (token, frequency) in &idx.tokens {
        let idf = 1.0 + f32::ln(idx.documents.len() as f32 / *frequency as f32);
        idx.idf.insert(token.clone(), idf);
    }
}
fn tfidf(idx: &mut Index) {
    for document in &mut idx.documents {
        for (term, frequency) in &document.tf {
            let idf = idx.idf.get(term).unwrap_or(&1.0);
            document.tfidf.insert(term.clone(), frequency * idf);
        }
    }
}
fn cosine_similarity(idx: &mut Index, query: &[String]) -> Vec<(String, f32)> {
    let q_tfidf = query_tfidf(idx, query);
    let mut similarities = Vec::new();
    for document in &idx.documents {
        let mut dot_product: f32 = 0.0;
        let mut query_magnitude: f32 = 0.0;
        let doc_magnitude: f32 = document.tfidf.values().map(|&v| v.powi(2)).sum();
        for (term, tfidf) in &q_tfidf {
            let doc_tfidf = document.tfidf.get(term).unwrap_or(&0.0);
            dot_product += tfidf * doc_tfidf;
            query_magnitude += tfidf.powi(2);
        }

        let similarity = dot_product / (query_magnitude.sqrt() * doc_magnitude.sqrt());
        similarities.push((document.filename.clone(), similarity));
    }

    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    similarities
}
fn query_tfidf(idx: &mut Index, query: &[String]) -> HashMap<String, f32> {
    let mut tfidf = HashMap::<String, f32>::new();
    let len = query.len() as f32;
    for term in query {
        let tf = 1.0 / len;

        let idf = idx.idf.get(term).unwrap_or(&1.0);
        tfidf.insert(term.to_string(), tf * idf);
    }
    println!("for query");
    for (term, weight) in &tfidf {
        println!("  {} => {}", term, weight);
    }
    tfidf
}
fn generate_snippets(
    document: &Document,
    query: &[String],
    context_size: usize,
) -> Option<Vec<String>> {
    let content = String::from_utf8_lossy(&document.raw_contents);
    let words: Vec<&str> = content.split_whitespace().collect();

    let snippets: Vec<String> = query
        .iter()
        .flat_map(|query_word| {
            words
                .iter()
                .enumerate()
                .filter(|&(_, word)| word.to_lowercase().contains(&query_word.to_lowercase()))
                .map(|(word_index, _)| {
                    let start = word_index.saturating_sub(context_size);
                    let end = (word_index + context_size + 1).min(words.len());
                    let snippet = words[start..end].join(" ");

                    if start > 0 {
                        format!("...{}", snippet)
                    } else {
                        snippet
                    }
                })
        })
        .collect();

    if snippets.is_empty() {
        None
    } else {
        Some(snippets)
    }
}
fn search(idx: &mut Index, query: &[String]) -> Vec<SearchResult> {
    let similarities = cosine_similarity(idx, query);
    let results: Vec<SearchResult> = similarities
        .into_iter()
        .filter_map(|(filename, similarity)| {
            idx.documents
                .iter()
                .find(|d| d.filename == filename)
                .and_then(|document| {
                    generate_snippets(document, query, 20).map(|snippets| SearchResult {
                        filename,
                        similarity,
                        snippets,
                    })
                })
        })
        .collect::<Vec<SearchResult>>();
    let mut results = results;
    results.sort_by(|a, b| a.similarity.partial_cmp(&b.similarity).unwrap());
    results
}
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    println!("In directory {file_path}");
    let query: Vec<String> = args[2..].to_vec();

    let all_documents = match init(file_path) {
        Ok(all_documents) => all_documents,
        Err(e) => return Err(e),
    };

    let mut all_documents = Index {
        documents: all_documents,
        tokens: HashMap::<String, i32>::new(),
        idf: HashMap::<String, f32>::new(),
    };

    tf(&mut all_documents);

    let mut count = 0;
    for document in &all_documents.documents {
        count += document.total_tokens_in_file;
    }
    println!("Processed {} tokens", count);

    idf(&mut all_documents);

    tfidf(&mut all_documents);

    let num_unique_tokens = all_documents.tokens.len();
    println!("There are {} unique tokens", num_unique_tokens);
    let results: Vec<SearchResult> = search(&mut all_documents, &query);
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
