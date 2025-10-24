use std::env;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings{
     pub database_url:String,
     pub host:String,
     pub port:u16,
}

impl Settings {
     pub fn from_env()->Self{
          dotenvy::dotenv().ok();
          //let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
          let database_url = "postgres://postgres:example@localhost:5432/postgres".into();
          let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
          let port = env::var("PORT").unwrap_or_else(|_| "3000".into()).parse().unwrap();
          Settings{
               database_url,
               host,
               port,
          }
     }
}