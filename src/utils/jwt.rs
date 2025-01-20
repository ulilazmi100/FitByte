use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web::dev::ServiceRequest;
use actix_web::{Error, HttpMessage};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (e.g., user email)
    pub exp: usize,  // Expiration time
}

/// Generates a JWT token for the given email
pub fn generate_token(email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: email.to_string(),
        exp: expiration,
    };

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
}

/// Async token validation using spawn_blocking for CPU-bound operations
async fn validate_token_async(token: &str, jwt_secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token = token.to_owned();
    let secret = jwt_secret.to_owned();
    
    actix_web::rt::task::spawn_blocking(move || {
        decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| data.claims)
    })
    .await
    .unwrap_or_else(|_| Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken)))
}

/// Async validator for Bearer authentication
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => return Err((actix_web::error::ErrorInternalServerError("JWT secret not configured"), req)),
    };

    match validate_token_async(credentials.token(), &jwt_secret).await {
        Ok(claims) => {
            // Manual expiration check
            let now = Utc::now().timestamp() as usize;
            if claims.exp < now {
                return Err((actix_web::error::ErrorUnauthorized("Token expired"), req));
            }
            
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(e) => {
            let error = match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => 
                    actix_web::error::ErrorUnauthorized("Token expired"),
                _ => actix_web::error::ErrorUnauthorized("Invalid token"),
            };
            Err((error, req))
        }
    }
}