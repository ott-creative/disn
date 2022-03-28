-- Add migration script here
CREATE TABLE IF NOT EXISTS vc_issuers (
    did VARCHAR NOT NULL PRIMARY KEY,
    service_address VARCHAR,
    status INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);