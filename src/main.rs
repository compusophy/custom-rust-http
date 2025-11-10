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
            <a href="/marketing" class="cta-button">Launch App</a>
        </div>
    </div>
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
        .route("/", get(marketing_page))
        .route("/marketing", get(marketing_page))
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
