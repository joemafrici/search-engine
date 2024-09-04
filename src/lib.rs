mod lexer;
use pdf_extract;
use std::collections::HashMap;
use std::f32;
use std::fs;
use std::io;
use std::path::Path;

use crate::lexer::tokenize;
use serde::{Deserialize, Serialize};
//use serde_json::Result;

#[derive(Debug)]
pub struct Document {
    filename: String,
    raw_contents: Vec<u8>,
    tf: HashMap<String, f32>,
    tfidf: HashMap<String, f32>,
    pub total_tokens_in_file: usize,
}
pub struct Index {
    pub documents: Vec<Document>,
    pub tokens: HashMap<String, i32>,
    idf: HashMap<String, f32>,
}
impl Index {
    pub fn new(file_path: &str) -> Result<Self, io::Error> {
        let documents = init(file_path)?;
        let mut index = Index {
            documents,
            tokens: HashMap::<String, i32>::new(),
            idf: HashMap::<String, f32>::new(),
        };
        index.build();
        Ok(index)
    }
    pub fn search(&mut self, query: &str) -> Vec<SearchResult> {
        let query_tokens: Vec<String> = tokenize(query);
        let similarities = cosine_similarity(self, &query_tokens);
        let results: Vec<SearchResult> = similarities
            .into_iter()
            .filter_map(|(filename, similarity)| {
                self.documents
                    .iter()
                    .find(|d| d.filename == filename)
                    .and_then(|document| {
                        generate_snippets(document, &query_tokens, 20).map(|snippets| {
                            SearchResult {
                                filename,
                                similarity,
                                snippets,
                            }
                        })
                    })
            })
            .collect::<Vec<SearchResult>>();
        let mut results = results;
        results.sort_by(|a, b| a.similarity.partial_cmp(&b.similarity).unwrap());
        results.reverse();
        results
    }
    fn build(&mut self) {
        self.tf();
        self.idf();
        self.tfidf();
    }
    fn tf(&mut self) {
        for document in &mut self.documents {
            let contents = String::from_utf8_lossy(&document.raw_contents);
            let tokens = tokenize(&contents);
            let num_tokens = tokens.len();
            document.total_tokens_in_file = num_tokens;
            for token in tokens {
                *document.tf.entry(token.clone()).or_insert(0.0) += 1.0;
                *self.tokens.entry(token).or_insert(0) += 1;
            }
            for count in document.tf.values_mut() {
                *count /= num_tokens as f32;
            }
        }
    }
    fn idf(&mut self) {
        for (token, frequency) in &self.tokens {
            let idf = 1.0 + f32::ln(self.documents.len() as f32 / *frequency as f32);
            self.idf.insert(token.clone(), idf);
        }
    }
    fn tfidf(&mut self) {
        for document in &mut self.documents {
            for (term, frequency) in &document.tf {
                let idf = self.idf.get(term).unwrap_or(&1.0);
                document.tfidf.insert(term.clone(), frequency * idf);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub filename: String,
    pub similarity: f32,
    pub snippets: Vec<String>,
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
            println!("reading {}", filename);
            let raw_contents = fs::read(&path)?;
            if filename.to_lowercase().ends_with(".pdf") {
                let text = pdf_extract::extract_text(&path)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                all_documents.push(Document {
                    filename,
                    raw_contents: text.into_bytes(),
                    tf: HashMap::new(),
                    tfidf: HashMap::new(),
                    total_tokens_in_file: 0,
                });
            } else {
                all_documents.push(Document {
                    filename,
                    raw_contents,
                    tf: HashMap::new(),
                    tfidf: HashMap::new(),
                    total_tokens_in_file: 0,
                });
            }
        }
    }
    Ok(all_documents)
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
    let mut snippet_ranges: Vec<(usize, usize, String)> = Vec::new();

    for query_word in query {
        for (word_index, word) in words.iter().enumerate() {
            if word.to_lowercase().contains(&query_word.to_lowercase()) {
                let start = word_index.saturating_sub(context_size);
                let end = (word_index + context_size + 1).min(words.len());

                let mut overlapped = false;
                for range in &mut snippet_ranges {
                    if (start <= range.1 && end >= range.0) || (range.0 <= end && range.1 >= start)
                    {
                        range.0 = range.0.min(start);
                        range.1 = range.1.max(end);
                        overlapped = true;
                        break;
                    }
                }

                if !overlapped {
                    let snippet = words[start..end].join(" ");
                    snippet_ranges.push((start, end, snippet));
                }
            }
        }
    }

    snippet_ranges.sort_by_key(|&(start, _, _)| start);

    let snippets: Vec<String> = snippet_ranges
        .into_iter()
        .enumerate()
        .map(|(i, (start, _, snippet))| {
            if i == 0 && start > 0 {
                format!("...{}", snippet)
            } else {
                snippet
            }
        })
        .collect();

    if snippets.is_empty() {
        None
    } else {
        Some(snippets)
    }
}
