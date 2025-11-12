use axum::{routing::{get, post, put, delete}, Json, Router, response::Html, extract::Path, response::sse::{Event, Sse}, extract::State};
use ethers::providers::{Http, Provider, Ws, StreamExt};
use ethers::middleware::Middleware;
use ethers::types::Address;
use serde::{Serialize, Deserialize};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::convert::Infallible;
use tokio::sync::{RwLock, broadcast};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use redis::{Client as RedisClient, aio::ConnectionManager};

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
struct ApiEndpoint {
    path: String,
    method: String,
    description: String,
    example_request: Option<String>,
    example_response: String,
    performance: Option<String>,
}

#[derive(Serialize)]
struct ApiDocs {
    name: String,
    version: String,
    base_url: String,
    endpoints: Vec<ApiEndpoint>,
}

#[derive(Clone)]
struct AppState {
    latest_block: Arc<RwLock<Option<u64>>>,
    redis: Arc<RedisClient>,
    redis_manager: Arc<ConnectionManager>,
    monitored_addresses: Arc<RwLock<Vec<Address>>>,
    alert_tx: broadcast::Sender<TransactionAlert>,
}

fn init_redis() -> Result<RedisClient, redis::RedisError> {
    // Get Redis URL from environment variable
    // Format: redis://:password@host:port or rediss://:password@host:port (TLS)
    // Railway Redis typically uses rediss:// (TLS)
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| {
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
    let masked_url = redis_url.replace(
        env::var("REDIS_PASSWORD").unwrap_or_default().as_str(), 
        "***"
    );
    println!("[REDIS] Connecting to Redis at: {}", masked_url);
    
    let client = RedisClient::open(redis_url.as_str())?;
    println!("[REDIS] Redis client created successfully!");
    
    Ok(client)
}

async fn marco() -> Json<PoloResponse> {
    println!("[API] /api/marco endpoint called");
    Json(PoloResponse {
        message: "polo".to_string(),
    })
}

// Example endpoint that uses Redis for persistent storage
#[derive(Serialize)]
struct RedisTestResponse {
    key: String,
    value: String,
    message: String,
}

async fn redis_test(state: axum::extract::State<AppState>) -> Result<Json<RedisTestResponse>, String> {
    println!("[API] /api/redis-test endpoint called");
    
    let mut conn = state.redis.get_multiplexed_async_connection().await
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

// Database CRUD structures
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DbRecord {
    id: String,
    data: serde_json::Value,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateRecordRequest {
    data: serde_json::Value,
}

#[derive(Serialize)]
struct DbResponse {
    success: bool,
    record: Option<DbRecord>,
    message: String,
}

#[derive(Serialize)]
struct DbListResponse {
    success: bool,
    records: Vec<DbRecord>,
    count: usize,
}

// Monitor address structures
#[derive(Serialize, Deserialize, Debug)]
struct MonitorAddressRequest {
    address: String,
}

#[derive(Serialize)]
struct MonitorAddressResponse {
    success: bool,
    message: String,
    monitored_addresses: Vec<String>,
}

#[derive(Serialize)]
struct MonitorListResponse {
    success: bool,
    addresses: Vec<String>,
    count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct TransactionAlert {
    block_number: u64,
    block_hash: String,
    address: String,
    role: String,
    tx_hash: String,
    from: String,
    to: String,
    value: String,
    gas: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    timestamp: u64,
}

// Helper function to deserialize timestamp from either u64 or hex string
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct TimestampVisitor;

    impl<'de> Visitor<'de> for TimestampVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u64 number or a hex string")
        }

        fn visit_u64<E>(self, value: u64) -> Result<u64, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_str<E>(self, value: &str) -> Result<u64, E>
        where
            E: de::Error,
        {
            // Try to parse as hex string (e.g., "0x69134139")
            if value.starts_with("0x") || value.starts_with("0X") {
                u64::from_str_radix(&value[2..], 16)
                    .map_err(|_| E::custom(format!("invalid hex timestamp: {}", value)))
            } else {
                // Try to parse as decimal string
                value.parse::<u64>()
                    .map_err(|_| E::custom(format!("invalid timestamp: {}", value)))
            }
        }
    }

    deserializer.deserialize_any(TimestampVisitor)
}

#[derive(Serialize)]
struct TransactionAlertsResponse {
    success: bool,
    alerts: Vec<TransactionAlert>,
    count: usize,
}

// CREATE - POST /api/db
async fn db_create(
    state: axum::extract::State<AppState>,
    Json(payload): Json<CreateRecordRequest>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] POST /api/db - Creating record");
    
    let mut conn = state.redis.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;
    
    // Generate ID
    let id = format!("db:{}", uuid::Uuid::new_v4().to_string());
    let now = chrono::Utc::now().to_rfc3339();
    
    let record = DbRecord {
        id: id.clone(),
        data: payload.data,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };
    
    let record_json = serde_json::to_string(&record)
        .map_err(|e| format!("Failed to serialize record: {}", e))?;
    
    redis::cmd("SET")
        .arg(&id)
        .arg(&record_json)
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| format!("Failed to SET record: {}", e))?;
    
    // Add to index set
    redis::cmd("SADD")
        .arg("db:index")
        .arg(&id)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to add to index: {}", e))?;
    
    Ok(Json(DbResponse {
        success: true,
        record: Some(record),
        message: "Record created successfully".to_string(),
    }))
}

// READ - GET /api/db/:id
async fn db_read(
    state: axum::extract::State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] GET /api/db/{} - Reading record", id);
    
    let mut conn = state.redis.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;
    
    // Handle both "db:uuid" format and just "uuid" format
    let key = if id.starts_with("db:") {
        id.clone()
    } else {
        format!("db:{}", id)
    };
    
    let record_json: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to GET record: {}", e))?;
    
    match record_json {
        Some(json) => {
            let mut record: DbRecord = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse record: {}", e))?;
            // Ensure ID matches the requested format
            record.id = key.clone();
            Ok(Json(DbResponse {
                success: true,
                record: Some(record),
                message: "Record found".to_string(),
            }))
        }
        None => Ok(Json(DbResponse {
            success: false,
            record: None,
            message: "Record not found".to_string(),
        })),
    }
}

// UPDATE - PUT /api/db/:id
async fn db_update(
    state: axum::extract::State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateRecordRequest>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] PUT /api/db/{} - Updating record", id);
    
    let mut conn = state.redis.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;
    
    // Handle both "db:uuid" format and just "uuid" format
    let key = if id.starts_with("db:") {
        id.clone()
    } else {
        format!("db:{}", id)
    };
    
    // Check if record exists
    let existing_json: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to GET record: {}", e))?;
    
    let (created_at, updated_at) = match existing_json {
        Some(json) => {
            let existing: DbRecord = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse existing record: {}", e))?;
            (existing.created_at, Some(chrono::Utc::now().to_rfc3339()))
        }
        None => (Some(chrono::Utc::now().to_rfc3339()), Some(chrono::Utc::now().to_rfc3339())),
    };
    
    let record = DbRecord {
        id: id.clone(),
        data: payload.data,
        created_at,
        updated_at,
    };
    
    let record_json = serde_json::to_string(&record)
        .map_err(|e| format!("Failed to serialize record: {}", e))?;
    
    redis::cmd("SET")
        .arg(&key)
        .arg(&record_json)
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| format!("Failed to SET record: {}", e))?;
    
    // Ensure in index
    redis::cmd("SADD")
        .arg("db:index")
        .arg(&key)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to add to index: {}", e))?;
    
    Ok(Json(DbResponse {
        success: true,
        record: Some(record),
        message: "Record updated successfully".to_string(),
    }))
}

