use axum::{
    extract::{Json},
    Router, routing::post, routing::get,
    response::Redirect,
    http::StatusCode,
    extract::Path,
    Extension,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::create_pool;
use crate::redis_queue::{create_pool as create_redis_pool};

use std::net::SocketAddr;
use axum::response::IntoResponse;

#[derive(Clone)]
struct AppState {
    db_pool: sqlx::PgPool,
    redis_pool:deadpool_redis::Pool,
    base_url:String,
}

#[derive(Deserialize)]
pub struct CreateReq{
    pub url:String,
    pub custom_alias: Option<String>,
}

#[derive(Serialize)]
pub struct CreateResp{
    short_url: String,
    code: String
}

#[derive(FromRow)]
struct UrlRow{
    id: i64,
    short_code: String,
    original_url: String,
    is_deleted: bool,
    expired_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub async fn run()->anyhow::Result<()>{
    let database_url=std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://shorty:shorty@localhost:5432/shorty".into());
    let redis_url=std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let base_url=std::env::var("BASE_URL").unwrap_or_else(|_| "http://mytinyurl.com".into());

    let db_pool=create_pool(&database_url).await?;
    let redis_pool=create_redis_pool(&redis_url).await?;
    let state = AppState{db_pool,redis_pool,base_url };
    let app = Router::new()
        .route("/api/shorten",post(create_short))
        // .route("/api/info/:code",get(info))
        // .route("/:code",get(redirect_code))
        .layer(Extension(state));
    let host = "127.0.0.1";
    let port ="3000";
    tracing::info!("listening on {}:{}", host,port);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}",host,port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn create_short(Extension(state): Extension<AppState>,Json(payload):Json<CreateReq>) -> impl IntoResponse {
    //validate url
    let url = match url::Url::parse(&payload.url){
        Ok(url) => url.to_string(),
        Err(_)=>return (StatusCode::BAD_REQUEST, "invalid url").into_response(),
    };
    // First, check if URL already exists
    if let Ok(Some(existing)) = sqlx::query!(
        "SELECT short_code FROM urls WHERE original_url = $1",
        url
    )
        .fetch_optional(&state.db_pool)
        .await
    {
        let short_url = format!("{}/{}", state.base_url.trim_end_matches('/'), existing.short_code);
        return (StatusCode::OK, axum::Json(CreateResp { short_url, code: existing.short_code })).into_response();
    }

    //handle custom alias or random
    let code = if let Some(alias)=payload.custom_alias{
        //attempt insert
        let res = sqlx::query!(
            r#"INSERT into urls (short_code,original_url) VALUES ($1,$2) RETURNING id"#,
            alias,
            url
        ).fetch_optional(&state.db_pool).await;
        match res{
            Ok(_)=>alias,
            Err(_)=>return (StatusCode::CONFLICT, "alias not available").into_response()
        }
    }else{
        //generate random code and insert
        let code= random_base62(7);
        let _ = sqlx::query!(
            r#"INSERT into urls (short_code, original_url) VALUES ($1,$2)"#,
            code,
            url
        ).execute(&state.db_pool)
            .await;
        code
    };
    let short_url = format!("{}/{}",state.base_url.trim_end_matches('/'),code);
    let resp = CreateResp{short_url,code};
    (StatusCode::CREATED, Json(resp)).into_response()
}

fn random_base62(len: usize) -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut buf = vec![0u8; len];
    let mut rng = rand::rngs::OsRng;
    use rand::RngCore;
    rng.fill_bytes(&mut buf);
    buf.iter().map(|b|CHARS[(*b as usize)%CHARS.len()] as char).collect()
}