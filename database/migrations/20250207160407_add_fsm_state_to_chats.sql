-- Add migration script here
ALTER TABLE chats
ADD COLUMN last_fsm_state VARCHAR(48) NULL;

ALTER TABLE messages
ADD COLUMN fsm_state VARCHAR(48) NULL;