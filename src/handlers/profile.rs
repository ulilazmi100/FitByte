use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::Utc;
use crate::models::user::User;
use crate::errors::AppError;
use crate::utils::validation::{validate_payload, validate_preference, validate_weight_unit, validate_height_unit};

#[derive(Deserialize, Validate, Clone)] // Add `Clone` trait
pub struct ProfileUpdate {
    #[validate(length(min = 2, max = 60, message = "Name must be between 2 and 60 characters"))]
    name: Option<String>,

    #[validate(url(message = "Invalid image URI"))]
    image_uri: Option<String>,

    #[validate(range(min = 10, max = 1000, message = "Weight must be between 10 and 1000"))]
    weight: Option<f64>,

    #[validate(range(min = 3, max = 250, message = "Height must be between 3 and 250"))]
    height: Option<f64>,

    #[validate(required(message = "Preference is required"))]
    preference: Option<String>,

    #[validate(required(message = "Weight unit is required"))]
    weight_unit: Option<String>,

    #[validate(required(message = "Height unit is required"))]
    height_unit: Option<String>,
}

#[derive(Serialize)]
struct ProfileResponse {
    preference: Option<String>,
    weight_unit: Option<String>,
    height_unit: Option<String>,
    weight: Option<f64>,
    height: Option<f64>,
    email: String,
    name: Option<String>,
    image_uri: Option<String>,
}

// GET /v1/user
pub async fn get_profile(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
) -> Result<HttpResponse, AppError> {
    // Extract token from headers
    let token = req.headers().get("Authorization")
        .and_then(|auth| auth.to_str().ok())
        .and_then(|auth| auth.split_whitespace().nth(1))
        .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;

    // Validate token
    let claims = crate::utils::jwt::validate_token(token)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    // Fetch user from database
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        claims.sub
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Return response
    Ok(HttpResponse::Ok().json(ProfileResponse {
        preference: user.preference,
        weight_unit: user.weight_unit,
        height_unit: user.height_unit,
        weight: user.weight,
        height: user.height,
        email: user.email,
        name: user.name,
        image_uri: user.image_uri,
    }))
}

// PATCH /v1/user
pub async fn update_profile(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    updates: web::Json<ProfileUpdate>,
) -> Result<HttpResponse, AppError> {
    // Clone the updates payload to avoid moving it
    let updates_clone = updates.clone();

    // Validate payload
    validate_payload(&updates_clone)?;

    // Validate preference, weight unit, and height unit
    if let Some(preference) = &updates_clone.preference {
        validate_preference(preference)?;
    }
    if let Some(weight_unit) = &updates_clone.weight_unit {
        validate_weight_unit(weight_unit)?;
    }
    if let Some(height_unit) = &updates_clone.height_unit {
        validate_height_unit(height_unit)?;
    }

    // Extract token from headers
    let token = req.headers().get("Authorization")
        .and_then(|auth| auth.to_str().ok())
        .and_then(|auth| auth.split_whitespace().nth(1))
        .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;

    // Validate token
    let claims = crate::utils::jwt::validate_token(token)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    // Fetch user from database
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        claims.sub
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Update user profile
    let now = Utc::now();
    sqlx::query!(
        "UPDATE users SET preference = $1, weight_unit = $2, height_unit = $3, weight = $4, height = $5, name = $6, image_uri = $7, updated_at = $8 WHERE user_id = $9",
        updates_clone.preference,
        updates_clone.weight_unit,
        updates_clone.height_unit,
        updates_clone.weight,
        updates_clone.height,
        updates_clone.name,
        updates_clone.image_uri,
        now,
        user.user_id
    )
    .execute(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?;

    // Return response
    Ok(HttpResponse::Ok().json(ProfileResponse {
        preference: updates_clone.preference.clone(),
        weight_unit: updates_clone.weight_unit.clone(),
        height_unit: updates_clone.height_unit.clone(),
        weight: updates_clone.weight,
        height: updates_clone.height,
        email: user.email,
        name: updates_clone.name.clone(),
        image_uri: updates_clone.image_uri.clone(),
    }))
}