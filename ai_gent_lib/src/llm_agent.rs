use crate::{
    fsm::{TransitionResult, FSM},
    llm_service::LLMStreamOut,
    GenaiLlmclient,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc::{self, Sender};

#[derive(Serialize, Deserialize, Debug)]
pub struct LLMReqSetting {
    pub sys_prompt: String,
    pub summary: String,
    pub context: Option<String>,
    pub messages: Vec<(String, String)>,
    pub temperature: Option<f32>,
    pub model: String,
    pub api_key: String,
    pub fsm_transition_prompt: Option<String>,
    pub summary_prompt: Option<String>,
    pub fsm_initial_state: String,
}

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
    pub next_state: Option<String>,
}

pub struct LLMAgent {
    pub fsm: FSM,
    pub llm_req_settings: LLMReqSetting,
    pub fsm_prompt: String,
    pub summary_prompt: String,
}

#[async_trait]
pub trait LLMClient {
    async fn generate(
        &self,
        prompt: &str,
        msg: &[(String, String)],
        temperature: Option<f32>,
    ) -> Result<String, anyhow::Error>;
    async fn generate_stream(
        &self,
        prompt: &str,
        msg: &[(String, String)],
        temperature: Option<f32>,
    ) -> LLMStreamOut;
}

pub struct AgentSettings {
    pub sys_prompt: String,
    pub fsm_prompt: String,
    pub summary_prompt: String,
    pub fsm_initial_state: String,
    pub model: String,
    pub api_key: String,
}

impl LLMAgent {
    pub fn new(fsm: FSM, agent_settings: AgentSettings) -> Self {
        let llm_req_setting = LLMReqSetting {
            summary: String::default(),
            sys_prompt: agent_settings.sys_prompt,
            messages: Vec::default(),
            temperature: None,
            context: None,
            model: agent_settings.model,
            api_key: agent_settings.api_key,
            fsm_transition_prompt: Some(agent_settings.fsm_prompt.clone()),
            summary_prompt: Some(agent_settings.fsm_prompt.clone()),
            fsm_initial_state: agent_settings.fsm_initial_state,
        };
        // Initialize prompts for each state here
        Self {
            fsm,
            fsm_prompt: agent_settings.fsm_prompt,
            summary_prompt: agent_settings.summary_prompt,
            llm_req_settings: llm_req_setting,
        }
    }

    pub fn set_context(&mut self, context: &str) {
        self.llm_req_settings.context = Some(context.to_string());
    }

    pub async fn set_current_state(
        &mut self,
        state: Option<String>,
        exec_state_actions: bool,
    ) -> Result<(), String> {
        if let Some(state) = state {
            self.fsm.set_initial_state(state, exec_state_actions).await
        } else {
            Err("fms set_current_state fail".into())
        }
    }

    pub async fn get_current_state(&self) -> Option<String> {
        self.fsm.get_current_state_name()
    }

    pub async fn process_message(
        &mut self,
        user_input: &str,
        tx: Option<Sender<(String, String)>>,
        temperature: Option<f32>,
    ) -> Result<String, anyhow::Error> {
        self.llm_req_settings.temperature = temperature;
        let mut last_message = Vec::<(String, String)>::new();

        self.llm_req_settings
            .messages
            .push(("user".into(), user_input.into()));
        last_message.push(("user".into(), user_input.into()));

        // Handle FSM state transition
        let current_state_name = self
            .fsm
            .get_current_state_name()
            .ok_or(anyhow::anyhow!("No current state"))?;

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
            current_state_name, available_transitions, self.llm_req_settings.summary
        );

        let fsm_prompt = [self.fsm_prompt.as_str(), msg.as_str()].join("\n");
        let llm_client = GenaiLlmclient {
            model: self.llm_req_settings.model.clone(),
            api_key: self.llm_req_settings.api_key.clone(),
        };
        let next_state = llm_client
            .generate(
                &fsm_prompt,
                &self.llm_req_settings.messages,
                self.llm_req_settings.temperature,
            )
            .await?;

        let next_fsm_step_response: LLMResponse = serde_json::from_str(&next_state)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM output: {e}, {}", next_state))?;

        if let Some(next_state) = &next_fsm_step_response.next_state {
            self.transition_state(next_state).await?;
        }

        let new_state_name = self
            .fsm
            .get_current_state_name()
            .ok_or(anyhow::anyhow!("No current state"))?;

        let next_states = self
            .fsm
            .available_transitions()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let (llm_output, next_state_name) = {
            let new_state = self.fsm.states.get_mut(&new_state_name).unwrap();

            let llm_req_setting = serde_json::to_string(&self.llm_req_settings).unwrap();
            new_state
                .set_attribute("llm_req_setting", llm_req_setting)
                .await;

            let next_state = if let Some(tx) = tx.clone() {
                // call LLM through the next_state.serve()
                new_state.start_service(tx, None, Some(next_states)).await
            } else {
                None
            };
            let llm_output = new_state.get_attribute("llm_output").await.unwrap();
            (llm_output, next_state)
        };

