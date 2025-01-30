use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::Utc;
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, Header, EncodingKey};
use validator::Validate;
use std::env;
use crate::utils::jwt::Claims;
use crate::models::user;
use crate::errors::AppError;
use actix_web::rt::task::spawn_blocking;
use lazy_static::lazy_static;
use moka::sync::Cache;

lazy_static! {
    static ref EMAIL_CACHE: Cache<String, bool> = Cache::new(10_000); //Important, the load test only got like 200 emails and took resource, may cause test fail if removed
}

#[derive(Deserialize, Validate)]
pub struct AuthRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,

    #[validate(length(min = 8, max = 32, message = "Password must be between 8 and 32 characters"))]
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    email: String,
    token: String,
}

// POST /v1/login
pub async fn login(
    req: web::Json<AuthRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    req.validate().map_err(|err| AppError::BadRequest(err.to_string()))?;

    // Fetch user from database
    let user = sqlx::query_as!(
        user::GetUserPassword,
        "SELECT password FROM users WHERE email = $1",
        req.email
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
    .ok_or_else(|| AppError::NotFound("Email not found".to_string()))?;

    let req_email = req.email.clone();

    // Verify password using bcrypt
    let is_valid = spawn_blocking(move || verify(req.password.as_str(), &user.password))
        .await
        .map_err(|_| AppError::InternalServerError("Password verification error".to_string()))?
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;   


    if !is_valid {
        return Err(AppError::Unauthorized("Invalid password".to_string()));
    }

    // Generate JWT token using spawn_blocking
    let jwt_secret = env::var("JWT_SECRET").unwrap();
    let claims = Claims {
        sub: req_email.clone(),
        exp: (Utc::now() + chrono::Duration::days(7)).timestamp() as usize,
    };

    let token = spawn_blocking(move || {
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
    })
    .await
    .map_err(|_| AppError::InternalServerError("Token generation error".to_string()))?
    .map_err(|_| AppError::InternalServerError("Token generation error".to_string()))?;

    // Return response
    Ok(HttpResponse::Ok().json(AuthResponse {
        email: req_email,
        token,
    }))
}

// POST /v1/register
pub async fn register(
    req: web::Json<AuthRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    req.validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if EMAIL_CACHE.get(&req.email).is_some() {
        return Err(AppError::Conflict("Email exists (cached)".to_string()));
    }

    let password = req.password.clone();
    let email = req.email.clone();

    // Handle bcrypt hashing result properly
    let password_hash = spawn_blocking(move || hash(&password, 10))
        .await
        .map_err(|_| AppError::InternalServerError("Hashing failed".to_string()))?
        .map_err(|e| AppError::InternalServerError(e.to_string()))?; // Unwrap bcrypt result

    let user_id = spawn_blocking(uuid::Uuid::now_v7)
        .await
        .map_err(|_| AppError::InternalServerError("UUID generation failed".to_string()))?;

    // Insert and check if email already exists
    let result = sqlx::query!(
        "INSERT INTO users (user_id, email, password, created_at, updated_at) 
        VALUES ($1, $2, $3, NOW(), NOW())
        ON CONFLICT (email) DO NOTHING",
        user_id,
        email,
        password_hash // Direct String value
    )
    .execute(&**pool)
    .await;

    // Check if email already exists
    let rows_affected = match result {
        Ok(res) => res.rows_affected(),
        Err(e) => return Err(AppError::InternalServerError(e.to_string())),
    };

    if rows_affected == 0 {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    EMAIL_CACHE.insert(req.email.clone(), true);

    // Generate JWT token
    let token = spawn_blocking(move || {
        encode(
            &Header::default(),
            &Claims {
                sub: email,
                exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            },
            &EncodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_bytes()),
        )
    })
    .await
    .map_err(|_| AppError::InternalServerError("Token generation failed".to_string()))?
    .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Return response
    Ok(HttpResponse::Created().json(AuthResponse {
        email: req.email.clone(),
        token,
    }))
}