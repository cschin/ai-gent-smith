use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

use ai_gent_lib::{fsm::FsmState, llm_agent::* , GenaiLlmclient};
use async_trait::async_trait;
use futures::StreamExt;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tron_app::TRON_APP;

#[derive(Default)]
pub struct ChatState {
    name: String,
    prompts: StatePrompts,
    config: StateConfig,
    attributes: HashMap<String, String>,
    handle: Option<JoinHandle<String>>,
}

impl LlmFsmStateInit for ChatState {
    fn new(name: &str, prompts: StatePrompts, config: StateConfig) -> Self {
        ChatState {
            name: name.to_string(),
            prompts,
            config,
            ..Default::default()
        }
    }
}

#[async_trait]
impl FsmState for ChatState {
    async fn on_enter(&self) {
        tracing::info!(target: TRON_APP, "Entering state: {}", self.name);
    }

    async fn on_exit(&self) {
        tracing::info!(target: TRON_APP, "Exiting state: {}", self.name);
    }

    async fn on_enter_mut(&mut self) {
        tracing::info!(target: TRON_APP, "Entering state (mut): {}", self.name);
    }

    async fn on_exit_mut(&mut self) {
        tracing::info!(target: TRON_APP, "Exiting state (mut): {}", self.name);
    }

    async fn start_service(
        &mut self,
        tx: Sender<(String, String, String)>,
        _rx: Option<Receiver<(String, String, String)>>,
        next_states: Option<Vec<String>>,
    ) -> Option<String> {
        let llm_req_setting: LlmReqSetting =
            serde_json::from_str(&self.get_attribute("llm_req_setting").await.unwrap()).unwrap();
        let prompt = self.prompts.chat.clone();
        let system_prompt = self.prompts.system.clone().unwrap_or("".into());
        let summary = llm_req_setting
            .memory
            .get("summary")
            .cloned()
            .unwrap_or_default();
        let summary = summary.last().cloned().unwrap_or_default();
        let summary = serde_json::from_value::<String>(summary).unwrap_or_default();
        let context = llm_req_setting
            .memory
            .get("context")
            .cloned()
            .unwrap_or_default();
        let context = context.last().cloned().unwrap_or_default();
        let context = serde_json::from_value::<String>(context.clone()).ok();

        let full_prompt = match prompt {
            Some(prompt) => match context {
                Some(context) => {
                    [
                        &system_prompt,
                        prompt.as_str(),
                        "\nHere is the summary of previous chat:\n",
                        "<SUMMARY>",
                        &summary,
                        "</SUMMARY>",
                        "\nHere is the current reference context:\n",
                        "<REFERENCES>",
                        &context,
                        "</REFERENCES>",
                    ]
                }
                .join("\n"),
                None => [
                    &system_prompt,
                    prompt.as_str(),
                    "\nHere is the summary of previous chat:\n",
                    "<SUMMARY>",
                    &summary,
                    "</SUMMARY>",
                    "\nHere is the current reference context:\n",
                ]
                .join("\n"),
            },
            None => "".into(),
        };
        if full_prompt.is_empty() {
            let _ = tx
                .send(("".into(), "error".into(), "no state prompt".into()))
                .await;
            return None;
        };
        let llm_client = GenaiLlmclient {
            model: llm_req_setting.model,
            api_key: llm_req_setting.api_key,
        };
        let messages = llm_req_setting.messages;
        let temperature = llm_req_setting.temperature;
        self.handle = Some(tokio::spawn(async move {
            let _ = tx
                .send((
                    "".into(),
                    "message".into(),
                    "LLM request sent, waiting for response\n".into(),
                ))
                .await;
            let mut llm_output = String::default();
            let mut llm_stream = llm_client
                .generate_stream(&full_prompt, &messages, temperature)
                .await;
            while let Some(result) = llm_stream.next().await {
                if let Some(output) = result {
                    llm_output.push_str(&output);
                    let _ = tx.send(("".into(), "token".into(), output)).await;
                };
            }
            let _ = tx
                .send(("".into(), "llm_output".into(), llm_output.clone()))
                .await;
            llm_output
        }));

        if let Some(handle) = self.handle.take() {
            let llm_output = tokio::join!(handle);
            let llm_output = llm_output.0.unwrap();
            self.set_attribute("llm_output", llm_output).await;
        } else {
            self.set_attribute("llm_output", "".into()).await;
        };
        if let Some(next_states) = next_states {
            if next_states.len() == 1 {
                Some(next_states.first().unwrap().clone())
            } else {
                None
            }
        } else {
            None
        }
    }

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

pub struct ChatAgent<LLMAgent> {
    pub base: LLMAgent,
}

impl<LLMAgent> Deref for ChatAgent<LLMAgent> {
    type Target = LLMAgent;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<LLMAgent> DerefMut for ChatAgent<LLMAgent> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ChatAgent<LlmFsmAgent> {
    pub async fn process_message(
        &mut self,
        user_input: &str,
        tx: Option<Sender<(String, String, String)>>,
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
            let _ = tx.send(("".into(), "clear".into(), "".into())).await;
            let _ = tx
                .send((
                    "".into(),
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
        let summary = self
            .llm_req_settings
            .memory
            .get("summary")
            .cloned()
            .unwrap_or_default();
        let summary = summary.last().cloned().unwrap_or_default();
        let msg = format!(
            "Current State: {}\nAvailable Next Steps: {}\n Summary of the previous chat:<summary>{}</summary> \n\n ",
            current_state_name, available_transitions, summary
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

        let next_fsm_step_response: LlmResponse = serde_json::from_str(&next_state)
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

        let llm_req_setting: String = serde_json::to_string(&self.llm_req_settings).unwrap();
        let (llm_output, next_state_name) = {
            let new_state = self.fsm.states.get_mut(&new_state_name).unwrap();
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
            let _ = tx.send(("".into(), "clear".into(), "".into())).await;
            let _ = tx
                .send((
                    "".into(),
                    "message".into(),
                    "generating chat summary".into(),
                ))
                .await;
        };

        let summary_prompt = self.summary_prompt.clone();
        let temperature = self.llm_req_settings.temperature;
        {
            let summary = self
                .llm_req_settings
                .memory
                .entry("summary".into())
                .or_default();
            let summary_value = summary.last().cloned().unwrap_or_default();
            let summary_str =
                &serde_json::from_value::<String>(summary_value.clone()).unwrap_or_default();
            let summary_prompt = [
                summary_prompt.as_str(),
                "<summary>",
                summary_str,
                "</summary>",
            ]
            .join("\n");
            let updated_summary = llm_client
                .generate(&summary_prompt, &last_message, temperature)
                .await?;
            summary.push(serde_json::from_str(&updated_summary).unwrap_or_default());
        }
        if let Some(tx) = tx {
            let _ = tx
                .send((
                    "".into(),
                    "message".into(),
                    "Summary generation complete. You can send new query now.".into(),
                ))
                .await;
            let _ = tx
                .send((
                    "".into(),
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
}
