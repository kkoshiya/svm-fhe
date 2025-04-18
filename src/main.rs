use tfhe::prelude::*;
use tfhe::{ set_server_key, ServerKey, ClientKey, CompressedCiphertextList, CompressedCiphertextListBuilder};
use std::io::Cursor;
use axum::{
    routing::{get, post}, Router, Json, extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_rusqlite::Connection;
use std::path::Path;
use std::fs;
use async_trait::async_trait;
use tokio::try_join;
mod keys;
mod types;
mod operations;
mod handlers;
mod cache;
use handlers::{handle_fhe8_add, handle_post, handle_encrypt8, handle_transfer, handle_view, handle_withdraw};
use crate::operations::{init_db, update_ciphertext, get_ciphertext, insert_ciphertext};

const DB_PATH: &str = "data/tfhe.db";
const zero_key: [u8; 32] = [0u8; 32];

#[derive(Clone)]
struct AppState {
    db: Arc<Connection>,
    server_key: Arc<ServerKey>,
    client_key: Arc<ClientKey>,
}

#[async_trait]
pub trait KeyAccess {
    fn get_server_key(&self) -> Arc<ServerKey>;
    fn get_client_key(&self) -> Arc<ClientKey>;
}

impl KeyAccess for AppState {
    fn get_server_key(&self) -> Arc<ServerKey> {
        self.server_key.clone()
    }
    fn get_client_key(&self) -> Arc<ClientKey> {
        self.client_key.clone()
    }
}

////////////////// Request structs //////////////////

#[derive(Deserialize)]
struct Request {
    key: [u8; 32],
    value: u64,
}

#[derive(Deserialize)]
struct Transfer {
    sender_key: [u8; 32],
    recipient_key: [u8; 32],
    transfer_value: [u8; 32],
}

#[derive(Deserialize)]
struct Withdraw {
    key: [u8; 32],
    value: [u8;32]
}

#[derive(Deserialize)]
struct Decrypt {
    key: [u8; 32],
}

//////////////////////// Response Structs ///////////////////////////

#[derive(Serialize)]
struct ViewResponse {
    result: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("data").exists() {
        fs::create_dir("data").expect("Failed to create data directory");
    }
    let state = AppState {
        db: Arc::new(Connection::open(DB_PATH).await?),
        server_key: Arc::new(keys::load_server_key()?),
        client_key: Arc::new(keys::load_client_key()?),
    };
    init_db(&state.db).await?;
    let app = Router::new()
        .route("/post", post(handle_post))
        .route("/encrypt8", post(handle_encrypt8))
        .route("/transfer", post(handle_transfer))
        .route("/decrypt", post(handle_view))
        .route("/withdraw", post(handle_withdraw))
        .route("/fhe8add", post(handle_fhe8_add))
        .with_state(state);

    println!("Server starting on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}