// DELETE - DELETE /api/db/:id
async fn db_delete(
    state: axum::extract::State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DbResponse>, String> {
    println!("[API] DELETE /api/db/{} - Deleting record", id);
    
    let mut conn = state.redis.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;
    
    // Handle both "db:uuid" format and just "uuid" format
    let key = if id.starts_with("db:") {
        id.clone()
    } else {
        format!("db:{}", id)
    };
    
    // Check if exists
    let exists: bool = redis::cmd("EXISTS")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to check existence: {}", e))?;
    
    if !exists {
        return Ok(Json(DbResponse {
            success: false,
            record: None,
            message: "Record not found".to_string(),
        }));
    }
    
    redis::cmd("DEL")
        .arg(&key)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to DELETE record: {}", e))?;
    
    // Remove from index
    redis::cmd("SREM")
        .arg("db:index")
        .arg(&key)
        .query_async::<i64>(&mut conn)
        .await
        .map_err(|e| format!("Failed to remove from index: {}", e))?;
    
    Ok(Json(DbResponse {
        success: true,
        record: None,
        message: "Record deleted successfully".to_string(),
    }))
}

// LIST - GET /api/db
async fn db_list(state: axum::extract::State<AppState>) -> Result<Json<DbListResponse>, String> {
    println!("[API] GET /api/db - Listing all records");
    
    let mut conn = state.redis.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;
    
    // Get all IDs from index
    let ids: Vec<String> = redis::cmd("SMEMBERS")
        .arg("db:index")
        .query_async(&mut conn)
        .await
        .map_err(|e| format!("Failed to get index: {}", e))?;
    
    let mut records = Vec::new();
    
    for id in ids {
        let record_json: Option<String> = redis::cmd("GET")
            .arg(&id)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Failed to GET record {}: {}", id, e))?;
        
        if let Some(json) = record_json {
            if let Ok(record) = serde_json::from_str::<DbRecord>(&json) {
                records.push(record);
            }
        }
    }
    
    let count = records.len();
    Ok(Json(DbListResponse {
        success: true,
        records,
        count,
    }))
}

// Add address to monitor - POST /api/monitor/add
async fn monitor_add(
    state: axum::extract::State<AppState>,
    Json(payload): Json<MonitorAddressRequest>,
) -> Result<Json<MonitorAddressResponse>, String> {
    println!("[API] POST /api/monitor/add - Adding address: {}", payload.address);
    
    let address: Address = payload.address.parse()
        .map_err(|_| "Invalid address format".to_string())?;
    
    let mut monitored = state.monitored_addresses.write().await;
    
    if monitored.contains(&address) {
        return Ok(Json(MonitorAddressResponse {
            success: false,
            message: "Address already being monitored".to_string(),
            monitored_addresses: monitored.iter().map(|a| format!("{:?}", a)).collect(),
        }));
    }
    
    monitored.push(address);
    let addresses: Vec<String> = monitored.iter().map(|a| format!("{:?}", a)).collect();
    
    println!("[MONITOR] Now monitoring {} address(es)", monitored.len());
    
    Ok(Json(MonitorAddressResponse {
        success: true,
        message: "Address added to monitoring list".to_string(),
        monitored_addresses: addresses,
    }))
}

// Remove address from monitor - DELETE /api/monitor/:address
async fn monitor_remove(
    state: axum::extract::State<AppState>,
    Path(address_str): Path<String>,
) -> Result<Json<MonitorAddressResponse>, String> {
    println!("[API] DELETE /api/monitor/{} - Removing address", address_str);
    
    let address: Address = address_str.parse()
        .map_err(|_| "Invalid address format".to_string())?;
    
    let mut monitored = state.monitored_addresses.write().await;
    
    let initial_len = monitored.len();
    monitored.retain(|&a| a != address);
    
    if monitored.len() == initial_len {
        return Ok(Json(MonitorAddressResponse {
            success: false,
            message: "Address not found in monitoring list".to_string(),
            monitored_addresses: monitored.iter().map(|a| format!("{:?}", a)).collect(),
        }));
    }
    
    let addresses: Vec<String> = monitored.iter().map(|a| format!("{:?}", a)).collect();
    
    println!("[MONITOR] Now monitoring {} address(es)", monitored.len());
    
    Ok(Json(MonitorAddressResponse {
        success: true,
        message: "Address removed from monitoring list".to_string(),
        monitored_addresses: addresses,
    }))
}

// List monitored addresses - GET /api/monitor
async fn monitor_list(
    state: axum::extract::State<AppState>,
) -> Json<MonitorListResponse> {
    let monitored = state.monitored_addresses.read().await;
    let addresses: Vec<String> = monitored.iter().map(|a| format!("{:?}", a)).collect();
    
    Json(MonitorListResponse {
        success: true,
        addresses: addresses.clone(),
        count: addresses.len(),
    })
}

// Get transaction alerts - GET /api/monitor/alerts (for initial load)
async fn monitor_alerts(
    state: axum::extract::State<AppState>,
) -> Json<TransactionAlertsResponse> {
    println!("[API] GET /api/monitor/alerts - Fetching alerts");
    
    let mut alerts = Vec::new();
    
    match state.redis.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            // Get list of alert keys (last 100)
            match redis::cmd("LRANGE")
                .arg("tx_alerts:list")
                .arg(0)
                .arg(99)
                .query_async::<Vec<String>>(&mut conn)
                .await
            {
                Ok(alert_keys) => {
                    // Fetch each alert
                    for key in alert_keys {
                        match redis::cmd("GET")
                            .arg(&key)
                            .query_async::<Option<String>>(&mut conn)
                            .await
                        {
                            Ok(Some(json_str)) => {
                                match serde_json::from_str::<TransactionAlert>(&json_str) {
                                    Ok(alert) => alerts.push(alert),
                                    Err(e) => {
                                        eprintln!("[API] Failed to parse alert {}: {}", key, e);
                                    }
                                }
                            }
                            Ok(None) => {
                                // Key expired or doesn't exist, skip it
                            }
                            Err(e) => {
                                eprintln!("[API] Failed to fetch alert {}: {}", key, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[API] Failed to fetch alert keys: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[API] Failed to get Redis connection: {}", e);
        }
    }
    
    // Sort by block number descending (newest first)
    alerts.sort_by(|a, b| b.block_number.cmp(&a.block_number));
    
    Json(TransactionAlertsResponse {
        success: true,
        alerts: alerts.clone(),
        count: alerts.len(),
    })
}

// SSE endpoint for real-time transaction alerts - GET /api/monitor/alerts/stream
async fn monitor_alerts_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    println!("[SSE] Client connected to /api/monitor/alerts/stream");
    
    let rx = state.alert_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| async move {
            match result {
                Ok(alert) => {
                    match serde_json::to_string(&alert) {
                        Ok(json) => Some(Ok(Event::default().data(json))),
                        Err(e) => {
                            eprintln!("[SSE] Failed to serialize alert: {}", e);
                            None
                        }
                    }
                }
                Err(BroadcastStreamRecvError::Lagged(skipped)) => {
                    eprintln!("[SSE] Client lagged, skipped {} messages", skipped);
                    None
                }
            }
        });
    
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive-text".to_string())
    )
}

async fn api_docs_json() -> Json<ApiDocs> {
    Json(ApiDocs {
        name: "RUSTful".to_string(),
        version: "1.0.0".to_string(),
        base_url: "https://custom-rust-http-production.up.railway.app".to_string(),
        endpoints: vec![
            ApiEndpoint {
                path: "/api/marco".to_string(),
                method: "GET".to_string(),
                description: "Simple test endpoint that returns 'polo' in response to 'marco'".to_string(),
                example_request: None,
                example_response: r#"{"message":"polo"}"#.to_string(),
                performance: Some("~30-50ms".to_string()),
            },
            ApiEndpoint {
                path: "/api/getCurrentBlock".to_string(),
                method: "GET".to_string(),
                description: "Returns the current Base mainnet block number.".to_string(),
                example_request: None,
                example_response: r#"{"block_number":"0x241337e","block_number_decimal":37827454}"#.to_string(),
                performance: None,
            },
            ApiEndpoint {
                path: "/api/db".to_string(),
                method: "POST".to_string(),
                description: "Create a new database record. Stores JSON data in Redis for maximum speed.".to_string(),
                example_request: Some(r#"{"data":{"name":"John","age":30}}"#.to_string()),
                example_response: r#"{"success":true,"record":{"id":"db:uuid","data":{"name":"John","age":30},"created_at":"2025-11-11T12:00:00Z","updated_at":"2025-11-11T12:00:00Z"},"message":"Record created successfully"}"#.to_string(),
                performance: Some("~5-15ms".to_string()),
            },
            ApiEndpoint {
                path: "/api/db".to_string(),
                method: "GET".to_string(),
                description: "List all database records. Returns all stored records with metadata.".to_string(),
                example_request: None,
                example_response: r#"{"success":true,"records":[{"id":"db:uuid","data":{...},"created_at":"...","updated_at":"..."}],"count":1}"#.to_string(),
                performance: Some("~10-30ms".to_string()),
            },
            ApiEndpoint {
                path: "/api/db/:id".to_string(),
                method: "GET".to_string(),
                description: "Read a specific database record by ID.".to_string(),
                example_request: None,
                example_response: r#"{"success":true,"record":{"id":"db:uuid","data":{...},"created_at":"...","updated_at":"..."},"message":"Record found"}"#.to_string(),
                performance: Some("~5-10ms".to_string()),
            },
            ApiEndpoint {
                path: "/api/db/:id".to_string(),
                method: "PUT".to_string(),
                description: "Update an existing database record by ID. Creates record if it doesn't exist.".to_string(),
                example_request: Some(r#"{"data":{"name":"Jane","age":25}}"#.to_string()),
                example_response: r#"{"success":true,"record":{"id":"db:uuid","data":{"name":"Jane","age":25},"created_at":"...","updated_at":"..."},"message":"Record updated successfully"}"#.to_string(),
                performance: Some("~5-15ms".to_string()),
            },
            ApiEndpoint {
                path: "/api/db/:id".to_string(),
                method: "DELETE".to_string(),
                description: "Delete a database record by ID.".to_string(),
                example_request: None,
                example_response: r#"{"success":true,"record":null,"message":"Record deleted successfully"}"#.to_string(),
                performance: Some("~5-10ms".to_string()),
            },
            ApiEndpoint {
                path: "/api/docs".to_string(),
                method: "GET".to_string(),
                description: "Returns API documentation as JSON listing all available endpoints".to_string(),
                example_request: None,
                example_response: r#"{"name":"Rust HTTP Blockchain API","version":"1.0.0",...}"#.to_string(),
                performance: Some("~10-20ms".to_string()),
            },
            ApiEndpoint {
                path: "/docs".to_string(),
                method: "GET".to_string(),
                description: "Returns API documentation as HTML page".to_string(),
                example_request: None,
                example_response: "HTML documentation page".to_string(),
                performance: Some("~10-20ms".to_string()),
            },
        ],
    })
}

async fn api_docs_html() -> Html<String> {
    let docs = api_docs_json().await.0;
    
    // Categorize endpoints
    let mut blockchain_endpoints = Vec::new();
    let mut database_endpoints = Vec::new();
    let mut utility_endpoints = Vec::new();
    let mut docs_endpoints = Vec::new();
    
    for endpoint in &docs.endpoints {
        let category = if endpoint.path.contains("Block") || endpoint.path.contains("block") {
            "blockchain"
        } else if endpoint.path.contains("/db") || endpoint.path.contains("/api/db") {
            "database"
        } else if endpoint.path.contains("docs") || endpoint.path.contains("Docs") {
            "documentation"
        } else {
            "utility"
        };
        
        match category {
            "blockchain" => blockchain_endpoints.push(endpoint),
            "database" => database_endpoints.push(endpoint),
            "documentation" => docs_endpoints.push(endpoint),
            _ => utility_endpoints.push(endpoint),
        }
    }
    
    // Generate sidebar navigation
    let mut sidebar_items = String::new();
    if !blockchain_endpoints.is_empty() {
        sidebar_items.push_str(r#"<div class="nav-category">Blockchain</div>"#);
        for endpoint in &blockchain_endpoints {
            let id = endpoint.path.replace("/", "-").replace("api-", "");
            sidebar_items.push_str(&format!("<a href=\"#{}\" class=\"nav-item\">{}</a>", id, endpoint.path));
        }
    }
    if !database_endpoints.is_empty() {
        sidebar_items.push_str(r#"<div class="nav-category">Database</div>"#);
        for endpoint in &database_endpoints {
            let id = format!("{}-{}", endpoint.method.to_lowercase(), endpoint.path.replace("/", "-").replace("api-", ""));
            sidebar_items.push_str(&format!("<a href=\"#{}\" class=\"nav-item\">{} {}</a>", id, endpoint.method, endpoint.path));
        }
    }
    if !utility_endpoints.is_empty() {
        sidebar_items.push_str(r#"<div class="nav-category">Utility</div>"#);
        for endpoint in &utility_endpoints {
            let id = endpoint.path.replace("/", "-").replace("api-", "");
            sidebar_items.push_str(&format!("<a href=\"#{}\" class=\"nav-item\">{}</a>", id, endpoint.path));
        }
    }
    if !docs_endpoints.is_empty() {
        sidebar_items.push_str(r#"<div class="nav-category">Documentation</div>"#);
        for endpoint in &docs_endpoints {
            let id = endpoint.path.replace("/", "-").replace("api-", "");
            sidebar_items.push_str(&format!("<a href=\"#{}\" class=\"nav-item\">{}</a>", id, endpoint.path));
        }
    }
    
    // Generate endpoint cards with IDs
    let mut endpoints_html = String::new();
    
    if !blockchain_endpoints.is_empty() {
        endpoints_html.push_str(r#"<h2 class="section-title" id="blockchain">Blockchain</h2>"#);
        for endpoint in &blockchain_endpoints {
            let id = endpoint.path.replace("/", "-").replace("api-", "");
            endpoints_html.push_str(&format_endpoint_card(endpoint, &id));
        }
    }
    
    if !database_endpoints.is_empty() {
        endpoints_html.push_str(r#"<h2 class="section-title" id="database">Database</h2>"#);
        for endpoint in &database_endpoints {
            let id = format!("{}-{}", endpoint.method.to_lowercase(), endpoint.path.replace("/", "-").replace("api-", "").replace(":", ""));
            endpoints_html.push_str(&format_endpoint_card(endpoint, &id));
        }
    }
    
    if !utility_endpoints.is_empty() {
        endpoints_html.push_str(r#"<h2 class="section-title" id="utility">Utility</h2>"#);
        for endpoint in &utility_endpoints {
            let id = endpoint.path.replace("/", "-").replace("api-", "");
            endpoints_html.push_str(&format_endpoint_card(endpoint, &id));
        }
    }
    
    if !docs_endpoints.is_empty() {
        endpoints_html.push_str(r#"<h2 class="section-title" id="documentation">Documentation</h2>"#);
        for endpoint in &docs_endpoints {
            let id = endpoint.path.replace("/", "-").replace("api-", "");
            endpoints_html.push_str(&format_endpoint_card(endpoint, &id));
        }
    }
    
    fn format_endpoint_card(endpoint: &ApiEndpoint, id: &str) -> String {
        let method_badge = match endpoint.method.as_str() {
            "GET" => r#"<span class="method-badge get">GET</span>"#,
            "POST" => r#"<span class="method-badge post">POST</span>"#,
            "PUT" => r#"<span class="method-badge put">PUT</span>"#,
            "DELETE" => r#"<span class="method-badge delete">DELETE</span>"#,
            _ => r#"<span class="method-badge">METHOD</span>"#,
        };
        
        let performance_html = endpoint.performance.as_ref()
            .map(|p| format!(r#"<div class="performance">⚡ {}</div>"#, p))
            .unwrap_or_default();
        
        let request_html = endpoint.example_request.as_ref()
            .map(|r| format!(r#"
                <div class="example">
                    <strong>Example Request:</strong>
                    <pre><code>{}</code></pre>
                </div>
            "#, r))
            .unwrap_or_default();
        
        format!(r#"
            <div class="endpoint-card" id="{}">
                <div class="endpoint-header">
                    {} <code class="path">{}</code>
                </div>
                <p class="description">{}</p>
                {}
                <div class="example">
                    <strong>Example Response:</strong>
                    <pre><code>{}</code></pre>
                </div>
                {}
            </div>
        "#, id, method_badge, endpoint.path, endpoint.description, request_html, endpoint.example_response, performance_html)
    }
    
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
    <title>RUSTful - API Documentation</title>
    <style>
        /* ============================================
           BRUTALIST MONOCHROME DESIGN SYSTEM
           Minimalist | Monospace | Geometric | Stark
           ============================================ */
        
        /* RESET & BASE */
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        /* TYPOGRAPHY - IBM PLEX MONO - WORLD CLASS READABILITY */
        body, html {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
            font-size: 15px;
            line-height: 1.7;
            letter-spacing: 0.3px;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            font-weight: 400;
        }}
        
        /* COLOR PALETTE - DARK MODE WITH GLOWY GREEN */
        :root {{
            --black: #000000;
            --dark: #0A0A0A;
            --dark-gray: #1A1A1A;
            --medium-gray: #2A2A2A;
            --light-gray: #3A3A3A;
            --white: #FFFFFF;
            --green-primary: #00D977;      /* Darker, more opaque green */
            --green-secondary: #00B366;    /* Even darker for secondary text */
            --green-accent: #00FF88;       /* Brighter for accents only */
            --green-glow: rgba(0, 217, 119, 0.5);  /* Glow effect */
        }}
        
        /* BASE LAYOUT - DARK MODE */
        body {{
            background: var(--black);
            color: var(--green-primary);
            min-height: 100vh;
        }}
        
        /* CONTAINER - BRUTALIST GEOMETRY */
        .container {{
            display: flex;
            max-width: 1600px;
            margin: 0 auto;
            background: var(--black);
            min-height: 100vh;
            border-left: 4px solid var(--green-primary);
            border-right: 4px solid var(--green-primary);
            box-shadow: 0 0 20px var(--green-glow);
        }}
        
        /* SIDEBAR - STARK GEOMETRY */
        .sidebar {{
            width: 300px;
            background: var(--dark);
            border-right: 4px solid var(--green-primary);
            padding: 0;
            position: sticky;
            top: 0;
            height: 100vh;
            overflow-y: auto;
        }}
        
        .sidebar-header {{
            padding: 24px;
            border-bottom: 4px solid var(--green-primary);
            background: var(--black);
            color: var(--green-primary);
        }}
        
        .sidebar-header h2 {{
            font-size: 16px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            margin-bottom: 8px;
            text-shadow: 0 0 10px var(--green-glow);
        }}
        
        .sidebar-header .version {{
            font-size: 12px;
            color: var(--green-secondary);
            letter-spacing: 1px;
        }}
        
        /* NAVIGATION - BRUTALIST LINKS */
        .nav-category {{
            font-size: 11px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--green-primary);
            padding: 16px 24px 8px;
            background: var(--dark);
            border-bottom: 2px solid var(--green-primary);
            text-shadow: 0 0 8px var(--green-glow);
        }}
        
        .nav-item {{
            display: block;
            padding: 12px 24px;
            color: var(--green-secondary);
            text-decoration: none;
            font-size: 13px;
            border-left: 4px solid transparent;
            background: var(--dark);
            border-bottom: 1px solid var(--dark-gray);
            transition: all 0.2s ease;
            line-height: 1.6;
        }}
        
        .nav-item:hover {{
            background: var(--black);
            color: var(--green-primary);
            border-left-color: var(--green-primary);
            text-shadow: 0 0 8px var(--green-glow);
        }}
        
        /* NAVIGATION BAR */
        .top-nav {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 20px 32px;
            border-bottom: 4px solid var(--green-primary);
            background: var(--black);
        }}
        
        .top-nav-brand {{
            font-size: 20px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 3px;
            color: var(--green-primary);
            text-shadow: 0 0 15px var(--green-glow);
            text-decoration: none;
        }}
        
        .top-nav-link {{
            font-size: 13px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--green-secondary);
            text-decoration: none;
            padding: 6px 14px;
            border: 2px solid var(--green-primary);
            transition: all 0.2s ease;
        }}
        
        .top-nav-link:hover {{
            background: var(--green-primary);
            color: var(--black);
            text-shadow: none;
        }}
        
        /* MAIN CONTENT */
        .main-content {{
            flex: 1;
            padding: 0;
            overflow-y: auto;
            background: var(--black);
        }}
        
        /* HEADER - BRUTALIST BANNER */
        .header {{
            background: var(--black);
            color: var(--green-primary);
            padding: 40px 32px;
            text-align: left;
            border-bottom: 4px solid var(--green-primary);
        }}
        
        .header h1 {{
            font-size: 28px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 3px;
            margin-bottom: 12px;
            text-shadow: 0 0 15px var(--green-glow);
            line-height: 1.3;
        }}
        
        .header .version {{
            font-size: 13px;
            color: var(--green-secondary);
            letter-spacing: 2px;
        }}
        
        /* SECTIONS */
        .section-title {{
            font-size: 20px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--green-primary);
            margin: 0;
            padding: 28px 32px;
            border-bottom: 4px solid var(--green-primary);
            background: var(--dark);
            text-shadow: 0 0 10px var(--green-glow);
        }}
        
        /* BASE URL BOX */
        .base-url {{
            background: var(--dark);
            color: var(--green-primary);
            padding: 20px 32px;
            border-bottom: 4px solid var(--green-primary);
        }}
        
        .base-url code {{
            font-size: 14px;
            letter-spacing: 1px;
            color: var(--green-primary);
            text-shadow: 0 0 8px var(--green-glow);
        }}
        
        /* ENDPOINT CARDS - BRUTALIST BOXES */
        .endpoint-card {{
            border: 4px solid var(--green-primary);
            padding: 32px;
            margin: 0;
            background: var(--black);
            border-top: none;
        }}
        
        .endpoint-card:last-child {{
            border-bottom: 4px solid var(--green-primary);
        }}
        
        .endpoint-header {{
            display: flex;
            align-items: center;
            gap: 16px;
            margin-bottom: 20px;
        }}
        
        /* METHOD BADGES - GLOWY STYLE */
        .method-badge {{
            padding: 6px 14px;
            font-weight: 600;
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 1.5px;
            border: 2px solid var(--green-primary);
            background: var(--black);
            color: var(--green-primary);
            text-shadow: 0 0 8px var(--green-glow);
        }}
        
        .method-badge.get {{
            background: var(--green-primary);
            color: var(--black);
            text-shadow: none;
        }}
        
        .method-badge.post {{
            background: var(--black);
            color: var(--green-primary);
            border: 2px solid var(--green-primary);
        }}
        
        .method-badge.put {{
            background: var(--dark-gray);
            color: var(--green-primary);
        }}
        
        .method-badge.delete {{
            background: var(--green-secondary);
            color: var(--black);
            text-shadow: none;
        }}
        
        /* PATH - MONOSPACE CODE */
        .path {{
            background: var(--dark);
            padding: 6px 12px;
            font-size: 14px;
            color: var(--green-primary);
            border: 2px solid var(--green-primary);
            letter-spacing: 0.5px;
            text-shadow: 0 0 6px var(--green-glow);
        }}
        
        /* DESCRIPTION - IMPROVED READABILITY */
        .description {{
            color: var(--green-secondary);
            margin-bottom: 20px;
            line-height: 1.8;
            font-size: 14px;
            max-width: 800px;
        }}
        
        /* EXAMPLE BOXES - IMPROVED READABILITY */
        .example {{
            background: var(--dark);
            border: 4px solid var(--green-primary);
            padding: 20px;
            margin-top: 20px;
        }}
        
        .example strong {{
            display: block;
            margin-bottom: 16px;
            color: var(--green-primary);
            font-size: 12px;
            text-transform: uppercase;
            letter-spacing: 1.5px;
            font-weight: 600;
            text-shadow: 0 0 6px var(--green-glow);
        }}
        
        .example pre {{
            background: var(--black);
            color: var(--green-primary);
            padding: 20px;
            border: 2px solid var(--green-primary);
            overflow-x: auto;
            font-size: 13px;
            line-height: 1.7;
            text-shadow: 0 0 8px var(--green-glow);
        }}
        
        .example code {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
        }}
        
        /* PERFORMANCE METRIC */
        .performance {{
            margin-top: 20px;
            color: var(--green-accent);
            font-weight: 600;
            font-size: 12px;
            text-transform: uppercase;
            letter-spacing: 1.5px;
            text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
        }}
        
        /* UTILITY CLASSES - IMPROVED READABILITY */
        h1, h2, h3, h4, h5, h6 {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            line-height: 1.4;
        }}
        
        code {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
            font-size: 0.95em;
        }}
        
        /* SCROLLBAR - GLOWY STYLE */
        ::-webkit-scrollbar {{
            width: 14px;
        }}
        
        ::-webkit-scrollbar-track {{
            background: var(--black);
            border-left: 2px solid var(--green-primary);
        }}
        
        ::-webkit-scrollbar-thumb {{
            background: var(--green-primary);
            border: 2px solid var(--black);
            box-shadow: 0 0 10px var(--green-glow);
        }}
        
        ::-webkit-scrollbar-thumb:hover {{
            background: var(--green-accent);
            box-shadow: 0 0 15px var(--green-glow);
        }}
        
        /* SELECTION - GLOWY */
        ::selection {{
            background: var(--green-primary);
            color: var(--black);
        }}
        
        ::-moz-selection {{
            background: var(--green-primary);
            color: var(--black);
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="sidebar">
            <div class="sidebar-header">
                <h2>{}</h2>
            </div>
            <nav>
                {}
            </nav>
        </div>
        <div class="main-content">
            <nav class="top-nav">
                <a href="/marketing" class="top-nav-brand">RUSTful</a>
                <a href="/marketing" class="top-nav-link">Home</a>
            </nav>
            <div class="header">
                <h1>{}</h1>
            </div>
            <div class="base-url">
                Base URL: <code>{}</code>
            </div>
            {}
        </div>
    </div>
</body>
</html>
    "#, docs.name, sidebar_items, docs.name, docs.base_url, endpoints_html);
    
    Html(html)
}

async fn marketing_page() -> Html<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
    <title>RUSTful - Ultra-Fast Blockchain API</title>
    <style>
        /* ============================================
           BRUTALIST DARK MODE DESIGN SYSTEM
           Minimalist | Monospace | Geometric | Stark
           ============================================ */
        
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body, html {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
            font-size: 15px;
            line-height: 1.7;
            letter-spacing: 0.3px;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            font-weight: 400;
            background: var(--black);
            color: var(--green-primary);
            min-height: 100vh;
        }}
        
        :root {{
            --black: #000000;
            --dark: #0A0A0A;
            --dark-gray: #1A1A1A;
            --green-primary: #00D977;
            --green-secondary: #00B366;
            --green-accent: #00FF88;
            --green-glow: rgba(0, 217, 119, 0.5);
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            border-left: 4px solid var(--green-primary);
            border-right: 4px solid var(--green-primary);
            box-shadow: 0 0 20px var(--green-glow);
        }}
        
        /* NAVIGATION */
        .nav {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 24px 32px;
            border-bottom: 4px solid var(--green-primary);
            background: var(--black);
        }}
        
        .nav-brand {{
            font-size: 24px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 3px;
            color: var(--green-primary);
            text-shadow: 0 0 15px var(--green-glow);
            text-decoration: none;
        }}
        
        .nav-link {{
            font-size: 14px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--green-secondary);
            text-decoration: none;
            padding: 8px 16px;
            border: 2px solid var(--green-primary);
            transition: all 0.2s ease;
        }}
        
        .nav-link:hover {{
            background: var(--green-primary);
            color: var(--black);
            text-shadow: none;
        }}
        
        /* HERO SECTION */
        .hero {{
            flex: 1;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            padding: 80px 32px;
            text-align: center;
            background: var(--black);
        }}
        
        .hero h1 {{
            font-size: 64px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 4px;
            margin-bottom: 24px;
            color: var(--green-primary);
            text-shadow: 0 0 20px var(--green-glow);
            line-height: 1.2;
        }}
        
        .hero-tagline {{
            font-size: 20px;
            color: var(--green-secondary);
            margin-bottom: 32px;
            max-width: 600px;
            line-height: 1.8;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        
        .hero-bullets {{
            list-style: none;
            margin-bottom: 48px;
            max-width: 600px;
        }}
        
        .hero-bullets li {{
            font-size: 16px;
            color: var(--green-secondary);
            margin-bottom: 16px;
            padding-left: 24px;
            position: relative;
            line-height: 1.8;
        }}
        
        .hero-bullets li::before {{
            content: "-";
            position: absolute;
            left: 0;
            color: var(--green-primary);
            font-weight: 700;
            font-size: 20px;
        }}
        
        .cta-button {{
            padding: 20px 48px;
            font-size: 18px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            border: 4px solid var(--green-primary);
            background: var(--black);
            color: var(--green-primary);
            text-decoration: none;
            text-shadow: 0 0 10px var(--green-glow);
            transition: all 0.2s ease;
            cursor: pointer;
            display: inline-block;
        }}
        
        .cta-button:hover {{
            background: var(--green-primary);
            color: var(--black);
            text-shadow: none;
            box-shadow: 0 0 30px var(--green-glow);
        }}
        
        ::selection {{
            background: var(--green-primary);
            color: var(--black);
        }}
        
        ::-moz-selection {{
            background: var(--green-primary);
            color: var(--black);
        }}
    </style>
</head>
<body>
    <div class="container">
        <nav class="nav">
            <a href="/marketing" class="nav-brand">RUSTful</a>
            <a href="/docs" class="nav-link">Docs</a>
        </nav>
        <div class="hero">
            <h1>RUSTful</h1>
            <p class="hero-tagline">Ultra-fast HTTP built with Rust</p>
            <a href="/app" class="cta-button">Launch App</a>
        </div>
    </div>
</body>
</html>
    "#);
    
    Html(html)
}

async fn app_page() -> Html<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
    <title>RUSTful - Address Monitor</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body, html {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
            font-size: 15px;
            line-height: 1.7;
            letter-spacing: 0.3px;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            font-weight: 400;
            background: var(--black);
            color: var(--green-primary);
            min-height: 100vh;
        }}
        
        :root {{
            --black: #000000;
            --dark: #0A0A0A;
            --dark-gray: #1A1A1A;
            --green-primary: #00D977;
            --green-secondary: #00B366;
            --green-accent: #00FF88;
            --green-glow: rgba(0, 217, 119, 0.5);
        }}
        
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            border-left: 4px solid var(--green-primary);
            border-right: 4px solid var(--green-primary);
            box-shadow: 0 0 20px var(--green-glow);
        }}
        
        .nav {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 24px 32px;
            border-bottom: 4px solid var(--green-primary);
            background: var(--black);
        }}
        
        .nav-brand {{
            font-size: 24px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 3px;
            color: var(--green-primary);
            text-shadow: 0 0 15px var(--green-glow);
            text-decoration: none;
        }}
        
        .nav-link {{
            font-size: 14px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--green-secondary);
            text-decoration: none;
            padding: 8px 16px;
            border: 2px solid var(--green-primary);
            transition: all 0.2s ease;
        }}
        
        .nav-link:hover {{
            background: var(--green-primary);
            color: var(--black);
            text-shadow: none;
        }}
        
        .app-content {{
            flex: 1;
            padding: 40px 32px;
            display: flex;
            flex-direction: column;
            gap: 32px;
        }}
        
        .section {{
            border: 4px solid var(--green-primary);
            background: var(--dark);
            padding: 24px;
        }}
        
        .section-title {{
            font-size: 20px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 2px;
            margin-bottom: 20px;
            color: var(--green-primary);
            text-shadow: 0 0 10px var(--green-glow);
        }}
        
        .form-group {{
            display: flex;
            gap: 12px;
            margin-bottom: 20px;
        }}
        
        .input {{
            flex: 1;
            padding: 12px 16px;
            border: 2px solid var(--green-primary);
            background: var(--black);
            color: var(--green-primary);
            font-family: 'IBM Plex Mono', monospace;
            font-size: 14px;
            outline: none;
        }}
        
        .input:focus {{
            box-shadow: 0 0 10px var(--green-glow);
        }}
        
        .button {{
            padding: 12px 24px;
            border: 2px solid var(--green-primary);
            background: var(--black);
            color: var(--green-primary);
            font-family: 'IBM Plex Mono', monospace;
            font-size: 14px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
            cursor: pointer;
            transition: all 0.2s ease;
        }}
        
        .button:hover {{
            background: var(--green-primary);
            color: var(--black);
            box-shadow: 0 0 15px var(--green-glow);
        }}
        
        .button:active {{
            transform: scale(0.98);
        }}
        
        .button-danger {{
            border-color: var(--green-secondary);
            color: var(--green-secondary);
        }}
        
        .button-danger:hover {{
            background: var(--green-secondary);
            color: var(--black);
        }}
        
        .address-list {{
            display: flex;
            flex-direction: column;
            gap: 12px;
        }}
        
        .address-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 12px 16px;
            border: 2px solid var(--green-secondary);
            background: var(--black);
        }}
        
        .address-text {{
            font-size: 14px;
            color: var(--green-primary);
            word-break: break-all;
        }}
        
        .alerts-container {{
            max-height: 600px;
            overflow-y: auto;
            display: flex;
            flex-direction: column;
            gap: 16px;
        }}
        
        .alert-item {{
            border: 2px solid var(--green-primary);
            background: var(--black);
            padding: 16px;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}
        
        .alert-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
            gap: 8px;
        }}
        
        .alert-title {{
            font-weight: 700;
            color: var(--green-accent);
            text-shadow: 0 0 8px var(--green-glow);
        }}
        
        .alert-badge {{
            padding: 4px 12px;
            border: 2px solid var(--green-primary);
            background: var(--dark);
            font-size: 12px;
            text-transform: uppercase;
        }}
        
        .alert-detail {{
            font-size: 13px;
            color: var(--green-secondary);
            word-break: break-all;
        }}
        
        .alert-detail strong {{
            color: var(--green-primary);
        }}
        
        .status-indicator {{
            display: inline-block;
            width: 8px;
            height: 8px;
            background: var(--green-primary);
            border-radius: 50%;
            margin-right: 8px;
            animation: pulse 2s infinite;
        }}
        
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}
        
        .empty-state {{
            text-align: center;
            padding: 40px;
            color: var(--green-secondary);
        }}
        
        ::selection {{
            background: var(--green-primary);
            color: var(--black);
        }}
        
        ::-moz-selection {{
            background: var(--green-primary);
            color: var(--black);
        }}
        
        ::-webkit-scrollbar {{
            width: 8px;
        }}
        
        ::-webkit-scrollbar-track {{
            background: var(--black);
        }}
        
        ::-webkit-scrollbar-thumb {{
            background: var(--green-primary);
            border: 1px solid var(--black);
        }}
        
        ::-webkit-scrollbar-thumb:hover {{
            background: var(--green-accent);
        }}
    </style>
