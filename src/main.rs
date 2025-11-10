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
    
    // Categorize endpoints
    let mut blockchain_endpoints = Vec::new();
    let mut utility_endpoints = Vec::new();
    let mut docs_endpoints = Vec::new();
    
    for endpoint in &docs.endpoints {
        let category = if endpoint.path.contains("Block") || endpoint.path.contains("block") {
            "blockchain"
        } else if endpoint.path.contains("docs") || endpoint.path.contains("Docs") {
            "documentation"
        } else {
            "utility"
        };
        
        match category {
            "blockchain" => blockchain_endpoints.push(endpoint),
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
        
        format!(r#"
            <div class="endpoint-card" id="{}">
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
        "#, id, method_badge, endpoint.path, endpoint.description, endpoint.example_response, performance_html)
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
    <title>{} - API Documentation</title>
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
        
        /* TYPOGRAPHY - IBM PLEX MONO */
        body, html {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
            font-size: 14px;
            line-height: 1.5;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }}
        
        /* COLOR PALETTE - DARK MODE WITH MATRIX GREEN */
        :root {{
            --black: #000000;
            --dark: #0A0A0A;
            --dark-gray: #1A1A1A;
            --medium-gray: #2A2A2A;
            --light-gray: #3A3A3A;
            --white: #FFFFFF;
            --matrix-green: #00FF41;
            --matrix-green-dark: #00CC33;
            --matrix-green-bright: #00FF88;
        }}
        
        /* BASE LAYOUT - DARK MODE */
        body {{
            background: var(--black);
            color: var(--matrix-green);
            min-height: 100vh;
        }}
        
        /* CONTAINER - BRUTALIST GEOMETRY */
        .container {{
            display: flex;
            max-width: 1600px;
            margin: 0 auto;
            background: var(--black);
            min-height: 100vh;
            border-left: 4px solid var(--matrix-green);
            border-right: 4px solid var(--matrix-green);
        }}
        
        /* SIDEBAR - STARK GEOMETRY */
        .sidebar {{
            width: 300px;
            background: var(--dark);
            border-right: 4px solid var(--matrix-green);
            padding: 0;
            position: sticky;
            top: 0;
            height: 100vh;
            overflow-y: auto;
        }}
        
        .sidebar-header {{
            padding: 24px;
            border-bottom: 4px solid var(--matrix-green);
            background: var(--black);
            color: var(--matrix-green);
        }}
        
        .sidebar-header h2 {{
            font-size: 16px;
            font-weight: bold;
            text-transform: uppercase;
            letter-spacing: 2px;
            margin-bottom: 8px;
        }}
        
        .sidebar-header .version {{
            font-size: 12px;
            color: var(--matrix-green-dark);
            letter-spacing: 1px;
        }}
        
        /* NAVIGATION - BRUTALIST LINKS */
        .nav-category {{
            font-size: 11px;
            font-weight: bold;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--matrix-green);
            padding: 16px 24px 8px;
            background: var(--dark);
            border-bottom: 2px solid var(--matrix-green);
        }}
        
        .nav-item {{
            display: block;
            padding: 12px 24px;
            color: var(--matrix-green-dark);
            text-decoration: none;
            font-size: 13px;
            border-left: 4px solid transparent;
            background: var(--dark);
            border-bottom: 1px solid var(--dark-gray);
            transition: none;
        }}
        
        .nav-item:hover {{
            background: var(--black);
            color: var(--matrix-green);
            border-left-color: var(--matrix-green);
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
            color: var(--matrix-green);
            padding: 32px;
            text-align: left;
            border-bottom: 4px solid var(--matrix-green);
        }}
        
        .header h1 {{
            font-size: 24px;
            font-weight: bold;
            text-transform: uppercase;
            letter-spacing: 3px;
            margin-bottom: 8px;
        }}
        
        .header .version {{
            font-size: 12px;
            color: var(--matrix-green-dark);
            letter-spacing: 2px;
        }}
        
        /* SECTIONS */
        .section-title {{
            font-size: 18px;
            font-weight: bold;
            text-transform: uppercase;
            letter-spacing: 2px;
            color: var(--matrix-green);
            margin: 0;
            padding: 24px 32px;
            border-bottom: 4px solid var(--matrix-green);
            background: var(--dark);
        }}
        
        /* BASE URL BOX */
        .base-url {{
            background: var(--dark);
            color: var(--matrix-green);
            padding: 16px 32px;
            border-bottom: 4px solid var(--matrix-green);
        }}
        
        .base-url code {{
            font-size: 13px;
            letter-spacing: 1px;
            color: var(--matrix-green);
        }}
        
        /* ENDPOINT CARDS - BRUTALIST BOXES */
        .endpoint-card {{
            border: 4px solid var(--matrix-green);
            padding: 24px 32px;
            margin: 0;
            background: var(--black);
            border-top: none;
        }}
        
        .endpoint-card:last-child {{
            border-bottom: 4px solid var(--matrix-green);
        }}
        
        .endpoint-header {{
            display: flex;
            align-items: center;
            gap: 16px;
            margin-bottom: 16px;
        }}
        
        /* METHOD BADGES - MATRIX STYLE */
        .method-badge {{
            padding: 4px 12px;
            font-weight: bold;
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 1px;
            border: 2px solid var(--matrix-green);
            background: var(--black);
            color: var(--matrix-green);
        }}
        
        .method-badge.get {{
            background: var(--matrix-green);
            color: var(--black);
        }}
        
        .method-badge.post {{
            background: var(--black);
            color: var(--matrix-green);
            border: 2px solid var(--matrix-green);
        }}
        
        .method-badge.put {{
            background: var(--dark-gray);
            color: var(--matrix-green);
        }}
        
        .method-badge.delete {{
            background: var(--matrix-green-dark);
            color: var(--black);
        }}
        
        /* PATH - MONOSPACE CODE */
        .path {{
            background: var(--dark);
            padding: 4px 8px;
            font-size: 13px;
            color: var(--matrix-green);
            border: 2px solid var(--matrix-green);
            letter-spacing: 0.5px;
        }}
        
        /* DESCRIPTION */
        .description {{
            color: var(--matrix-green-dark);
            margin-bottom: 16px;
            line-height: 1.6;
            font-size: 13px;
        }}
        
        /* EXAMPLE BOXES */
        .example {{
            background: var(--dark);
            border: 4px solid var(--matrix-green);
            padding: 16px;
            margin-top: 16px;
        }}
        
        .example strong {{
            display: block;
            margin-bottom: 12px;
            color: var(--matrix-green);
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        
        .example pre {{
            background: var(--black);
            color: var(--matrix-green);
            padding: 16px;
            border: 2px solid var(--matrix-green);
            overflow-x: auto;
            font-size: 12px;
            line-height: 1.5;
        }}
        
        .example code {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
        }}
        
        /* PERFORMANCE METRIC */
        .performance {{
            margin-top: 16px;
            color: var(--matrix-green-bright);
            font-weight: bold;
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        
        /* UTILITY CLASSES */
        h1, h2, h3, h4, h5, h6 {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
            font-weight: bold;
            text-transform: uppercase;
            letter-spacing: 2px;
        }}
        
        code {{
            font-family: 'IBM Plex Mono', 'Courier New', 'Monaco', 'Menlo', 'Consolas', monospace;
        }}
        
        /* SCROLLBAR - MATRIX STYLE */
        ::-webkit-scrollbar {{
            width: 12px;
        }}
        
        ::-webkit-scrollbar-track {{
            background: var(--black);
            border-left: 2px solid var(--matrix-green);
        }}
        
        ::-webkit-scrollbar-thumb {{
            background: var(--matrix-green);
            border: 2px solid var(--black);
        }}
        
        ::-webkit-scrollbar-thumb:hover {{
            background: var(--matrix-green-bright);
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="sidebar">
            <div class="sidebar-header">
                <h2>{}</h2>
                <div class="version">v{}</div>
            </div>
            <nav>
                {}
            </nav>
        </div>
        <div class="main-content">
            <div class="header">
                <h1>{}</h1>
                <div class="version">Version {}</div>
            </div>
            <div class="base-url">
                Base URL: <code>{}</code>
            </div>
            {}
        </div>
    </div>
</body>
</html>
    "#, docs.name, docs.name, docs.version, sidebar_items, docs.name, docs.version, docs.base_url, endpoints_html);
    
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
