mod api;
mod worker;
mod db;
mod redis_queue;

#[tokio::main]
async fn main() ->anyhow::Result<()>{
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
    tracing::info!("Starting server");
    api::run().await?;
    Ok(())
}
