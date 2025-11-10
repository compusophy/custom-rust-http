use axum::{routing::get, Json, Router, response::Html};
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
}

async fn marco() -> Json<PoloResponse> {
    Json(PoloResponse {
        message: "polo".to_string(),
    })
}

async fn api_docs_json() -> Json<ApiDocs> {
    Json(ApiDocs {
        name: "Rust HTTP Blockchain API".to_string(),
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
                description: "Returns the current Base mainnet block number. Uses WebSocket streaming for sub-100ms response times.".to_string(),
                example_request: None,
                example_response: r#"{"block_number":"0x241337e","block_number_decimal":37827454}"#.to_string(),
                performance: Some("~35-70ms (WebSocket cached)".to_string()),
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
    
    let endpoints_html: String = docs.endpoints.iter().map(|endpoint| {
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
        
        format!(r#"
            <div class="endpoint-card">
                <div class="endpoint-header">
                    {} <code class="path">{}</code>
                </div>
                <p class="description">{}</p>
                <div class="example">
                    <strong>Example Response:</strong>
                    <pre><code>{}</code></pre>
                </div>
                {}
            </div>
        "#, method_badge, endpoint.path, endpoint.description, endpoint.example_response, performance_html)
    }).collect();
    
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - API Documentation</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 2rem;
            color: #333;
        }}
        .container {{
            max-width: 900px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem;
            text-align: center;
        }}
        .header h1 {{
            font-size: 2rem;
            margin-bottom: 0.5rem;
        }}
        .header .version {{
            opacity: 0.9;
            font-size: 1rem;
        }}
        .content {{
            padding: 2rem;
        }}
        .endpoint-card {{
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            transition: transform 0.2s, box-shadow 0.2s;
        }}
        .endpoint-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
        }}
        .endpoint-header {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
            margin-bottom: 1rem;
        }}
        .method-badge {{
            padding: 0.25rem 0.75rem;
            border-radius: 4px;
            font-weight: 600;
            font-size: 0.875rem;
            text-transform: uppercase;
        }}
        .method-badge.get {{
            background: #10b981;
            color: white;
        }}
        .method-badge.post {{
            background: #3b82f6;
            color: white;
        }}
        .method-badge.put {{
            background: #f59e0b;
            color: white;
        }}
        .method-badge.delete {{
            background: #ef4444;
            color: white;
        }}
        .path {{
            background: #f3f4f6;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.9rem;
            color: #667eea;
        }}
        .description {{
            color: #666;
            margin-bottom: 1rem;
            line-height: 1.6;
        }}
        .example {{
            background: #f9fafb;
            border-left: 3px solid #667eea;
            padding: 1rem;
            border-radius: 4px;
            margin-top: 1rem;
        }}
        .example strong {{
            display: block;
            margin-bottom: 0.5rem;
            color: #333;
        }}
        .example pre {{
            background: #1e293b;
            color: #e2e8f0;
            padding: 1rem;
            border-radius: 4px;
            overflow-x: auto;
            font-size: 0.875rem;
        }}
        .example code {{
            font-family: 'Monaco', 'Courier New', monospace;
        }}
        .performance {{
            margin-top: 0.75rem;
            color: #10b981;
            font-weight: 500;
            font-size: 0.875rem;
        }}
        .base-url {{
            background: #f3f4f6;
            padding: 1rem;
            border-radius: 8px;
            margin-bottom: 2rem;
            text-align: center;
        }}
        .base-url code {{
            color: #667eea;
            font-size: 1.1rem;
            font-weight: 600;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{}</h1>
            <div class="version">Version {}</div>
        </div>
        <div class="content">
            <div class="base-url">
                Base URL: <code>{}</code>
            </div>
            <h2 style="margin-bottom: 1.5rem; color: #333;">API Endpoints</h2>
            {}
        </div>
    </div>
</body>
</html>
    "#, docs.name, docs.name, docs.version, docs.base_url, endpoints_html);
    
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
