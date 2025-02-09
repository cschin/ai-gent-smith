use crate::{
    fsm::{TransitionResult, FSM},
    llm_service::LLMStreamOut,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use futures::StreamExt;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct FSMAgentConfig {
    pub states: Vec<String>,
    pub transitions: Vec<(String, String)>,
    pub initial_state: String,
    pub prompts: HashMap<String, String>,
    pub sys_prompt: String,
    pub summary_prompt: String,
    pub fsm_prompt: String,
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
    fsm_prompt: String,
    summary_prompt: String,
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

    pub fn set_fsm_prompt(mut self, prompt: String) -> Self {
        self.fsm_prompt = prompt;
        self
    }

    pub fn set_summary_prompt(mut self, prompt: String) -> Self {
        self.summary_prompt = prompt;
        self
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let config: FSMAgentConfig = serde_json::from_str(json_str)?;
        Ok(Self {
            states: config.states,
            transitions: config.transitions,
            initial_state: Some(config.initial_state),
            prompts: config.prompts,
            fsm_prompt: config.fsm_prompt,
            sys_prompt: config.sys_prompt,
            summary_prompt: config.summary_prompt,
        })
    }

    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        let config: FSMAgentConfig = toml::from_str(toml_str)?;
        Ok(Self {
            states: config.states,
            transitions: config.transitions,
            initial_state: Some(config.initial_state),
            prompts: config.prompts,
            fsm_prompt: config.fsm_prompt,
            sys_prompt: config.sys_prompt,
            summary_prompt: config.summary_prompt,
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
            fsm_prompt: self.fsm_prompt,
            sys_prompt: self.sys_prompt,
            summary_prompt: self.summary_prompt,
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
    pub context: Option<String>,
    pub temperature: Option<f32>,
    pub messages: Vec<(String, String)>, // (role, message)
}

#[async_trait]
pub trait LLMClient {
    async fn generate(
        &self,
        prompt: &str,
        msg: &[(String, String)],
        temperature: Option<f32>,
    ) -> String;
    async fn generate_stream(
        &self,
        prompt: &str,
        msg: &[(String, String)],
        temperature: Option<f32>,
    ) -> LLMStreamOut;
}

impl<C: LLMClient> LLMAgent<C> {
    pub fn new(llm_client: C, fsm: FSM, fsm_config: &FSMAgentConfig) -> Self {
        // Initialize prompts for each state here
        Self {
            fsm,
            llm_client,
            summary: String::default(),
            sys_prompt: fsm_config.sys_prompt.clone(),
            fsm_prompt: fsm_config.fsm_prompt.clone(),
            summary_prompt: fsm_config.summary_prompt.clone(),
            messages: Vec::default(),
            temperature: None,
            context: None,
        }
    }

    pub fn set_context(&mut self, context: &str) {
        self.context = Some(context.to_string());
    }

    pub async fn set_current_state(&mut self, state: Option<String>, exec_state_actions: bool) -> Result<(), String> {
        if let Some(state) = state {
            self.fsm
                .set_initial_state(state, exec_state_actions)
                .await
        } else {
            Err("fms set_current_state fail".into())
        }
    }

    pub async fn get_current_state(&self) -> Option<String> {
        self.fsm.current_state()
    }

    pub async fn process_message(
        &mut self,
        user_input: &str,
        tx: Option<Sender<(String, String)>>,
        temperature: Option<f32>,
    ) -> Result<String, String> {
        self.temperature = temperature;
        let mut last_message = Vec::<(String, String)>::new();

        self.messages.push(("user".into(), user_input.into()));
        last_message.push(("user".into(), user_input.into()));

        // Handle FSM state transition
        let current_state_name = self.fsm.current_state().ok_or("No current state")?;
        if let Some(tx) = tx.clone() {
            let _ = tx.send(("clear".into(), "".into())).await;
            let _ = tx
                .send((
                    "message".into(),
                    "determining the agent's next state".into(),
                ))
                .await;
        };

        let available_transitions = self
            .fsm
            .available_transitions()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join("/");

        let msg = format!(
            "Current State: {}\nAvailable Next Steps: {}\n Summary of the previous chat:<summary>{}</summary> \n\n ",
            current_state_name, available_transitions, self.summary
        );

        let fsm_prompt = [self.fsm_prompt.as_str(), msg.as_str()].join("\n");

        let next_state = self
            .llm_client
            .generate(&fsm_prompt, &self.messages, self.temperature)
            .await;

        let next_fsm_step_response: LLMResponse = serde_json::from_str(&next_state)
            .map_err(|e| format!("Failed to parse LLM output: {e}, {}", next_state))?;

        if let Some(next_state) = &next_fsm_step_response.next_state {
            self.transition_state(next_state).await?;
        }

        let next_state_name = self.fsm.current_state().ok_or("No current state")?;
        let next_state = self.fsm.states.get(&next_state_name).unwrap();

        // Process the last message for chat
        if let Some(prompt) = next_state.get_attribute("prompt").await {
            let prompt = if let Some(context) = self.context.as_ref() {
                [
                    self.sys_prompt.as_str(),
                    prompt.as_str(),
                    "\nHere is the summary of previous chat:\n",
                    "<SUMMARY>",
                    &self.summary,
                    "</SUMMARY>",
                    "\nHere is the current reference context:\n",
                    "<REFERENCES>",
                    context,
                    "</REFERENCES>",
                ]
                .join("\n")
            } else {
                [
                    self.sys_prompt.as_str(),
                    prompt.as_str(),
                    "\nHere is the summary of previous chat:\n",
                    "<SUMMARY>",
                    &self.summary,
                    "</SUMMARY>",
                ]
                .join("\n")
            };

            // tracing::info!(target: "tron_app", "full prompt: {}", prompt);

            let llm_output = if let Some(tx) = tx.clone() {
                let _ = tx
                    .send((
                        "message".into(),
                        "LLM request sent, waiting for response\n".into(),
                    ))
                    .await;
                let mut llm_output = String::default();

                let mut llm_stream = self
                    .llm_client
                    .generate_stream(&prompt, &self.messages, self.temperature)
                    .await;

                while let Some(result) = llm_stream.next().await {
                    if let Some(output) = result {
                        llm_output.push_str(&output);
                        let _ = tx.send(("token".into(), output)).await;
                    };
                }
                let _ = tx.send(("llm_output".into(), llm_output.clone())).await;
                llm_output
            } else {
                self.llm_client
                    .generate(&prompt, &self.messages, self.temperature)
                    .await
            };

            // Generate Summary
            self.messages.push(("assistant".into(), llm_output.clone()));
            last_message.push(("assistant".into(), llm_output.clone()));

            if let Some(tx) = tx.clone() {
                let _ = tx.send(("clear".into(), "".into())).await;
                let _ = tx
                    .send(("message".into(), "generating chat summary".into()))
                    .await;
            };

            let summary_prompt = [
                self.summary_prompt.as_str(),
                "<summary>",
                self.summary.as_str(),
                "</summary>",
            ]
            .join("\n");

            self.summary = self
                .llm_client
                .generate(&summary_prompt, &last_message, self.temperature)
                .await;

            // Status update
            if let Some(tx) = tx {
                let _ = tx
                    .send((
                        "message".into(),
                        "Summary generation complete. You can send new query now.".into(),
                    ))
                    .await;

                let _ = tx
                    .send((
                        "message".into(),
                        format!(
                            "state transition: {} -> {}",
                            current_state_name, next_state_name
                        ),
                    ))
                    .await;
            }

            Ok(llm_output)
        } else {
            Ok("".to_string())
        }
    }

    pub async fn transition_state(&mut self, next_state: &str) -> Result<(), String> {
        match self.fsm.transition(next_state.into()).await {
            (TransitionResult::Success, _) => {
                // tracing::info!("Transitioned to state: {}", next_state);
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
        async fn generate(
            &self,
            _prompt: &str,
            _msg: &[(String, String)],
            _temperature: Option<f32>,
        ) -> String {
            r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
                .to_string()
        }
        async fn generate_stream(
            &self,
            _prompt: &str,
            _msg: &[(String, String)],
            _temperature: Option<f32>,
        ) -> LLMStreamOut {
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

        let fsm = FSMBuilder::from_config::<crate::fsm::DefaultFSMChatState>(
            &fsm_config,
            HashMap::default(),
        )
        .unwrap()
        .build()
        .unwrap();
        let llm_client = MockLLMClient;

        let fsm_config_str = include_str!("../../ai_gent_tools/dev_config/fsm_config.json");

        let fsm_config = FSMAgentConfigBuilder::from_json(fsm_config_str)
            .unwrap()
            .build()
            .unwrap();

        let mut agent = LLMAgent::new(llm_client, fsm, &fsm_config);

        let result = agent.process_message("Test input", None, None).await;
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
