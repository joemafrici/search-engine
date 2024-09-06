use crate::document::Document;
use crate::index::Index;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub filename: String,
    pub similarity: f32,
    pub snippets: Vec<String>,
}

pub fn cosine_similarity(idx: &mut Index, query: &[String]) -> Vec<(String, f32)> {
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
pub fn generate_snippets(
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
