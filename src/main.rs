use axum::{routing::get, Json, Router};
use ethers::providers::{Http, Provider, Ws, StreamExt};
use ethers::middleware::Middleware;
use serde::Serialize;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize)]
struct PoloResponse {
    message: String,
}

#[derive(Serialize)]
struct BlockResponse {
    block_number: String,
    block_number_decimal: u64,
}

#[derive(Clone)]
struct AppState {
    latest_block: Arc<RwLock<Option<u64>>>,
}

async fn marco() -> Json<PoloResponse> {
    Json(PoloResponse {
        message: "polo".to_string(),
    })
}

async fn get_current_block_handler(state: axum::extract::State<AppState>) -> Json<BlockResponse> {
    if let Some(block_number) = *state.latest_block.read().await {
        Json(BlockResponse {
            block_number: format!("0x{:x}", block_number),
            block_number_decimal: block_number,
        })
    } else {
        // Fallback to HTTP if cache is empty
        match get_current_block().await {
            Ok(response) => response,
            Err(_error) => Json(BlockResponse {
                block_number: "error".to_string(),
                block_number_decimal: 0,
            }),
        }
    }
}

async fn get_current_block() -> Result<Json<BlockResponse>, String> {
    // Using Alchemy custom API key for optimal performance
    let rpc_url = "https://base-mainnet.g.alchemy.com/v2/MQ6e6fTn3VEz_P0yeS6518oalCmYIfCx";

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

async fn websocket_block_streamer(state: AppState) {
    loop {
        // WebSocket URL for Alchemy
        let ws_url = "wss://base-mainnet.g.alchemy.com/v2/MQ6e6fTn3VEz_P0yeS6518oalCmYIfCx";

        match Provider::<Ws>::connect(ws_url).await {
            Ok(provider) => {
                let provider = Arc::new(provider);
                println!("WebSocket connected to Alchemy");

                // Subscribe to new blocks
                match provider.subscribe_blocks().await {
                    Ok(mut stream) => {
                        println!("Subscribed to new blocks");

                        while let Some(block) = stream.next().await {
                            if let Some(number) = block.number {
                                let block_num = number.as_u64();
                                *state.latest_block.write().await = Some(block_num);
                                println!("Updated block number: {}", block_num);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to subscribe to blocks: {:?}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
            Err(e) => {
                println!("WebSocket connection failed: {:?}, retrying in 5 seconds", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize shared state for block number caching
    let latest_block = Arc::new(RwLock::new(None));
    let app_state = AppState {
        latest_block: latest_block.clone(),
    };

    // Start WebSocket block streamer in background
    let streamer_state = app_state.clone();
    tokio::spawn(async move {
        websocket_block_streamer(streamer_state).await;
    });

    // Build the application with routes
    let app = Router::new()
        .route("/api/marco", get(marco))
        .route("/api/getCurrentBlock", get({
            let state = app_state.clone();
            move || get_current_block_handler(axum::extract::State(state))
        }))
        .with_state(app_state);

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