</head>
<body>
    <div class="container">
        <nav class="nav">
            <a href="/marketing" class="nav-brand">RUSTful</a>
            <a href="/docs" class="nav-link">Docs</a>
        </nav>
        <div class="app-content">
            <div class="section">
                <div class="section-title">
                    <span class="status-indicator"></span>
                    Monitor Addresses
                </div>
                <div class="form-group">
                    <input type="text" id="addressInput" class="input" placeholder="0x..." />
                    <button class="button" onclick="addAddress()">Add Address</button>
                </div>
                <div id="addressList" class="address-list"></div>
            </div>
            
            <div class="section">
                <div class="section-title">
                    🚨 Transaction Alerts
                </div>
                <div id="alertsContainer" class="alerts-container">
                    <div class="empty-state">Loading alerts...</div>
                </div>
            </div>
        </div>
    </div>
    <script>
        async function loadMonitoredAddresses() {{
            try {{
                const response = await fetch('/api/monitor');
                const data = await response.json();
                const listEl = document.getElementById('addressList');
                
                if (data.addresses.length === 0) {{
                    listEl.innerHTML = '<div class="empty-state">No addresses being monitored</div>';
                    return;
                }}
                
                listEl.innerHTML = data.addresses.map(addr => `
                    <div class="address-item">
                        <span class="address-text">${{addr}}</span>
                        <button class="button button-danger" onclick="removeAddress('${{addr}}')">Remove</button>
                    </div>
                `).join('');
            }} catch (error) {{
                console.error('Error loading addresses:', error);
            }}
        }}
        
        async function addAddress() {{
            const input = document.getElementById('addressInput');
            const address = input.value.trim();
            
            if (!address) {{
                alert('Please enter an address');
                return;
            }}
            
            if (!address.startsWith('0x') || address.length !== 42) {{
                alert('Invalid address format. Must be 0x followed by 40 hex characters.');
                return;
            }}
            
            try {{
                const response = await fetch('/api/monitor/add', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ address: address }})
                }});
                
                const data = await response.json();
                
                if (data.success) {{
                    input.value = '';
                    loadMonitoredAddresses();
                }} else {{
                    alert('Error: ' + data.message);
                }}
            }} catch (error) {{
                console.error('Error adding address:', error);
                alert('Error adding address: ' + error.message);
            }}
        }}
        
        async function removeAddress(address) {{
            if (!confirm('Remove this address from monitoring?')) {{
                return;
            }}
            
            try {{
                const response = await fetch(`/api/monitor/${{encodeURIComponent(address)}}`, {{
                    method: 'DELETE'
                }});
                
                const data = await response.json();
                
                if (data.success) {{
                    loadMonitoredAddresses();
                }} else {{
                    alert('Error: ' + data.message);
                }}
            }} catch (error) {{
                console.error('Error removing address:', error);
                alert('Error removing address: ' + error.message);
            }}
        }}
        
        function renderAlert(alert) {{
            const valueEth = (parseInt(alert.value) / 1e18).toFixed(6);
            const date = new Date(alert.timestamp * 1000).toLocaleString();
            return `
                <div class="alert-item">
                    <div class="alert-header">
                        <div class="alert-title">🚨 Transaction Alert</div>
                        <div class="alert-badge">Block #${{alert.block_number}}</div>
                    </div>
                    <div class="alert-detail"><strong>Address:</strong> ${{alert.address}}</div>
                    <div class="alert-detail"><strong>Role:</strong> ${{alert.role}}</div>
                    <div class="alert-detail"><strong>TX Hash:</strong> ${{alert.tx_hash}}</div>
                    <div class="alert-detail"><strong>From:</strong> ${{alert.from}}</div>
                    <div class="alert-detail"><strong>To:</strong> ${{alert.to}}</div>
                    <div class="alert-detail"><strong>Value:</strong> ${{valueEth}} ETH (${{alert.value}} wei)</div>
                    <div class="alert-detail"><strong>Gas:</strong> ${{alert.gas}}</div>
                    <div class="alert-detail"><strong>Time:</strong> ${{date}}</div>
                </div>
            `;
        }}
        
        async function loadInitialAlerts() {{
            try {{
                const response = await fetch('/api/monitor/alerts');
                const container = document.getElementById('alertsContainer');
                
                if (!response.ok) {{
                    const errorText = await response.text();
                    console.error('Error loading alerts:', response.status, errorText);
                    container.innerHTML = '<div class="empty-state">Error loading alerts: HTTP ' + response.status + '</div>';
                    return;
                }}
                
                const data = await response.json();
                
                if (!data.success) {{
                    const errorMsg = data.message || 'Unknown error';
                    container.innerHTML = '<div class="empty-state">Error loading alerts: ' + errorMsg + '</div>';
                    return;
                }}
                
                if (data.alerts.length === 0) {{
                    container.innerHTML = '<div class="empty-state">No transaction alerts yet. Add addresses to monitor to see alerts here.</div>';
                    return;
                }}
                
                container.innerHTML = data.alerts.map(alert => renderAlert(alert)).join('');
            }} catch (error) {{
                console.error('Error loading initial alerts:', error);
                const container = document.getElementById('alertsContainer');
                container.innerHTML = '<div class="empty-state">Error loading alerts: ' + error.message + '</div>';
            }}
        }}
        
        function setupSSE() {{
            const container = document.getElementById('alertsContainer');
            const eventSource = new EventSource('/api/monitor/alerts/stream');
            
            eventSource.onmessage = function(event) {{
                try {{
                    const alert = JSON.parse(event.data);
                    if (alert && alert.tx_hash) {{
                        // Prepend new alert to the top
                        const currentContent = container.innerHTML;
                        if (currentContent.includes('empty-state')) {{
                            container.innerHTML = renderAlert(alert);
                        }} else {{
                            container.innerHTML = renderAlert(alert) + currentContent;
                        }}
                        console.log('[SSE] Received new alert:', alert.tx_hash);
                    }}
                }} catch (error) {{
                    console.error('[SSE] Error parsing alert:', error);
                }}
            }};
            
            eventSource.onerror = function(error) {{
                console.error('[SSE] Connection error:', error);
                // Try to reconnect after 3 seconds
                setTimeout(() => {{
                    eventSource.close();
                    setupSSE();
                }}, 3000);
            }};
            
            // Store eventSource for cleanup if needed
            window.alertEventSource = eventSource;
        }}
        
        // Allow Enter key to submit address
        document.getElementById('addressInput').addEventListener('keypress', function(e) {{
            if (e.key === 'Enter') {{
                addAddress();
            }}
        }});
        
        // Initial load
        loadMonitoredAddresses();
        loadInitialAlerts();
        
        // Setup SSE for real-time updates
        setupSSE();
    </script>
