use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct Activity {
    pub activity_id: Uuid,
    pub user_id: Uuid,
    pub activity_type: String,
    pub done_at: chrono::DateTime<Utc>,
    pub duration_in_minutes: i32,
    pub calories_burned: i32,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}