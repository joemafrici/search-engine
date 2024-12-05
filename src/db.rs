use crate::document::Document;
use rusqlite::{Connection, Result};

use std::collections::HashMap;

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("search_engine.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL UNIQUE,
            raw_contents BLOB NOT NULL,
            tf TEXT NOT NULL,
            tfidf TEXT NOT NULL,
            total_tokens_in_file TEXT NOT NULL
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
        let tf: HashMap<String, f32> = serde_json::from_str(row.get::<_, String>(2)?.as_str())
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let tfidf: HashMap<String, f32> =
            serde_json::from_str(row.get::<_, String>(3)?.as_str())
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(Document {
            filename: row.get(0)?,
            raw_contents: row.get(1)?,
            tf,
            tfidf,
            total_tokens_in_file: row.get::<_, i64>(4)? as usize,
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
