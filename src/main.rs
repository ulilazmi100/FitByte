mod handlers;
mod models;
mod utils;
mod db;
mod errors;

use actix_web::{web, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use dotenv::dotenv;
use sqlx::PgPool;
use std::env;
use log::info;
use crate::utils::s3::create_s3_client;
use env_logger::Env;
use actix_web::middleware::Logger;
use actix_web_httpauth::middleware::HttpAuthentication;
use std::collections::HashMap;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Initialize S3 client
    let s3_client = create_s3_client().await;

    // Validate JWT secret
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    if jwt_secret.is_empty() {
        panic!("JWT_SECRET cannot be empty");
    }

    // Initialize the database pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to the database");

    // Fetch the server bind address from an environment variable, default to "127.0.0.1:8080"
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    info!("Starting server at {}", bind_address);

    // Authentication middleware
    let auth = HttpAuthentication::bearer(crate::utils::jwt::validator);

    // Set up Prometheus metrics
    let mut labels = HashMap::new();
    labels.insert("app".to_string(), "fitbyte_cakalang".to_string()); // Add custom labels
    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .const_labels(labels)
        .build()
        .expect("Failed to create Prometheus metrics");

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default()) // Logging middleware
            .wrap(prometheus.clone()) // Prometheus metrics middleware
            .app_data(web::Data::new(pool.clone())) // Database pool
            .app_data(web::Data::new(s3_client.clone())) // S3 client
            .service(
                web::resource("/v1/login")
                    .route(web::post().to(handlers::auth::login)),
            )
            .service(
                web::resource("/v1/register")
                    .route(web::post().to(handlers::auth::register)),
            )
            .service(
                web::resource("/v1/user")
                    .wrap(auth.clone())
                    .route(web::get().to(handlers::profile::get_profile))
                    .route(web::patch().to(handlers::profile::update_profile)),
            )
            .service(
                web::resource("/v1/file")
                    .wrap(auth.clone())
                    .route(web::post().to(handlers::file::upload_file)),
            )
            .service(
                web::resource("/v1/activity")
                    .wrap(auth.clone())
                    .route(web::get().to(handlers::activity::get_activities))
                    .route(web::post().to(handlers::activity::create_activity)),
            )
            .service(
                web::resource("/v1/activity/{activityId}")
                    .wrap(auth.clone())
                    .route(web::patch().to(handlers::activity::update_activity))
                    .route(web::delete().to(handlers::activity::delete_activity)),
            )
    })
    .bind(&bind_address)?
    .run()
    .await
}