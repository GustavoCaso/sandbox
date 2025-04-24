use std::fmt;
use std::net::SocketAddr;

use axum::{
    Router,
    extract::Json,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

// Define a struct to represent the JSON data
#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(rename(deserialize = "start_cursor", serialize = "startCursor"))]
    start_cursor: String,
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "Error serializing Body"),
        }
    }
}

#[tokio::main]
async fn main() {
    // Create a router with our routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/hello", get(hello_handler))
        .route("/info", get(info_handler))
        .route("/submit", post(submit_handler));

    // Create a TCP listener bound to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server listening on {}", addr);

    // Start the server with the router
    axum::serve(listener, app).await.unwrap();
}

// Handler functions for each route

// Root path - return a welcome message
async fn root_handler() -> Html<&'static str> {
    Html("Hello! Welcome to our Rust HTTP server")
}

// /hello path - return a different message
async fn hello_handler() -> Html<&'static str> {
    Html("Hello, World!")
}

// /info path - return some server info
async fn info_handler() -> Html<&'static str> {
    Html("Server running on Rust 🦀")
}

// /submit path - handle POST requests with JSON body
async fn submit_handler(Json(body): Json<Body>) -> impl IntoResponse {
    // Successfully parsed JSON data
    println!("Received valid JSON body: {:?}", body);
    (StatusCode::OK, Json(body))
}
