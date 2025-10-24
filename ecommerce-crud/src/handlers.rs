use crate::errors::AppError;
use crate::models::{CreateProduct,UpdateProduct};
use crate::repositories::ProductRepo;
use axum::{
    extract::{ Extension,Path,Query},
    Json
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_product(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateProduct>,
)->Result<Json<serde_json::Value>, AppError>{
    payload.validate().map_err(|e|AppError::Validation(e.to_string()))?;
    let repo=ProductRepo::new(&pool);
    let created = repo.create_product(payload).await?;
    Ok(Json(serde_json::json!({"data": created})))
}

pub async fn list_products(Extension(pool): Extension<PgPool>,Query(params):Query<ListParams>)->Result<Json<serde_json::Value>, AppError>{
    let repo=ProductRepo::new(&pool);
    let limit=params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let items=repo.list(limit, offset).await?;
    Ok(Json(serde_json::json!({"data":items})))
}

pub async fn get_product(Extension(pool): Extension<PgPool>,Path(product_id):Path<Uuid>)->Result<Json<serde_json::Value>, AppError>{
    let repo=ProductRepo::new(&pool);
    let item=repo.get(product_id).await?;
    Ok(Json(serde_json::json!({"data":item})))
}

pub async fn update_product(Extension(pool): Extension<PgPool>,Path(id):Path<Uuid>,Json(payload):Json<UpdateProduct>)->Result<Json<serde_json::Value>, AppError>{
    let repo=ProductRepo::new(&pool);
    let item=repo.update(id, payload).await?;
    Ok(Json(serde_json::json!({"data":item})))
}

pub async fn delete_product(Extension(pool): Extension<PgPool>,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>, AppError>{
    let repo=ProductRepo::new(&pool);
    repo.delete(id).await?;
    Ok(Json(serde_json::json!({"status":"deleted"})))
}