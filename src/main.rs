use axum::{routing::get, Json, Router};
use ethers::providers::{Http, Provider};
use ethers::middleware::Middleware;
use serde::Serialize;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Serialize)]
struct PoloResponse {
    message: String,
}

#[derive(Serialize)]
struct BlockResponse {
    block_number: String,
    block_number_decimal: u64,
}

async fn marco() -> Json<PoloResponse> {
    Json(PoloResponse {
        message: "polo".to_string(),
    })
}

async fn get_current_block_handler() -> Json<BlockResponse> {
    match get_current_block().await {
        Ok(response) => response,
        Err(_error) => Json(BlockResponse {
            block_number: "error".to_string(),
            block_number_decimal: 0,
        }),
    }
}

async fn get_current_block() -> Result<Json<BlockResponse>, String> {
    // Using Alchemy's Base mainnet RPC for better performance
    let rpc_url = "https://base-mainnet.g.alchemy.com/v2/demo"; // Free tier, replace with your API key for production

    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|_| "Failed to create provider".to_string())?;

    let provider = Arc::new(provider);

    match provider.get_block_number().await {
        Ok(block_number) => {
            let block_hex = format!("0x{:x}", block_number);
            Ok(Json(BlockResponse {
                block_number: block_hex,
                block_number_decimal: block_number.as_u64(),
            }))
        }
        Err(_) => Err("Failed to get block number from RPC".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // Build the application with routes
    let app = Router::new()
        .route("/api/marco", get(marco))
        .route("/api/getCurrentBlock", get(get_current_block_handler));

    // Get port from environment variable or default to 3000
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on {}", addr);

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
