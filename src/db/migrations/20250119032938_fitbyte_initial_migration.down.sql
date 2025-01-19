DROP INDEX IF EXISTS idx_activities_user_done;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_activities_user_id;
DROP INDEX IF EXISTS idx_activities_done_at;
DROP INDEX IF EXISTS idx_activities_activity_type;
DROP INDEX IF EXISTS idx_activities_calories_burned;

DROP TABLE IF EXISTS activities;

DROP TABLE IF EXISTS users;

DROP EXTENSION IF EXISTS "uuid-ossp";
