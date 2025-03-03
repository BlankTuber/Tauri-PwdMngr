use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    #[sqlx(rename = "id")]
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Password {
    #[sqlx(rename = "id")]
    pub id: String,
    pub user_id: String,
    pub website: String,
    pub website_url: Option<String>,
    pub encrypted_username: String,
    pub encrypted_password: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}