CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    preference VARCHAR,
    weight_unit VARCHAR,
    height_unit VARCHAR,
    weight FLOAT,
    height FLOAT,
    name VARCHAR,
    image_uri VARCHAR,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE activities (
    activity_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(user_id),
    activity_type VARCHAR NOT NULL,
    done_at TIMESTAMPTZ NOT NULL,
    duration_in_minutes INT NOT NULL,
    calories_burned INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);

CREATE INDEX IF NOT EXISTS idx_activities_user_id ON activities (user_id);
CREATE INDEX IF NOT EXISTS idx_activities_done_at ON activities (done_at);
CREATE INDEX IF NOT EXISTS idx_activities_activity_type ON activities (activity_type);
CREATE INDEX IF NOT EXISTS idx_activities_calories_burned ON activities (calories_burned);

CREATE INDEX IF NOT EXISTS idx_activities_user_done ON activities (user_id, done_at);