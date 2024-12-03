//use crate::document::Document;
//use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
//use sqlx::Error;
//
//pub async fn init_db() -> Result<SqlitePool, Error> {
//    // figure out this address for docker container
//    let db_url =
//        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:search_engine.db".to_string());
//    let pool = SqlitePoolOptions::new()
//        .max_connections(5)
//        .connect(&db_url)
//        .await?;
//    sqlx::query(
//        r#"
//    CREATE TABLE IF NOT EXISTS documents (
//        id INTEGER PRIMARY KEY AUTOINCREMENT,
//        filename TEXT NOT NULL,
//        raw_contents BLOB NOT NULL,
//        tf TEXT NOT NULL,
//        tfidf TEXT NOT NULL,
//        total_tokens_in_file INTEGER NOT NULL
//    );
//    "#,
//    )
//    .execute(&pool)
//    .await?;
//    Ok(pool)
//}
//pub async fn insert_document(pool: &SqlitePool, doc: &Document) -> Result<(), Error> {
//    sqlx::query!(
//        "INSERT INTO documents (filename, raw_contents, tf, tfidf, total_tokens_in_file)
//    VALUES (?, ?, ?, ?, ?)",
//        doc.filename,
//        doc.raw_contents,
//        serde_json::to_value(&doc.tf).unwrap(),
//        serde_json::to_value(&doc.tfidf).unwrap(),
//        doc.total_tokens_in_file as i32,
//    )
//    .execute(pool)
//    .await?;
//    Ok(())
//}
//pub async fn get_all_documents(pool: &SqlitePool) -> Result<Vec<Document>, Error> {
//    let rows = sqlx::query!(
//        "SELECT filename, raw_contents, tf, tfidf, total_tokens_in_file FROM documents"
//    )
//    .fetch_all(pool)
//    .await?;
//
//    let documents = rows
//        .into_iter()
//        .map(|row| Document {
//            filename: row.filename,
//            raw_contents: row.raw_contents,
//            tf: serde_json::from_str(row.tf).unwrap(),
//            tfidf: serde_json::from_str(row.tfidf).unwrap(),
//            total_tokens_in_file: row.total_tokens_in_file as usize,
//        })
//        .collect();
//    Ok(documents)
//}
