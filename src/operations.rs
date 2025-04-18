use axum::http::StatusCode;
use tfhe::{FheUint8, FheUint64, CompressedCiphertextList};
use tfhe::prelude::*;
use std::path::Path;
use std::fs;
use tokio_rusqlite::Connection;

const DB_PATH: &str = "data/tfhe.db";

pub async fn get_prepared_ciphertext(key: [u8; 32]) -> Result<FheUint64, StatusCode> {
    let serialized_data = get_ciphertext(key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deserialized_compressed: CompressedCiphertextList = bincode::deserialize(&serialized_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    deserialized_compressed.get(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_prepared_ciphertext_8(key: [u8; 32]) -> Result<FheUint8, StatusCode> {
    let serialized_data = get_ciphertext(key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deserialized_compressed: CompressedCiphertextList = bincode::deserialize(&serialized_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    deserialized_compressed.get(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_ciphertext(key: [u8; 32]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_PATH).await?;
    
    conn.call(move |conn| {
        conn.query_row(
            "SELECT ciphertext FROM computations WHERE key = ?",
            [key],
            |row| row.get(0)
        )
    }).await.map_err(Into::into)
}

pub async fn init_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = Path::new(DB_PATH).parent() {
        println!("Creating directory at: {:?}", parent);
        fs::create_dir_all(parent)?;
    }
    conn.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS computations (
                key CHAR(32) NOT NULL PRIMARY KEY,
                ciphertext BLOB NOT NULL
            )",
            (),
        ).map_err(|e| {
            println!("Database error: {}", e);
            e
        })?;
        Ok(())
    })
    .await?;
    Ok(())
}

pub async fn update_ciphertext(key: [u8; 32], new_ciphertext: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_PATH).await?;
    conn.call(move |conn| {
        let mut stmt = conn.prepare(
            "UPDATE computations SET ciphertext = ? WHERE key = ?"
        )?;
        let rows_affected = stmt.execute((&new_ciphertext, &key))?;
        if rows_affected == 0 {
            println!("No row found with the given key");
        } else {
            println!("Updated ciphertext for key: {:?}", key);
        }
        Ok(())
    }).await?;
    Ok(())
}

pub async fn insert_ciphertext(key: [u8; 32], ciphertext: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_PATH).await?;
    println!("inserting ciphertext via helper");
    conn.call(move |conn| {
        conn.execute(
            "INSERT OR REPLACE INTO computations (key, ciphertext) VALUES (?1, ?2)",
            (key, ciphertext),
        ).map_err(|e| {
            println!("Insert error: {}", e);
            e
        })?;
        Ok(())
    }).await?;
    Ok(())
}