-- Add migration script here
ALTER TABLE messages
ADD COLUMN role VARCHAR(32);
