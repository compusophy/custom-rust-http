use axum::{extract::State, Json};
use serde::Serialize;

use crate::state::AppState;
use crate::types::PoloResponse;

pub async fn marco() -> Json<PoloResponse> {
    println!("[API] /api/marco endpoint called");
    Json(PoloResponse {
        message: "polo".to_string(),
    })
}

#[derive(Serialize)]
pub struct RedisTestResponse {
    key: String,
    value: String,
    message: String,
}

pub async fn redis_test(
    State(state): State<AppState>,
) -> Result<Json<RedisTestResponse>, String> {
    println!("[API] /api/redis-test endpoint called");

    let mut conn = state
        .redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let test_key = "rust_app:test";
    let test_value = "Hello from Rust! This is persistent storage.";

    // Set a value
    redis::cmd("SET")
        .arg(test_key)
        .arg(&test_value)
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| format!("Failed to SET: {}", e))?;

    // Get the value back
    let stored_value: String = redis::cmd("GET")
        .arg(test_key)
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| format!("Failed to GET: {}", e))?;

    Ok(Json(RedisTestResponse {
        key: test_key.to_string(),
        value: stored_value,
        message: "Successfully stored and retrieved from Redis!".to_string(),
    }))
}


