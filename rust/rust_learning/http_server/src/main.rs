use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{
    Router,
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::signal;

// Define a struct to represent the JSON data
#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(rename(deserialize = "start_cursor", serialize = "startCursor"))]
    start_cursor: String,
}

// Custom response struct for standardized JSON responses
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// Custom error type
#[derive(Debug)]
enum AppError {
    InvalidInput(String),
    NotFound(String),
    ServerError(String),
}

// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    request_count: Arc<Mutex<usize>>,
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "Error serializing Body"),
        }
    }
}

// Implement IntoResponse for AppError for easy error handling
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::ServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let api_response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(error_message),
        };

        (status, Json(api_response)).into_response()
    }
}

// Query parameters struct
#[derive(Deserialize)]
struct FilterParams {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[tokio::main]
async fn main() {
    // Initialize shared state
    let state = AppState {
        request_count: Arc::new(Mutex::new(0)),
    };

    // Create a router with our routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/hello", get(hello_handler))
        .route("/info", get(info_handler))
        .route("/items", get(get_items))
        .route("/items/:id", get(get_item_by_id))
        .route("/submit", post(submit_handler))
        // Apply middleware to all routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            logging_middleware,
        ))
        .with_state(state);

    // Create a TCP listener bound to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server listening on {}", addr);

    // Start the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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
async fn info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let count = *state.request_count.lock().unwrap();

    let info = format!("Server running on Rust 🦀\nProcessed {} requests", count);
    Html(info)
}

// /submit path - handle POST requests with JSON body
async fn submit_handler(Json(body): Json<Body>) -> impl IntoResponse {
    // Successfully parsed JSON data
    println!("Received valid JSON body: {:?}", body);

    let response = ApiResponse {
        success: true,
        data: Some(body),
        error: None,
    };

    (StatusCode::OK, Json(response))
}

// /items - demonstrate query parameters
async fn get_items(Query(params): Query<FilterParams>) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    // Simulate items retrieval
    let items: Vec<String> = (offset..offset + limit)
        .map(|i| format!("Item {}", i))
        .collect();

    let response = ApiResponse {
        success: true,
        data: Some(items),
        error: None,
    };

    Ok((StatusCode::OK, Json(response)))
}

// /items/:id - demonstrate path parameters
async fn get_item_by_id(Path(id): Path<String>) -> Result<impl IntoResponse, AppError> {
    // Simulate database lookup
    if id == "42" {
        let response = ApiResponse {
            success: true,
            data: Some(format!("Item details for ID: {}", id)),
            error: None,
        };
        Ok((StatusCode::OK, Json(response)))
    } else {
        Err(AppError::NotFound(format!("Item with ID {} not found", id)))
    }
}

// Middleware for logging requests
async fn logging_middleware(
    headers: HeaderMap,
    State(state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Increment request counter
    {
        let mut count = state.request_count.lock().unwrap();
        *count += 1;
    }

    // Log the request
    let path = request.uri().path().to_owned();
    let method = request.method().clone();
    println!("[REQUEST] {} {}", method, path);

    // Optional: Log certain headers
    if let Some(user_agent) = headers.get("user-agent") {
        println!("[USER-AGENT] {:?}", user_agent);
    }

    // Process the request and get the response
    let response = next.run(request).await;

    // Log the response status
    println!("[RESPONSE] Status: {}", response.status());

    response
}

// Signal handler for graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            println!("Received termination signal, starting graceful shutdown");
        },
    }

    // Give ongoing requests a moment to complete
    tokio::time::sleep(Duration::from_secs(1)).await;
}
