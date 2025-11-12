use std::convert::Infallible;

use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::Json;
use ethers::types::Address;
use futures::stream::StreamExt;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;

use crate::state::AppState;
use crate::types::{
    MonitorAddressRequest, MonitorAddressResponse, MonitorListResponse, TransactionAlert,
    TransactionAlertsResponse,
};

pub async fn monitor_add(
    State(state): State<AppState>,
    Json(payload): Json<MonitorAddressRequest>,
) -> Result<Json<MonitorAddressResponse>, String> {
    println!(
        "[API] POST /api/monitor/add - Adding address: {}",
        payload.address
    );

    let address: Address = payload
        .address
        .parse()
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

pub async fn monitor_remove(
    State(state): State<AppState>,
    Path(address_str): Path<String>,
) -> Result<Json<MonitorAddressResponse>, String> {
    println!(
        "[API] DELETE /api/monitor/{} - Removing address",
        address_str
    );

    let address: Address = address_str
        .parse()
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

pub async fn monitor_list(State(state): State<AppState>) -> Json<MonitorListResponse> {
    let monitored = state.monitored_addresses.read().await;
    let addresses: Vec<String> = monitored.iter().map(|a| format!("{:?}", a)).collect();

    Json(MonitorListResponse {
        success: true,
        addresses: addresses.clone(),
        count: addresses.len(),
    })
}

pub async fn monitor_alerts(State(state): State<AppState>) -> Json<TransactionAlertsResponse> {
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
                            Ok(Some(json_str)) => match serde_json::from_str::<TransactionAlert>(&json_str) {
                                Ok(alert) => alerts.push(alert),
                                Err(e) => {
                                    eprintln!("[API] Failed to parse alert {}: {}", key, e);
                                }
                            },
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

pub async fn monitor_alerts_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    println!("[SSE] Client connected to /api/monitor/alerts/stream");

    let rx = state.alert_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| async move {
        match result {
            Ok(alert) => match serde_json::to_string(&alert) {
                Ok(json) => Some(Ok(Event::default().data(json))),
                Err(e) => {
                    eprintln!("[SSE] Failed to serialize alert: {}", e);
                    None
                }
            },
            Err(BroadcastStreamRecvError::Lagged(skipped)) => {
                eprintln!("[SSE] Client lagged, skipped {} messages", skipped);
                None
            }
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive-text".to_string()),
    )
}


