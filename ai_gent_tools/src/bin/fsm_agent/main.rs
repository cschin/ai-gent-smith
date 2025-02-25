use ai_gent_lib::fsm_chat_state::FsmAgentState;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tokio::io::{self, AsyncReadExt};
use tokio::sync::mpsc::{Receiver, Sender};

use anyhow::Result;

use ai_gent_lib::llm_agent::{AgentSettings, LlmFsmAgent, LlmFsmAgentConfigBuilder, LlmFsmBuilder};

use tokio::sync::mpsc;

use clap::Parser;

// Define a struct to represent the command line arguments
#[derive(Parser)]
#[command(
    name = "AI-Gent Smith",
    version = "0.1",
    author = "Jason Chin",
    about = "Who don't write an AI-agent these days?"
)]
struct Cli {
    /// Path to the file to read
    #[arg(short, long)]
    config_file: String,
    #[arg(short, long)]
    input: Option<String>,
}

use std::collections::HashMap;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Parse the command line arguments
    let cli = Cli::parse();

    // Read the file into a string
    let content = fs::read_to_string(cli.config_file)?;

    let fsm_config = LlmFsmAgentConfigBuilder::from_toml(&content)?.build()?;

    let fsm =
        LlmFsmBuilder::from_config::<FsmAgentState>(&fsm_config, HashMap::default())?.build()?;

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
            env_name: "OPENAI_API_KEY".to_string(),
        })
        .unwrap();

    let llm_req_setting = AgentSettings {
        sys_prompt: fsm_config.system_prompt,
        fsm_prompt: fsm_config.fsm_prompt,
        summary_prompt: fsm_config.summary_prompt,
        model: "gpt-4o-mini".into(),
        api_key,
        fsm_initial_state: fsm_config.initial_state,
        tools: fsm_config.tools,
        total_state_transition_limit: Some(32),
    };
    let mut agent = LlmFsmAgent::new(fsm, llm_req_setting);

    // tracing::info!("agent config: {}", fsm_config.to_json().unwrap());

    //write_agent_config_to_file(&fsm_config);

    let (fsm_tx, fsm_rx) = mpsc::channel::<(String, String, String)>(1);
    let (send_msg, rcv_msg) = mpsc::channel::<(String, String)>(8);
    let agent_handler = tokio::spawn(async move {
        agent
            .agent_message_service(rcv_msg, fsm_tx.clone(), None)
            .await
    });

    match cli.input {
        Some(input) if input == "-" => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).await?;
            run_non_interactive(send_msg, fsm_rx, buffer).await?;
        }
        Some(input) => {
            // Non-interactive mode with provided input
            run_non_interactive(send_msg, fsm_rx, input).await?;
        }
        None => {
            // Interactive mode
            run_interactive(send_msg, fsm_rx).await?;
        }
    }

    let _ = tokio::join!(agent_handler).0.unwrap();
    Ok(())
}

async fn run_interactive(
    send_msg: Sender<(String, String)>,
    mut fsm_rx: Receiver<(String, String, String)>,
) -> Result<(), anyhow::Error> {

    let mut rl = DefaultEditor::new()?;

    println!("\n ========== Welcome to the Ai-gent Smith. ========== \n Type 'exit' to quit.");

    loop {
        let readline = rl.readline("\n>> ");
        match readline {
            Ok(user_input) => {
                if user_input.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(user_input.as_str());

                let _ = send_msg.send(("task".into(), user_input.clone())).await;
                let _ = send_msg.send(("message".into(), user_input)).await;

                process_fsm_messages(&mut fsm_rx).await;
            }
            Err(ReadlineError::Interrupted) => {
                let _ = send_msg.send(("terminate".into(), "".into())).await;
                fsm_rx.close();
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                let _ = send_msg.send(("terminate".into(), "".into())).await;
                fsm_rx.close();
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                let _ = send_msg.send(("terminate".into(), "".into())).await;
                fsm_rx.close();
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

async fn run_non_interactive(
    send_msg: Sender<(String, String)>,
    mut fsm_rx: Receiver<(String, String, String)>,
    user_input: String,
) -> Result<(), anyhow::Error> {
    // Send the user input as a task and message
    let _ = send_msg.send(("task".into(), user_input.clone())).await;
    let _ = send_msg.send(("message".into(), user_input)).await;

    process_fsm_messages(&mut fsm_rx).await;

    Ok(())
}

async fn process_fsm_messages(fsm_rx: &mut mpsc::Receiver<(String, String, String)>) -> Vec<String> {
    let mut state_service_count = 0;
    let mut llm_output = Vec::new();

    while let Some(message) = fsm_rx.recv().await {
        match (message.0.as_str(), message.1.as_str()) {
            (_, "state") => {
                state_service_count += 1;
                println!(
                    "\n\n-------{:02}/32 Agent State: {}\n",
                    state_service_count, message.2
                );
            }

            (_, "next_state") => {
                state_service_count += 1;
                println!(
                    "\n\n-------Next State: {}\n",
                    message.2
                );
            }
            (s, "token") if s != "MakeSummary" => {
                print!("{}", message.2);
            }
            (state_name, "exec_output") => {
                println!(
                    "exec_output received, state:{}, len={}",
                    state_name,
                    message.2.len()
                );
                println!("{}", message.2);
                llm_output.push(message.2);
            }
            (_, "llm_output") => {
                llm_output.push(message.2);
            }
            (state_name, "error") => {
                eprintln!(
                    "Error received from state '{}': '{}'",
                    state_name, message.2
                )
            }
            (_, "message_processed") => {
                println!("message_processed, processing complete");
                break;
            }
            _ => {}
        }
    }

    llm_output
}