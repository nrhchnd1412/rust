mod config;
mod db;
mod errors;
mod models;
mod repositories;
mod handlers;
mod routes;
use db::create_pool;


#[tokio::main]
async fn main()->anyhow::Result<()>{
    let settings= config::Settings::from_env();
    println!("{:?}", settings);
    let pool = create_pool(&settings.database_url).await?;
    let app = routes::router(pool.clone());
    let host = settings.host;
    let port = settings.port;
    println!("Host {}, port {}", host, port);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}",host,port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
