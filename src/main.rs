use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::env;
use std::net::SocketAddr;

#[derive(Serialize)]
struct PoloResponse {
    message: String,
}

async fn marco() -> Json<PoloResponse> {
    Json(PoloResponse {
        message: "polo".to_string(),
    })
}

#[tokio::main]
async fn main() {
    // Build the application with a route
    let app = Router::new()
        .route("/api/marco", get(marco));

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
