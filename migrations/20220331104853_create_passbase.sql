-- Add migration script here
CREATE TABLE IF NOT EXISTS passbase (
    id VARCHAR NOT NULL PRIMARY KEY,
    identity VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);