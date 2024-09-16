use crate::fsm::{FSMState, FiniteStateMachine, FiniteStateMachineBuilder, TransitionResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;



#[derive(Deserialize, Serialize)]
pub struct LLMResponse {
    message: String,
    tool: Option<String>,
    tool_input: Option<String>,
    next_state: Option<String>,
}

pub struct ToolManager {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolManager {
    fn new() -> Self {
        let mut tools = HashMap::new();
        // Register tools here
        // tools.insert("search".to_string(), Box::new(SearchTool));
        // tools.insert("calculator".to_string(), Box::new(CalculatorTool));
        ToolManager { tools }
    }

    async fn use_tool(&self, tool_name: &str, input: &str) -> Result<String, String> {
        if let Some(tool) = self.tools.get(tool_name) {
            tool.run(input).await
        } else {
            Err(format!("Tool not found: {}", tool_name))
        }
    }
}

#[async_trait]
trait Tool: Send + Sync {
    async fn run(&self, input: &str) -> Result<String, String>;
}

pub struct LLMAgent<C: LLMClient> {
    pub fsm: FiniteStateMachine,
    pub llm_client: C,
    pub tool_manager: ToolManager,
    pub prompts: HashMap<String, String>,
}

#[async_trait]
pub trait LLMClient {
    async fn generate(&self, prompt: &str) -> String;
}

impl<C: LLMClient> LLMAgent<C> {
    pub fn new(fsm: FiniteStateMachine, llm_client: C) -> Self {
        // Initialize prompts for each state here
        let tool_manager = ToolManager::new();
        Self {
            fsm,
            llm_client,
            tool_manager,
            prompts: HashMap::default(),
        }
    }

    pub async fn process_input(&mut self, user_input: &str) -> Result<String, String> {
        let current_state = self.fsm.current_state().ok_or("No current state")?;
        let prompt = self
            .prompts
            .get(&current_state)
            .ok_or("No prompt for current state")?;

        let full_prompt = format!(
            "{}\nCurrent State: {}\nAvailable Transitions: {:?}\nUser Input: {}",
            prompt,
            current_state,
            self.fsm.available_transitions(),
            user_input
        );

        let llm_output = self.llm_client.generate(&full_prompt).await;
        let response: LLMResponse = serde_json::from_str(&llm_output)
            .map_err(|e| format!("Failed to parse LLM output: {}", e))?;

        let mut final_response = response.message.clone();

        // Use tool if specified
        if let (Some(tool), Some(tool_input)) = (response.tool, response.tool_input) {
            match self.tool_manager.use_tool(&tool, &tool_input).await {
                Ok(tool_output) => {
                    final_response.push_str(&format!("\nTool '{}' output: {}", tool, tool_output));
                }
                Err(e) => {
                    final_response.push_str(&format!("\nError using tool '{}': {}", tool, e));
                }
            }
        }

        // Transition state if specified
        if let Some(next_state) = &response.next_state {
            self.transition_state(next_state).await?;
        }

        Ok(final_response)
    }

    pub async fn transition_state(&mut self, next_state: &str) -> Result<(), String> {
        match self.fsm.transition(next_state.into()).await {
            (TransitionResult::Success, _) => {
                println!("Transitioned to state: {}", next_state);
                Ok(())
            }
            (TransitionResult::InvalidTransition, _) => {
                Err(format!("Invalid transition to state: {}", next_state))
            }
            (TransitionResult::NoTransitionAvailable, _) => {
                Err(format!("No transition available to state: {}", next_state))
            }
            (TransitionResult::NoCurrentState, _) => Err("No current state".to_string()),
        }
    }
}

// Example state implementations
pub struct InitialState;
pub struct ProcessingState;

#[async_trait]
impl FSMState for InitialState {
    async fn on_enter(&self) {
        println!("Entering Initial State");
    }

    async fn on_exit(&self) {
        println!("Exiting Initial State");
    }

    async fn on_enter_mut(&mut self) {
        println!("Entering Initial State (mut)");
    }

    async fn on_exit_mut(&mut self) {
        println!("Exiting Initial State (mut)");
    }

    fn name(&self) -> String {
        "InitialState".to_string()
    }
}

#[async_trait]
impl FSMState for ProcessingState {
    async fn on_enter(&self) {
        println!("Entering Processing State");
    }

    async fn on_exit(&self) {
        println!("Exiting Processing State");
    }

    async fn on_enter_mut(&mut self) {
        println!("Entering Processing State (mut)");
    }

    async fn on_exit_mut(&mut self) {
        println!("Exiting Processing State (mut)");
    }

    fn name(&self) -> String {
        "ProcessingState".to_string()
    }
}

// Example tool implementation
struct SearchTool;

#[async_trait]
impl Tool for SearchTool {
    async fn run(&self, input: &str) -> Result<String, String> {
        // Implement search logic here
        Ok(format!("Search results for: {}", input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock LLM Client for testing

    struct MockLLMClient;

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn generate(&self, _prompt: &str) -> String {
            r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
                .to_string()
        }
    }

    // Mock Tool for testing
    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        async fn run(&self, input: &str) -> Result<String, String> {
            Ok(format!("Mock tool output for: {}", input))
        }
    }

    #[tokio::test]
    async fn test_llm_agent_process_input() {
        let mut fsm_builder = FiniteStateMachineBuilder::new();
        fsm_builder = fsm_builder.add_state("Initial".to_string(), Box::new(InitialState));
        fsm_builder = fsm_builder.add_state("Processing".to_string(), Box::new(ProcessingState));
        fsm_builder = fsm_builder.add_transition("Initial".to_string(), "Processing".to_string());
        fsm_builder = fsm_builder.set_initial_state("Initial".to_string());

        let fsm = fsm_builder.build().unwrap();

        let llm_client = MockLLMClient;

        let mut agent = LLMAgent::new(fsm, llm_client);

        // Add a mock prompt
        agent.prompts.insert(
            "Initial".to_string(),
            "This is the initial state prompt.".to_string(),
        );

        agent.prompts.insert(
            "Processing".to_string(),
            "This is the processing state prompt.".to_string(),
        );

        // Add a mock tool
        agent
            .tool_manager
            .tools
            .insert("mock_tool".to_string(), Box::new(MockTool));

        let result = agent.process_input("Test input").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test response");
    }

    #[tokio::test]
    async fn test_tool_manager() {
        let mut tool_manager = ToolManager::new();
        tool_manager
            .tools
            .insert("mock_tool".to_string(), Box::new(MockTool));

        let result = tool_manager.use_tool("mock_tool", "test input").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock tool output for: test input");

        let error_result = tool_manager
            .use_tool("non_existent_tool", "test input")
            .await;
        assert!(error_result.is_err());
        assert_eq!(
            error_result.unwrap_err(),
            "Tool not found: non_existent_tool"
        );
    }

    #[tokio::test]
    async fn test_fsm_transitions() {
        let mut fsm_builder = FiniteStateMachineBuilder::new();
        fsm_builder = fsm_builder.add_state("Initial".to_string(), Box::new(InitialState));
        fsm_builder = fsm_builder.add_state("Processing".to_string(), Box::new(ProcessingState));
        fsm_builder = fsm_builder.add_transition("Initial".to_string(), "Processing".to_string());
        fsm_builder = fsm_builder.set_initial_state("Initial".to_string());
        let mut fsm = fsm_builder.build().unwrap();

        assert_eq!(fsm.current_state(), Some("Initial".to_string()));

        let (result, _) = fsm.transition("Processing".into()).await;
        assert_eq!(result, TransitionResult::Success);
        assert_eq!(fsm.current_state(), Some("Processing".to_string()));

        let (result, _) = fsm.transition("NonExistentState".into()).await;
        assert_eq!(result, TransitionResult::NoTransitionAvailable);
    }
}
