-- Add type column to agents table
ALTER TABLE agents
ADD COLUMN type VARCHAR(20) NOT NULL DEFAULT 'basic',
ADD CONSTRAINT check_agent_type CHECK (type IN ('basic', 'advanced'));

