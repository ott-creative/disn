-- Add migration script here
CREATE TABLE IF NOT EXISTS passbase (
    tx_hash character varying NOT NULL,
    -- 0: pending 1: success -1: failture
    send_status INTEGER NOT NULL DEFAULT 0,
    block_number bigint,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);