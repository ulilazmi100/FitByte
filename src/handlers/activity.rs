use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::{activity::Activity, activity::GetActivityCreatedAt, user::GetUserId};
use crate::errors::AppError;
use crate::utils::jwt::Claims;

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRequest {
    #[validate(required(message = "Activity type is required"))]
    #[validate(length(min = 1, message = "Activity type cannot be empty"))]
    activity_type: Option<String>,

    #[validate(required(message = "Done at is required"))]
    #[validate(length(min = 1, message = "Done at cannot be empty"))]
    done_at: Option<String>,

    #[validate(required(message = "Duration is required"))]
    #[validate(range(min = 1, message = "Duration must be at least 1 minute"))]
    duration_in_minutes: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResponse {
    activity_id: Uuid,
    activity_type: String,
    done_at: String,
    duration_in_minutes: i32,
    calories_burned: i32,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetActivitiesQuery {
    limit: Option<i32>,
    offset: Option<i32>,
    activity_type: Option<String>,
    done_at_from: Option<String>,
    done_at_to: Option<String>,
    calories_burned_min: Option<i32>,
    calories_burned_max: Option<i32>,
}

// Helper function to calculate calories burned
fn calculate_calories_burned(activity_type: &str, duration: i32) -> Result<i32, AppError> {
    match activity_type {
        "Walking" | "Yoga" | "Stretching" => Ok(4 * duration),
        "Cycling" | "Swimming" | "Dancing" => Ok(8 * duration),
        "Hiking" | "Running" | "HIIT" | "JumpRope" => Ok(10 * duration),
        _ => Err(AppError::BadRequest("Invalid activity type".to_string())),
    }
}

// POST /v1/activity
pub async fn create_activity(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    payload: web::Json<ActivityRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate payload
    payload.validate().map_err(|err| AppError::BadRequest(err.to_string()))?;

    let extensions = req.extensions();
    let claims = extensions.get::<Claims>().unwrap();

    // Fetch user from database
    let user = sqlx::query_as!(
        GetUserId,
        "SELECT user_id FROM users WHERE email = $1",
        claims.sub
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Parse done_at date
    let done_at = DateTime::parse_from_rfc3339(&payload.done_at.as_ref().unwrap())
        .map_err(|_| AppError::BadRequest("Invalid date format".to_string()))?
        .with_timezone(&Utc);

    // Calculate calories burned
    let calories_burned = calculate_calories_burned(
        payload.activity_type.as_ref().unwrap(),
        payload.duration_in_minutes.unwrap(),
    )?;

    // Insert activity into database
    let activity_id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query!(
        "INSERT INTO activities (activity_id, user_id, activity_type, done_at, duration_in_minutes, calories_burned, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        activity_id,
        user.user_id,
        payload.activity_type.as_ref().unwrap(),
        done_at,
        payload.duration_in_minutes.unwrap(),
        calories_burned,
        now,
        now
    )
    .execute(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?;

    // Return response
    Ok(HttpResponse::Created().json(ActivityResponse {
        activity_id,
        activity_type: payload.activity_type.clone().unwrap(),
        done_at: payload.done_at.clone().unwrap(),
        duration_in_minutes: payload.duration_in_minutes.unwrap(),
        calories_burned,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    }))
}

// GET /v1/activity
pub async fn get_activities(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    query: web::Query<GetActivitiesQuery>,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>().unwrap();

    // Fetch user from database
    let user = sqlx::query_as!(
        GetUserId,
        "SELECT user_id FROM users WHERE email = $1",
        claims.sub
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| {
        AppError::InternalServerError(format!(
            "Database error: {}", e
        ))
    })?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Build query
    let limit = query.limit.unwrap_or(5);
    let offset = query.offset.unwrap_or(0);
    let mut sql_query = "SELECT * FROM activities WHERE user_id = $1".to_string();
    let mut params: Vec<String> = vec![user.user_id.to_string()];

    if let Some(activity_type) = &query.activity_type {
        sql_query.push_str(" AND activity_type = $2");
        params.push(activity_type.clone());
    }

    if let Some(done_at_from) = &query.done_at_from {
        sql_query.push_str(" AND done_at >= $3::timestamptz");
        params.push(done_at_from.clone());
    }

    if let Some(done_at_to) = &query.done_at_to {
        sql_query.push_str(" AND done_at <= $4::timestamptz");
        params.push(done_at_to.clone());
    }

    if let Some(calories_burned_min) = query.calories_burned_min {
        sql_query.push_str(" AND calories_burned >= $5");
        params.push(calories_burned_min.to_string());
    }

    if let Some(calories_burned_max) = query.calories_burned_max {
        sql_query.push_str(" AND calories_burned <= $6");
        params.push(calories_burned_max.to_string());
    }

    sql_query.push_str(" LIMIT $7 OFFSET $8");
    params.push(limit.to_string());
    params.push(offset.to_string());

    // Fetch activities for the user
    let activities = sqlx::query_as::<_, Activity>(&sql_query)
        .bind(&user.user_id)
        .bind(&query.activity_type)
        .bind(&query.done_at_from)
        .bind(&query.done_at_to)
        .bind(&query.calories_burned_min)
        .bind(&query.calories_burned_max)
        .bind(limit)
        .bind(offset)
        .fetch_all(&**pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Database error: {}", e
            ))
        })?;

    // Return response
    Ok(HttpResponse::Ok().json(activities))
}

// PATCH /v1/activity/:activityId
pub async fn update_activity(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    activity_id: web::Path<Uuid>,
    payload: web::Json<ActivityRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate payload
    payload.validate().map_err(|err| AppError::BadRequest(err.to_string()))?;

    let extensions = req.extensions();
    let claims = extensions.get::<Claims>().unwrap();

    // Fetch user from database
    let user = sqlx::query_as!(
        GetUserId,
        "SELECT user_id FROM users WHERE email = $1",
        claims.sub
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Fetch activity from database
    let activity = sqlx::query_as!(
        GetActivityCreatedAt,
        "SELECT created_at FROM activities WHERE activity_id = $1 AND user_id = $2",
        *activity_id,
        user.user_id
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
    .ok_or_else(|| AppError::NotFound("Activity not found".to_string()))?;

    // Parse done_at date
    let done_at = DateTime::parse_from_rfc3339(&payload.done_at.as_ref().unwrap())
        .map_err(|_| AppError::BadRequest("Invalid date format".to_string()))?
        .with_timezone(&Utc);

    // Calculate calories burned
    let calories_burned = calculate_calories_burned(
        payload.activity_type.as_ref().unwrap(),
        payload.duration_in_minutes.unwrap(),
    )?;

    // Update activity in database
    let now = Utc::now();
    sqlx::query!(
        "UPDATE activities SET activity_type = $1, done_at = $2, duration_in_minutes = $3, calories_burned = $4, updated_at = $5 WHERE activity_id = $6",
        payload.activity_type.as_ref().unwrap(),
        done_at,
        payload.duration_in_minutes.unwrap(),
        calories_burned,
        now,
        *activity_id
    )
    .execute(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?;

    // Return response
    Ok(HttpResponse::Ok().json(ActivityResponse {
        activity_id: *activity_id,
        activity_type: payload.activity_type.clone().unwrap(),
        done_at: payload.done_at.clone().unwrap(),
        duration_in_minutes: payload.duration_in_minutes.unwrap(),
        calories_burned,
        created_at: activity.created_at.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    }))
}

// DELETE /v1/activity/:activityId
pub async fn delete_activity(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    activity_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>().unwrap();

    // Fetch user from database
    let user = sqlx::query_as!(
        GetUserId,
        "SELECT user_id FROM users WHERE email = $1",
        claims.sub
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Delete activity from database
    sqlx::query!(
        "DELETE FROM activities WHERE activity_id = $1 AND user_id = $2",
        *activity_id,
        user.user_id
    )
    .execute(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?;

    // Return response
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Activity deleted successfully" })))
}