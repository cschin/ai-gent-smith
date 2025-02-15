use crate::{
    fsm::{FsmState, TransitionResult, FiniteStateMachine},
    llm_service::LLMStreamOut,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::{self, Receiver, Sender};



pub trait LlmFsmStateInit {
    fn new(name: &str, prompts:StatePrompts, config: StateConfig) -> Self;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LlmReqSetting {
    pub summary: String,
    pub context: Option<String>,
    pub messages: Vec<(String, String)>,
    pub temperature: Option<f32>,
    pub model: String,
    pub api_key: String,
    pub fsm_initial_state: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct StatePrompts {
    pub system: Option<String>,
    pub chat: Option<String>,
    pub fsm: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct StateConfig {
    pub extract_code: Option<bool>,
    pub execute_code: Option<bool>,
    pub disable_llm_request: Option<bool>,
    pub update_context: Option<bool>,
    pub append_to_context: Option<bool>,
    pub ignore_llm_output: Option<bool>,
    pub get_msg: Option<bool>,
}


#[derive(Clone, Default)]
pub struct DefaultFsmChatState {
    name: String,
    attributes: HashMap<String, String>,
    _prompts: StatePrompts,
    _config: StateConfig,
}

impl LlmFsmStateInit for DefaultFsmChatState {
    fn new(name: &str, _prompts: StatePrompts, _config: StateConfig) -> Self {
        DefaultFsmChatState {
            name: name.to_string(),
            _prompts,
            _config,
            ..Default::default()
        }
    }
}

#[async_trait]
impl FsmState for DefaultFsmChatState {
    async fn set_attribute(&mut self, k: &str, v: String) {
        self.attributes.insert(k.to_string(), v);
    }

    async fn get_attribute(&self, k: &str) -> Option<String> {
        self.attributes.get(k).cloned()
    }

    async fn clone_attribute(&self, k: &str) -> Option<String> {
        self.attributes.get(k).cloned()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct FsmBuilder {
    states: HashMap<String, Box<dyn FsmState>>,
    transitions: HashMap<String, HashSet<String>>,
    current_state: Option<String>,
}

impl Default for FsmBuilder {
    fn default() -> Self {
        Self::new()
    }
}


impl FsmBuilder {
    pub fn new() -> Self {
        FsmBuilder {
            states: HashMap::new(),
            transitions: HashMap::new(),
            current_state: None,
        }
    }

    pub fn add_state(mut self, name: String, state: Box<dyn FsmState>) -> Self {
        self.states.insert(name, state);
        self
    }

    pub fn add_transition(mut self, from: String, to: String) -> Self {
        self.transitions.entry(from).or_default().insert(to);
        self
    }

    pub fn set_initial_state(mut self, state: String) -> Self {
        self.current_state = Some(state);
        self
    }

    pub fn from_config<S: LlmFsmStateInit + FsmState + 'static>(
        config: &FsmAgentConfig,
        mut state_map: HashMap<String, S>,
    ) -> Result<Self, anyhow::Error> {
        let mut builder = FsmBuilder {
            states: HashMap::new(),
            transitions: HashMap::new(),
            current_state: Some(config.initial_state.clone()),
        };

        // Add states
        for state_name in &config.states {
            let state_prompt = config
                .state_prompts
                .get(state_name)
                .unwrap_or(&StatePrompts{..Default::default()})
                .clone();
            let state_config = config
                .state_config
                .clone()
                .unwrap_or_default()
                .get(state_name)
                .unwrap_or(&StateConfig{..Default::default()})
                .clone();
            let state = state_map
                .remove(state_name)
                .unwrap_or(S::new(state_name, state_prompt, state_config));

            builder.states.insert(state_name.clone(), Box::new(state));
        }

        // Add transitions
        for (from, to) in &config.transitions {
            builder
                .transitions
                .entry(from.clone())
                .or_default()
                .insert(to.clone());
        }

        // Validate initial state
        if !builder.states.contains_key(&config.initial_state) {
            return Err(anyhow::anyhow!("Initial state not found in states"));
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<FiniteStateMachine, anyhow::Error> {
        if self.states.is_empty() {
            return Err(anyhow::anyhow!("FSM must have at least one state".to_string()));
        }

        if self.current_state.is_none() {
            return Err(anyhow::anyhow!("Initial state must be set".to_string()));
        }

        // Validate that all states in transitions exist
        for (from, tos) in &self.transitions {
            if !self.states.contains_key(from) {
                return Err(anyhow::anyhow!("Transition from non-existent state: {}", from));
            }
            for to in tos {
                if !self.states.contains_key(to) {
                    return Err(anyhow::anyhow!("Transition to non-existent state: {}", to));
                }
            }
        }

        Ok(FiniteStateMachine {
            states: self.states,
            transitions: self.transitions,
            current_state: self.current_state,
        })
    }
}


#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct FsmAgentConfig {
    pub states: Vec<String>,
    pub transitions: Vec<(String, String)>,
    pub initial_state: String,
    pub state_prompts: HashMap<String, StatePrompts>,
    pub state_config: Option<HashMap<String, StateConfig>>,
    pub system_prompt: String,
    pub summary_prompt: String,
    pub fsm_prompt: String,
}

impl FsmAgentConfig {
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
pub struct FsmAgentConfigBuilder {
    states: Vec<String>,
    transitions: Vec<(String, String)>,
    initial_state: String,
    state_prompts: HashMap<String, StatePrompts>,
    state_config: Option<HashMap<String, StateConfig>>,
    fsm_prompt: String,
    summary_prompt: String,
    system_prompt: String,
}

impl FsmAgentConfigBuilder {
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
        self.initial_state = state;
        self
    }

    pub fn add_prompt(mut self, state: String, prompts: StatePrompts) -> Self {
        self.state_prompts.insert(state, prompts);
        self
    }

    pub fn set_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
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
        let config: FsmAgentConfig = serde_json::from_str(json_str)?;
        Ok(Self {
            states: config.states,
            transitions: config.transitions,
            initial_state: config.initial_state,
            state_prompts: config.state_prompts,
            state_config: config.state_config,
            fsm_prompt: config.fsm_prompt,
            system_prompt: config.system_prompt,
            summary_prompt: config.summary_prompt,
        })
    }

    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        let config: FsmAgentConfig = toml::from_str(toml_str)?;
        // if the fsm of system prompt is not set for a state, replace it with the global one
        let state_prompts = config
            .state_prompts
            .iter()
            .map(|(state_name, prompt)| {
                let new_prompt = StatePrompts {
                    system: Some(
                        prompt
                            .system
                            .clone()
                            .unwrap_or(config.system_prompt.clone()),
                    ),
                    fsm: Some(prompt.fsm.clone().unwrap_or(config.fsm_prompt.clone())),
                    chat: prompt.clone().chat,
                };
                (state_name.clone(), new_prompt)
            })
            .collect::<HashMap<String, StatePrompts>>();

        Ok(Self {
            states: config.states,
            transitions: config.transitions,
            initial_state: config.initial_state,
            state_prompts,
            state_config: config.state_config,
            fsm_prompt: config.fsm_prompt,
            system_prompt: config.system_prompt,
            summary_prompt: config.summary_prompt,
        })
    }

    pub fn build(self) -> Result<FsmAgentConfig, anyhow::Error> {
        if self.states.is_empty() {
            return Err(anyhow::anyhow!("At least one state is required"));
        }

        Ok(FsmAgentConfig {
            states: self.states,
            transitions: self.transitions,
            initial_state: self.initial_state,
            state_prompts: self.state_prompts,
            state_config: self.state_config,
            fsm_prompt: self.fsm_prompt,
            system_prompt: self.system_prompt,
            summary_prompt: self.summary_prompt,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LlmResponse {
    #[serde(default)]
    message: String,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    tool_input: Option<String>,
    pub next_state: Option<String>,
}

pub struct LlmAgent {
    pub fsm: FiniteStateMachine,
    pub llm_req_settings: LlmReqSetting,
    pub fsm_prompt: String,
    pub summary_prompt: String,
}

#[async_trait]
pub trait LlmClient {
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

impl LlmAgent {
    pub fn new(fsm: FiniteStateMachine, agent_settings: AgentSettings) -> Self {
        let llm_req_setting = LlmReqSetting {
            summary: String::default(),
            messages: Vec::default(),
            temperature: None,
            context: None,
            model: agent_settings.model,
            api_key: agent_settings.api_key,
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

    pub async fn fsm_message_service(
        &mut self,
        user_input: &str,
        tx: Option<Sender<(String, String, String)>>,
        temperature: Option<f32>,
    ) -> Result<String, anyhow::Error> {
        self.llm_req_settings.temperature = temperature;
        self.llm_req_settings
            .messages
            .push(("user".into(), user_input.into()));

        // let current_state_name = self
        //     .fsm
        //     .get_current_state_name()
        //     .ok_or(anyhow::anyhow!("No current state"))?;
        // println!("trace: current_state: {}", current_state_name);

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
                .ok_or(anyhow::anyhow!(" FSM Error: No current state"))?;

            // println!("trace: current_state: {}", current_state_name);

            let next_states = if let Some(next_state) = self.fsm.available_transitions() {
                next_state.iter().cloned().collect::<Vec<_>>()
            } else {
                break;
            };

            // println!("trace: available_transitions: {:?}", next_states);

            let current_state = self.fsm.states.get_mut(&current_state_name).unwrap();

            let llm_req_setting = serde_json::to_string(&self.llm_req_settings).unwrap();
            current_state
                .set_attribute("llm_req_setting", llm_req_setting)
                .await;

            let (fsm_tx, fsm_rx) = mpsc::channel::<(String, String, String)>(16);
            let tx = tx.clone();
            let handle = get_fsm_state_communication_handle(tx, fsm_rx);

            if let Some(next_state_name) = current_state
                .start_service(fsm_tx, None, Some(next_states))
                .await
            {
                let (llm_output, new_context, append_context) = tokio::join!(handle).0.unwrap();
                self.update_message_and_context(llm_output, new_context, append_context);
                let _ = self.transition_state(&next_state_name).await;
                let next_state = self.fsm.states.get(&next_state_name).unwrap();
                if next_state.get_attribute("get_msg").await.is_some() {
                    break;
                }
            } else {
                let (llm_output, new_context, append_context) = tokio::join!(handle).0.unwrap();
                self.update_message_and_context(llm_output, new_context, append_context);
                break;
            }
        }

        let llm_output = "".into();
        Ok(llm_output)
    }

    fn update_message_and_context(
        &mut self,
        llm_output: Option<String>,
        new_context: Option<String>,
        append_context: Option<String>,
    ) {
        self.llm_req_settings
            .messages
            .push(("bot".into(), llm_output.unwrap_or("".into())));

        if let Some(new_context) = new_context {
            self.llm_req_settings.context = Some(new_context);
        } else if let Some(append_context) = Some(append_context) {
            self.llm_req_settings.context = Some(
                [
                    self.llm_req_settings.context.clone().unwrap_or("".into()),
                    append_context.unwrap_or("".into()),
                ]
                .join("\n"),
            );
        };
    }

    pub async fn transition_state(&mut self, next_state: &str) -> Result<(), anyhow::Error> {
        match self.fsm.make_transition_to(next_state.into()).await {
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

fn get_fsm_state_communication_handle(
    tx: Option<Sender<(String, String, String)>>,
    mut fsm_rx: Receiver<(String, String, String)>,
) -> tokio::task::JoinHandle<(Option<String>, Option<String>, Option<String>)> {
    tokio::spawn(async move {
        let mut llm_output = None;
        let mut new_context = None;
        let mut append_context = None;
        if let Some(tx) = tx {
            while let Some((a, t, r)) = fsm_rx.recv().await {
                match t.as_str() {
                    "llm_output" => {
                        llm_output = Some(r.clone());
                    }
                    "code" | "update_context" => {
                        new_context = Some(r.clone());
                    }
                    "append_context" => {
                        append_context = Some(r.clone());
                    }
                    _ => {}
                }
                let _ = tx.send((a, t, r)).await;
            }
        };
        (llm_output, new_context, append_context)
    })
}

#[cfg(test)]
mod tests {

    use crate::fsm::FsmState;

    use super::*;

    // Example state implementations
    pub struct InitialState;
    pub struct ProcessingState;

    #[async_trait]
    impl FsmState for InitialState {
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
    impl FsmState for ProcessingState {
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
    async fn test_agent_creation() {
        let test_prompt = StatePrompts {
            system: None,
            chat: Some("test".into()),
            fsm: Some("".into()),
        };

        let fsm_config = FsmAgentConfigBuilder::new()
            .add_state("Initial".to_string())
            .add_state("Processing".to_string())
            .add_transition("Initial".to_string(), "Processing".to_string())
            .set_initial_state("Initial".to_string())
            .add_prompt("Initial".to_string(), test_prompt.clone())
            .add_prompt("Processing".to_string(), test_prompt)
            .set_system_prompt("".into())
            .build()
            .unwrap();

        let fsm = FsmBuilder::from_config::<DefaultFsmChatState>(
            &fsm_config,
            HashMap::default(),
        )
        .unwrap()
        .build()
        .unwrap();

        let fsm_config_str = include_str!("../dev_config/fsm_config.toml");

        let fsm_config = FsmAgentConfigBuilder::from_toml(fsm_config_str)
            .unwrap()
            .build()
            .unwrap();

        let agent_settings = AgentSettings {
            sys_prompt: fsm_config.system_prompt,
            fsm_prompt: fsm_config.fsm_prompt,
            summary_prompt: fsm_config.summary_prompt,
            model: "".into(),
            api_key: "".into(),
            fsm_initial_state: "Initial".into(),
        };

        let _agent = LlmAgent::new(fsm, agent_settings);
    }

    #[tokio::test]
    async fn test_fsm_transitions() {
        let mut fsm_builder = FsmBuilder::new();
        fsm_builder = fsm_builder.add_state("Initial".to_string(), Box::new(InitialState));
        fsm_builder = fsm_builder.add_state("Processing".to_string(), Box::new(ProcessingState));
        fsm_builder = fsm_builder.add_transition("Initial".to_string(), "Processing".to_string());
        fsm_builder = fsm_builder.set_initial_state("Initial".to_string());
        let mut fsm = fsm_builder.build().unwrap();

        assert_eq!(fsm.get_current_state_name(), Some("Initial".to_string()));

        let (result, _) = fsm.make_transition_to("Processing".into()).await;
        assert_eq!(result, TransitionResult::Success);
        assert_eq!(fsm.get_current_state_name(), Some("Processing".to_string()));

        let (result, _) = fsm.make_transition_to("NonExistentState".into()).await;
        assert_eq!(result, TransitionResult::NoTransitionAvailable);
    }


    #[derive(Debug, Default)]
    struct TestState {
        name: String,
        attributes: HashMap<String, String>,
    }

    #[async_trait]
    impl FsmState for TestState {
        async fn on_enter(&self) {
            println!("Entering state: {}", self.name);
        }

        async fn on_exit(&self) {
            println!("Exiting state: {}", self.name);
        }

        async fn on_enter_mut(&mut self) {
            println!("Entering state (mut): {}", self.name);
        }

        async fn on_exit_mut(&mut self) {
            println!("Exiting state (mut): {}", self.name);
        }

        async fn set_attribute(&mut self, k: &str, v: String) {
            self.attributes.insert(k.into(), v);
        }

        async fn clone_attribute(&self, k: &str) -> Option<String> {
            self.attributes.get(k).cloned()
        }

        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[tokio::test]
    
    async fn test_finite_state_machine_builder() {
        let fsm = FsmBuilder::new()
            .add_state(
                "State1".to_string(),
                Box::new(TestState {
                    name: "State1".to_string(),
                    attributes: HashMap::default(),
                }),
            )
            .add_state(
                "State2".to_string(),
                Box::new(TestState {
                    name: "State2".to_string(),
                    attributes: HashMap::default(),
                }),
            )
            .add_state(
                "State3".to_string(),
                Box::new(TestState {
                    name: "State3".to_string(),
                    attributes: HashMap::default(),
                }),
            )
            .add_transition("State1".to_string(), "State2".to_string())
            .add_transition("State2".to_string(), "State3".to_string())
            .add_transition("State3".to_string(), "State1".to_string())
            .set_initial_state("State1".to_string())
            .build();

        assert!(fsm.is_ok(), "FSM builder should succeed");
        let mut fsm = fsm.unwrap();

        assert_eq!(fsm.current_state, Some("State1".to_string()));

        // Test valid transition
        let (result, new_state) = fsm.make_transition_to("State2".to_string()).await;
        assert_eq!(result, TransitionResult::Success);
        assert_eq!(new_state, Some("State2".to_string()));

        // Test invalid transition
        let (result, new_state) = fsm.make_transition_to("State1".to_string()).await;
        assert_eq!(result, TransitionResult::InvalidTransition);
        assert_eq!(new_state, Some("State2".to_string()));

        // Test valid transition
        let (result, new_state) = fsm.make_transition_to("State3".to_string()).await;
        assert_eq!(result, TransitionResult::Success);
        assert_eq!(new_state, Some("State3".to_string()));
    }
}
