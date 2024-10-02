use crate::{
    fsm::{TransitionResult, FSM},
    llm_service::LLMStreamOut,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct FSMAgentConfig {
    pub states: Vec<String>,
    pub transitions: Vec<(String, String)>,
    pub initial_state: String,
    pub prompts: HashMap<String, String>,
    pub sys_prompt: String,
}

impl FSMAgentConfig {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Default)]
pub struct FSMAgentConfigBuilder {
    states: Vec<String>,
    transitions: Vec<(String, String)>,
    initial_state: Option<String>,
    prompts: HashMap<String, String>,
    sys_prompt: String,
}

impl FSMAgentConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_state(mut self, state: String) -> Self {
        self.states.push(state);
        self
    }

    pub fn add_transition(mut self, from: String, to: String) -> Self {
        self.transitions.push((from, to));
        self
    }

    pub fn set_initial_state(mut self, state: String) -> Self {
        self.initial_state = Some(state);
        self
    }

    pub fn add_prompt(mut self, state: String, prompt: String) -> Self {
        self.prompts.insert(state, prompt);
        self
    }

    pub fn set_sys_prompt(mut self, prompt: String) -> Self {
        self.sys_prompt = prompt;
        self
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let config: FSMAgentConfig = serde_json::from_str(json_str)?;
        Ok(Self {
            states: config.states,
            transitions: config.transitions,
            initial_state: Some(config.initial_state),
            prompts: config.prompts,
            sys_prompt: config.sys_prompt,
        })
    }

    pub fn build(self) -> Result<FSMAgentConfig, &'static str> {
        if self.states.is_empty() {
            return Err("At least one state is required");
        }
        if self.initial_state.is_none() {
            return Err("Initial state must be set");
        }

        Ok(FSMAgentConfig {
            states: self.states,
            transitions: self.transitions,
            initial_state: self.initial_state.unwrap(),
            prompts: self.prompts,
            sys_prompt: self.sys_prompt,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LLMResponse {
    #[serde(default)]
    message: String,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    tool_input: Option<String>,
    next_state: Option<String>,
}

pub struct LLMAgent<C: LLMClient> {
    pub fsm: FSM,
    pub llm_client: C,
    pub sys_prompt: String,
    pub fsm_prompt: String,
    pub summary_prompt: String,
    pub summary: String,
    pub messages: Vec<(String, String)>, // (role, message)
}

#[async_trait]
pub trait LLMClient {
    async fn generate(&self, prompt: &str, msg: &[(String, String)]) -> String;

    async fn generate_stream(&self, prompt: &str, msg: &str) -> LLMStreamOut;
}



impl<C: LLMClient> LLMAgent<C> {
    pub fn new(
        fsm: FSM,
        llm_client: C,
        sys_prompt: &str,
        fsm_prompt: &str,
        summary_prompt: &str,
    ) -> Self {
        // Initialize prompts for each state here
        Self {
            fsm,
            llm_client,
            summary: String::default(),
            sys_prompt: sys_prompt.into(),
            fsm_prompt: fsm_prompt.into() ,
            summary_prompt: summary_prompt.into(),
            messages: Vec::default(),
        }
    }

    pub async fn process_input(&mut self, user_input: &str) -> Result<String, String> {
        let mut last_message = Vec::<(String, String)>::new();

        let current_state_name = self.fsm.current_state().ok_or("No current state")?;
        let current_state = self.fsm.states.get(&current_state_name).unwrap();
        // println!("current_state: {:?} {:?}", current_state,  self.prompts.get(&current_state));

        self.messages.push(("user".into(), user_input.into()));
        last_message.push(("user".into(), user_input.into()));
        if let Some(prompt) = current_state.get_attribute("prompt").await {
            let msg = format!(
                "Current State: {}\nAvailable Transitions: {:?}",
                current_state_name,
                self.fsm.available_transitions(),
            );

            let fsm_prompt = [self.fsm_prompt.as_str(), msg.as_str()].join("\n");

            let prompt = [
                self.sys_prompt.as_str(),
                prompt.as_str(),
                "\nHere is the current summary:\n",
                &self.summary,
            ]
            .join("\n");

            let summary_prompt = [self.summary_prompt.as_str(), self.summary.as_str()].join("\n");

            tracing::info!("summary prompt: {}\n\n", summary_prompt);

            let llm_output = self.llm_client.generate(&prompt, &self.messages).await;

            self.messages.push(("assistant".into(), llm_output.clone()));
            last_message.push(("assistant".into(), llm_output.clone()));

            tracing::info!("raw output: {}\n", llm_output);

            let next_state = self.llm_client.generate(&fsm_prompt, &self.messages).await;

            self.summary = self
                .llm_client
                .generate(&summary_prompt, &last_message)
                .await;

            tracing::info!("summary: {}", self.summary);
            tracing::info!("next_state raw: {}", next_state);

            let mut response: LLMResponse = serde_json::from_str(&next_state)
                .map_err(|e| format!("Failed to parse LLM output: {e}, {}", next_state))?;

            response.message = llm_output;

            tracing::info!(
                "resp: {:?} /n NEXT STATE:{:?}\n",
                response,
                response.next_state
            );

            // placeholder for tool usage

            // Transition state if specified
            if let Some(next_state) = &response.next_state {
                self.transition_state(next_state).await?;
            }

            Ok(response.message)
        } else {
            Ok("".to_string())
        }
    }

    pub async fn transition_state(&mut self, next_state: &str) -> Result<(), String> {
        match self.fsm.transition(next_state.into()).await {
            (TransitionResult::Success, _) => {
                tracing::info!("Transitioned to state: {}", next_state);
                Ok(())
            }
            (TransitionResult::InvalidTransition, _) => Err(format!(
                "Invalid transition to state:{:?} -> {}",
                self.fsm.current_state(),
                next_state
            )),
            (TransitionResult::NoTransitionAvailable, _) => {
                Err(format!("No transition available to state: {}", next_state))
            }
            (TransitionResult::NoCurrentState, _) => Err("No current state".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fsm::{FSMBuilder, FSMState};

    use super::*;

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

        async fn set_attribute(&mut self, _k: &str, _v: String) {}
        async fn clone_attribute(&self, _k: &str) -> Option<String> {
            None
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

        async fn set_attribute(&mut self, _k: &str, _v: String) {}
        async fn clone_attribute(&self, _k: &str) -> Option<String> {
            None
        }
    }

    // Mock LLM Client for testing

    struct MockLLMClient;

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn generate(&self, _prompt: &str, _msg: &[(String, String)]) -> String {
            r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
                .to_string()
        }
        async fn generate_stream(&self, _prompt: &str, _msg: &str) -> LLMStreamOut {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_llm_agent_process_input() {
        let fsm_config = FSMAgentConfigBuilder::new()
            .add_state("Initial".to_string())
            .add_state("Processing".to_string())
            .add_transition("Initial".to_string(), "Processing".to_string())
            .set_initial_state("Initial".to_string())
            .add_prompt(
                "Initial".to_string(),
                "This is the initial state prompt.".to_string(),
            )
            .add_prompt(
                "Processing".to_string(),
                "This is the processing state prompt.".to_string(),
            )
            .set_sys_prompt("".into())
            .build()
            .unwrap();

        let fsm = FSMBuilder::from_config(&fsm_config)
            .unwrap()
            .build()
            .unwrap();
        let llm_client = MockLLMClient;

        let mut agent = LLMAgent::new(fsm, llm_client, "", "", "");

        let result = agent.process_input("Test input").await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
        );
    }

    #[tokio::test]
    async fn test_fsm_transitions() {
        let mut fsm_builder = FSMBuilder::new();
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
