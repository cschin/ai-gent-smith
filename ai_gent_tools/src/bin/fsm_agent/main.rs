use ai_gent_lib::fsm_chat_state::FSMChatState;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use anyhow::Result;

use ai_gent_lib::llm_agent::{
    AgentSettings, LlmFsmAgent, LlmFsmAgentConfigBuilder, LlmFsmBuilder, 
};

use tokio::sync::mpsc;

use clap::Parser;
use std::fs;

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
}

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command line arguments
    let args = Cli::parse();

    // Read the file into a string
    let content = fs::read_to_string(args.config_file)?;

    let fsm_config = LlmFsmAgentConfigBuilder::from_toml(&content)?.build()?;

    let fsm =
        LlmFsmBuilder::from_config::<FSMChatState>(&fsm_config, HashMap::default())?.build()?;

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
            env_name: "OPENAI_API_KEY".to_string(),
        })
        .unwrap();

    let llm_req_setting = AgentSettings {
        sys_prompt: fsm_config.system_prompt,
        fsm_prompt: fsm_config.fsm_prompt,
        summary_prompt: fsm_config.summary_prompt,
        model: "gpt-4o".into(),
        api_key,
        fsm_initial_state: fsm_config.initial_state,
        tools: fsm_config.tools,
        total_state_transition_limit: None,
    };
    let mut agent = LlmFsmAgent::new(fsm, llm_req_setting);

    // tracing::info!("agent config: {}", fsm_config.to_json().unwrap());

    //write_agent_config_to_file(&fsm_config);

    println!("\n ========== Welcome to the Ai-gent Smith. ========== \n Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?; // Use DefaultEditor instead

    let (fsm_tx, mut fsm_rx) = mpsc::channel::<(String, String, String)>(8);
    let (send_msg, rcv_msg) = mpsc::channel::<(String, String)>(8);
    let agent_handler = tokio::spawn(async move {
        agent
            .fsm_message_service(rcv_msg, fsm_tx.clone(), None)
            .await
    });

    loop {
        let readline = rl.readline("\n>> ");
        match readline {
            Ok(user_input) => {
                if user_input.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(user_input.as_str());

                // let _ = send_msg.send(("clear_message".into(), "".into())).await;
                let _ = send_msg.send(("task".into(), user_input.clone())).await;

                // this should the last command sent, it will trigger the server to start to response 
                let _ = send_msg.send(("message".into(), user_input)).await;

                let mut llm_output = Vec::<String>::new();

                while let Some(message) = fsm_rx.recv().await {
                    match (message.0.as_str(), message.1.as_str()) {
                        (_, "state") => {
                            println!("\n\n--------- Agent State: {}\n", message.2);
                        }
                        (s, "token") if s != "MakeSummary" => {
                            print!("{}", message.2);
                        }
                        (_, "output") => {
                            print!("{}", message.2);
                            llm_output.push(message.2);
                        }
                        (state_name, "exec_output") => {
                            println!("exec_output received, state:{}, len={}", state_name, message.2.len());
                            println!("{}", message.2);
                            llm_output.push(message.2);
                        }
                        (_, "llm_output") => {
                            llm_output.push(message.2);
                        }
                        (state_name, "error") => {
                            eprintln!("Error received from state '{}': '{}'", state_name, message.2)
                        }
                        (_, "message_processed") => {
                            println!("message_processed, wait for the next user input"); // clear rustyline's buffer
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                let _ = send_msg.send(("terminate".into(), "".into())).await;
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                let _ = send_msg.send(("terminate".into(), "".into())).await;
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                let _ = send_msg.send(("terminate".into(), "".into())).await;
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    // let _ = tokio::join!(message_handler).0.unwrap();
    let _ = tokio::join!(agent_handler).0.unwrap();
    Ok(())
}
