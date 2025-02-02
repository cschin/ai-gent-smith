-- Add migration script here
ALTER TABLE agents
ADD COLUMN asset_id integer NULL,
ADD CONSTRAINT fk_agent_asset FOREIGN KEY (asset_id) REFERENCES assets(asset_id);
DROP TABLE agent_assets
