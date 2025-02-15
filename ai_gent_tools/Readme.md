# FSM Agent CLI

This project implements a Finite State Machine (FSM) based AI agent using a Large Language Model (LLM) for natural language processing and interaction. The agent is designed to handle complex conversations and tasks through a command-line interface.

## Features

1. **FSM-based Architecture**: Utilizes a Finite State Machine to manage conversation flow and agent states.

2. **LLM Integration**: Integrates with a Large Language Model (likely GPT-4) for natural language understanding and generation.

3. **Asynchronous Processing**: Uses Tokio for asynchronous operations, allowing for efficient handling of LLM requests and responses.

4. **Docker-based Code Execution**: Capable of executing Python code snippets in a isolated Docker environment for safety and consistency.

5. **Configurable States and Prompts**: Uses a TOML configuration file to define FSM states, transitions, and associated prompts.

6. **Interactive CLI**: Provides a command-line interface for user interaction with the agent.

7. **Stream Processing**: Processes LLM responses as a stream, allowing for real-time output.

8. **Error Handling**: Implements robust error handling for various scenarios, including LLM failures and user interruptions.

## Key Components

- `FSMChatState`: Represents a state in the Finite State Machine.
- `LLMAgent`: Main agent class that manages the FSM and LLM interactions.
- `GenaiLlmclient`: Client for interacting with the LLM API.
- `run_code_in_docker`: Function to execute Python code snippets in a Docker container.

## Usage

1. Set up the necessary environment variables (e.g., `OPENAI_API_KEY`).
2. Configure the FSM states and prompts in the `fsm_config.toml` file.
3. Run the application to start the interactive CLI.
4. Type your queries or commands at the prompt.
5. Type 'exit' to quit the application.

## Dependencies

- Tokio for asynchronous runtime
- Serde for serialization/deserialization
- Anyhow for error handling
- Rustyline for the interactive CLI
- Docker for code execution (optional)
