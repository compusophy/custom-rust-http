use crate::types::TransactionAlert;
use ethers::types::Address;
use redis::Client as RedisClient;
use std::env;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub latest_block: Arc<RwLock<Option<u64>>>,
    pub redis: Arc<RedisClient>,
    pub monitored_addresses: Arc<RwLock<Vec<Address>>>,
    pub alert_tx: broadcast::Sender<TransactionAlert>,
}

pub fn init_redis() -> Result<RedisClient, redis::RedisError> {
    // Get Redis URL from environment variable
    // Format: redis://:password@host:port or rediss://:password@host:port (TLS)
    // Railway Redis typically uses rediss:// (TLS)
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| {
        // Fallback: construct from individual env vars for Railway
        // Railway provides: hopper.proxy.rlwy.net:29794 with REDIS_PASSWORD
        let host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let password = env::var("REDIS_PASSWORD").ok();

        // Railway Redis typically requires TLS, so use rediss://
        let protocol = if env::var("REDIS_TLS").unwrap_or_else(|_| "true".to_string()) == "true" {
            "rediss"
        } else {
            "redis"
        };

        if let Some(pwd) = password {
            format!("{}://:{}@{}:{}", protocol, pwd, host, port)
        } else {
            format!("{}://{}:{}", protocol, host, port)
        }
    });

    // Mask password in logs
    let masked_url = redis_url.replace(env::var("REDIS_PASSWORD").unwrap_or_default().as_str(), "***");
    println!("[REDIS] Connecting to Redis at: {}", masked_url);

    let client = RedisClient::open(redis_url.as_str())?;
    println!("[REDIS] Redis client created successfully!");

    Ok(client)
}

