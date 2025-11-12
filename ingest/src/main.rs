use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use turso::{Builder, Connection};

#[derive(Clone)]
pub struct AppState {
    pub db_url: String,
    pub db: Arc<Connection>,
}

/// Initialize API routes
pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/test", get(test_endpoint))
        .route("/list", get(list_records))
}

/// Health check endpoint
async fn health_check() -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("service".to_string(), "ore-ingest".to_string());
    response.insert("phase".to_string(), "2-axum-turso".to_string());
    Ok(Json(response))
}

/// Test endpoint
async fn test_endpoint() -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Axum + Turso test works!".to_string(),
    );
    response.insert(
        "timestamp".to_string(),
        chrono::Utc::now().to_rfc3339().to_string(),
    );
    Ok(Json(response))
}

/// List records from database
async fn list_records(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("message".to_string(), "Database test endpoint".to_string());
    response.insert("db_url".to_string(), state.db_url.clone());

    // Test database connection with a simple query
    match state.db.execute("SELECT 1", ()).await {
        Ok(_) => {
            response.insert("db_status".to_string(), "connected".to_string());
        }
        Err(e) => {
            response.insert("db_status".to_string(), format!("error: {}", e));
        }
    }

    Ok(Json(response))
}

/// Initialize database connection
async fn init_database(db_url: &str) -> Result<Arc<Connection>> {
    println!("🗄️  Initializing database connection to: {}", db_url);

    let db = Builder::new_local(db_url).build().await?;

    let conn = db.connect()?;
    let conn = Arc::new(conn);
    println!("✅ Database connection established");

    // Create test table if it doesn't exist
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS test_records (
            id TEXT PRIMARY KEY,
            created_at TEXT NOT NULL,
            message TEXT
        )
    "#;

    conn.execute(create_table_sql, ()).await?;
    println!("✅ Database table initialized");

    Ok(conn)
}

/// Start the API server
pub async fn start_api_server(db_url: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing database...");
    let db = init_database(&db_url).await?;

    let app_state = AppState { db_url, db };

    let app = create_api_routes().with_state(app_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🚀 API server started on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Main function with API features enabled
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Phase 2: Axum + Turso debug");
    println!("📍 Working directory: {:?}", std::env::current_dir());
    println!("🔧 Environment variables:");
    for (key, value) in std::env::vars() {
        println!("  {}={}", key, value);
    }

    // Use environment variable for port, fallback to 8080
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "test.db".to_string());

    println!("🎯 Phase 2: Axum + Turso test");
    println!("📍 Port: {}", port);
    println!("🗄️  Database URL: {}", db_url);
    println!("🔧 About to start API server with database...");

    if let Err(e) = start_api_server(db_url, port).await {
        eprintln!("❌ Failed to start API server: {}", e);
        return Err(anyhow::anyhow!("API server failed: {}", e));
    }

    println!("✅ API server started successfully");
    Ok(())
}
