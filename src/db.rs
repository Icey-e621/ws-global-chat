use chrono::{DateTime, Utc};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::env;

use crate::db::secrets::get_secret;

mod secrets;

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub message_id: i64,
    pub username: String,
    pub content: String,
    pub created_at: std::option::Option<DateTime<Utc>>,
}

pub async fn create_pool() -> Result<MySqlPool, sqlx::Error> {
    
    let database_url = get_secret(env::var("DATABASE_URL_NAME")
        .expect("The name of the secret containing the full database url must be passed").as_str());

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}