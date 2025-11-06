# Rust HTTP Server

A simple, fast HTTP server built with Rust and Axum.

## API

- `GET /api/marco` - Returns JSON: `{"message": "polo"}`

## Running Locally

```bash
cargo run
```

The server will start on port 3000 (or the PORT environment variable if set).

## Deployment

This project is designed to be deployed on Railway. The server automatically reads the PORT environment variable set by Railway.

## Testing

```bash
curl http://localhost:3000/api/marco
# Returns: {"message":"polo"}
```