</body>
</html>
    "#);
    
    Html(html)
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

// Scan historical blocks for transactions involving monitored addresses
async fn scan_historical_blocks(state: AppState, provider: Arc<Provider<Ws>>, start_block: u64, end_block: u64) {
    println!("[SCAN] Starting historical block scan from {} to {}", start_block, end_block);
    
    let monitored = state.monitored_addresses.read().await.clone();
    if monitored.is_empty() {
        println!("[SCAN] No addresses to monitor, skipping scan");
        return;
    }
    
    let mut found_count = 0;
    
    // Scan backwards from end_block to start_block
    for block_num in (start_block..=end_block).rev() {
        if let Ok(Some(block)) = provider.get_block(block_num).await {
            for tx_hash in block.transactions {
                if let Ok(Some(tx)) = provider.get_transaction(tx_hash).await {
                    let from = tx.from;
                    let to = tx.to;
                    
                    for monitored_addr in &monitored {
                        let mut found = false;
                        let mut role = String::new();
                        
                        if from == *monitored_addr {
                            found = true;
                            role = "FROM".to_string();
                        } else if let Some(to_addr) = to {
                            if to_addr == *monitored_addr {
                                found = true;
                                role = "TO".to_string();
                            }
                        }
                        
                        if found {
                            found_count += 1;
                            let addr_str = format!("{:?}", monitored_addr);
                            
                            println!("\n🔍 HISTORICAL TRANSACTION FOUND 🔍");
                            println!("Block: {}", block_num);
                            println!("Address: {} (as {})", addr_str, role);
                            println!("Transaction Hash: {:?}", tx.hash);
                            println!("From: {:?}", tx.from);
                            println!("To: {:?}", tx.to);
                            println!("Value: {:?} wei", tx.value);
                            
                            // Store in Redis
                            let redis_clone = state.redis.clone();
                            let tx_hash_str = format!("{:?}", tx.hash);
                            let block_hash_str = block.hash.map(|h| format!("{:?}", h)).unwrap_or_default();
                            let tx_data = serde_json::json!({
                                "block_number": block_num,
                                "block_hash": block_hash_str,
                                "address": addr_str.clone(),
                                "role": role.clone(),
                                "tx_hash": tx_hash_str.clone(),
                                "from": format!("{:?}", tx.from),
                                "to": format!("{:?}", tx.to),
                                "value": tx.value.to_string(),
                                "gas": tx.gas.to_string(),
                                "timestamp": block.timestamp,
                                "historical": true,
                            });
                            
                            let key = format!("tx_alert:{}:{}", addr_str, tx_hash_str);
                            let tx_json = serde_json::to_string(&tx_data).unwrap_or_default();
                            
                            if let Ok(mut conn) = redis_clone.get_multiplexed_async_connection().await {
                                let _ = redis::cmd("SETEX")
                                    .arg(&key)
                                    .arg(86400)
                                    .arg(&tx_json)
                                    .query_async::<String>(&mut conn)
                                    .await;
                                
                                let _ = redis::cmd("LPUSH")
                                    .arg("tx_alerts:list")
                                    .arg(&key)
                                    .query_async::<i64>(&mut conn)
                                    .await;
                            }
                            
                            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                        }
                    }
                }
            }
        }
        
        // Progress indicator every 100 blocks
        if block_num % 100 == 0 {
            println!("[SCAN] Scanned to block {}, found {} transactions so far", block_num, found_count);
        }
        
        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    println!("[SCAN] Historical scan complete! Found {} total transactions", found_count);
}

async fn websocket_block_streamer(state: AppState) {
    // Check if block update logging is enabled (default: false)
    // Set LOG_BLOCK_UPDATES=true or LOG_BLOCK_UPDATES=1 to enable
    let log_block_updates = match env::var("LOG_BLOCK_UPDATES") {
        Ok(val) => val.to_lowercase() == "true" || val == "1",
        Err(_) => false,
    };
    
    // Check if we should scan historical blocks
    let scan_historical = match env::var("SCAN_HISTORICAL_BLOCKS") {
        Ok(val) => val.to_lowercase() == "true" || val == "1",
        Err(_) => false,
    };
    
    let scan_blocks_back = env::var("SCAN_BLOCKS_BACK")
        .unwrap_or_else(|_| "1000".to_string())
        .parse::<u64>()
        .unwrap_or(1000);
    
    loop {
        // WebSocket URL for Alchemy
        let ws_url = "wss://base-mainnet.g.alchemy.com/v2/MQ6e6fTn3VEz_P0yeS6518oalCmYIfCx";

        match Provider::<Ws>::connect(ws_url).await {
            Ok(provider) => {
                let provider = Arc::new(provider);
                println!("WebSocket connected to Alchemy");
                
                // Get current block number for historical scanning
                let current_block = if scan_historical {
                    match provider.get_block_number().await {
                        Ok(num) => Some(num.as_u64()),
                        Err(e) => {
                            println!("[SCAN] Failed to get current block number: {:?}", e);
                            None
                        }
                    }
                } else {
                    None
                };
                
                // Scan historical blocks if enabled
                if scan_historical {
                    if let Some(current) = current_block {
                        let start_block = current.saturating_sub(scan_blocks_back);
                        let state_clone = state.clone();
                        let provider_clone = provider.clone();
                        tokio::spawn(async move {
                            scan_historical_blocks(state_clone, provider_clone, start_block, current).await;
                        });
                    }
                }

                // Subscribe to new blocks
                match provider.subscribe_blocks().await {
                    Ok(mut stream) => {
                        println!("Subscribed to new blocks");
                        
                        // Clone provider for fetching transaction details
                        let provider_clone = provider.clone();

                        while let Some(block) = ethers::providers::StreamExt::next(&mut stream).await {
                            if let Some(number) = block.number {
                                let block_num = number.as_u64();
                                *state.latest_block.write().await = Some(block_num);
                                if log_block_updates {
                                    println!("Updated block number: {}", block_num);
                                }
                                
                                // Monitor transactions for specific addresses
                                let monitored = state.monitored_addresses.read().await.clone();
                                if !monitored.is_empty() {
                                    // Fetch full block with transactions (subscribe_blocks only returns headers)
                                    let provider_ref = provider_clone.clone();
                                    let block_hash = block.hash;
                                    
                                    // Get full block with transaction details
                                    let full_block_result = if let Some(hash) = block_hash {
                                        provider_ref.get_block_with_txs(hash).await
                                    } else {
                                        provider_ref.get_block_with_txs(block_num).await
                                    };
                                    
                                    match full_block_result {
                                        Ok(Some(full_block)) => {
                                            let tx_count = full_block.transactions.len();
                                            println!("[MONITOR] Checking {} transactions in block {} against {} monitored address(es)", 
                                                    tx_count, block_num, monitored.len());
                                            
                                            for tx in &full_block.transactions {
                                                // Check if transaction involves any monitored address
                                                let from = tx.from;
                                                let to = tx.to;
                                                
                                                // Debug: log first few transactions
                                                if log_block_updates && full_block.transactions.iter().position(|t| t.hash == tx.hash).unwrap_or(0) < 3 {
                                                    println!("[MONITOR] Checking tx {:?} - From: {:?}, To: {:?}", tx.hash, from, to);
                                                }
                                                
                                                for monitored_addr in &monitored {
                                                    let mut found = false;
                                                    let mut role = String::new();
                                                    
                                                    // Direct address comparison
                                                    if from == *monitored_addr {
                                                        found = true;
                                                        role = "FROM".to_string();
                                                    } else if let Some(to_addr) = to {
                                                        if to_addr == *monitored_addr {
                                                            found = true;
                                                            role = "TO".to_string();
                                                        }
                                                    }
                                                    
                                                    // Debug: log comparison details (always log matches, optionally log all comparisons)
                                                    if found {
                                                        println!("[MONITOR] ✅ MATCH FOUND! Address {:?} found as {} in tx {:?}", monitored_addr, role, tx.hash);
                                                    } else if log_block_updates {
                                                        println!("[MONITOR] Comparing - From: {:?} == {:?}? {}", from, monitored_addr, from == *monitored_addr);
                                                        if let Some(to_addr) = to {
                                                            println!("[MONITOR] Comparing - To: {:?} == {:?}? {}", to_addr, monitored_addr, to_addr == *monitored_addr);
                                                        }
                                                    }
                                                
                                                    if found {
                                                        let addr_str = format!("{:?}", monitored_addr);
                                                        let role_clone = role.clone();
                                                        
                                                        println!("\n🚨 TRANSACTION ALERT 🚨");
                                                        println!("Block: {}", block_num);
                                                        println!("Address: {} (as {})", addr_str, role_clone);
                                                        println!("Transaction Hash: {:?}", tx.hash);
                                                        println!("From: {:?}", tx.from);
                                                        println!("To: {:?}", tx.to);
                                                        println!("Value: {:?} wei", tx.value);
                                                        println!("Gas: {:?}", tx.gas);
                                                        
                                                        // Get receipt for more details
                                                        let provider_ref2 = provider_clone.clone();
                                                        if let Ok(Some(receipt)) = provider_ref2.get_transaction_receipt(tx.hash).await {
                                                            println!("Status: {:?}", receipt.status);
                                                            println!("Gas Used: {:?}", receipt.gas_used);
                                                            if let Some(contract) = receipt.contract_address {
                                                                println!("Contract Created: {:?}", contract);
                                                            }
                                                            println!("Logs: {}", receipt.logs.len());
                                                        }
                                                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                                                        
                                                        // Create alert struct
                                                        let tx_hash_str = format!("{:?}", tx.hash);
                                                        let block_hash_str = format!("{:?}", block.hash);
                                                        let alert = TransactionAlert {
                                                            block_number: block_num,
                                                            block_hash: block_hash_str.clone(),
                                                            address: addr_str.clone(),
                                                            role: role_clone.clone(),
                                                            tx_hash: tx_hash_str.clone(),
                                                            from: format!("{:?}", tx.from),
                                                            to: format!("{:?}", tx.to),
                                                            value: tx.value.to_string(),
                                                            gas: tx.gas.to_string(),
                                                            timestamp: block.timestamp.as_u64(),
                                                        };
                                                        
                                                        // Broadcast alert to SSE clients
                                                        let alert_tx_clone = state.alert_tx.clone();
                                                        if let Err(e) = alert_tx_clone.send(alert.clone()) {
                                                            eprintln!("[MONITOR] Failed to broadcast alert: {}", e);
                                                        }
                                                        
                                                        // Store in Redis for webhook/querying
                                                        let redis_manager = state.redis_manager.clone();
                                                        let tx_data = serde_json::json!({
                                                            "block_number": block_num,
                                                            "block_hash": block_hash_str,
                                                            "address": addr_str.clone(),
                                                            "role": role_clone.clone(),
                                                            "tx_hash": tx_hash_str.clone(),
                                                            "from": format!("{:?}", tx.from),
                                                            "to": format!("{:?}", tx.to),
                                                            "value": tx.value.to_string(),
                                                            "gas": tx.gas.to_string(),
                                                            "timestamp": block.timestamp.as_u64(),
                                                        });
                                                        
                                                        let key = format!("tx_alert:{}:{}", addr_str, tx_hash_str);
                                                        let tx_json = serde_json::to_string(&tx_data).unwrap_or_default();
                                                        
                                                        tokio::spawn(async move {
                                                            let mut conn = (*redis_manager).clone();
                                                            
                                                            // Store alert with 24 hour TTL
                                                            match redis::cmd("SETEX")
                                                                .arg(&key)
                                                                .arg(86400)
                                                                .arg(&tx_json)
                                                                .query_async::<String>(&mut conn)
                                                                .await
                                                            {
                                                                Ok(_) => println!("[REDIS] Stored alert: {}", key),
                                                                Err(e) => eprintln!("[REDIS] Failed to store alert {}: {}", key, e),
                                                            }
                                                            
                                                            // Add to list of alerts
                                                            match redis::cmd("LPUSH")
                                                                .arg("tx_alerts:list")
                                                                .arg(&key)
                                                                .query_async::<i64>(&mut conn)
                                                                .await
                                                            {
                                                                Ok(count) => println!("[REDIS] Added to list (total: {})", count),
                                                                Err(e) => eprintln!("[REDIS] Failed to add to list: {}", e),
                                                            }
                                                            
                                                            // Keep list size manageable (last 1000 alerts)
                                                            let _ = redis::cmd("LTRIM")
                                                                .arg("tx_alerts:list")
                                                                .arg(0)
                                                                .arg(999)
                                                                .query_async::<String>(&mut conn)
                                                                .await;
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            if log_block_updates {
                                                println!("[MONITOR] Block {} not found when fetching full block", block_num);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[MONITOR] Error fetching full block {}: {:?}", block_num, e);
                                        }
                                    }
                                }
                                
                                // Debug: Inspect block structure (every 100th block)
                                if block_num % 100 == 0 {
                                    println!("\n=== BLOCK STRUCTURE DEBUG ===");
                                    println!("Block number: {:?}", block.number);
                                    println!("Block hash: {:?}", block.hash);
                                    println!("Parent hash: {:?}", block.parent_hash);
                                    println!("Timestamp: {:?}", block.timestamp);
                                    println!("Gas used: {:?}", block.gas_used);
                                    println!("Gas limit: {:?}", block.gas_limit);
                                    println!("Base fee: {:?}", block.base_fee_per_gas);
                                    println!("Transaction count (from stream): {:?}", block.transactions.len());
                                    
                                    // Try to get full block
                                    let provider_debug = provider_clone.clone();
                                    if let Some(full_block) = provider_debug.get_block_with_txs(block_num).await.ok().flatten() {
                                        println!("Transaction count (from full block): {:?}", full_block.transactions.len());
                                        
                                        // Inspect transaction hashes (full_block.transactions contains full Transaction objects)
                                        println!("\n--- Transaction Hashes (first 3) ---");
                                        for (i, tx) in full_block.transactions.iter().take(3).enumerate() {
                                            println!("  Transaction {}: {:?}", i + 1, tx.hash);
                                        }
                                        
                                        // Show transaction details from full block (already have them!)
                                        if let Some(first_tx) = full_block.transactions.first() {
                                            println!("\n--- Transaction Details (from full block) ---");
                                            println!("  Hash: {:?}", first_tx.hash);
                                            println!("  From: {:?}", first_tx.from);
                                            println!("  To: {:?}", first_tx.to);
                                            println!("  Value: {:?}", first_tx.value);
                                            println!("  Gas: {:?}", first_tx.gas);
                                            println!("  Gas Price: {:?}", first_tx.gas_price);
                                            println!("  Max Fee Per Gas: {:?}", first_tx.max_fee_per_gas);
                                            println!("  Max Priority Fee Per Gas: {:?}", first_tx.max_priority_fee_per_gas);
                                            println!("  Input length: {} bytes", first_tx.input.len());
                                            println!("  Nonce: {:?}", first_tx.nonce);
                                            println!("  Transaction Type: {:?}", first_tx.transaction_type);
                                            println!("  Access List: {:?}", first_tx.access_list);
                                            println!("  Chain ID: {:?}", first_tx.chain_id);
                                            
                                            // Try to get receipt for more info
                                            let provider_ref2 = provider_clone.clone();
                                            let tx_hash = first_tx.hash;
                                            if let Ok(Some(receipt)) = provider_ref2.get_transaction_receipt(tx_hash).await {
                                                println!("\n  --- Transaction Receipt ---");
                                                println!("    Status: {:?}", receipt.status);
                                                println!("    Gas Used: {:?}", receipt.gas_used);
                                                println!("    Cumulative Gas Used: {:?}", receipt.cumulative_gas_used);
                                                println!("    Effective Gas Price: {:?}", receipt.effective_gas_price);
                                                println!("    Logs Count: {:?}", receipt.logs.len());
                                                println!("    Contract Address: {:?}", receipt.contract_address);
                                            }
                                        }
                                    } else {
                                        println!("Could not fetch full block for debugging");
                                    }
                                    println!("=== END DEBUG ===\n");
                                }
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
    // Initialize Redis connection
    let redis_client = match init_redis() {
        Ok(client) => Arc::new(client),
        Err(e) => {
            eprintln!("[ERROR] Failed to initialize Redis: {}", e);
            eprintln!("[ERROR] Server will start without Redis support");
            // Create a dummy client - this will fail if used, but allows server to start
            // In production, you might want to exit here instead
            panic!("Redis connection required");
        }
    };
    
    // Initialize Redis connection manager for connection pooling
    // Use the same Redis client to create the connection manager
    // Retry a few times in case of temporary network issues
    let mut redis_manager = None;
    let mut retries = 5;
    let mut retry_delay = 2;
    
    println!("[REDIS] Creating connection manager...");
    while redis_manager.is_none() && retries > 0 {
        match ConnectionManager::new(redis_client.as_ref().clone()).await {
            Ok(manager) => {
                println!("[REDIS] Connection manager created successfully!");
                redis_manager = Some(Arc::new(manager));
            }
            Err(e) => {
                retries -= 1;
                if retries > 0 {
                    eprintln!("[REDIS] Failed to create connection manager: {}. Retrying in {} seconds... ({} retries left)", e, retry_delay, retries);
                    tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay)).await;
                    retry_delay = std::cmp::min(retry_delay + 1, 5); // Incremental backoff, max 5 seconds
                } else {
                    eprintln!("[ERROR] Failed to create Redis connection manager after {} retries: {}", 5, e);
                    eprintln!("[ERROR] Check your REDIS_URL environment variable and Railway Redis connection");
                    eprintln!("[ERROR] Make sure the Redis instance is accessible from your network");
                    panic!("Redis connection manager required - check Railway Redis connection");
                }
            }
        }
    }
    
    let redis_manager = redis_manager.expect("Redis connection manager should be initialized");
    
    // Initialize shared state for block number caching
    let latest_block = Arc::new(RwLock::new(None));
    
    // Initialize monitored addresses (can be set via env var or default)
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
    
    // Create broadcast channel for SSE alerts (buffer up to 1000 alerts)
    let (alert_tx, _) = broadcast::channel(1000);
    
    let app_state = AppState {
        latest_block: latest_block.clone(),
        redis: redis_client,
        redis_manager: redis_manager.clone(),
        monitored_addresses: Arc::new(RwLock::new(monitored_addresses)),
        alert_tx: alert_tx.clone(),
    };

    // Start WebSocket block streamer in background
    let streamer_state = app_state.clone();
    tokio::spawn(async move {
        websocket_block_streamer(streamer_state).await;
    });

    // Build the application with routes
    let app = Router::new()
        .route("/", get(marketing_page))
        .route("/marketing", get(marketing_page))
        .route("/app", get(app_page))
        .route("/api/marco", get(marco))
        .route("/api/redis-test", get(|s: axum::extract::State<AppState>| redis_test(s)))
        .route("/api/getCurrentBlock", get({
            let state = app_state.clone();
            move || get_current_block_handler(axum::extract::State(state))
        }))
        .route("/api/db", post({
            let _state = app_state.clone();
            move |s: axum::extract::State<AppState>, payload: Json<CreateRecordRequest>| db_create(s, payload)
        }))
        .route("/api/db", get({
            let _state = app_state.clone();
            move |s: axum::extract::State<AppState>| db_list(s)
        }))
        .route("/api/db/:id", get({
            let _state = app_state.clone();
            move |s: axum::extract::State<AppState>, path: Path<String>| db_read(s, path)
        }))
        .route("/api/db/:id", put({
            let _state = app_state.clone();
            move |s: axum::extract::State<AppState>, path: Path<String>, payload: Json<CreateRecordRequest>| db_update(s, path, payload)
        }))
        .route("/api/db/:id", delete({
            let _state = app_state.clone();
            move |s: axum::extract::State<AppState>, path: Path<String>| db_delete(s, path)
        }))
        .route("/api/monitor", get(|s: axum::extract::State<AppState>| monitor_list(s)))
        .route("/api/monitor/add", post(|s: axum::extract::State<AppState>, payload: Json<MonitorAddressRequest>| monitor_add(s, payload)))
        .route("/api/monitor/:address", delete(|s: axum::extract::State<AppState>, path: Path<String>| monitor_remove(s, path)))
        .route("/api/monitor/alerts", get(|s: axum::extract::State<AppState>| monitor_alerts(s)))
        .route("/api/monitor/alerts/stream", get({
            let state = app_state.clone();
            move || monitor_alerts_stream(State(state))
        }))
        .route("/api/docs", get(api_docs_json))
        .route("/docs", get(api_docs_html))
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

