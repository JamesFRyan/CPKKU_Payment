use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Liveness/readiness only for now (dev prompt section 34). Readiness will
    // start checking PostgreSQL/Redis/RabbitMQ once those adapters land.
    let app = Router::new()
        .route("/health/live", get(|| async { "ok" }))
        .route("/health/ready", get(|| async { "ok" }));

    let addr = std::env::var("PAYMENT_API_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind payment-api listener");

    tracing::info!(%addr, "payment-api skeleton listening");

    axum::serve(listener, app)
        .await
        .expect("payment-api server error");
}
