-- Create conversation_turns table for AI Mirror conversation storage
-- Migration: 001_create_conversation_turns

CREATE TABLE IF NOT EXISTS conversation_turns (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    user_id BIGINT NOT NULL,
    speaker VARCHAR(10) NOT NULL CHECK (speaker IN ('user', 'ai')),
    content TEXT NOT NULL,
    word_count INT,
    sentiment REAL,
    depth_level SMALLINT CHECK (depth_level >= 1 AND depth_level <= 5),
    virtue_signals JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    timestamp TIMESTAMPTZ NOT NULL
);

-- Index for efficient session queries
CREATE INDEX idx_conversation_session ON conversation_turns(session_id, created_at DESC);

-- Index for user queries
CREATE INDEX idx_conversation_user ON conversation_turns(user_id, created_at DESC);

-- Index for session + user queries
CREATE INDEX idx_conversation_session_user ON conversation_turns(session_id, user_id);
