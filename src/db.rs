use chrono::{DateTime, Utc};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::env;

use crate::db::secrets::get_secret;

mod secrets;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ChatMessage {
    #[serde(skip_serializing)]
    pub message_id: i64,
    pub user_id: i32,
    pub username: String,
    pub content: String,
    #[serde(skip_serializing)]
    pub created_at: std::option::Option<DateTime<Utc>>,
}

impl<'de> serde::Deserialize<'de> for ChatMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct RawChatMessage {
            user_id: i32,
            username: String,
            content: String,
        }

        let raw = RawChatMessage::deserialize(deserializer)?;
        Ok(ChatMessage {
            message_id: 0,
            user_id: raw.user_id,
            username: raw.username,
            content: raw.content,
            created_at: None,
        })
    }
}

pub async fn create_pool() -> Result<MySqlPool, sqlx::Error> {
    
    let database_url = get_secret(env::var("DATABASE_URL_NAME")
        .expect("The name of the secret containing the full database url must be passed").as_str());

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}