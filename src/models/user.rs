use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub password: String,
    pub preference: Option<String>,
    pub weight_unit: Option<String>,
    pub height_unit: Option<String>,
    pub weight: Option<f64>,
    pub height: Option<f64>,
    pub name: Option<String>,
    pub image_uri: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub struct GetUserPassword {
    pub password: String,
}

pub struct GetUserProfile {
    pub preference: Option<String>,
    pub weight_unit: Option<String>,
    pub height_unit: Option<String>,
    pub weight: Option<f64>,
    pub height: Option<f64>,
    pub name: Option<String>,
    pub image_uri: Option<String>,
}

pub struct GetUserId {
    pub user_id: Uuid,
}