use pms_platform::{auth, create_app, db};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env if present (silently ok if missing)
    dotenvy::dotenv().ok();

    // Validate JWT_SECRET early so fail-fast happens before any I/O.
    // This call will process::exit(1) in production if the secret is unset.
    let _secret = auth::jwt_secret();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pms_platform=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create uploads directory
    tokio::fs::create_dir_all(pms_platform::upload_dir())
        .await
        .expect("Failed to create uploads directory");

    // Create static directory (where frontend/static pages live)
    tokio::fs::create_dir_all("frontend/static")
        .await
        .expect("Failed to create frontend/static directory");

    // Initialize database
    let pool = db::init_pool().await;
    db::run_migrations(&pool).await;
    db::seed_data(&pool).await;

    // Build application router
    let app = create_app(pool);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    tracing::info!("🚀 PMS Platform running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server error");
}
