-- Create vocabulary_words table
CREATE TABLE vocabulary_words (
    id SERIAL PRIMARY KEY,
    word VARCHAR(255) UNIQUE NOT NULL,
    definition TEXT NOT NULL,
    genai_image_prompt TEXT,
    genai_audio_prompt TEXT
);

-- Create node_vocabulary_tags table
CREATE TABLE node_vocabulary_tags (
    node_id INTEGER NOT NULL REFERENCES nodes(id),
    word_id INTEGER NOT NULL REFERENCES vocabulary_words(id),
    PRIMARY KEY (node_id, word_id)
);

-- Create choice_vocabulary_tags table
CREATE TABLE choice_vocabulary_tags (
    choice_id INTEGER NOT NULL REFERENCES choices(id),
    word_id INTEGER NOT NULL REFERENCES vocabulary_words(id),
    PRIMARY KEY (choice_id, word_id)
);

-- Remove tags column from nodes table
ALTER TABLE nodes DROP COLUMN tags;
