# Rust HTTP Server

A simple, fast HTTP server built with Rust and Axum.

## API

- `GET /api/marco` - Returns JSON: `{"message": "polo"}`
- `GET /api/getCurrentBlock` - Returns current Base mainnet block number: `{"block_number":"0x241337e","block_number_decimal":37827454}`
- `POST /api/deployContract` - Deploys a smart contract to Base mainnet

## Running Locally

```bash
cargo run
```

The server will start on port 3000 (or the PORT environment variable if set).

## Deployment

This project is designed to be deployed on Railway. The server automatically reads the PORT environment variable set by Railway.

## Testing

```bash
# Test the marco endpoint
curl http://localhost:3000/api/marco
# Returns: {"message":"polo"}

# Test the getCurrentBlock endpoint
curl http://localhost:3000/api/getCurrentBlock
# Returns: {"block_number":"0x241337e","block_number_decimal":37827454}

# Test the deployContract endpoint (requires DEPLOYER_PRIVATE_KEY env var)
curl -X POST http://localhost:3000/api/deployContract \
  -H "Content-Type: application/json" \
  -d '{"bytecode": "0x608060405234801561001057600080fd5b5060df8061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c80636d4ce63c14602d575b600080fd5b60336049565b604051603e91906067565b60405180910390f35b60006001905090565b6000819050919050565b6061816055565b82525050565b6000602082019050607a6000830184605a565b9291505056fea2646970667358221220c0f1c8e3c8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b8b64736f6c63430008060033"}'
# Returns: {"contract_address":"0x...", "transaction_hash":"0x...", "block_number":"..."}
```
