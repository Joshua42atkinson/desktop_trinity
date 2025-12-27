-- Step 1: Ensure the UUID generation extension is available
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Step 2: Update the table definition to use UUID and automatically generate it.
CREATE TABLE reflection_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id BIGINT NOT NULL,
    challenge_name VARCHAR(255) NOT NULL,
    reflection_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);