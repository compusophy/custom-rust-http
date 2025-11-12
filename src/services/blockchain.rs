use std::env;
use std::sync::Arc;

use crate::state::AppState;
use crate::types::BlockResponse;
use crate::types::TransactionAlert;
use axum::Json;
use ethers::middleware::Middleware;
use ethers::providers::{Http, Provider, StreamExt, Ws};
use ethers::types::Address;
use serde_json::json;

// Clanker contract address for token deployments
const CLANKER_CONTRACT: &str = "0xE85A59c628F7d27878ACeB4bf3b35733630083a9";
// deployToken method selector bytes (first 4 bytes of keccak256("deployToken((tuple,tuple,tuple,tuple,tuple[]))"))
const DEPLOY_TOKEN_SELECTOR_BYTES: [u8; 4] = [0xdf, 0x40, 0x22, 0x4a];

/// Classify a transaction based on its to address and input data
fn classify_transaction(to: Option<Address>, input_data: &[u8]) -> String {
    // Check if transaction is to Clanker contract
    if let Some(to_addr) = to {
        let to_str = format!("{:?}", to_addr).to_lowercase();
        let clanker_lower = CLANKER_CONTRACT.to_lowercase();
        
        if to_str == clanker_lower {
            // Check if input data starts with deployToken selector
            if input_data.len() >= 4 && input_data[0..4] == DEPLOY_TOKEN_SELECTOR_BYTES {
                return "Deployment".to_string();
            }
        }
    }
    
    "Other".to_string()
}

pub async fn get_current_block() -> Result<Json<BlockResponse>, String> {
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
pub async fn scan_historical_blocks(
    state: AppState,
    provider: Arc<Provider<Ws>>,
    start_block: u64,
    end_block: u64,
) {
    println!(
        "[SCAN] Starting historical block scan from {} to {}",
        start_block, end_block
    );

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

                            // Classify transaction
                            let category = classify_transaction(tx.to, &tx.input);
                            let input_data_str = if tx.input.len() > 0 {
                                Some(format!("0x{}", tx.input.iter().map(|b| format!("{:02x}", b)).collect::<String>()))
                            } else {
                                None
                            };
                            
                            // Store in Redis
                            let redis_clone = state.redis.clone();
                            let tx_hash_str = format!("{:?}", tx.hash);
                            let block_hash_str = block.hash.map(|h| format!("{:?}", h)).unwrap_or_default();
                            let tx_data = json!({
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
                                "category": category.clone(),
                                "input_data": input_data_str.clone(),
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
            println!(
                "[SCAN] Scanned to block {}, found {} transactions so far",
                block_num, found_count
            );
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    println!(
        "[SCAN] Historical scan complete! Found {} total transactions",
        found_count
    );
}

pub async fn websocket_block_streamer(state: AppState) {
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

                        while let Some(block) = stream.next().await {
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
                                            for tx in &full_block.transactions {
                                                // Check if transaction involves any monitored address
                                                let from = tx.from;
                                                let to = tx.to;

                                                // Debug: log first few transactions
                                                if log_block_updates
                                                    && full_block
                                                        .transactions
                                                        .iter()
                                                        .position(|t| t.hash == tx.hash)
                                                        .unwrap_or(0)
                                                        < 3
                                                {
                                                    println!(
                                                        "[MONITOR] Checking tx {:?} - From: {:?}, To: {:?}",
                                                        tx.hash, from, to
                                                    );
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
                                                        println!(
                                                            "[MONITOR] ✅ MATCH FOUND! Address {:?} found as {} in tx {:?}",
                                                            monitored_addr, role, tx.hash
                                                        );
                                                    } else if log_block_updates {
                                                        println!(
                                                            "[MONITOR] Comparing - From: {:?} == {:?}? {}",
                                                            from,
                                                            monitored_addr,
                                                            from == *monitored_addr
                                                        );
                                                        if let Some(to_addr) = to {
                                                            println!(
                                                                "[MONITOR] Comparing - To: {:?} == {:?}? {}",
                                                                to_addr,
                                                                monitored_addr,
                                                                to_addr == *monitored_addr
                                                            );
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
                                                        if let Ok(Some(receipt)) =
                                                            provider_ref2.get_transaction_receipt(tx.hash).await
                                                        {
                                                            println!("Status: {:?}", receipt.status);
                                                            println!("Gas Used: {:?}", receipt.gas_used);
                                                            if let Some(contract) = receipt.contract_address {
                                                                println!("Contract Created: {:?}", contract);
                                                            }
                                                            println!("Logs: {}", receipt.logs.len());
                                                        }
                                                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                                                        // Classify transaction
                                                        let category = classify_transaction(tx.to, &tx.input);
                                                        let input_data_str = if tx.input.len() > 0 {
                                                            Some(format!("0x{}", tx.input.iter().map(|b| format!("{:02x}", b)).collect::<String>()))
                                                        } else {
                                                            None
                                                        };
                                                        
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
                                                            category: category.clone(),
                                                            input_data: input_data_str.clone(),
                                                        };

                                                        // Broadcast alert to SSE clients
                                                        let alert_tx_clone = state.alert_tx.clone();
                                                        if let Err(e) = alert_tx_clone.send(alert.clone()) {
                                                            eprintln!("[MONITOR] Failed to broadcast alert: {}", e);
                                                        }

                                                        // Store in Redis for webhook/querying
                                                        let redis_client = state.redis.clone();
                                                        let tx_data = json!({
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
                                                            "category": category.clone(),
                                                            "input_data": input_data_str.clone(),
                                                        });

                                                        let key = format!("tx_alert:{}:{}", addr_str, tx_hash_str);
                                                        let tx_json = serde_json::to_string(&tx_data).unwrap_or_default();

                                                        tokio::spawn(async move {
                                                            match redis_client.get_multiplexed_async_connection().await {
                                                                Ok(mut conn) => {
                                                                    // Store alert with 24 hour TTL
                                                                    match redis::cmd("SETEX")
                                                                        .arg(&key)
                                                                        .arg(86400)
                                                                        .arg(&tx_json)
                                                                        .query_async::<String>(&mut conn)
                                                                        .await
                                                                    {
                                                                        Ok(_) => println!("[REDIS] Stored alert: {}", key),
                                                                        Err(e) => eprintln!(
                                                                            "[REDIS] Failed to store alert {}: {}",
                                                                            key, e
                                                                        ),
                                                                    }

                                                                    // Add to list of alerts
                                                                    match redis::cmd("LPUSH")
                                                                        .arg("tx_alerts:list")
                                                                        .arg(&key)
                                                                        .query_async::<i64>(&mut conn)
                                                                        .await
                                                                    {
                                                                        Ok(count) => println!(
                                                                            "[REDIS] Added to list (total: {})",
                                                                            count
                                                                        ),
                                                                        Err(e) => eprintln!(
                                                                            "[REDIS] Failed to add to list: {}",
                                                                            e
                                                                        ),
                                                                    }

                                                                    // Keep list size manageable (last 1000 alerts)
                                                                    let _ = redis::cmd("LTRIM")
                                                                        .arg("tx_alerts:list")
                                                                        .arg(0)
                                                                        .arg(999)
                                                                        .query_async::<String>(&mut conn)
                                                                        .await;
                                                                }
                                                                Err(e) => {
                                                                    eprintln!("[REDIS] Failed to get connection: {}", e);
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            if log_block_updates {
                                                println!(
                                                    "[MONITOR] Block {} not found when fetching full block",
                                                    block_num
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[MONITOR] Error fetching full block {}: {:?}", block_num, e);
                                        }
                                    }
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


