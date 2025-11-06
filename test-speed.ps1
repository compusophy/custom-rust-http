# Simple API speed test for getCurrentBlock endpoint
$url = "https://custom-rust-http-production.up.railway.app/api/getCurrentBlock"
$count = 10

Write-Host "Testing Rust API Speed: $url"
Write-Host "Running $count requests..."
Write-Host ""

$times = @()
$totalTime = 0
$successCount = 0

for ($i = 1; $i -le $count; $i++) {
    $start = Get-Date
    try {
        $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 10
        $end = Get-Date
        $duration = ($end - $start).TotalMilliseconds

        $times += $duration
        $totalTime += $duration
        $successCount++

        Write-Host "Request $i : $([math]::Round($duration, 2))ms - $($response.StatusCode)"
    }
    catch {
        Write-Host "Request $i : FAILED - $($_.Exception.Message)"
    }
}

if ($successCount -gt 0) {
    $avgTime = $totalTime / $successCount
    $minTime = ($times | Measure-Object -Minimum).Minimum
    $maxTime = ($times | Measure-Object -Maximum).Maximum

    Write-Host ""
    Write-Host "Results:"
    Write-Host "Average: $([math]::Round($avgTime, 2))ms"
    Write-Host "Min: $([math]::Round($minTime, 2))ms"
    Write-Host "Max: $([math]::Round($maxTime, 2))ms"
    Write-Host "Success: $successCount/$count requests"
}
else {
    Write-Host "All requests failed!"
}
