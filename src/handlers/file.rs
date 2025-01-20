use actix_web::{web, HttpResponse, HttpRequest, Error};
use aws_sdk_s3::Client as S3Client;
use uuid::Uuid;
use std::env;
use serde_json::json;
use actix_multipart::Multipart;
use futures_util::StreamExt;
use tokio::sync::oneshot;
use log::{info, error};
use infer;

// Define the type alias for the upload result
type UploadResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn upload_file(
    req: HttpRequest,
    s3_client: web::Data<S3Client>,
    payload: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut multipart = Multipart::new(&req.headers(), payload);
    let mut file_data = Vec::new();
    let mut file_size = 0;

    // Collect file data
    while let Some(item) = multipart.next().await {
        let mut field = item.map_err(|err| {
            error!("Invalid multipart field: {:?}", err);
            actix_web::error::ErrorBadRequest("Invalid multipart field")
        })?;

        if field.name() != "file" {
            error!("Invalid field name: expected 'file'");
            return Err(actix_web::error::ErrorBadRequest("Invalid field name: expected 'file'"));
        }

        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|err| {
                error!("Failed to read chunk: {:?}", err);
                actix_web::error::ErrorBadRequest("Failed to read chunk")
            })?;
            file_size += chunk.len();
            if file_size > 102400 {
                error!("File size exceeds 100KiB limit");
                return Err(actix_web::error::ErrorBadRequest("File size exceeds 100KiB limit"));
            }
            file_data.extend_from_slice(&chunk);
        }
    }

    if file_data.is_empty() {
        error!("File part is missing");
        return Err(actix_web::error::ErrorBadRequest("File part is missing"));
    }

    info!("File size: {}", file_size);

    // Detect file type
    let file_type = infer::get(&file_data).ok_or_else(|| {
        error!("Unable to detect file type");
        actix_web::error::ErrorBadRequest("Unable to detect file type")
    })?;

    info!("Detected file type: {:?}", file_type.mime_type());

    if !["image/jpeg", "image/jpg", "image/png"].contains(&file_type.mime_type()) {
        error!("Only JPEG, JPG, and PNG files are allowed");
        return Err(actix_web::error::ErrorBadRequest("Only JPEG, JPG, and PNG files are allowed"));
    }

    // Generate a unique file name using UUID
    let file_id = Uuid::new_v4();
    let file_name = format!("{}.{}", file_id, file_type.extension());

    // Generate the S3 URI
    let bucket_name = env::var("AWS_S3_BUCKET").map_err(|_| {
        error!("AWS_S3_BUCKET environment variable not set");
        actix_web::error::ErrorInternalServerError("AWS_S3_BUCKET not set")
    })?;
    let s3_uri = format!("s3://{}/{}", bucket_name, file_name);

    info!("Uploading file to S3: {}", s3_uri);

    // Upload the file to S3
    let (tx, rx) = oneshot::channel::<UploadResult>();
    let s3_client_clone = s3_client.clone();

    tokio::spawn(async move {
        match s3_client_clone.put_object()
            .bucket(&bucket_name)
            .key(&file_name)
            .body(file_data.into())
            .send()
            .await
        {
            Ok(_) => {
                let _ = tx.send(Ok(()));
            }
            Err(err) => {
                let _ = tx.send(Err(err.into()));
            }
        }
    });

    match rx.await {
        Ok(Ok(())) => {
            // Return the S3 URI
            Ok(HttpResponse::Ok().json(json!({ "uri": s3_uri })))
        }
        Ok(Err(err)) => {
            error!("Failed to upload to S3: {:?}", err);
            Err(actix_web::error::ErrorInternalServerError("Failed to upload to S3"))
        }
        Err(_) => {
            error!("Upload task canceled");
            Err(actix_web::error::ErrorServiceUnavailable("Upload task canceled"))
        }
    }
}