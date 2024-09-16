pub mod fsm;
pub mod llm_agent;


use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use async_trait::async_trait;
use fsm::{FiniteStateMachine, FSMState};
use anyhow::Result;

// Mock LLM Agent
struct MockLLMAgent;

impl MockLLMAgent {
    fn new() -> Self {
        MockLLMAgent
    }

    async fn process(&self, input: &str) -> String {
        format!("Processed: {}", input)
    }
}

// Define states for the FSM
struct InitialState;
struct ProcessingState;
struct ResponseState;

#[async_trait]
impl FSMState for InitialState {
    async fn on_enter(&self) { println!("Entered Initial State"); }
    async fn on_exit(&self) { println!("Exited Initial State"); }
    async fn on_enter_mut(&mut self) {}
    async fn on_exit_mut(&mut self) {}
    fn name(&self) -> String { "Initial".to_string() }
}

#[async_trait]
impl FSMState for ProcessingState {
    async fn on_enter(&self) { println!("Entered Processing State"); }
    async fn on_exit(&self) { println!("Exited Processing State"); }
    async fn on_enter_mut(&mut self) {}
    async fn on_exit_mut(&mut self) {}
    fn name(&self) -> String { "Processing".to_string() }
}

#[async_trait]
impl FSMState for ResponseState {
    async fn on_enter(&self) { println!("Entered Response State"); }
    async fn on_exit(&self) { println!("Exited Response State"); }
    async fn on_enter_mut(&mut self) {}
    async fn on_exit_mut(&mut self) {}
    fn name(&self) -> String { "Response".to_string() }
}




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fsm = FiniteStateMachine::new();
    fsm.add_state("Initial".to_string(), Box::new(InitialState));
    fsm.add_state("Processing".to_string(), Box::new(ProcessingState));
    fsm.add_state("Response".to_string(), Box::new(ResponseState));

    fsm.add_transition("Initial".to_string(), "Processing".to_string());
    fsm.add_transition("Processing".to_string(), "Response".to_string());
    fsm.add_transition("Response".to_string(), "Initial".to_string());

    fsm.set_initial_state("Initial".to_string()).await?;

    let llm_agent = MockLLMAgent::new();

    println!("Welcome to the LLM Agent CLI. Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?;  // Use DefaultEditor instead
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(line.as_str());

                let _ = fsm.transition("Processing".to_string()).await;
                let response = llm_agent.process(&line).await;
                fsm.transition("Response".to_string()).await;
                println!("Agent: {}", response);
                fsm.transition("Initial".to_string()).await;

                if let Some(current_state) = fsm.current_state() {
                    println!("Current state: {}", current_state);
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