        let next_state_name = if let Some(next_state_name) = next_state_name {
            self.transition_state(&next_state_name).await?;
            next_state_name
        } else {
            "NoTransition".into()
        };

        self.llm_req_settings
            .messages
            .push(("assistant".into(), llm_output.clone()));
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
            self.llm_req_settings.summary.as_str(),
            "</summary>",
        ]
        .join("\n");

        self.llm_req_settings.summary = llm_client
            .generate(
                &summary_prompt,
                &last_message,
                self.llm_req_settings.temperature,
            )
            .await?;

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
                        "state transition: {} -> {} -> {}",
                        current_state_name, new_state_name, next_state_name
                    ),
                ))
                .await;
        }
        Ok(llm_output)
    }

    pub async fn fsm_message_service(
        &mut self,
        user_input: &str,
        _tx: Option<Sender<(String, String)>>,
        temperature: Option<f32>,
    ) -> Result<String, anyhow::Error> {
        self.llm_req_settings.temperature = temperature;
        self.llm_req_settings
            .messages
            .push(("user".into(), user_input.into()));

        let current_state_name = self
            .fsm
            .get_current_state_name()
            .ok_or(anyhow::anyhow!("No current state"))?;

        println!("trace: current_state: {}", current_state_name);

        if self.fsm.available_transitions().is_none() {
            let _ = self
                .fsm
                .set_initial_state(self.llm_req_settings.fsm_initial_state.clone(), true)
                .await;
        };

        loop {
            let current_state_name = self
                .fsm
                .get_current_state_name()
                .ok_or(anyhow::anyhow!("No current state"))?;

            println!("trace: current_state: {}", current_state_name);

            let next_states = if let Some(next_state) = self.fsm.available_transitions() {
                next_state.iter().cloned().collect::<Vec<_>>()
            } else {
                break;
            };

            println!("trace: available_transitions: {:?}", next_states);

            let current_state = self.fsm.states.get_mut(&current_state_name).unwrap();
            let llm_req_setting = serde_json::to_string(&self.llm_req_settings).unwrap();
            current_state
                .set_attribute("llm_req_setting", llm_req_setting)
                .await;
            let (fsm_tx, mut fsm_rx) = mpsc::channel::<(String, String)>(16);

            let handle = tokio::spawn(async move {
                while let Some((t, r)) = fsm_rx.recv().await {
                    println!("tag: {}, message:{}", t, r);
                }
            });

            if let Some(next_state) = current_state
                .start_service(fsm_tx, None, Some(next_states))
                .await
            {
                let _ = tokio::join!(handle);
                let _ = self.transition_state(&next_state).await;
            } else {
                let _ = tokio::join!(handle);
                break;
            }
        }

        let llm_output = "".into();
        Ok(llm_output)
    }

    pub async fn transition_state(&mut self, next_state: &str) -> Result<(), anyhow::Error> {
        match self.fsm.transition(next_state.into()).await {
            (TransitionResult::Success, _) => {
                // tracing::info!("Transitioned to state: {}", next_state);
                Ok(())
            }
            (TransitionResult::InvalidTransition, _) => Err(anyhow::anyhow!(
                "Invalid transition to state:{:?} -> {}",
                self.fsm.get_current_state_name(),
                next_state
            )),
            (TransitionResult::NoTransitionAvailable, _) => Err(anyhow::anyhow!(
                "No transition available to state: {}",
                next_state
            )),
            (TransitionResult::NoCurrentState, _) => Err(anyhow::anyhow!("No current state")),
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

        let fsm_config_str = include_str!("../../ai_gent_tools/dev_config/fsm_config.json");

        let fsm_config = FSMAgentConfigBuilder::from_json(fsm_config_str)
            .unwrap()
            .build()
            .unwrap();

        let agent_settings = AgentSettings {
            sys_prompt: fsm_config.sys_prompt,
            fsm_prompt: fsm_config.fsm_prompt,
            summary_prompt: fsm_config.summary_prompt,
            model: "".into(),
            api_key: "".into(),
            fsm_initial_state: "Initial".into(),
        };

        let mut agent = LLMAgent::new(fsm, agent_settings);

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

        assert_eq!(fsm.get_current_state_name(), Some("Initial".to_string()));

        let (result, _) = fsm.transition("Processing".into()).await;
        assert_eq!(result, TransitionResult::Success);
        assert_eq!(fsm.get_current_state_name(), Some("Processing".to_string()));

        let (result, _) = fsm.transition("NonExistentState".into()).await;
        assert_eq!(result, TransitionResult::NoTransitionAvailable);
    }
}
