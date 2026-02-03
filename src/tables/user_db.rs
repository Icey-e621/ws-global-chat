use argon2::{
    Argon2, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::ChatMessage;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,          // maps to INT
    pub username: String, // maps to VARCHAR
    #[serde(skip_serializing)]
    pub password_hash: String, // maps to VARCHAR
    pub created_at: DateTime<Utc>, // maps to TIMESTAMP
}

pub async fn get_chat_history(
    pool: &sqlx::MySqlPool,
    limit: i32,
) -> Result<Vec<ChatMessage>, sqlx::Error> {
    sqlx::query_as!(
        ChatMessage,
        r#"
        SELECT 
            m.id as message_id, 
            u.username, 
            m.content, 
            m.created_at
        FROM messages m
        JOIN app_users u ON m.user_id = u.id
        ORDER BY m.created_at ASC
        LIMIT ?
        "#,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn find_user_by_username(
    pool: &sqlx::MySqlPool,
    name: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, username, password_hash, created_at FROM app_users WHERE username = ?",
        name
    )
    .fetch_one(pool)
    .await
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Error hashing password")
        .to_string()
}

pub fn verify_password(password: &str, hashstr: &str) -> Result<(), argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let hash = match argon2::PasswordHash::parse(hashstr, argon2::password_hash::Encoding::B64) {
        Ok(parsed_hash) => {parsed_hash},
        Err(e) => {
            print!("error: couldnt parse hash: {:?}",hashstr);
            return Err(argon2::password_hash::Error::PhcStringField);
        }
    };
    argon2.verify_password(password.as_bytes(), &hash)
}

pub async fn create_user(
    pool: &sqlx::MySqlPool,
    username: &str,
    raw_password: &str,
) -> Result<u64, sqlx::Error> {
    // 1. Hash the password using the function we talked about earlier
    let hashed_password = hash_password(raw_password);

    // 2. Insert into the database
    let result = sqlx::query!(
        r#"
        INSERT INTO app_users (username, password_hash)
        VALUES (?, ?)
        "#,
        username,
        hashed_password
    )
    .execute(pool)
    .await?;

    // Returns the number of rows affected (should be 1)
    Ok(result.rows_affected())
}
