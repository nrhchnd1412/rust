use crate::errors::AppError;
use crate::models::{CreateProduct,UpdateProduct,Product};
use sqlx::{PgPool};
use uuid::Uuid;

pub struct  ProductRepo<'a>{
    pool: &'a PgPool,
}

impl<'a> ProductRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self{pool}
    }

    pub async fn create_product(&self, item: CreateProduct) -> Result<Product, AppError> {
        let rec=sqlx::query_as::<_,Product>(
            r#"
                    INSERT INTO products (id,name,description,price_cents,sku,created_at,updated_at)
                    values ($1,$2,$3,$4,$5,now(),now())
                    RETURNING *
                    "#,
            )
            .bind(Uuid::new_v4())
            .bind(item.name)
            .bind(item.description)
            .bind(item.price_cents)
            .bind(item.sku)
            .fetch_one(self.pool)
            .await?;
        Ok(rec)
    }

    pub async fn list(&self,limit:i64,offset:i64) -> Result<Vec<Product>, AppError> {
        let recs=sqlx::query_as::<_,Product>(
            r#"
                        select * from products
                        ORDER BY created_at DESC
                        limit $1 offset $2"#
            )
            .bind(limit)
        .bind(offset)
            .fetch_all(self.pool)
            .await?;
        Ok(recs)
    }

    pub async fn get(&self, id:Uuid) -> Result<Product, AppError> {
        let rec=sqlx::query_as::<_,Product>(
            r"
                   select * from products where id=$1"
            )
        .bind(id)
            .fetch_optional(self.pool)
            .await?;
        rec.ok_or(AppError::NotFound)
    }

    pub async fn update(&self, id:Uuid,input: UpdateProduct) -> Result<Product, AppError> {
        let mut prod = self.get(id).await?;
        if let Some(name) = input.name {
            prod.name = name;
        }
        if let Some(description) = input.description {
            prod.description = Some(description);
        }
        if let Some(price_cents) = input.price_cents {
            prod.price_cents = price_cents;
        }
        if let Some(sku) = input.sku {
            prod.sku = sku;
        }
        let rec=sqlx::query_as::<_,Product>(
            r#"update products
                 set name=$1, description=$2, price_cents=$3, sku=$4, updated_at=now()
                 where id=$5 returning *"#,
        )
            .bind(prod.name)
            .bind(prod.description)
            .bind(prod.price_cents)
            .bind(prod.sku)
            .fetch_one(self.pool)
            .await?;
        Ok(rec)
    }

    pub async fn delete(&self, id:Uuid) -> Result<(), AppError> {
        let res=sqlx::query("Delete from products where id=$1"
        )
            .bind(id)
            .execute(self.pool)
            .await?;
        if res.rows_affected()==0{
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}