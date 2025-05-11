-- Add migration script here

ALTER TABLE text_embedding
ADD COLUMN meta_data JSONB NOT NULL DEFAULT '{}'::jsonb;
