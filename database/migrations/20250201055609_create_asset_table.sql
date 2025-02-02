-- Add migration script here
-- Asset table
CREATE TABLE assets (
    asset_id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    URL TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    last_used TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    CHECK (status IN ('active', 'inactive', 'deleted'))
);

-- AgentTools table (for many-to-many relationship between Agents and Assets)
CREATE TABLE agent_assets (
    agent_id INTEGER NOT NULL,
    asset_id INTEGER NOT NULL,
    PRIMARY KEY (agent_id, asset_id),
    FOREIGN KEY (agent_id) REFERENCES agents(agent_id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(asset_id) ON DELETE CASCADE
);
