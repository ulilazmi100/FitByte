# FitByte Backend

A backend application for tracking fitness activities, ProjectSprint Batch 3 Week 2 project..

## Setup

1. Clone the repository.
2. Create a `.env` file and set the required environment variables.
3. Run the database migrations:
   ```bash
   sqlx migrate run
   ```
4. Build and run the application:
   ```bash
   cargo run
   ```

## API Endpoints

- `POST /v1/login`: User login.
- `POST /v1/register`: User registration.
- `GET /v1/user`: Retrieve user profile.
- `PATCH /v1/user`: Update user profile.
- `POST /v1/file`: Upload a file.
- `POST /v1/activity`: Create a new activity.
- `GET /v1/activity`: Retrieve activities.
- `PATCH /v1/activity/:activityId`: Update an activity.
- `DELETE /v1/activity/:activityId`: Delete an activity.

## Environment Variables

- `DATABASE_URL`: The connection string for the PostgreSQL database.
- `JWT_SECRET`: The secret key used for JWT token generation.
- `AWS_ACCESS_KEY_ID`: The AWS access key ID for S3 integration.
- `AWS_SECRET_ACCESS_KEY`: The AWS secret access key for S3 integration.
- `AWS_REGION`: The AWS region for S3 integration.
- `AWS_S3_BUCKET_NAME`: The S3 bucket name for file uploads.