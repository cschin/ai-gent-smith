

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use anyhow::Result;
use async_trait::async_trait;
use ai_gent_lib::fsm::FSMBuilder;

use ai_gent_lib::llm_agent::{FSMAgentConfigBuilder, LLMAgent, LLMClient};

// use futures::StreamExt;
use ai_gent_lib::llm_service::{openai_service, openai_stream_service, LLMStreamOut};
struct TestLLMClient {}


#[async_trait]
impl LLMClient for TestLLMClient {
    async fn generate(&self, prompt: &str, msgs: &[(String, String)]) -> String {
        // r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
        //     .to_string()
        openai_service(prompt, msgs).await
    }

    async fn generate_stream(&self, prompt: &str, msg: &str) -> LLMStreamOut {
        openai_stream_service(prompt, msg).await
    }
}

// #[derive(Debug)]
// struct InitialState<C: LLMClient + Sync + Send> {
//     attributes: HashMap<String, String>,
//     llm_client: Arc<C>,
// }

// #[async_trait]
// impl<C: LLMClient + Sync + Send> FSMState for InitialState<C> {
//     async fn on_exit(&self) {
//         println!("Exited then LMM Processing State");
//     }
//     async fn on_enter_mut(&mut self) {
//         println!("Entered the LMM Processing State");
//         let guard = self.llm_client.clone();
//         let prompt = self.attributes.get("prompt").unwrap();
//         let msg = self.attributes.get("msg").unwrap();
//         let mut llm_stream = guard.generate_stream(prompt, msg).await;
//         while let Some(result) = llm_stream.next().await {
//             if let Some(output) = result {
//                 print!("{}", output);
//             }
//         }
//         println!();
//     }
//     async fn on_exit_mut(&mut self) {}
//     fn name(&self) -> String {
//         "exit the LMM Processing processing".to_string()
//     }
//     async fn set_attribute(&mut self, k: &str, v: String) {
//         self.attributes.insert(k.into(), v);
//     }
//     async fn clone_attribute(&self, k: &str) -> Option<String> {
//         self.attributes.get(k).cloned()
//     }
// }

const FSM_CONFIG: &str = include_str!("../../../../ai_gent_lib/dev_config/fsm_config.json");

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fsm_config = FSMAgentConfigBuilder::from_json(FSM_CONFIG)?.build()?;
   
    let fsm = FSMBuilder::from_config(&fsm_config)?.build()?;

    let llm_client = TestLLMClient {};
    let mut agent = LLMAgent::new(fsm, llm_client);

    tracing::info!("agent config: {}", fsm_config.to_json().unwrap());

    //write_agent_config_to_file(&fsm_config); 
   

    println!("Welcome to the LLM Agent CLI. Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?; // Use DefaultEditor instead
    loop {
        let readline = rl.readline("\n>> ");
        match readline {
            Ok(line) => {
                if line.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(line.as_str());

                match agent.process_input(&line).await {
                    Ok(res) => println!("Response: {}", res),
                    Err(err) => println!("LLM error, please retry your question. {:?}", err),
                }

                if let Some(current_state) = agent.fsm.current_state() {
                    tracing::info!("Current state: {}", current_state);
                }
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

    Ok(())
}
