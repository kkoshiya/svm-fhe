use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use tfhe::{
    FheUint8,
    FheUint64,
    CompressedCiphertextListBuilder,
    set_server_key,
};
use tfhe::prelude::*;
use tokio::try_join;
use crate::{
    AppState,
    KeyAccess,
    operations::{self, update_ciphertext, insert_ciphertext},
    types::{
        Request,
        Transfer,
        Decrypt,
        Withdraw,
        ViewResponse,
        zero_key,
    },
};

pub async fn handle_post(State(state): State<AppState>, Json(payload): Json<Request>) -> Result<StatusCode, StatusCode> {
    println!("Received value: {}, key: {:?}", payload.value, payload.key);
    let client_key = state.get_client_key();
    let server_key = state.get_server_key();
    set_server_key((*server_key).clone());
    let value = FheUint64::encrypt(payload.value, &*client_key);
    println!("Encrypted value type: {:?}", std::any::type_name_of_val(&value));
    let compressed = CompressedCiphertextListBuilder::new()
        .push(value)
        .build()
        .map_err(|e| {
            println!("Compression error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
    });
    println!("Serializing compressed value...");
    let serialized_data = bincode::serialize(&compressed.unwrap())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    insert_ciphertext(payload.key, serialized_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("hit the end of post");
    Ok(StatusCode::OK)
}

pub async fn handle_encrypt8(State(state): State<AppState>, Json(payload): Json<Request>) -> Result<StatusCode, StatusCode> {
    println!("Received value: {}, key: {:?}", payload.value, payload.key);
    let client_key = state.get_client_key();
    let server_key = state.get_server_key();
    set_server_key((*server_key).clone());
    let value = FheUint8::encrypt(payload.value, &*client_key);
    println!("Encrypted value type: {:?}", std::any::type_name_of_val(&value));
    let compressed = CompressedCiphertextListBuilder::new()
        .push(value)
        .build()
        .map_err(|e| {
            println!("Compression error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
    });
    println!("Serializing compressed value...");
    let serialized_data = bincode::serialize(&compressed.unwrap())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    insert_ciphertext(payload.key, serialized_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("hit the end of encrypt8");
    Ok(StatusCode::OK)
}

pub async fn handle_transfer(State(state): State<AppState>, Json(payload): Json<Transfer>) -> Result<StatusCode, StatusCode> {
    println!("=== TRANSFER REQUEST RECEIVED ===");
    let server_key = state.get_server_key();
    set_server_key((*server_key).clone());
    println!("handle_transfer hit!!!!!!!!");

    println!("Attempting to fetch sender ciphertext...");
    println!("Sender key: {:?}", payload.sender_key);
    println!("Reciver key: {:?}", payload.recipient_key);
    println!("transfer key: {:?}", payload.transfer_value);
    println!("Fetching all required values...");
    let (sender_value, recipient_value, transfer_value, zero_value) = try_join!(
        operations::get_prepared_ciphertext(payload.sender_key),
        operations::get_prepared_ciphertext(payload.recipient_key),
        operations::get_prepared_ciphertext(payload.transfer_value),
        operations::get_prepared_ciphertext(zero_key)
    ).map_err(|e| {
        println!("Error fetching values: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    println!("Successfully fetched all values");

    println!("about to start operations");
    let condition = sender_value.ge(&transfer_value);
    let real_amount = condition.if_then_else(&transfer_value, &zero_value);
    let new_sender_value = &sender_value - &real_amount;
    let new_recipient_value = &recipient_value + &real_amount;
    println!("ending operations");

    let compressed_sender = CompressedCiphertextListBuilder::new()
        .push(new_sender_value.clone())
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let serialized_sender = bincode::serialize(&compressed_sender)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    update_ciphertext(payload.sender_key, serialized_sender.clone()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let compressed_recipient = CompressedCiphertextListBuilder::new()
        .push(new_recipient_value.clone())
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let serialized_recipient = bincode::serialize(&compressed_recipient)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    update_ciphertext(payload.recipient_key, serialized_recipient.clone()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn handle_view(
    State(state): State<AppState>, 
    Json(payload): Json<Decrypt>
) -> Result<Json<ViewResponse>, StatusCode> {
    println!("Received key bytes: {:?}", payload.key);  // Debug incoming data
    
    let client_key = state.get_client_key();
    let server_key = state.get_server_key();
    set_server_key((*server_key).clone());

    // Add error logging
    let value = operations::get_prepared_ciphertext(payload.key)
        .await
        .map_err(|e| {
            println!("Error preparing ciphertext: {:?}", e);  // Log the actual error
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("Successfully prepared ciphertext");  // Confirm success
    
    let decrypted: u64 = value.decrypt(&client_key);
    println!("Decrypted value: {}", decrypted);  // Log decrypted value
    
    Ok(Json(ViewResponse { result: decrypted }))
}

pub async fn handle_withdraw(State(state): State<AppState>, 
Json(payload): Json<Withdraw>
) -> Result<Json<ViewResponse>, StatusCode> {
    let client_key = state.get_client_key();
    let server_key = state.get_server_key();
    set_server_key((*server_key).clone());
    
    let balance = operations::get_prepared_ciphertext(payload.key)
        .await?;
    let transfer = operations::get_prepared_ciphertext(payload.value)
        .await?;
    let zero_value = operations::get_prepared_ciphertext(zero_key).await?;
        
    let condition = balance.ge(&transfer);
    let real_amount = condition.if_then_else(&transfer, &zero_value);
    let new_balance = balance - real_amount;

    let compressed = CompressedCiphertextListBuilder::new()
        .push(new_balance.clone())
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let serialized_data = bincode::serialize(&compressed)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    update_ciphertext(payload.key, serialized_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let decrypted: u64 = new_balance.decrypt(&client_key);
    Ok(Json(ViewResponse { result: decrypted }))
}

#[derive(Debug, serde::Deserialize)]
pub struct Fhe8AddRequest {
    pub lhs_key: [u8; 32],
    pub rhs_key: [u8; 32],
    pub result_key: [u8; 32],
}

pub async fn handle_fhe8_add(
    State(state): State<AppState>, 
    Json(payload): Json<Fhe8AddRequest>
) -> Result<StatusCode, StatusCode> {
    println!("=== FHE 8 ADD REQUEST RECEIVED ===");

    let client_key = state.get_client_key();
    let server_key = state.get_server_key();

    set_server_key((*server_key).clone());

    let lhs = operations::get_prepared_ciphertext_8(payload.lhs_key).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rhs = operations::get_prepared_ciphertext_8(payload.rhs_key).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Downcast to FheUint8 if needed, or assume FheUint8
    let result = &lhs + &rhs;
    
    let number1: u8 = lhs.decrypt(&client_key);
    let number2: u8 = rhs.decrypt(&client_key);
    let decrypted: u8 = result.decrypt(&client_key);
    println!("Number 1 value: {}", number1);
    println!("Number 2 value: {}", number2);
    println!("Result value: {}", decrypted);
    
    let compressed = CompressedCiphertextListBuilder::new()
        .push(result)
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let serialized_data = bincode::serialize(&compressed)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    insert_ciphertext(payload.result_key, serialized_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("Successfully prepared ciphertext");  // Confirm success

    Ok(StatusCode::OK)
}
