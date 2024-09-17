pub mod fsm;
pub mod llm_agent;
pub mod llm_service;

use std::{collections::HashMap, sync::Arc};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use anyhow::Result;
use async_trait::async_trait;
use fsm::{FSMState, FiniteStateMachine};

use llm_agent::{LLMAgent, LLMClient};

use futures::StreamExt;
use llm_service::{openai_stream_service, LLMStreamOut};
struct TestLLMClient {}

#[async_trait]
impl LLMClient for TestLLMClient {
    async fn generate(&self, _prompt: &str, _msg: &str) -> String {
        r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
            .to_string()
    }

    async fn generate_stream(&self, prompt: &str, msg: &str) -> LLMStreamOut {
        openai_stream_service(prompt, msg).await
    }
}

// Define states for the FSM
#[derive(Debug, Default)]
struct WaitInputState;

#[derive(Debug)]
struct ProcessingState<C: LLMClient + Sync + Send> {
    attributes: HashMap<String, String>,
    llm_client: Arc<C>,
}
struct ResponseState;

#[async_trait]
impl FSMState for WaitInputState {
    async fn on_enter(&self) {
        println!("Entered WaitForInput State");
    }
    async fn on_exit(&self) {
        println!("Exited WaitForInput State");
    }
    async fn on_enter_mut(&mut self) {}
    async fn on_exit_mut(&mut self) {}
    async fn set_attribute(&mut self, _k: &str, _v: String) {}
    async fn clone_attribute(&self, _k: &str) -> Option<String> {
        None
    }
    fn name(&self) -> String {
        "WaitForInput".to_string()
    }
}

#[async_trait]
impl<C: LLMClient + Sync + Send> FSMState for ProcessingState<C> {
    async fn on_enter(&self) {}
    async fn on_exit(&self) {
        println!("Exited Processing State");
    }
    async fn on_enter_mut(&mut self) {
        println!("Entered Processing State");
        let guard = self.llm_client.clone();
        let prompt = self.attributes.get("prompt").unwrap();
        let msg = self.attributes.get("msg").unwrap();
        let mut llm_stream = guard.generate_stream(prompt, msg).await;
        while let Some(result) = llm_stream.next().await {
            if let Some(output) = result {
                print!("{}", output);
            }
        }
        println!();
    }
    async fn on_exit_mut(&mut self) {}
    fn name(&self) -> String {
        "Processing".to_string()
    }
    async fn set_attribute(&mut self, k: &str, v: String) {
        self.attributes.insert(k.into(), v);
    }
    async fn clone_attribute(&self, k: &str) -> Option<String> {
        self.attributes.get(k).cloned()
    }
}

#[async_trait]
impl FSMState for ResponseState {
    async fn on_enter(&self) {
        println!("Entered Response State");
    }
    async fn on_exit(&self) {
        println!("Exited Response State");
    }
    async fn on_enter_mut(&mut self) {}
    async fn on_exit_mut(&mut self) {}
    fn name(&self) -> String {
        "Response".to_string()
    }
    async fn set_attribute(&mut self, _k: &str, _v: String) {}
    async fn clone_attribute(&self, _k: &str) -> Option<String> {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fsm = FiniteStateMachine::new();
    let llm_client = TestLLMClient {};

    fsm.add_state("WaitForInput".to_string(), Box::new(WaitInputState));
    fsm.add_state(
        "Processing".to_string(),
        Box::new(ProcessingState {
            llm_client: Arc::new(llm_client),
            attributes: HashMap::<String, String>::default(),
        }),
    );

    fsm.add_state("Response".to_string(), Box::new(ResponseState));

    fsm.add_transition("WaitForInput".to_string(), "Processing".to_string());
    fsm.add_transition("Processing".to_string(), "Response".to_string());
    fsm.add_transition("Response".to_string(), "WaitForInput".to_string());

    fsm.set_initial_state("WaitForInput".to_string()).await?;

    let llm_client = TestLLMClient {};
    let mut agent = LLMAgent::new(fsm, llm_client);

    println!("Welcome to the LLM Agent CLI. Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?; // Use DefaultEditor instead
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(line.as_str());

                let processing_state = agent.fsm.states.get_mut("Processing").unwrap();

                processing_state
                    .set_attribute("prompt", "test prompt".into())
                    .await;

                processing_state.set_attribute("msg", line).await;

                let _ = agent.fsm.transition("Processing".to_string()).await;

                agent.fsm.transition("Response".to_string()).await;
                agent.fsm.transition("WaitForInput".to_string()).await;

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
