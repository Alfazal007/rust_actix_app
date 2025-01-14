-- Add migration script here
CREATE TABLE todos (
    id SERIAL PRIMARY KEY,
    creator_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    completed BOOLEAN DEFAULT FALSE,
    title VARCHAR(255) NOT NULL
);
