-- Add migration script here
ALTER TABLE vc_issuers ALTER COLUMN service_address TYPE INTEGER USING (trim(service_address)::integer);