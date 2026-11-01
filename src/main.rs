use pms_platform::{auth, create_app, db};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => signal.recv().await,
            Err(e) => {
                tracing::error!(error = %e, "Failed to install SIGTERM handler");
                std::future::pending::<Option<()>>().await
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Option<()>>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() {
    // Load .env if present (silently ok if missing)
    dotenvy::dotenv().ok();

    // Initialize tracing FIRST so all startup logs and warnings are captured
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pms_platform=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Validate JWT_SECRET early so fail-fast happens before any I/O.
    // This call will process::exit(1) in production if the secret is unset or insecure.
    let _secret = auth::jwt_secret();

    let is_production =
        std::env::var("RUST_ENV").unwrap_or_default().to_lowercase() == "production";
    if is_production && std::env::var("TRUST_PROXY").is_err() {
        tracing::warn!(
            "⚠️ TRUST_PROXY is unset in production mode. Reverse proxy IP headers will not be trusted."
        );
    }

    // Verify required frontend static files are present at runtime
    if !std::path::Path::new("frontend/static/index.html").exists() {
        eprintln!(
            "\n\
            ╔══════════════════════════════════════════════════════════════════════════╗\n\
            ║  FATAL: Required static file frontend/static/index.html is missing.      ║\n\
            ║  PMS platform requires the frontend/static directory to serve pages.   ║\n\
            ╚══════════════════════════════════════════════════════════════════════════╝\n"
        );
        std::process::exit(1);
    }

    // Create uploads directory
    tokio::fs::create_dir_all(pms_platform::upload_dir())
        .await
        .expect("Failed to create uploads directory");

    // Initialize database
    let pool = db::init_pool().await;
    db::run_migrations(&pool).await;
    db::seed_data(&pool).await;
    db::spawn_revoked_token_pruner(pool.clone());

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
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Server error");
}
