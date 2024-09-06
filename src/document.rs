use std::collections::HashMap;
#[derive(Debug)]
pub struct Document {
    pub filename: String,
    pub raw_contents: Vec<u8>,
    pub tf: HashMap<String, f32>,
    pub tfidf: HashMap<String, f32>,
    pub total_tokens_in_file: usize,
}
