-- Add migration script here
ALTER TABLE chats
ADD COLUMN status VARCHAR(16) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'deleted'));
