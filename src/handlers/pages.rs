use axum::response::Html;

pub async fn marketing_page() -> Html<String> {
    let html = format!(
        r#"
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
    "#
    );

    Html(html)
}

pub async fn app_page() -> Html<String> {
    let html = format!(
        r#"
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
                    🚀 Deployments
                </div>
                <div id="deploymentsContainer" class="alerts-container">
                    <div class="empty-state">Loading deployments...</div>
                </div>
            </div>
            
            <div class="section">
                <div class="section-title">
                    🚨 Other Transactions
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
            const category = alert.category || 'Other';
            const isDeployment = category === 'Deployment';
            const titleIcon = isDeployment ? '🚀' : '🚨';
            const titleText = isDeployment ? 'Token Deployment' : 'Transaction Alert';
            
            return `
                <div class="alert-item">
                    <div class="alert-header">
                        <div class="alert-title">${{titleIcon}} ${{titleText}}</div>
                        <div class="alert-badge">Block #${{alert.block_number}}</div>
                    </div>
                    <div class="alert-detail"><strong>Category:</strong> ${{category}}</div>
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
                const alertsContainer = document.getElementById('alertsContainer');
                const deploymentsContainer = document.getElementById('deploymentsContainer');
                
                if (!response.ok) {{
                    const errorText = await response.text();
                    console.error('Error loading alerts:', response.status, errorText);
                    alertsContainer.innerHTML = '<div class="empty-state">Error loading alerts: HTTP ' + response.status + '</div>';
                    deploymentsContainer.innerHTML = '<div class="empty-state">Error loading deployments: HTTP ' + response.status + '</div>';
                    return;
                }}
                
                const data = await response.json();
                
                if (!data.success) {{
                    const errorMsg = data.message || 'Unknown error';
                    alertsContainer.innerHTML = '<div class="empty-state">Error loading alerts: ' + errorMsg + '</div>';
                    deploymentsContainer.innerHTML = '<div class="empty-state">Error loading deployments: ' + errorMsg + '</div>';
                    return;
                }}
                
                // Separate deployments from other transactions
                const deployments = data.alerts.filter(alert => alert.category === 'Deployment');
                const otherAlerts = data.alerts.filter(alert => alert.category !== 'Deployment');
                
                if (deployments.length === 0) {{
                    deploymentsContainer.innerHTML = '<div class="empty-state">No deployments yet.</div>';
                }} else {{
                    deploymentsContainer.innerHTML = deployments.map(alert => renderAlert(alert)).join('');
                }}
                
                if (otherAlerts.length === 0) {{
                    alertsContainer.innerHTML = '<div class="empty-state">No other transactions yet.</div>';
                }} else {{
                    alertsContainer.innerHTML = otherAlerts.map(alert => renderAlert(alert)).join('');
                }}
            }} catch (error) {{
                console.error('Error loading initial alerts:', error);
                const alertsContainer = document.getElementById('alertsContainer');
                const deploymentsContainer = document.getElementById('deploymentsContainer');
                alertsContainer.innerHTML = '<div class="empty-state">Error loading alerts: ' + error.message + '</div>';
                deploymentsContainer.innerHTML = '<div class="empty-state">Error loading deployments: ' + error.message + '</div>';
            }}
        }}
        
        function setupSSE() {{
            const alertsContainer = document.getElementById('alertsContainer');
            const deploymentsContainer = document.getElementById('deploymentsContainer');
            const eventSource = new EventSource('/api/monitor/alerts/stream');
            
            eventSource.onmessage = function(event) {{
                try {{
                    const alert = JSON.parse(event.data);
                    if (alert && alert.tx_hash) {{
                        const category = alert.category || 'Other';
                        const isDeployment = category === 'Deployment';
                        const targetContainer = isDeployment ? deploymentsContainer : alertsContainer;
                        
                        // Prepend new alert to the top
                        const currentContent = targetContainer.innerHTML;
                        if (currentContent.includes('empty-state')) {{
                            targetContainer.innerHTML = renderAlert(alert);
                        }} else {{
                            targetContainer.innerHTML = renderAlert(alert) + currentContent;
                        }}
                        console.log('[SSE] Received new alert:', alert.tx_hash, 'Category:', category);
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
    "#
    );

    Html(html)
}


