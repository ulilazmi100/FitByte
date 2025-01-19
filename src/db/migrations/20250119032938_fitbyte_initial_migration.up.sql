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