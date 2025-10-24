use axum::routing::{get, post, put, delete};
use axum::Router;
use sqlx::PgPool;
use crate::handlers::*;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/api/products",post(create_product).get(list_products))
        .route(
            "/api/products/{id}",
            get(get_product).put(update_product).delete(delete_product)
        )
        .layer(axum::Extension(pool))
}