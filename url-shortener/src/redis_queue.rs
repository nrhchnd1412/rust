use deadpool_redis::{Config,Pool};
use redis::AsyncCommands;

pub async fn create_pool(redis_url:&str)->anyhow::Result<Pool>{
    let cfg = Config::from_url(redis_url);
    let pool=cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    Ok(pool)
}