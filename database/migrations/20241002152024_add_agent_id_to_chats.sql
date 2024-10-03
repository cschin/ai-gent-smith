-- Add agent_id column to Chats table
ALTER TABLE chats
ADD COLUMN agent_id INTEGER;

-- Add foreign key constraint for agent_id
ALTER TABLE chats
ADD CONSTRAINT fk_chats_agent
FOREIGN KEY (agent_id) REFERENCES agents(agent_id) ON DELETE SET NULL;

