-- Add context_tag and complexity_tier to vocabulary_words table
ALTER TABLE vocabulary_words
ADD COLUMN context_tag VARCHAR(255),
ADD COLUMN complexity_tier INTEGER;

-- Create player_mastery table
CREATE TABLE player_mastery (
    id SERIAL PRIMARY KEY,
    player_id INTEGER NOT NULL,
    word_id INTEGER NOT NULL REFERENCES vocabulary_words(id),
    times_used INTEGER NOT NULL DEFAULT 0,
    is_mastered BOOLEAN NOT NULL DEFAULT false,
    last_used_at TIMESTAMPTZ,
    UNIQUE (player_id, word_id)
);

-- Create word_usage_logs table
CREATE TABLE word_usage_logs (
    id SERIAL PRIMARY KEY,
    player_id INTEGER NOT NULL,
    word_id INTEGER NOT NULL REFERENCES vocabulary_words(id),
    context_used VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
