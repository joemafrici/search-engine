use crate::document::Document;
use rusqlite::{Connection, Result};

use std::collections::HashMap;

pub fn init_db() -> Result<Connection> {
    // when on macos
    // let conn = Connection::open("/Users/deepwater/Documents/search_engine.db")?;
    let conn = Connection::open("/home/deepwater/Documents/search_engine.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL UNIQUE,
            raw_contents BLOB NOT NULL,
            tf TEXT NOT NULL,
            tfidf TEXT NOT NULL,
            total_tokens_in_file INTEGER NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

pub fn insert_document(conn: &Connection, doc: &Document) -> Result<()> {
    let tf_json = serde_json::to_string(&doc.tf)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let tfidf_json = serde_json::to_string(&doc.tfidf)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    conn.execute(
        "INSERT OR REPLACE INTO documents (filename, raw_contents, tf, tfidf, total_tokens_in_file) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &doc.filename,
            &doc.raw_contents,
            &tf_json,
            &tfidf_json,
            doc.total_tokens_in_file as i64
        ),
    )?;

    Ok(())
}

pub fn get_all_documents(conn: &Connection) -> Result<Vec<Document>> {
    let mut stmt = conn.prepare("SELECT * from documents")?;

    let document_iter = stmt.query_map([], |row| {
        println!("reading row from database");

        let filename: String = row.get(1)?;
        println!("got filename: {}", filename);

        let raw_contents: Vec<u8> = match row.get(2) {
            Ok(contents) => contents,
            Err(e) => {
                println!("Error reading raw contents: {:?}", e);
                return Err(e);
            }
        };
        println!("got raw_contents of length: {}", raw_contents.len());

        let tf_str: String = row.get(3)?;

        let tf: HashMap<String, f32> = match serde_json::from_str(&tf_str) {
            Ok(map) => map,
            Err(e) => {
                println!("Error parsing tf json: {}", e);
                return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(e)));
            }
        };
        println!("parsed tf map");

        let tfidf_str: String = row.get(4)?;
        let tfidf: HashMap<String, f32> = match serde_json::from_str(&tfidf_str) {
            Ok(map) => map,
            Err(e) => {
                println!("Error parsing tfidf json: {}", e);
                return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(e)));
            }
        };
        println!("parsed tfidf map");

        let total_tokens: i64 = row.get(5)?;
        println!("got total tokens: {}", total_tokens);

        Ok(Document {
            filename,
            raw_contents,
            tf,
            tfidf,
            total_tokens_in_file: total_tokens as usize,
        })
    })?;

    document_iter.collect()
}

pub fn get_document_by_filename(conn: &Connection, filename: &str) -> Result<Option<Document>> {
    // TODO: does the get() fail if I do SELECT * instead of the specific column names???
    let mut stmt = conn.prepare("SELECT * from documents WHERE filename = ?")?;

    let mut rows = stmt.query([filename])?;

    if let Some(row) = rows.next()? {
        let tf: HashMap<String, f32> = serde_json::from_str(row.get::<_, String>(2)?.as_str())
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let tfidf: HashMap<String, f32> =
            serde_json::from_str(row.get::<_, String>(3)?.as_str())
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(Some(Document {
            filename: row.get(0)?,
            raw_contents: row.get(1)?,
            tf,
            tfidf,
            total_tokens_in_file: row.get::<_, i64>(4)? as usize,
        }))
    } else {
        Ok(None)
    }
}
