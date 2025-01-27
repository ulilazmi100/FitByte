use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use argon2::{Argon2, password_hash::PasswordHasher, password_hash::SaltString, PasswordVerifier};
use jsonwebtoken::{encode, Header, EncodingKey};
use validator::Validate;
use std::env;
use rand;
use crate::utils::jwt::Claims;
use crate::models::user;
use crate::errors::AppError;

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

    // Verify password
    let parsed_hash = argon2::PasswordHash::new(&user.password)
        .map_err(|_| AppError::InternalServerError("Invalid password hash".to_string()))?;
    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid password".to_string()))?;

    // Generate JWT token
    let claims = Claims {
        sub: req.email.clone(),
        exp: (Utc::now() + chrono::Duration::days(7)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref()),
    )
    .map_err(|_| AppError::InternalServerError("Token generation error".to_string()))?;

    // Return response
    Ok(HttpResponse::Ok().json(AuthResponse {
        email: req.email.clone(),
        token,
    }))
}

// POST /v1/register
pub async fn register(
    req: web::Json<AuthRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    req.validate().map_err(|err| AppError::BadRequest(err.to_string()))?;

    // Check if email already exists
    if sqlx::query!("SELECT email FROM users WHERE email = $1", req.email)
        .fetch_optional(&**pool)
        .await
        .map_err(|_| AppError::InternalServerError("Database error".to_string()))?
        .is_some()
    {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // Hash password
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError("Hashing error".to_string()))?
        .to_string();

    // Insert user into database
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query!(
        "INSERT INTO users (user_id, email, password, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
        user_id, req.email, password_hash, now, now
    )
    .execute(&**pool)
    .await
    .map_err(|_| AppError::InternalServerError("Database error".to_string()))?;

    // Generate JWT token
    let claims = Claims {
        sub: req.email.clone(),
        exp: (Utc::now() + chrono::Duration::days(7)).timestamp() as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref()),
    )
    .map_err(|_| AppError::InternalServerError("Token generation error".to_string()))?;

    // Return response
    Ok(HttpResponse::Created().json(AuthResponse {
        email: req.email.clone(),
        token,
    }))
}