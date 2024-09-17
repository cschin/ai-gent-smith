use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionResult {
    Success,
    InvalidTransition,
    NoTransitionAvailable,
    NoCurrentState,
}

#[async_trait]
pub trait FSMState: Send + Sync {
    async fn on_enter(&self) {}
    async fn on_exit(&self) {}
    async fn on_enter_mut(&mut self) {}
    async fn on_exit_mut(&mut self) {}
    async fn set_attribute(&mut self, _k: &str, _v: String) {}
    async fn clone_attribute(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn name(&self) -> String {
        unimplemented!()
    }
}

pub struct FiniteStateMachine {
    pub states: HashMap<String, Box<dyn FSMState>>,
    transitions: HashMap<String, HashSet<String>>,
    current_state: Option<String>,
}

pub struct FiniteStateMachineBuilder {
    states: HashMap<String, Box<dyn FSMState>>,
    transitions: HashMap<String, HashSet<String>>,
    current_state: Option<String>,
}

impl Default for FiniteStateMachineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FiniteStateMachineBuilder {
    pub fn new() -> Self {
        FiniteStateMachineBuilder {
            states: HashMap::new(),
            transitions: HashMap::new(),
            current_state: None,
        }
    }

    pub fn add_state(mut self, name: String, state: Box<dyn FSMState>) -> Self {
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

    pub fn build(self) -> Result<FiniteStateMachine, String> {
        if self.states.is_empty() {
            return Err("FSM must have at least one state".to_string());
        }

        if self.current_state.is_none() {
            return Err("Initial state must be set".to_string());
        }

        // Validate that all states in transitions exist
        for (from, tos) in &self.transitions {
            if !self.states.contains_key(from) {
                return Err(format!("Transition from non-existent state: {}", from));
            }
            for to in tos {
                if !self.states.contains_key(to) {
                    return Err(format!("Transition to non-existent state: {}", to));
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

impl Default for FiniteStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl FiniteStateMachine {
    pub fn new() -> Self {
        FiniteStateMachine {
            states: HashMap::new(),
            transitions: HashMap::new(),
            current_state: None,
        }
    }

    pub fn add_state(&mut self, name: String, state: Box<dyn FSMState>) {
        self.states.insert(name.clone(), state);
        self.transitions.entry(name).or_default();
    }

    pub fn add_transition(&mut self, from: String, to: String) {
        self.transitions.entry(from).or_default().insert(to);
    }

    pub fn available_transitions(&self) -> Option<HashSet<String>> {
        self.current_state
            .as_ref()
            .and_then(|current| self.transitions.get(current).cloned())
    }

    pub fn current_state(&self) -> Option<String> {
        self.current_state.clone()
    }

    pub async fn set_initial_state(&mut self, state: String) -> Result<(), String> {
        if self.states.contains_key(&state) {
            if let Some(current_state) = &self.current_state {
                self.states
                    .get_mut(current_state)
                    .unwrap()
                    .on_exit_mut()
                    .await;
                self.states.get(current_state).unwrap().on_exit().await;
            }
            self.current_state = Some(state.clone());
            self.states.get_mut(&state).unwrap().on_enter_mut().await;
            self.states.get(&state).unwrap().on_enter().await;
            Ok(())
        } else {
            Err("State does not exist".to_string())
        }
    }

    pub async fn transition(&mut self, to: String) -> (TransitionResult, Option<String>) {
        if let Some(current_state) = &self.current_state {
            if let Some(valid_transitions) = self.transitions.get(current_state) {
                if valid_transitions.contains(&to) {
                    self.states
                        .get_mut(current_state)
                        .unwrap()
                        .on_exit_mut()
                        .await;
                    self.states.get(current_state).unwrap().on_exit().await;
                    self.current_state = Some(to.clone());
                    self.states.get_mut(&to).unwrap().on_enter_mut().await;
                    self.states.get(&to).unwrap().on_enter().await;
                    (TransitionResult::Success, Some(to))
                } else {
                    (
                        TransitionResult::InvalidTransition,
                        Some(current_state.clone()),
                    )
                }
            } else {
                (
                    TransitionResult::NoTransitionAvailable,
                    Some(current_state.clone()),
                )
            }
        } else {
            (TransitionResult::NoCurrentState, None)
        }
    }
}

// Test harness
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[derive(Debug, Default)]
    struct TestState {
        name: String,
        attributes: HashMap<String, String>,
    }

    #[async_trait]
    impl FSMState for TestState {
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
    async fn test_on_enter() {
        let state = TestState {
            name: "TestState".to_string(),
            ..Default::default()
        };

        state.on_enter().await;
        // You might want to capture stdout and check the output
    }

    #[tokio::test]
    async fn test_on_exit() {
        let state = TestState {
            name: "TestState".to_string(),
            ..Default::default()
        };

        state.on_exit().await;
        // You might want to capture stdout and check the output
    }

    #[tokio::test]
    async fn test_finite_state_machine() {
        let mut fsm = FiniteStateMachine::new();

        fsm.add_state(
            "State1".to_string(),
            Box::new(TestState {
                name: "State1".to_string(),
                attributes: HashMap::default(),
            }),
        );
        fsm.add_state(
            "State2".to_string(),
            Box::new(TestState {
                name: "State2".to_string(),
                attributes: HashMap::default(),
            }),
        );
        fsm.add_state(
            "State3".to_string(),
            Box::new(TestState {
                name: "State3".to_string(),
                attributes: HashMap::default(),
            }),
        );
        fsm.add_transition("State1".to_string(), "State2".to_string());
        fsm.add_transition("State2".to_string(), "State3".to_string());
        fsm.add_transition("State3".to_string(), "State1".to_string());

        assert!(fsm.set_initial_state("State1".to_string()).await.is_ok());
        assert_eq!(fsm.current_state, Some("State1".to_string()));

        match fsm.transition("State2".to_string()).await {
            (TransitionResult::Success, Some(new_state)) => {
                println!("Transitioned to {}", new_state);
                assert_eq!(new_state, "State2");
            }
            (status, _) => {
                println!("Transition failed: {:?}", status);
                panic!("Transition should have succeeded");
            }
        }

        // Test invalid transition
        match fsm.transition("State1".to_string()).await {
            (TransitionResult::Success, _) => {
                panic!("Transition should have failed");
            }
            (status, Some(new_state)) => {
                println!("Transition failed as expected: {:?}", status);
                assert_eq!(new_state, "State2");
            }
            (status, None) => {
                println!("Transition failed as expected: {:?}", status);
            }
        }
    }
    #[tokio::test]
    async fn test_finite_state_machine_builder() {
        let fsm = FiniteStateMachineBuilder::new()
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
        let (result, new_state) = fsm.transition("State2".to_string()).await;
        assert_eq!(result, TransitionResult::Success);
        assert_eq!(new_state, Some("State2".to_string()));

        // Test invalid transition
        let (result, new_state) = fsm.transition("State1".to_string()).await;
        assert_eq!(result, TransitionResult::InvalidTransition);
        assert_eq!(new_state, Some("State2".to_string()));

        // Test valid transition
        let (result, new_state) = fsm.transition("State3".to_string()).await;
        assert_eq!(result, TransitionResult::Success);
        assert_eq!(new_state, Some("State3".to_string()));
    }
}
