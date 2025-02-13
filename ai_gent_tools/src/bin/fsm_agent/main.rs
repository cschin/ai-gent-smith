use ai_gent_lib::GenaiLlmclient;
use async_trait::async_trait;
use futures::StreamExt;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use ai_gent_lib::fsm::{FSMBuilder, FSMState, FSMStateInit};
use anyhow::Result;
use tokio::sync::mpsc::{self, Receiver, Sender};

use ai_gent_lib::llm_agent::{
    self, AgentSettings, FSMAgentConfigBuilder, LLMAgent, LLMClient, LLMResponse, StatePrompts,
};
use tokio::task::JoinHandle;

// use futures::StreamExt;

pub struct FSMChatState {
    name: String,
    attributes: HashMap<String, String>,
    handle: Option<JoinHandle<String>>,
}

impl FSMStateInit for FSMChatState {
    fn new(name: &str, prompts: &StatePrompts) -> Self {
        let mut attributes = HashMap::new();
        if let Some(chat_prompt) = prompts.chat.clone() {
            attributes.insert("prompt.chat".to_string(), chat_prompt);
        }
        if let Some(system_prompt) = prompts.system.clone() {
            attributes.insert("prompt.system".to_string(), system_prompt);
        }
        if let Some(fsm_prompt) = prompts.fsm.clone() {
            attributes.insert("prompt.fsm".to_string(), fsm_prompt);
        }
        FSMChatState {
            name: name.to_string(),
            attributes,
            handle: None,
        }
    }
}

#[async_trait]
impl FSMState for FSMChatState {

    async fn start_service(
        &mut self,
        tx: Sender<(String, String)>,
        _rx: Option<Receiver<(String, String)>>,
        next_states: Option<Vec<String>>,
    ) -> Option<String> {
        let llm_req_settings: llm_agent::LLMReqSetting =
            serde_json::from_str(&self.get_attribute("llm_req_setting").await.unwrap()).unwrap();

        let summary = if !llm_req_settings.summary.is_empty() {
            ["<SUMMARY>", &llm_req_settings.summary, "</SUMMARY>"].join("\n")
        } else {
            "".into()
        };

        let context = if let Some(context) = llm_req_settings.context {
            ["<CONTEXT>", &context, "</CONTEXT>"].join("\n")
        } else {
            "".into()
        };

        let system_prompt = self
            .get_attribute("prompt.system")
            .await
            .unwrap_or(llm_req_settings.system_prompt);

        let chat_prompt = self.get_attribute("prompt.chat").await.unwrap_or("".into());

        if system_prompt.len() + chat_prompt.len() > 0 {
            let full_prompt = [system_prompt, summary, context, chat_prompt].join("\n");
            let llm_client = GenaiLlmclient {
                model: llm_req_settings.model.clone(),
                api_key: llm_req_settings.api_key.clone(),
            };

            let messages = llm_req_settings.messages.clone();
            let temperature = llm_req_settings.temperature;
            self.handle = Some(tokio::spawn(async move {
                let _ = tx
                    .send((
                        "message".into(),
                        "LLM request sent, waiting for response\n".into(),
                    ))
                    .await;
                let mut llm_output = String::default();
                println!("llm_request");
                let mut llm_stream = llm_client
                    .generate_stream(&full_prompt, &messages, temperature)
                    .await;
                while let Some(result) = llm_stream.next().await {
                    if let Some(output) = result {
                        llm_output.push_str(&output);
                        let _ = tx.send(("token".into(), output)).await;
                    };
                }
                let _ = tx.send(("llm_output".into(), llm_output.clone())).await;
                llm_output
            }));

            if let Some(handle) = self.handle.take() {
                let llm_output = tokio::join!(handle);
                let llm_output = llm_output.0.unwrap();
                self.set_attribute("llm_output", llm_output).await;
            } else {
                self.set_attribute("llm_output", "".into()).await;
            };
        };
        {
            // get the the FSM state
            if let Some(next_states) = next_states {
                if next_states.len() == 1 {
                    Some(next_states.first().unwrap().clone())
                } else if let Some(fsm_prompt) = llm_req_settings.fsm_transition_prompt {
                    let available_transitions = next_states.join(", ");
                    let msg = format!(
                        r#"Current State: {}\nAvailable Next State: {}\n Summary of the previous chat:<SUMMARY>{}</SUMMARY> \n\n "#,
                        self.name, available_transitions, llm_req_settings.summary
                    );
                    let fsm_prompt = [fsm_prompt, msg].join("\n");
                    let llm_client = GenaiLlmclient {
                        model: llm_req_settings.model.clone(),
                        api_key: llm_req_settings.api_key.clone(),
                    };
                    let next_state = llm_client
                        .generate(
                            &fsm_prompt,
                            &llm_req_settings.messages,
                            llm_req_settings.temperature,
                        )
                        .await
                        .unwrap();

                    let next_fsm_state_response: LLMResponse = serde_json::from_str(&next_state)
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to parse LLM output: {e}, {}", next_state)
                        })
                        .unwrap();
                    next_fsm_state_response.next_state
                } else {
                    None
                }
            } else {
                None
            }
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

const FSM_CONFIG: &str = include_str!("../../../dev_config/fsm_config.toml");

use std::collections::HashMap;
use std::io::{stdout, Write}; //for flush()

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fsm_config = FSMAgentConfigBuilder::from_toml(FSM_CONFIG)?.build()?;

    let fsm = FSMBuilder::from_config::<FSMChatState>(&fsm_config, HashMap::default())?.build()?;

    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| genai::resolver::Error::ApiKeyEnvNotFound {
            env_name: "OPENAI_API_KEY".to_string(),
        })
        .unwrap();

    let llm_req_setting = AgentSettings {
        sys_prompt: fsm_config.system_prompt,
        fsm_prompt: fsm_config.fsm_prompt,
        summary_prompt: fsm_config.summary_prompt,
        model: "gpt-4o".into(),
        api_key,
        fsm_initial_state: fsm_config.initial_state,
    };
    let mut agent = LLMAgent::new(fsm, llm_req_setting);

    // tracing::info!("agent config: {}", fsm_config.to_json().unwrap());

    //write_agent_config_to_file(&fsm_config);

    println!("Welcome to the LLM Agent CLI. Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?; // Use DefaultEditor instead
    let (tx, mut rx) = mpsc::channel::<(String, String)>(8);

    let t = tokio::spawn(async move {
        while let Some(_message) = rx.recv().await {
            stdout().write(b".").and(stdout().flush()).unwrap();
        }
    });

    loop {
        let readline = rl.readline("\n>> ");
        match readline {
            Ok(line) => {
                if line.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(line.as_str());

                match agent
                    .fsm_message_service(&line, Some(tx.clone()), None)
                    .await
                {
                    Ok(res) => println!("\nResponse: {}", res),
                    Err(err) => println!("LLM error, please retry your question. {:?}", err),
                }

                // if let Some(current_state) = agent.fsm.current_state() {
                //     tracing::info!("Current state: {}", current_state);
                // }
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
    t.abort();
    Ok(())
}
