use thiserror::Error;
use serde_json::json;
use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status,message)=match &self{
            AppError::Db(e)=>{
                tracing::error!("DB error {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR,self.to_string())
            }
            AppError::NotFound => (StatusCode::NOT_FOUND,self.to_string()),
            AppError::Validation(_)=>(StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR,self.to_string()),
        };
        let body = Json(json!({
            "error": message,
        }));
        (status, body).into_response()
    }
}