-- Add migration script here
CREATE EXTENSION vector;

-- Create the text_embedding table
CREATE TABLE text_embedding (
    id SERIAL PRIMARY KEY,
    asset_id INTEGER REFERENCES assets(asset_id) NOT NULL,
    text TEXT,
    span INT4RANGE,
    embedding_vector vector(768) NOT NULL,
    two_d_embedding vector(2),
    filename VARCHAR(1024),
    title VARCHAR(1024),
    UNIQUE (asset_id, filename, title, text)
);

-- Create an index on the embedding_vector column for faster similarity searches
CREATE INDEX ON text_embedding USING ivfflat (embedding_vector vector_cosine_ops);

-- Create an index on the two_d_embedding column for faster similarity searches
CREATE INDEX ON text_embedding USING ivfflat (two_d_embedding vector_cosine_ops);
