-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    creator_level INTEGER NOT NULL DEFAULT 0
);

-- Create projects table
CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES users(id),
    title VARCHAR(255) NOT NULL,
    risk_tier INTEGER NOT NULL DEFAULT 0,
    learning_objectives TEXT,
    starting_node_id INTEGER
);

-- Create nodes table
CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    title VARCHAR(255) NOT NULL,
    content TEXT,
    position_x INTEGER NOT NULL DEFAULT 0,
    position_y INTEGER NOT NULL DEFAULT 0,
    tags TEXT
);

-- Create choices table
CREATE TABLE choices (
    id SERIAL PRIMARY KEY,
    source_node_id INTEGER NOT NULL REFERENCES nodes(id),
    target_node_id INTEGER NOT NULL REFERENCES nodes(id),
    choice_text VARCHAR(255) NOT NULL
);
