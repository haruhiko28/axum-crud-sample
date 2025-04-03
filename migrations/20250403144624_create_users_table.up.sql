-- Add up migration script here
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    address TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);