use crate::db::{get_all_documents, init_db};
use crate::document::Document;
use crate::lexer::tokenize;
use crate::search::generate_snippets;
use crate::search::{cosine_similarity, SearchResult};
use epub::doc::EpubDoc;
use html2text::from_read;
use pdf_extract;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Default)]
pub struct Index {
    pub documents: Vec<Document>,
    pub tokens: HashMap<String, i32>,
    pub idf: HashMap<String, f32>,
    pub conn: rusqlite::Connection,
}
pub struct IndexStats {
    pub total_tokens: usize,
    pub unique_tokens: usize,
}
impl Index {
    pub fn new(file_path: &str) -> Result<Self, io::Error> {
        let conn = init_db().expect("Should have been able to initialize database");
        let db_documents = get_all_documents(&conn);

        let documents = init(file_path)?;
        let mut index = Index {
            documents,
            tokens: HashMap::<String, i32>::new(),
            idf: HashMap::<String, f32>::new(),
            conn,
        };
        index.build();
        Ok(index)
    }
    pub fn search(&mut self, query: &str) -> Vec<SearchResult> {
        let query_tokens: Vec<String> = self.parse_query(query);
        let similarities = cosine_similarity(self, &query_tokens);
        let results: Vec<SearchResult> = similarities
            .into_iter()
            .filter_map(|(filename, similarity)| {
                if similarity > 0.0 {
                    self.documents
                        .iter()
                        .find(|d| d.filename == filename)
                        .and_then(|document| {
                            generate_snippets(document, &query_tokens, 80).map(|snippets| {
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
        println!("got {} search results for: {}", results.len(), query);
        results.reverse();
        results
    }
    pub fn get_stats(&mut self) -> IndexStats {
        let mut total_tokens = 0;
        for doc in &self.documents {
            total_tokens += doc.total_tokens_in_file;
        }

        IndexStats {
            total_tokens,
            unique_tokens: self.tokens.len(),
        }
    }
    pub fn get_all_doc_names(&mut self) -> Vec<String> {
        self.documents
            .iter()
            .map(|doc| doc.filename.clone())
            .collect()
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
    fn parse_query(&mut self, query: &str) -> Vec<String> {
        let mut in_quotes = false;
        let mut current_phrase = String::new();
        let mut query_tokens = Vec::<String>::new();

        for c in query.chars() {
            match c {
                '"' => {
                    in_quotes = !in_quotes;

                    if !in_quotes && !current_phrase.is_empty() {
                        query_tokens.push(current_phrase.clone());
                        current_phrase.clear();
                    }
                }
                /// taco "taco bell" taco
                ' ' => {
                    if !in_quotes && !current_phrase.is_empty() {
                        query_tokens.push(current_phrase.clone());
                        current_phrase.clear();
                    }
                    if in_quotes {
                        current_phrase.push(c);
                    }
                }
                _ => {
                    current_phrase.push(c);
                }
            }
        }
        query_tokens.push(current_phrase.clone());
        query_tokens
    }
}
fn init(file_path: &str) -> Result<Vec<Document>, io::Error> {
    // I think this should accept the db_documents and do a comparison
    // to see if the document already exists in the db...
    // if it does then don't add it
    println!("reading filepath {}", file_path);
    let dir = Path::new(file_path);
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("path is not directory: {}", file_path),
        ));
    }

    let mut file_paths = Vec::new();
    fn collect_files(dir: &Path, files: &mut Vec<String>) -> Result<(), io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files)?;
            } else if path.is_file() {
                if let Some(path_str) = path.to_str() {
                    files.push(path_str.to_string());
                }
            }
        }
        Ok(())
    }
    collect_files(dir, &mut file_paths)?;

    let conn = db::init_db()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Database error: {}", e)))?;

    let mut all_documents = db::get_all_documents(&conn).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to get documents from database: {}", e),
        )
    })?;

    for file_path in file_paths {
        let path = Path::new(&file_path);
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            if !all_documents.iter().any(|doc| doc.filename == filename) {
                println!("Processing new file: {}", filename);
                match process_file(path, filename) {
                    Ok(document) => {
                        if let Err(e) = db::insert_document(&conn, &document) {
                            eprintln!("Failed to store document {} in database: {}", filename, e);
                        }
                        all_documents.push(document)
                    }
                    Err(e) => eprintln!("Error processing file {}: {}", filename, e),
                }
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
    match std::panic::catch_unwind(|| pdf_extract::extract_text(path)) {
        Ok(result) => match result {
            Ok(text) => Ok(Document {
                filename: filename.to_string(),
                raw_contents: text.into_bytes(),
                tf: HashMap::new(),
                tfidf: HashMap::new(),
                total_tokens_in_file: 0,
            }),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("PDF extraction error for {}: {}", filename, e),
            )),
        },
        Err(_) => {
            eprintln!("pdf_extract lib panic while processing {}", filename);
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("pdf_extract lib crash while processing {}", filename),
            ))
        }
    }
}
fn process_epub(path: &Path, filename: &str) -> Result<Document, io::Error> {
    let mut doc = EpubDoc::new(&path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open EPUB: {}", e)))?;
    let mut content = String::new();
    let len = doc.spine.len();
    for _ in 0..len {
        if let Some((_, html_content)) = doc.get_current_str() {
            let text = from_read(html_content.as_bytes(), html_content.len());
            content.push_str(&text);
            content.push_str("\n");
        };
        doc.go_next();
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
