use serde::{Deserialize,Serialize};
use uuid::Uuid;
use chrono::{DateTime,Utc};
use validator::Validate;

#[derive(Debug,Serialize,sqlx::FromRow)]
pub struct Product{
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub sku: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug,Deserialize,Validate)]
pub struct CreateProduct{
    #[validate(length(min=1))]
    pub name: String,
    pub description: Option<String>,
    #[validate(range(min=0))]
    pub price_cents: i64,
    #[validate(length(min=1))]
    pub sku: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProduct {
    #[validate(length(min = 1))]
    pub name: Option<String>,

    pub description: Option<String>,

    #[validate(range(min = 0))]
    pub price_cents: Option<i64>,

    #[validate(length(min = 1))]
    pub sku: Option<String>,
}