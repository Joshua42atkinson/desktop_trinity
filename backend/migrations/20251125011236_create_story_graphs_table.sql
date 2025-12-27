-- Create story_graphs table
CREATE TABLE story_graphs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    nodes JSONB NOT NULL,
    connections JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
