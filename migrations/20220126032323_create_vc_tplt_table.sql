-- Add migration script here
CREATE TABLE IF NOT EXISTS vc_tplts (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    purpose VARCHAR NOT NULL,
    fields VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);