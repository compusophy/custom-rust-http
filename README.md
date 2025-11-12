# Rust HTTP Server

A simple, fast HTTP server built with Rust and Axum.

## API

- `GET /api/marco` - Returns JSON: `{"message": "polo"}`
- `GET /api/redis-test` - Tests Redis connection by storing and retrieving a value
- `GET /api/getCurrentBlock` - Returns current Base mainnet block number: `{"block_number":"0x241337e","block_number_decimal":37827454}`
- `GET /api/docs` or `GET /docs` - Returns API documentation with all available endpoints

## Running Locally

```bash
cargo run
.\dev.ps1
```

The server will start on port 3000 (or the PORT environment variable if set).

### Redis Setup

The server uses Redis for persistent storage. Set up environment variables:

**Option 1: Use REDIS_URL (recommended)**
```bash
REDIS_URL=redis://:password@host:port
```

**Option 2: Use individual variables (for Railway)**
```bash
REDIS_HOST=redis.railway.internal
REDIS_PORT=6379
REDIS_PASSWORD=your_password_here
```

For Railway deployment, use the private internal network (`redis.railway.internal`) for better performance and security. These variables are automatically available when you link your Redis service in Railway.

## Deployment

This project is designed to be deployed on Railway. The server automatically reads the PORT environment variable set by Railway.

## Testing

```bash
# Test the marco endpoint
curl http://localhost:3000/api/marco
# Returns: {"message":"polo"}

# Test Redis connection
curl http://localhost:3000/api/redis-test
# Returns: {"key":"rust_app:test","value":"Hello from Rust! This is persistent storage.","message":"Successfully stored and retrieved from Redis!"}

# Test the getCurrentBlock endpoint
curl http://localhost:3000/api/getCurrentBlock
# Returns: {"block_number":"0x241337e","block_number_decimal":37827454}

# View API documentation
curl http://localhost:3000/api/docs
# or
curl http://localhost:3000/docs
# Returns: Complete API documentation JSON
```
