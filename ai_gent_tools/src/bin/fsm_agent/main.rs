use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use ai_gent_lib::fsm::FSMBuilder;
use anyhow::Result;
use tokio::sync::mpsc;

use ai_gent_lib::llm_agent::{AgentSettings, FSMAgentConfigBuilder, LLMAgent};

// use futures::StreamExt;

const FSM_CONFIG: &str = include_str!("../../../dev_config/fsm_config.json");


use std::collections::HashMap;
use std::io::{stdout, Write}; //for flush()

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fsm_config = FSMAgentConfigBuilder::from_json(FSM_CONFIG)?.build()?;

    let fsm = FSMBuilder::from_config::<ai_gent_lib::fsm::DefaultFSMChatState>(&fsm_config, HashMap::default())?.build()?;

    let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
        env_name: "OPENAI_API_KEY".to_string()}).unwrap();

    let fsm_config = FSMAgentConfigBuilder::from_json(FSM_CONFIG).unwrap().build().unwrap();
    let llm_req_setting = AgentSettings {
        sys_prompt: fsm_config.sys_prompt,
        fsm_prompt: fsm_config.fsm_prompt,
        summary_prompt: fsm_config.summary_prompt,
        model: "gpt-4o".into(),
        api_key,
    };
    let mut agent = LLMAgent::new(fsm, llm_req_setting);

    // tracing::info!("agent config: {}", fsm_config.to_json().unwrap());

    //write_agent_config_to_file(&fsm_config);

    println!("Welcome to the LLM Agent CLI. Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?; // Use DefaultEditor instead
    let (tx, mut rx) = mpsc::channel::<(String,String)>(8);

    let t = tokio::spawn(async move {
        while let Some(_message) = rx.recv().await {
            stdout().write(b".").and(stdout().flush()).unwrap();
        }
    });

    loop {
        let readline = rl.readline("\n>> ");
        match readline {
            Ok(line) => {
                if line.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(line.as_str());

                match agent.process_message(&line, Some(tx.clone()), None).await {
                    Ok(res) => println!("\nResponse: {}", res),
                    Err(err) => println!("LLM error, please retry your question. {:?}", err),
                }

                // if let Some(current_state) = agent.fsm.current_state() {
                //     tracing::info!("Current state: {}", current_state);
                // }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    t.abort();
    Ok(())
}
