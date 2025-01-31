# AI-Gent Smith

![AI-Gent Smith](https://github.com/cschin/ai-gent-smith/blob/main/misc/images/ai_gent_web.png?raw=true)

"AI-Gent Smith" is a simple project demonstrating the use of Rust and various LLM libraries to build a web-based LLM agent library for users. A user can create a simple agent by providing prompts and selecting a backend model. However, it is designed to support LLM agents with "state," allowing the agent to use different prompts or take different actions depending on the current state of the dialogue.

A user can define a finite state machine for state transitions and specify the prompts (or actions) that determine how the agent responds in a given state. State transitions can also be determined using an LLM. I have not fully tested it with more complex scenarios yet, but it works well for simple Q&A cases.

A user can create multiple agents, which can be updated or deleted through a web UI. Conversations can be saved and browsed. The project supports retrieval-augmented generation (RAG), and in the future, we plan to add support for "asset" creation and management.

Currently, we provide a small document dataset from an precision FDA challenge: [PrecisionFDA Generative Artificial Intelligence (GenAI) Community Challenge: Democratizing and Demystifying AI as the default dataset](https://precision.fda.gov/challenges/34/intro), with pre-configured agents allowing users to query related documents to answer a set of questions for the challenge.


## Table of Contents

- [Installation](#installation)
- [Technologies Used](#technologies-used)
- [Project Structure](#project-structure)
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
6. create the database: `pushd database; bash bootstraping.sh; popd`, you may need to setup the correct `PGUSER` and `PGPASSWORD` for authentication to create the database with `sqlx` (see `ai-gent-smith/ai_gent_web/docker/load_db.sh` for example). 
7. Copy the data file: 
```
cp data/all_embedding.jsonl.gz /opt/data/all_embedding.jsonl.gz
```
If you don't have access to `/opt`, you can modify the source to a place where you have
permission for the data file.
8. Set up the environment variables: 
    - `OPENAI_API_KEY` for OpenAI APIs
    - `ANTHROPIC_API_KEY` for Anthropic APIs
    - `DATABASE_URL` for AI-Gent Smith to accese the postgresql database, for example if your
    user name is `db_user`, you can do `expose DATABASE_URL=postgres://db_user@localhost/ai_gent` under a command line shell prompt if your postgresql user name is `db_user`.
8. Build the project: 
```
cd ai_gent_web
cargo run --release
```
9. Point your browser to `http://127.0.0.1:8080` to use the AI-Gent Smith
10. It may take a while of the web server to start up as it needs download model weight (for tokenization and embedding vector from Hugging face)


### For Mac ARM64 (Apple Silicon M1/M2/M3/M4) Docker Users

If you are not an experienced developer of Rust and PostgreSQL, you should just try it out
using docker as the database installation is taken care of. 

1. Install Docker Desktop for Mac from [https://www.docker.com/products/docker-desktop/](https://www.docker.com/products/docker-desktop/)
2. Run the prebuilt docker container: 
```
docker run -p 8080:8080 -e OPENAI_API_KEY=$OPENAI_API_KEY cschin/ai_gent_web-arm64
```
You can add `ANTHROPIC_API_KEY` too.
3. Point your browser to `http://127.0.0.1:8080` to use the AI-Gent Smith
4. It may take a while of the web server to start up as it needs download model weight (for tokenization and embedding vector from Hugging face)


### For Intel/AMD64 architecture Docker Users

If you are not an experienced developer of Rust and PostgreSQL, you should just try it out
using docker as the database installation is taken care of. 

1. Install Docker Desktop from [https://www.docker.com/products/docker-desktop/](https://www.docker.com/products/docker-desktop/) or from [CLI] (https://docs.docker.com/engine/install/)
2. Run the prebuilt docker container: 
```
docker run -p 8080:8080 -e OPENAI_API_KEY=$OPENAI_API_KEY cschin/ai_gent_web-amd64
```
You can add `ANTHROPIC_API_KEY` too.
3. Point your browser to `http://127.0.0.1:8080` to use the AI-Gent Smith
4. It may take a while of the web server to start up as it needs download model weight (for tokenization and embedding vector from Hugging face)


## Technologies Used

- [Rust](https://www.rust-lang.org)
- [Tron: a server side Ruse UI library for web UI with htmx](https://github.com/cschin/tron)  
- [Candle: A Rust deep learning library developed by Hugging face ](https://github.com/huggingface/candle)
- [genai: supporting multi LLM vendor API calls](https://github.com/jeremychone/rust-genai)

## Project Structure

Brief overview of the main directories and their purposes:

- `/ai_gent_lib`: Supporting libraries
- `/ai_gent_tools`: Command line tools for development
- `/ai_gent_web`: The web UI application
- `/database`: For database support
- `/misc`: Miscellaneous 

## License
```
Copyright (c) <2025> <copyright Jason Chin>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```




