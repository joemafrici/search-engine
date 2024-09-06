use crate::document::Document;
use crate::lexer::tokenize;
use crate::search::generate_snippets;
use crate::search::{cosine_similarity, SearchResult};
use epub::doc::EpubDoc;
use pdf_extract;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
pub struct Index {
    pub documents: Vec<Document>,
    pub tokens: HashMap<String, i32>,
    pub idf: HashMap<String, f32>,
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
                if similarity > 0.0 {
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
                } else {
                    None
                }
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
fn init(file_path: &str) -> Result<Vec<Document>, io::Error> {
    println!("reading filepath {}", file_path);
    let mut all_documents = Vec::<Document>::new();
    let dir = Path::new(file_path);
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("path is not directory: {}", file_path),
        ));
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read directory entry: {}", e),
            )
        })?;
        let path = entry.path();
        if path.is_file() {
            let filename = path
                .file_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to extract file name"))?
                .to_string_lossy()
                .into_owned();
            println!("Attempting to read file: {}", filename);
            match process_file(&path, &filename) {
                Ok(document) => all_documents.push(document),
                Err(e) => eprintln!("Error processing file {}: {}", filename, e),
            }
        }
    }
    if all_documents.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "No valid documents found in directory",
        ))
    } else {
        Ok(all_documents)
    }
}
fn process_file(path: &Path, filename: &str) -> Result<Document, io::Error> {
    if filename.to_lowercase().ends_with(".pdf") {
        process_pdf(path, filename)
    } else if filename.to_lowercase().ends_with(".epub") {
        process_epub(path, filename)
    } else if filename.to_lowercase().ends_with(".txt") {
        process_txt(path, filename)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("unsopported filetype for: {}", filename),
        ))
    }
}
fn process_pdf(path: &Path, filename: &str) -> Result<Document, io::Error> {
    let text =
        pdf_extract::extract_text(&path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(Document {
        filename: filename.to_string(),
        raw_contents: text.into_bytes(),
        tf: HashMap::new(),
        tfidf: HashMap::new(),
        total_tokens_in_file: 0,
    })
}
fn process_epub(path: &Path, filename: &str) -> Result<Document, io::Error> {
    let mut doc = EpubDoc::new(&path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut content = String::new();
    let len = doc.spine.len();
    for _ in 1..len {
        if doc.go_next() {
            let (thing, _) = doc.get_current_str().unwrap();
            content.push_str(&thing);
        }
    }
    Ok(Document {
        filename: filename.to_string(),
        raw_contents: content.into_bytes(),
        tf: HashMap::new(),
        tfidf: HashMap::new(),
        total_tokens_in_file: 0,
    })
}
fn process_txt(path: &Path, filename: &str) -> Result<Document, io::Error> {
    let raw_contents = fs::read(&path)?;
    Ok(Document {
        filename: filename.to_string(),
        raw_contents,
        tf: HashMap::new(),
        tfidf: HashMap::new(),
        total_tokens_in_file: 0,
    })
}
