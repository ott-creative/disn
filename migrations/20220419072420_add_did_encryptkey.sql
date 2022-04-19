-- Add migration script here
ALTER TABLE dids ADD COLUMN encrypt_public_key VARCHAR NOT NULL DEFAULT '';
ALTER TABLE dids ADD COLUMN encrypt_private_key VARCHAR NOT NULL DEFAULT '';