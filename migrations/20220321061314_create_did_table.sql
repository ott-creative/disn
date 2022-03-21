-- Add migration script here
CREATE TABLE IF NOT EXISTS dids (
    id VARCHAR NOT NULL PRIMARY KEY,
    jwk VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);