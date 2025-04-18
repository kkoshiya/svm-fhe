use tokio_rusqlite::Connection;
use std::path::Path;
use tfhe::prelude::*;
use tfhe::{CompressedCiphertextList, ClientKey, FheUint8, FheUint64, set_server_key};
mod keys;

const DB_PATH: &str = "data/tfhe.db";

pub async fn test_first_value_zero() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting database test...");
    let client_key = keys::load_client_key()?;
    let server_key = keys::load_server_key()?;
    
    // Set the server key before any operations
    set_server_key(server_key);
    
    let conn = Connection::open(DB_PATH).await?;
    let blob = conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT ciphertext FROM computations LIMIT 1")?;
        let blob: Vec<u8> = stmt.query_row([], |row| row.get(0))?;
        Ok(blob)
    }).await?;
    println!("Retrieved blob of size: {} bytes", blob.len());
    
    let compressed: CompressedCiphertextList = bincode::deserialize(&blob)?;
    let value: FheUint64 = compressed.get(0)?.unwrap();
    let decrypted: u64 = value.decrypt(&client_key);
    
    println!("Decrypted value: {}", decrypted);
    assert_eq!(decrypted, 0);
    Ok(())
}

pub async fn test_last_values() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting last values test...");
    let client_key = keys::load_client_key()?;
    let server_key = keys::load_server_key()?;
    
    // Set the server key before any operations
    set_server_key(server_key);
    
    let conn = Connection::open(DB_PATH).await?;
    
    let rows = conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT key, ciphertext FROM computations ORDER BY ROWID DESC LIMIT 3")?;
        let rows: Vec<(Vec<u8>, Vec<u8>)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }).await?;

    println!("Retrieved {} rows", rows.len());
    
    for (i, (key, blob)) in rows.iter().enumerate() {
        let compressed: CompressedCiphertextList = bincode::deserialize(&blob)?;
        let value: FheUint64 = compressed.get(0)?.unwrap();
        let decrypted: u64 = value.decrypt(&client_key);
        println!("Row {}: Key = {:?}, Value = {}", i + 1, key, decrypted);
    }

    Ok(())
}

pub async fn test_last_values_fhe8_add() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting last values test...");
    let client_key = keys::load_client_key()?;
    let server_key = keys::load_server_key()?;
    
    // Set the server key before any operations
    set_server_key(server_key);
    
    let conn = Connection::open(DB_PATH).await?;
    
    let rows = conn.call(|conn| {
        let mut stmt = conn.prepare("SELECT key, ciphertext FROM computations ORDER BY ROWID DESC LIMIT 3")?;
        let rows: Vec<(Vec<u8>, Vec<u8>)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }).await?;

    println!("Retrieved {} rows", rows.len());
    
    for (i, (key, blob)) in rows.iter().enumerate() {
        let compressed: CompressedCiphertextList = bincode::deserialize(&blob)?;
        let value: FheUint8 = compressed.get(0)?.unwrap();
        let decrypted: u8 = value.decrypt(&client_key);
        println!("Row {}: Key = {:?}, Value = {}", i + 1, key, decrypted);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_first_value_zero().await;
    test_last_values().await
}