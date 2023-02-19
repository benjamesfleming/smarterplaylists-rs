-- Add migration script here
CREATE TABLE users (
    id                    INT(11) PRIMARY KEY,
    spotify_username      VARCHAR(255) NOT NULL,
    spotify_email         VARCHAR(255) NOT NULL,
    spotify_access_token  TEXT,
    spotify_refresh_token TEXT
);