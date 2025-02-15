use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::Utc;
use crate::models::user::{GetUserProfile, GetUserId};
use crate::errors::AppError;
use crate::utils::validation::{validate_preference, validate_weight_unit, validate_height_unit, validate_url};
use crate::utils::jwt::Claims;

#[derive(Deserialize, Validate, Clone)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
    // Extract claims from request extensions
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Invalid token in claim".to_string()))?;

    // Fetch user from database
    let user = sqlx::query_as!(
        GetUserProfile,
        "SELECT preference, weight_unit, height_unit, weight, height, name, image_uri FROM users WHERE email = $1",
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
        email: claims.sub.clone(),
        name: user.name,
        image_uri: user.image_uri,
    }))
}

// Helper function to check for null values in the input
fn has_null_fields(updates: &ProfileUpdate) -> bool {
    updates.name.is_none() || updates.image_uri.is_none() || updates.weight.is_none() ||
    updates.height.is_none() || updates.preference.is_none() || updates.weight_unit.is_none() ||
    updates.height_unit.is_none()
}

// PATCH /v1/user
pub async fn update_profile(
    req: HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    updates: web::Json<ProfileUpdate>,
) -> Result<HttpResponse, AppError> {
    // Extract claims from request extensions
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Invalid token in claim".to_string()))?;

    // Validate payload

    // Check for null values in the input
    if has_null_fields(&updates) {
        return Err(AppError::BadRequest("Fields cannot be null if provided".to_string()));
    }

    // Validate preference, weight unit, and height unit
    if let Some(preference) = &updates.preference {
        validate_preference(preference)?;
    }
    if let Some(weight_unit) = &updates.weight_unit {
        validate_weight_unit(weight_unit)?;
    }
    if let Some(height_unit) = &updates.height_unit {
        validate_height_unit(height_unit)?;
    }

    if let Some(uri) = &updates.image_uri {
        if uri.is_empty() {
            return Err(AppError::BadRequest("Image URI cannot be empty if provided".to_string()));
        }
        validate_url(uri)?;
    }

    // Validate the entire payload, including the image URI
    updates.validate().map_err(|err| AppError::BadRequest(err.to_string()))?;

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

    // Update user profile
    let now = Utc::now();
    sqlx::query!(
        "UPDATE users SET preference = $1, weight_unit = $2, height_unit = $3, weight = $4, height = $5, name = $6, image_uri = $7, updated_at = $8 WHERE user_id = $9",
        updates.preference,
        updates.weight_unit,
        updates.height_unit,
        updates.weight,
        updates.height,
        updates.name,
        updates.image_uri,
        now,
        user.user_id
    )
    .execute(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?;

    // Return response
    Ok(HttpResponse::Ok().json(ProfileResponse {
        preference: updates.preference.clone(),
        weight_unit: updates.weight_unit.clone(),
        height_unit: updates.height_unit.clone(),
        weight: updates.weight,
        height: updates.height,
        email: claims.sub.clone(),
        name: updates.name.clone(),
        image_uri: updates.image_uri.clone(),
    }))
}