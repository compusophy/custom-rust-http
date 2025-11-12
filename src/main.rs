mod app;
mod handlers;
mod services;
mod state;
mod types;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use ethers::types::Address;
use tokio::sync::{broadcast, RwLock};
use crate::services::blockchain::websocket_block_streamer;
use crate::state::{init_redis, AppState};

#[tokio::main]
async fn main() {
    // Initialize Redis connection
    let redis_client = match init_redis() {
        Ok(client) => Arc::new(client),
        Err(e) => {
            eprintln!("[ERROR] Failed to initialize Redis: {}", e);
            eprintln!("[ERROR] Server will start without Redis support");
            panic!("Redis connection required");
        }
    };

    // Shared state
    let latest_block = Arc::new(RwLock::new(None));

    // Initialize monitored addresses (env: MONITORED_ADDRESSES)
    let default_monitored = env::var("MONITORED_ADDRESSES")
        .unwrap_or_else(|_| "0x09b55B6c70e3ff2fF79358d81f5DAfe5e5FbEBbc".to_string());
    let monitored_addresses: Vec<Address> = default_monitored
        .split(',')
        .filter_map(|addr| {
            let trimmed = addr.trim();
            match trimmed.parse::<Address>() {
                Ok(a) => {
                    println!("[MONITOR] Parsed address: {} -> {:?}", trimmed, a);
                    Some(a)
                }
                Err(e) => {
                    eprintln!("[MONITOR] Failed to parse address '{}': {:?}", trimmed, e);
                    None
                }
            }
        })
        .collect();
    println!("[MONITOR] Monitoring {} address(es):", monitored_addresses.len());
    for addr in &monitored_addresses {
        println!("  - {:?} (checksum: {})", addr, format!("{:?}", addr));
    }
    if monitored_addresses.is_empty() {
        eprintln!("[WARNING] No valid addresses to monitor! Check MONITORED_ADDRESSES env var.");
    }

    let (alert_tx, _) = broadcast::channel(1000);

    let app_state = AppState {
        latest_block: latest_block.clone(),
        redis: redis_client,
        monitored_addresses: Arc::new(RwLock::new(monitored_addresses)),
        alert_tx: alert_tx.clone(),
    };

    // Start WebSocket block streamer
    let streamer_state = app_state.clone();
    tokio::spawn(async move {
        websocket_block_streamer(streamer_state).await;
    });

    // Build app and serve
    let app = app::build_router(app_state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}