-- Mock data for AI Agent Chat System

-- Users
INSERT INTO users (username, email, password_hash, created_at, last_login) VALUES
('john_doe', 'john@example.com', 'hashed_password_1', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('jane_smith', 'jane@example.com', 'hashed_password_2', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
('bob_johnson', 'bob@example.com', 'hashed_password_3', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- Agents
INSERT INTO agents (user_id, name, description, status, configuration, created_at, last_used) VALUES
(1, 'Assistant', 'General-purpose AI assistant', 'active', '{"language": "en", "model": "gpt-3.5-turbo"}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2, 'Coder', 'AI for coding assistance', 'active', '{"language": "en", "specialization": "python"}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3, 'Analyst', 'Data analysis AI', 'active', '{"language": "en", "data_sources": ["csv", "sql"]}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- Chats
INSERT INTO chats (user_id, title, created_at, updated_at) VALUES
(1, 'General Conversation', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(2, 'Python Help', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(3, 'Data Analysis Project', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- Messages
INSERT INTO messages (chat_id, user_id, agent_id, content, message_type, timestamp) VALUES
(1, 1, 1, 'Hello, how can I help you today?', 'text', CURRENT_TIMESTAMP),
(1, 1, NULL, 'I need help with a general question.', 'text', CURRENT_TIMESTAMP),
(2, 2, 2, 'What Python concept do you need help with?', 'text', CURRENT_TIMESTAMP),
(2, 2, NULL, 'I''m struggling with decorators.', 'text', CURRENT_TIMESTAMP),
(3, 3, 3, 'What kind of data are we analyzing today?', 'text', CURRENT_TIMESTAMP),
(3, 3, NULL, 'I have a CSV file with sales data.', 'text', CURRENT_TIMESTAMP);

-- Tools
INSERT INTO tools (name, description, created_at) VALUES
('Calculator', 'Performs mathematical calculations', CURRENT_TIMESTAMP),
('CodeExecutor', 'Executes code snippets', CURRENT_TIMESTAMP),
('DataVisualizer', 'Creates charts and graphs', CURRENT_TIMESTAMP);

-- AgentTools
INSERT INTO agent_tools (agent_id, tool_id) VALUES
(1, 1),
(2, 2),
(3, 3),
(1, 2),
(1, 3);

-- ToolUsage
INSERT INTO tool_usage (agent_id, tool_id, chat_id, message_id, used_at) VALUES
(1, 1, 1, 1, CURRENT_TIMESTAMP),
(2, 2, 2, 3, CURRENT_TIMESTAMP),
(3, 3, 3, 5, CURRENT_TIMESTAMP);