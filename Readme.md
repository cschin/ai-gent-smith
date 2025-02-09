# AI-Gent Smith

![AI-Gent Smith](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/ai_gent_web.png?raw=true)

"AI-Gent Smith" is a simple project demonstrating the use of Rust and various LLM libraries to build a web-based LLM agent library for users. A user can create a simple agent by providing prompts and selecting a backend model. However, it is designed to support LLM agents with "state," allowing the agent to use different prompts or take different actions depending on the current state of the dialogue.

A user can define a finite state machine for state transitions and specify the prompts (or actions) that determine how the agent responds in a given state. State transitions can also be determined using an LLM. I have not fully tested it with more complex scenarios yet, but it works well for simple Q&A cases.

A user can create multiple agents, which can be updated or deleted through a web UI. Conversations are saved automatics and can browsed. A user can download JSON output of the chats as well. The project supports retrieval-augmented generation (RAG), and in the future, we plan to add support for "asset" creation and management.

Currently, we provide a small document dataset from an precision FDA challenge: [PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge: Democratizing and Demystifying AI as the default dataset](https://precision.fda.gov/challenges/34/intro), with pre-configured agents allowing users to query related documents to answer a set of questions for the challenge.


## Table of Contents

- [Installation](#installation)
- [Finite State Machine Agent](#finite-state-machine-agent)
- [Usage](#usage)
- [Technologies Used](#technologies-used)
- [Project Structure](#project-structure)
- [Supported LLM Vendors and Models](#supported-llm-vendors-and-models)
- [License](#license)

## Installation

### For Rust Developers

1. Ensure you have Rust installed on your system. If not, install it from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

2. Install `sqlx-cli` with `cargo`, 
```
cargo install sqlx-cli
```

3. Install `postgresql` (version 14+) for your system and create a database user with the permission to create a database in your development environment. (see `ai-gent-smith/ai_gent_web/docker/setup_db.sh` for an example). 

4. Clone the repository: 

```
git clone https://github.com/cschin/ai-gent-smith
```

5. Navigate to the project directory: 

```
cd ai-gent-smith/
```

6. create the database: 

You need to have a functional postgreSQL database running with pgvector extension built. (You can see an example how to setup a local postgreSQL + pgvetcor in the Docker file `ai_gent_web/docker/Dockerfile` for Ubuntu 24.04.)

Set up the environmental variable `DATABASE_URL` for the database location.

For example if your user name is `db_user`, you can do `export DATABASE_URL=postgres://db_user@localhost/ai_gent` under a command line shell prompt if your user name is `db_user` for a local postgreSQL setup.

You can use the script `database/create_empty_db.sh` to create an empty database named `ai_gent` as long as you have permission to create and modify a database in your PostgreSQL setup. You need to set up the environmental variable `DATABASE_URL` so the `sqlx` knows the user name and the database name. You may setup `PGUSER` and `PGPASSWORD` if you can use the environmental variables for the authentication to use the database. Here is the context of the script and you need to be in the `database` directory so the `sqlx` can access all database migration files:

```
## assuming the DATABASE_URL environment variable is setup properly and a database named "ai_gent" does not exist.
sqlx database create
sqlx migrate run
```

You can also use the script in `bootstraping.sh` in the `database` directory. It uses `psql` to create the database and load pre-existing examples of agents, assets, and chat sessions. You still need to set the correct `PGUSER` and `PGPASSWORD` or have other means to get the authentication to access your database. See `ai-gent-smith/ai_gent_web/docker/load_db.sh` for an example on how to set it up in a Docker container. 

7. Set up the environment variables for LLM API calls: 
    - `OPENAI_API_KEY` for OpenAI APIs
    - `ANTHROPIC_API_KEY` for Anthropic APIs

8. Build the project: 

```
cd ai_gent_web
cargo run --release
```

9. Point your browser to `http://127.0.0.1:8080` to use the AI-Gent Smith

10. It may take a while of the web server to start up as it needs download model weight (for tokenization and embedding vector from Hugging face)

![Chat Interface](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/chat_interface.png?raw=true)

### For Mac ARM64 (Apple Silicon M1/M2/M3/M4) Docker Users

If you are not an experienced developer of Rust and PostgreSQL, you should just try it out using docker as the database installation is taken care of. 

1. Install Docker Desktop for Mac from [https://www.docker.com/products/docker-desktop/](https://www.docker.com/products/docker-desktop/)

2. Run the prebuilt docker container: 

```
docker run -p 8080:8080 -e OPENAI_API_KEY=$OPENAI_API_KEY cschin/ai_gent_web-arm64
```

You can add `ANTHROPIC_API_KEY` too.

3. Point your browser to `http://127.0.0.1:8080` to use the AI-Gent Smith

4. It may take a while of the web server to start up as it needs download model weight (for tokenization and embedding vector from Hugging face)


### For Intel/AMD64 architecture Docker Users

If you are not an experienced developer of Rust and PostgreSQL, you should just try it out using docker as the database installation is taken care of. 

1. Install Docker Desktop from [https://www.docker.com/products/docker-desktop/](https://www.docker.com/products/docker-desktop/) or from [CLI] (https://docs.docker.com/engine/install/)

2. Run the prebuilt docker container: 

```
docker run -p 8080:8080 -e OPENAI_API_KEY=$OPENAI_API_KEY cschin/ai_gent_web-amd64
```
You can add `ANTHROPIC_API_KEY` too.

3. Point your browser to `http://127.0.0.1:8080` to use the AI-Gent Smith

4. It may take a while of the web server to start up as it needs download model weight (for tokenization and embedding vector from Hugging face)

## Finite State Machine Agent

### Create a "Finite State Machine" Agent

In Ai-Gent Smith, we implement the agent using a finite state machine. The agent can be in a specific state at any given time. It will use this state to generate corresponding prompts and apply different state-dependent prompts. This allows the agent to simulate certain dialog scenarios or perform specific tasks before responding.  

For example, it might be desirable for the agent to infer the user's role and tailor its responses accordingly. A user with a different role may receive a more suitable response. To achieve this, we can utilize a simple finite state machine to manage state transitions in a chat session.  

The user can specify states and their associated prompts through a configuration file to create a specific agent. You can refer to the file [`ai_gent_web/dev_config/fda_challenge_example_1.toml`](ai_gent_web/dev_config/fda_challenge_example_1.toml) for an example.  

The following section in the configuration file sets up the possible states, the transitions between them, and the initial state:  


```toml
states = ["StandBy", 
          "ForChiefExecutiveOfficer", 
          "ForScientist", 
          "ForRegulatoryConsultant", 
          "ForCustomer"]

transitions = [
  ["StandBy", "StandBy"],
  
  ["StandBy", "ForChiefExecutiveOfficer"],
  ["StandBy", "ForScientist"],
  ["StandBy", "ForRegulatoryConsultant"],
  ["StandBy", "ForCustomer"],
  
  ["ForChiefExecutiveOfficer", "ForChiefExecutiveOfficer"],
  ["ForChiefExecutiveOfficer", "ForScientist"],
  ["ForChiefExecutiveOfficer", "ForRegulatoryConsultant"],
  ["ForChiefExecutiveOfficer", "ForCustomer"],

  ["ForScientist", "ForChiefExecutiveOfficer"],
  ["ForScientist", "ForScientist"],
  ["ForScientist", "ForRegulatoryConsultant"],
  ["ForScientist", "ForCustomer"],

 ...
]

initial_state = "StandBy"
```

In this example, we have a "StandBy" state. Once the agent receives user input, it uses another "FSM" agent and the transition table above to "guess" what the user's role might be. We send the user input along with a special "fsm_prompt" that utilizes an LLM to determine the next finite state machine state.  

In our case, the states following "StandBy" represent a set of possible roles that a user can have.  For example, the user could be a CEO, a Regulatory Consultant, a Customer, or a Scientist.  

If the user does not provide any specific useful information for the LLM to make a guess, it may transition back to the "StandBy" mode.  


Here is an excerpt of the `fsm_prompt` (see [`ai_gent_web/dev_config/fda_challenge_example_1.toml`](ai_gent_web/dev_config/fda_challenge_example_1.toml) for a full example):  

```toml
fsm_prompt = """
Please determine the next state based on the user's message and the current state.
The state represents the possible role the agent should adopt in answering the user's question.
...
```

We can specify the prompt for each state in prompt section:
```toml
[prompts]

StandBy = """ You are in the StandBy state. Your goal is to inform the user that you
are ready to answer new questions if they have any. Welcome the user and encourage
them to ask new questions. """
...
```

Once the state of the agent is determined, a final prompt will be generated using the `sys_prompt` and the state prompt, along with the user's query and search context from the user's input (in the case of retrieval-augmented generation) to get a response from the LLM APIs.  

In the current codebase, we do not send the entire conversation history. Instead, we generate a summary of the conversation each time we receive a response from the LLM. For each user query, we send the previous summary along with the new user query to the LLM.  

This means the agent does not have true "long-term memory" if the summarization only considers the latest message.  

However, this behavior can be easily modified. We could send the last few messages along with the summary or extend the summary length to retain more context. This is an area worth experimenting with for different use cases. One can control how summarization is done by modifying the `summary_prompt`  in the configuration file.  

The configuration toml file will be de-serialized to the following Rust struct:
```rust
pub struct FSMAgentConfig {
    pub states: Vec<String>,
    pub transitions: Vec<(String, String)>,
    pub initial_state: String,
    pub prompts: HashMap<String, String>,
    pub sys_prompt: String,
    pub summary_prompt: String,
    pub fsm_prompt: String,
}
```
If any required field is missing in the configuration file, the UI will not allow the configuration to be loaded when creating a new finite state machine agent through the UI.  

We also provide an interface to create a "Basic Agent," which has only three states:  `InitialResponse`, `FollowUp`, and `StandBy`. You can customize the prompts used for `InitialResponse` and `FollowUp` directly through the UI.  

You can find the configuration for a "Basic Agent" in the file  
[`ai_gent_web/templates/simple_agent_config.toml.template`](ai_gent_web/templates/simple_agent_config.toml.template).  

## Usage 

### Create Asset JSONL file

The web server provides API for some generic text chunking and get embedding vector from PDF file. You can use the script `supporting_scripts/pdf_to_embedding` to generate the `jsonl` file for creating new asset for a new RAG agent.

For example, if you have a collection of the PDF in a directory `pdf_files/`, you can run

```
python pdf_to_embedding.py --input-dir=pdf_files/ -o asset.jsonl 
```

It generates the `asset.jsonl` that can be used to upload the Ai-Gent Smith through UI. Not that the script connects to the local Ai-Gent Smith server (http://127.0.0.1:8080) through a HTTP request to get the embedding vectors. You need to start the Ai-Gent Simth before you can use `pdf_to_embedding.py`. See more in [`supporting_scripts/pdf_to_embedding`](supporting_scripts/pdf_to_embedding).

A pregenerated asset file for the pFDA challenge is at [`ai_gent_web/dev_config/fda_challenge_example_asset.jsonl.gz`](ai_gent_web/dev_config/fda_challenge_example_asset.jsonl.gz)`. You can upload the file through the UI interface (see below).

### Upload the Asset JSONL
The following screenshot shows how to upload an asset through "Asset Library > Create Asset" on the left panel
![CreateAsset1](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAsset1.png?raw=true)

If the asset file is big, it may take a while for the system to process, please wait until you to see the dialog box like below showing up to continue.
![CreateAsset2](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAsset2.png?raw=true)

Once the asset is showing up in the "Asset Library", you can click the "show" button on the asset card to see some of the content in the asset.
![CreateAsset3](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAsset3.png?raw=true)

You can see the table of content of the asset
![CreateAsset4](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAsset4.png?raw=true)

#### Embedding Map
For each set of asset (collections of documents), the `pdf_to_embedding.py` generate a two dimensional UMAP project from the high dimensional embedding space too. You can clikc the plot, the document that is "near" where you click in the embedding space will be highlighted.  ![CreateAsset5](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAsset5.png?raw=true)


### Create a Finite State Machine Agent

Use the "Agent Library > Create An Advanced Agent" button to show the Finite State Machine Agent configuration form: 

![CreateAgent1](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAgent1.png?raw=true)

You can select a specific model and an asset for the new agent.
![CreateAgent2](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAgent2.png?raw=true)

### Create a Basic Agent

Use the "Agent Library > Create A Basic Agent" button to show the Finite State Machine Agent configuration form: 

![CreateAgent3](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/CreateAgent3.png?raw=true)

### Use The Agents
Use the "Agent Library > Show Agent Library" to show current available library, click the "use" to start work with a chat agent.
![UseAgent1](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/UseAgent1.png?raw=true)

Once you click the "Use" button, the chat interface will appear. You can enter your queries in the input field at the bottom and click the "Send" button to retrieve the relevant context in the asset associated with the agent and process your query through the LLM.  

If you want to see the retrieved context without sending the query to an LLM model, you can use the "Search Asset" button. 

The asset search results are displayed on the right side of the workspace, and you can adjust various retrieval parameters using the sliders in the lower-right corner.  

Additionally, you can download the current  chat and agent configuration using the "Download" button or start a new session  with the "New Session" button.  
![UseAgent2](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/UseAgent2.png?raw=true)

#### Chat Examples

You can find the previous chat session using the "Sessions" tab on the left panel for different time periods
![ChatSession1.png](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/ChatSession1.png?raw=true)


Here are some previously generated chat log:

- [With gpt-4o](https://github.com/cschin/ai-gent-smith/blob/main/misc/chat_output/chat-gpt-4o.json)
- [With gpt-4o-mini](https://github.com/cschin/ai-gent-smith/blob/main/misc/chat_output/chat-gpt-4o-mini.json)
- [With gpt-3.5-turbo](https://github.com/cschin/ai-gent-smith/blob/main/misc/chat_output/chat-gpt-3.5-turbo.json)
- [With gpt-4o and a party goer personality](https://github.com/cschin/ai-gent-smith/blob/main/misc/chat_output/chat-gpt-4o-party-mode.json)


## Technologies Used

- [Rust](https://www.rust-lang.org)
- [Tron](https://github.com/cschin/tron): a server side Ruse UI library for Web developed by Jason Chin with [axum](https://github.com/tokio-rs/axum) and [htmx](https://htmx.org)  
- [Candle](https://github.com/huggingface/candle): Minimalist ML framework for Rust developed by Huggingface 
- [genai](https://github.com/jeremychone/rust-genai): supporting multi LLM vendor API calls
- [PostgreSQL](https://www.postgresql.org) + [pgvector](https://github.com/pgvector/pgvector) for vector query
- See the `Cargo.toml` files for many other great tools used for this project.

## Project Structure

Brief overview of the main directories and their purposes:

- `/ai_gent_lib`: Supporting libraries
- `/ai_gent_tools`: Command line tools for development
- `/ai_gent_web`: The web UI application
- `/database`: For database support
- `/misc`: Miscellaneous 


## Supported LLM Vendors and Models

- OpenAI: gpt-4o, gpt-4o-mini, gpt-3.5-turbo, o3-mini
- Anthropic: claude-3-haiku-20240307, claude-3-5-sonnet-20241022

It can support [Ollama](https://ollama.com) too but you need to modify a few lines of the source code.

## License

```
Copyright (c) <2025> <copyright Jason Chin>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```
