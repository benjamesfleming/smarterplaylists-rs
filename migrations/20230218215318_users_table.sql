-- Add migration script here
CREATE TABLE users (
    id                    CHAR(26) PRIMARY KEY NOT NULL, -- ULID
    spotify_id            VARCHAR(255) NOT NULL,
    spotify_username      VARCHAR(255) NOT NULL,
    spotify_email         VARCHAR(255) NOT NULL,
    spotify_access_token  TEXT
);
CREATE UNIQUE INDEX users_email ON users (spotify_email);