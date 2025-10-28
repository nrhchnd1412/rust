// src/main.rs
use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

use rate_limiter::{BucketConfig, InMemoryLimiter, RateLimiter};

#[tokio::main]
async fn main() ->anyhow::Result<()>{
    // create limiter: 5 tokens/sec, burst 10
    let cfg = BucketConfig { rate: 5.0, capacity: 10.0 };
    let limiter = Arc::new(InMemoryLimiter::new(cfg));
    let app = Router::new()
        .route("/", get(root))
        .layer(Extension(limiter));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}","127.0.0.1","3000")).await?;
    println!("listening on {}", "localhost");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root(Extension(limiter): Extension<Arc<InMemoryLimiter>>) -> impl IntoResponse {
    let key = "global"; // in real cases use IP/api-key/user-id
    let ok = limiter.try_acquire(key, 1).await;
    if ok {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::TOO_MANY_REQUESTS, "rate limited")
    }
}
