use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;

#[derive(Serialize)]
struct PoloResponse {
    message: String,
}

#[derive(Serialize)]
struct BlockResponse {
    block_number: String,
    block_number_decimal: u64,
}

#[derive(Serialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<String>,
    id: u32,
}

#[derive(Deserialize)]
struct RpcResponse {
    result: String,
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
    let client = reqwest::Client::new();
    let rpc_url = "https://mainnet.base.org";

    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "eth_blockNumber".to_string(),
        params: vec![],
        id: 1,
    };

    match client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<RpcResponse>().await {
                Ok(rpc_response) => {
                    // Remove "0x" prefix and parse as hex
                    let hex_block = rpc_response.result.trim_start_matches("0x");
                    match u64::from_str_radix(hex_block, 16) {
                        Ok(block_number) => Ok(Json(BlockResponse {
                            block_number: rpc_response.result,
                            block_number_decimal: block_number,
                        })),
                        Err(_) => Err("Failed to parse block number".to_string()),
                    }
                }
                Err(_) => Err("Failed to parse RPC response".to_string()),
            }
        }
        Err(_) => Err("Failed to connect to Base RPC".to_string()),
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
