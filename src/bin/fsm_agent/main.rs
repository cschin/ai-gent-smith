pub mod fsm;
pub mod llm_agent;
pub mod llm_service;

use std::{collections::HashMap, sync::Arc};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use anyhow::Result;
use async_trait::async_trait;
use fsm::{FSMState, FiniteStateMachine, FiniteStateMachineBuilder};

use llm_agent::{FSMAgentConfig, FSMAgentConfigBuilder, LLMAgent, LLMClient};

// use futures::StreamExt;
use llm_service::{openai_service, openai_stream_service, LLMStreamOut};
struct TestLLMClient {}

const SYS_PROMPT: &str = include_str!("../../../sys_prompt");
const STANDBY_PROMPT: &str = include_str!("../../../standby_prompt");
const RESPONSE_PROMPT: &str = include_str!("../../../resp_prompt"); 
const FOLLOWUP_PROMPT: &str = include_str!("../../../followup_prompt"); 

#[async_trait]
impl LLMClient for TestLLMClient {
    async fn generate(&self, prompt: &str, msgs: &Vec<(String, String)>) -> String {
        // r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
        //     .to_string()
        openai_service(prompt, msgs).await 
    }

    async fn generate_stream(&self, prompt: &str, msg: &str) -> LLMStreamOut {
        openai_stream_service(prompt, msg).await
    }
}

struct StandBy;
struct InitialResponseState;
struct AskFollowUpQuestionState;

#[async_trait]
impl FSMState for StandBy {
    async fn on_enter(&self) {
        println!("Entered StandBy State");
    }
    async fn on_exit(&self) {
        println!("Exited StandBy State");
    }
    async fn clone_attribute(&self, _k: &str) -> Option<String> {
        None
    }
    fn name(&self) -> String {
        "StandBy".to_string()
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

#[async_trait]
impl FSMState for AskFollowUpQuestionState {
    async fn on_enter(&self) {
        println!("Entered AskFollowUpQuestion State");
    }
    async fn on_exit(&self) {
        println!("Exited AskFollowUpQuestion State");
    }
    fn name(&self) -> String {
        "AskFollowUpQuestion".to_string()
    }
}

#[async_trait]
impl FSMState for InitialResponseState {
    async fn on_enter(&self) {
        println!("Entered Response State");
    }
    async fn on_exit(&self) {
        println!("Exited Response State");
    }
    fn name(&self) -> String {
        "InitialResponseState".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let fsm_config = FSMAgentConfigBuilder::new()
    .add_state("StandBy".to_string())
    .add_state("InitialResponse".to_string())
    .add_state("AskFollowUpQuestion".to_string())
    .add_transition("StandBy".to_string(), "StandBy".to_string())
    .add_transition("StandBy".to_string(), "InitialResponse".to_string())
    .add_transition("StandBy".to_string(), "AskFollowUpQuestion".to_string())
    .add_transition("InitialResponse".to_string(), "AskFollowUpQuestion".to_string())
    .add_transition("InitialResponse".to_string(), "StandBy".to_string())
    .add_transition("AskFollowUpQuestion".to_string(), "AskFollowUpQuestion".to_string())
    .add_transition("AskFollowUpQuestion".to_string(), "StandBy".to_string())
    .set_initial_state("StandBy".to_string())
    .add_prompt("StandBy".to_string(), [SYS_PROMPT, STANDBY_PROMPT].join("\n"))
    .add_prompt("InitialResponse".to_string(), [SYS_PROMPT, RESPONSE_PROMPT].join("\n"))
    .add_prompt("AskFollowUpQuestion".to_string(), [SYS_PROMPT, FOLLOWUP_PROMPT].join("\n"))
    .set_sys_prompt(SYS_PROMPT.into())
    .build().unwrap();

    let fsm = FiniteStateMachineBuilder::from_config(&fsm_config)?
    .build()?;

    let llm_client = TestLLMClient {};
    let mut agent = LLMAgent::new(fsm, llm_client);


    println!("agent config: {}", fsm_config.to_json().unwrap());
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


                if let Ok(res) = agent.process_input(&line).await {
                    println!("Response: {}", res);
                } else {
                    println!("LLM error, please retry your question.");
                }

                if let Some(current_state) = agent.fsm.current_state() {
                    println!("Current state: {}", current_state);
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
