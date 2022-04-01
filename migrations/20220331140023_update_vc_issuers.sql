-- Add migration script here
ALTER TABLE vc_issuers ADD COLUMN name VARCHAR UNIQUE;