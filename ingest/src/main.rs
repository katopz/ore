use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use ore_api::state::Config;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::collections::HashMap;
use std::panic;
use std::sync::Arc;
use tokio::net::TcpListener;
use turso::{Builder, Connection};

#[derive(Clone)]
pub struct AppState {
    pub db_url: String,
    pub db: Arc<Connection>,
    pub solana_client: Arc<RpcClient>,
    pub ore_config: Arc<Config>,
}

/// Initialize API routes
pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/test", get(test_endpoint))
        .route("/list", get(list_records))
        .route("/solana", get(test_solana))
        .route("/ore", get(test_ore_api))
}

/// Health check endpoint
async fn health_check() -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("service".to_string(), "ore-ingest".to_string());
    response.insert("phase".to_string(), "4-full-stack".to_string());
    Ok(Json(response))
}

/// Test endpoint
async fn test_endpoint() -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Axum + Turso + Solana + ore-api test works!".to_string(),
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

/// Test Solana connection
async fn test_solana(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("message".to_string(), "Solana test endpoint".to_string());

    // Test Solana connection
    match state.solana_client.get_latest_blockhash() {
        Ok(blockhash) => {
            response.insert("solana_status".to_string(), "connected".to_string());
            response.insert("latest_blockhash".to_string(), blockhash.to_string());
        }
        Err(e) => {
            response.insert("solana_status".to_string(), format!("error: {}", e));
        }
    }

    Ok(Json(response))
}

/// Test ore-api integration
async fn test_ore_api(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("message".to_string(), "ORE API test endpoint".to_string());

    // Test ore-api configuration access
    response.insert("ore_status".to_string(), "loaded".to_string());
    response.insert(
        "config_admin".to_string(),
        state.ore_config.admin.to_string(),
    );
    response.insert(
        "config_fee_collector".to_string(),
        state.ore_config.fee_collector.to_string(),
    );

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

/// Initialize Solana client
fn init_solana_client() -> Result<Arc<RpcClient>> {
    println!("🔗 Initializing Solana client...");

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    println!("📍 Using Solana RPC URL: {}", rpc_url);

    let client = Arc::new(RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));
    println!("✅ Solana client initialized: {}", rpc_url);

    Ok(client)
}

/// Initialize ore-api configuration
fn init_ore_config() -> Result<Arc<Config>> {
    println!("⛏️  Initializing ORE API configuration...");

    // Create a default ore configuration for testing
    let config = Config {
        admin: solana_sdk::pubkey::Pubkey::default(),
        bury_authority: solana_sdk::pubkey::Pubkey::default(),
        fee_collector: solana_sdk::pubkey::Pubkey::default(),
        swap_program: solana_sdk::pubkey::Pubkey::default(),
        var_address: solana_sdk::pubkey::Pubkey::default(),
        buffer: 0,
    };

    let config = Arc::new(config);
    println!("✅ ORE API configuration initialized");

    Ok(config)
}

/// Start the API server
pub async fn start_api_server(db_url: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing database...");
    let db = init_database(&db_url).await?;
    println!("✅ Database initialized successfully");

    println!("🔧 Initializing Solana client...");
    let solana_client = init_solana_client()?;
    println!("✅ Solana client initialized successfully");

    println!("🔧 Initializing ORE API configuration...");
    let ore_config = init_ore_config()?;
    println!("✅ ORE API configuration initialized successfully");

    println!("🔧 Creating app state...");
    let app_state = AppState {
        db_url,
        db,
        solana_client,
        ore_config,
    };

    println!("🔧 Creating API routes...");
    let app = create_api_routes().with_state(app_state);

    println!("🔧 Binding to port {}...", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("✅ Bound to port {}", port);

    println!("🚀 API server started on http://0.0.0.0:{}", port);

    println!("🔧 Starting axum server...");
    axum::serve(listener, app).await?;
    println!("✅ Server completed");
    Ok(())
}

/// Set up panic hook to capture any panics
fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("🚨 PANIC OCCURRED:");
        eprintln!(
            "  Location: {}",
            panic_info
                .location()
                .unwrap_or_else(|| panic::Location::caller())
        );
        eprintln!("  Message: {}", panic_info);
        eprintln!("  Backtrace:");
        eprintln!("{:?}", backtrace::Backtrace::new());
    }));
}

/// Main function with API features enabled
#[tokio::main]
async fn main() -> Result<()> {
    // Set up panic hook first
    setup_panic_hook();

    println!("🚀 Starting Phase 4: Full Stack (Axum + Turso + Solana + ORE API) debug");
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

    println!("🎯 Phase 4: Full Stack test");
    println!("📍 Port: {}", port);
    println!("🗄️  Database URL: {}", db_url);
    println!("🔧 About to start API server with database, Solana, and ORE API...");

    // Just call start_api_server directly since we're already in tokio runtime
    if let Err(e) = start_api_server(db_url, port).await {
        eprintln!("❌ Failed to start API server: {}", e);
        return Err(anyhow::anyhow!("API server failed: {}", e));
    }

    println!("✅ API server started successfully");
    Ok(())
}
