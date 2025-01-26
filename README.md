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


## Test Results

Functional Test:
![image](https://github.com/user-attachments/assets/cb3489ad-37e5-4828-ab90-3e81c6ec956b)


Load Test:
1. Commit: d3dc3752e662ded8eb07c2df4fb0989b0e9e315d [d3dc375]
    "prod-cakalang-fafa" = {
      allow_internet = true
      allow_view_ec2 = true
      ec2_instances  = ["t4g.small"]
      db_type        = "postgres"
      db_disk        = "standard"
      db_instances   = ["t4g.small"]
    }
![image](https://github.com/user-attachments/assets/f8e8464f-dddf-4860-954f-769bbddf3a14)
