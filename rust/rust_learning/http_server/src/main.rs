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
// Import tracing macros and subscriber
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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
        let error_message = match &self {
            AppError::InvalidInput(msg) => msg,
            AppError::NotFound(msg) => msg,
            AppError::ServerError(msg) => msg,
        };
        
        // Log the error
        error!(?self, "Request error occurred");
        
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
    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // Default to info level if RUST_LOG env var is not set
            "http_server=info,tower_http=debug,axum=debug".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting HTTP server");
    
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
        .layer(middleware::from_fn_with_state(state.clone(), logging_middleware))
        .with_state(state);

    // Create a TCP listener bound to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!(%addr, "Server listening");

    // Start the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

// Handler functions for each route

// Root path - return a welcome message
#[instrument(skip_all)]
async fn root_handler() -> Html<&'static str> {
    info!("Handling root request");
    Html("Hello! Welcome to our Rust HTTP server")
}

// /hello path - return a different message
#[instrument(skip_all)]
async fn hello_handler() -> Html<&'static str> {
    info!("Handling hello request");
    Html("Hello, World!")
}

// /info path - return some server info
#[instrument(skip_all)]
async fn info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let count = *state.request_count.lock().unwrap();
    info!(request_count = count, "Handling info request");

    let info = format!("Server running on Rust 🦀\nProcessed {} requests", count);
    Html(info)
}

// /submit path - handle POST requests with JSON body
#[instrument(skip_all, fields(start_cursor))]
async fn submit_handler(Json(body): Json<Body>) -> impl IntoResponse {
    // Add relevant fields to the span
    tracing::Span::current().record("start_cursor", &body.start_cursor);
    
    info!("Received POST request with JSON body");
    debug!(?body, "Request body details");
    
    let response = ApiResponse {
        success: true,
        data: Some(body),
        error: None,
    };
    
    info!("Returning successful response");
    (StatusCode::OK, Json(response))
}

// /items - demonstrate query parameters
#[instrument(skip_all, fields(limit, offset))]
async fn get_items(Query(params): Query<FilterParams>) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    
    // Add fields to the current span
    tracing::Span::current()
        .record("limit", &limit)
        .record("offset", &offset);
    
    info!("Getting items with pagination");
    
    // Simulate items retrieval
    let items: Vec<String> = (offset..offset + limit)
        .map(|i| format!("Item {}", i))
        .collect();
    
    debug!(item_count = items.len(), "Items retrieved");
    
    let response = ApiResponse {
        success: true,
        data: Some(items),
        error: None,
    };
    
    Ok((StatusCode::OK, Json(response)))
}

// /items/:id - demonstrate path parameters
#[instrument(skip_all, fields(item_id = %id))]
async fn get_item_by_id(Path(id): Path<String>) -> Result<impl IntoResponse, AppError> {
    info!("Looking up item");
    
    // Simulate database lookup
    if id == "42" {
        info!("Item found");
        let response = ApiResponse {
            success: true,
            data: Some(format!("Item details for ID: {}", id)),
            error: None,
        };
        Ok((StatusCode::OK, Json(response)))
    } else {
        warn!("Item not found");
        Err(AppError::NotFound(format!("Item with ID {} not found", id)))
    }
}

// Middleware for logging requests
#[instrument(skip_all, fields(method, path))]
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
        debug!(request_count = *count, "Incremented request counter");
    }

    // Log the request
    let path = request.uri().path().to_owned();
    let method = request.method().clone();
    
    // Add values to the current tracing span
    tracing::Span::current()
        .record("method", &method.as_str())
        .record("path", &path);
    
    info!("Request received");
    
    // Optional: Log certain headers
    if let Some(user_agent) = headers.get("user-agent") {
        if let Ok(ua_str) = user_agent.to_str() {
            debug!(user_agent = ua_str, "User agent header");
        }
    }

    // Process the request and get the response
    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();
    
    // Log the response status
    info!(
        status = response.status().as_u16(),
        latency = ?duration,
        "Response sent"
    );
    
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
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received termination signal, starting graceful shutdown");
        },
    }
    
    // Give ongoing requests a moment to complete
    info!("Waiting for in-flight requests to complete");
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Shutdown complete");
}
