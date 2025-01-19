use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::env;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (e.g., user email)
    pub exp: usize,  // Expiration time
}

/// Generates a JWT token for the given email.
pub fn generate_token(email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: email.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref()),
    )
}

/// Validates a JWT token and returns the claims if valid.
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map(|data| data.claims)
}

/// Validator function for the `HttpAuthentication::bearer` middleware.
/// This function checks if the provided Bearer token is valid.
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();

    match validate_token(token) {
        Ok(claims) => {
            // Attach claims to the request extensions
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            // Return an error with the original request
            Err((actix_web::error::ErrorUnauthorized("Invalid token"), req))
        }
    }
}