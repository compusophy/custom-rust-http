# Hot reloading dev script for Rust HTTP server
# This will watch for file changes and automatically rebuild and restart the server

Write-Host "Killing processes on port 3000..." -ForegroundColor Yellow

# Find and kill processes using port 3000
$processes = netstat -ano | findstr :3000
if ($processes) {
    $processes | ForEach-Object {
        $parts = $_ -split '\s+'
        $processId = $parts[-1]
        if ($processId -match '^\d+$') {
            Write-Host "Killing process $processId on port 3000..." -ForegroundColor Red
            taskkill /PID $processId /F 2>$null
        }
    }
    Start-Sleep -Seconds 1
    Write-Host "Port 3000 cleared!" -ForegroundColor Green
} else {
    Write-Host "No processes found on port 3000" -ForegroundColor Green
}

Write-Host "`nStarting hot reloading dev server..." -ForegroundColor Green
Write-Host "Make sure cargo-watch is installed: cargo install cargo-watch" -ForegroundColor Yellow

# Load .env file if it exists
if (Test-Path .env) {
    Write-Host "Loading .env file..." -ForegroundColor Cyan
    Get-Content .env | ForEach-Object {
        if ($_ -match '^\s*([^#][^=]*)=(.*)$') {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            [Environment]::SetEnvironmentVariable($name, $value, "Process")
        }
    }
}

# Check if REDIS_URL is set
if (-not $env:REDIS_URL) {
    Write-Host "`nWARNING: REDIS_URL not set!" -ForegroundColor Red
    Write-Host "Set REDIS_URL with your Railway Redis connection string:" -ForegroundColor Yellow
    Write-Host "  Example: rediss://:password@hopper.proxy.rlwy.net:29794" -ForegroundColor Yellow
    Write-Host "  Or set REDIS_HOST, REDIS_PORT, and REDIS_PASSWORD" -ForegroundColor Yellow
    Write-Host ""
}

cargo watch -x run

