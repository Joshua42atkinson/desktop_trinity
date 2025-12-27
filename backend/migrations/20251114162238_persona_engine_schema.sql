-- Create archetypes table
CREATE TABLE archetypes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT
);

-- Create stats table
CREATE TABLE stats (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL
);

-- Create archetype_stat_buffs table
CREATE TABLE archetype_stat_buffs (
    archetype_id INTEGER NOT NULL REFERENCES archetypes(id),
    stat_id INTEGER NOT NULL REFERENCES stats(id),
    buff_value INTEGER NOT NULL,
    PRIMARY KEY (archetype_id, stat_id)
);

-- Create dilemmas table
CREATE TABLE dilemmas (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    dilemma_text TEXT NOT NULL
);

-- Create dilemma_choices table
CREATE TABLE dilemma_choices (
    id SERIAL PRIMARY KEY,
    dilemma_id INTEGER NOT NULL REFERENCES dilemmas(id),
    choice_text VARCHAR(255) NOT NULL
);

-- Create dilemma_choice_archetype_points table
CREATE TABLE dilemma_choice_archetype_points (
    dilemma_choice_id INTEGER NOT NULL REFERENCES dilemma_choices(id),
    archetype_id INTEGER NOT NULL REFERENCES archetypes(id),
    points INTEGER NOT NULL,
    PRIMARY KEY (dilemma_choice_id, archetype_id)
);

-- Modify users table to add primary_archetype_id
ALTER TABLE users ADD COLUMN primary_archetype_id INTEGER REFERENCES archetypes(id);

-- Modify choices table to add required_archetype_id
ALTER TABLE choices ADD COLUMN required_archetype_id INTEGER REFERENCES archetypes(id);
