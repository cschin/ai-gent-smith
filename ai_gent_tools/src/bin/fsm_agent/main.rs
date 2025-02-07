use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use ai_gent_lib::fsm::FSMBuilder;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use ai_gent_lib::llm_agent::{FSMAgentConfigBuilder, LLMAgent, LLMClient};

// use futures::StreamExt;
use ai_gent_lib::llm_service::{genai_service, genai_stream_service, LLMStreamOut};
struct TestLLMClient {
    model: String,
    api_key: String,
}

//const SYS_PROMPT: &str = include_str!("../../../dev_config/sys_prompt");
//const FSM_PROMPT: &str = include_str!("../../../dev_config/fsm_prompt");
//const SUMMARY_PROMPT: &str = include_str!("../../../dev_config/summary_prompt");

#[async_trait]
impl LLMClient for TestLLMClient {
    async fn generate(&self, prompt: &str, msgs: &[(String, String)], temperature: Option<f32>) -> String {
        // r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
        //     .to_string()
        let t = temperature.unwrap_or(0.5); 
        genai_service(prompt, msgs, &self.model, &self.api_key, t).await
    }

    async fn generate_stream(&self, prompt: &str, msgs: &[(String, String)], temperature: Option<f32>) -> LLMStreamOut {
        let t = temperature.unwrap_or(0.5); 
        genai_stream_service(prompt, msgs, &self.model, &self.api_key, t).await
    }
}

const FSM_CONFIG: &str = include_str!("../../../dev_config/fsm_config.json");

// use std::fs::File;
// use std::io::Write;
// use llm_agent::FSMAgentConfig;
// fn write_agent_config_to_file(fsm_config: &FSMAgentConfig) -> Result<(), std::io::Error> {
//     let json_output = fsm_config.to_json().unwrap();
//     let mut file = File::create("agent_config.json")?;
//     file.write_all(json_output.as_bytes())?;
//     tracing::info!("Agent config written to agent_config.json");
//     Ok(())
// }

use std::io::{stdout, Write}; //for flush()

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fsm_config = FSMAgentConfigBuilder::from_json(FSM_CONFIG)?.build()?;

    let fsm = FSMBuilder::from_config(&fsm_config)?.build()?;

    let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
        env_name: "OPENAI_API_KEY".to_string()}).unwrap();

    let llm_client = TestLLMClient {
        model: "gpt-4o".into(),
        api_key
    };
    let fsm_config = FSMAgentConfigBuilder::from_json(FSM_CONFIG).unwrap().build().unwrap();
    let mut agent = LLMAgent::new(llm_client, fsm, &fsm_config);

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
