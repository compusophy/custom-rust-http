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

cargo watch -x run

