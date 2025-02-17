use std::collections::HashMap;

use async_trait::async_trait;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tera::Tera;
use tokio::{sync::mpsc::{Receiver, Sender}, task::JoinHandle};

use crate::{fsm::FsmState, llm_agent::{self, *}, GenaiLlmclient};


// use futures::StreamExt;
#[derive(Default)]
pub struct FSMChatState {
    name: String,
    attributes: HashMap<String, String>,
    prompts: StatePrompts,
    config: StateConfig,
    handle: Option<JoinHandle<String>>,
}

impl LlmFsmStateInit for FSMChatState {
    fn new(name: &str, prompts: StatePrompts, config: StateConfig) -> Self {
        let mut attributes = HashMap::<String, String>::default();
        if let Some(wait_for_msg) = config.wait_for_msg {
            if wait_for_msg {
                attributes.insert("wait_for_msg".into(), "true".into());
            }
        }
        FSMChatState {
            name: name.to_string(),
            attributes,
            prompts,
            config,
            ..Default::default()
        }
    }
}

async fn get_llm_req_process_handle(
    state_name: String,
    tx: Sender<(String, String, String)>,
    messages: Vec<(String, String)>,
    full_prompt: String,
    temperature: Option<f32>,
    ignore_llm_output: bool,
    model_api_key: (String, String),
) -> JoinHandle<String> {
    // let messages = llm_req_settings.messages.clone();
    // let temperature = llm_req_settings.temperature;
    // let ignore_llm_output = self.config.ignore_llm_output.unwrap_or(false);
    let llm_client = GenaiLlmclient {
        model: model_api_key.0,
        api_key: model_api_key.1,
    };
    tokio::spawn(async move {
        let _ = tx
            .send((
                state_name.clone(),
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
                if !ignore_llm_output {
                    let _ = tx.send((state_name.clone(), "token".into(), output)).await;
                };
            };
        }
        if !ignore_llm_output {
            let _ = tx
                .send((state_name.clone(), "llm_output".into(), llm_output.clone()))
                .await;
        };
        llm_output
    })
}

fn extract_code(input: &str) -> String {
    let start_tag = "<code>";
    let end_tag = "</code>";

    match (input.find(start_tag), input.rfind(end_tag)) {
        (Some(start), Some(end)) if start < end => {
            let start_index = start + start_tag.len();
            input[start_index..end].trim().to_string()
        }
        _ => String::new(),
    }
}

fn run_code_in_docker(code: &str) -> (String, String) {
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    // Create a temporary file to store the Python code
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", code).unwrap();
    let temp_file_path = temp_file.path().to_str().unwrap();

    // Run the Docker command
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &format!("{}:/tmp/code.py", temp_file_path),
            "python-ext",
            "/tmp/code.py",
        ])
        .output()
        .expect("Failed to execute Docker command");

    // Capture stdin and stdout
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (stdout, stderr)
}

fn escape_json_string(input: &str) -> String {
    input
        .replace('\\', "\\\\") // Escape backslash first!
        .replace('"', "\\\"") // Escape double quotes
        .replace('\n', "\\n") // Escape newlines
        .replace('\r', "\\r") // Escape carriage return
        .replace('\t', "\\t") // Escape tabs
        .replace('\x08', "\\b") // Escape backspace
        .replace('\x0C', "\\f") // Escape form feed
}

#[async_trait]
impl FsmState for FSMChatState {
    async fn start_service(
        &mut self,
        tx: Sender<(String, String, String)>,
        _rx: Option<Receiver<(String, String, String)>>,
        next_states: Option<Vec<String>>,
    ) -> Option<String> {
        let llm_req_settings: llm_agent::LlmReqSetting =
            serde_json::from_str(&self.get_attribute("llm_req_setting").await.unwrap()).unwrap();

        let state_name = self.name.clone();
        let _ = tx
            .send((state_name.clone(), "state".into(), state_name.clone()))
            .await;

        let messages = if self.config.use_only_last_message.unwrap_or(false) {
            vec![llm_req_settings
                .messages
                .last()
                .cloned()
                .unwrap_or_default()]
        } else {
            llm_req_settings.messages.clone()
        };

        let summary = if !self.config.disable_summary.unwrap_or(false) {
            if let Some(summary) = llm_req_settings.memory.get("summary") {
                let summary = summary.last().cloned().unwrap_or_default();
                let summary =
                    serde_json::from_value::<String>(summary.clone()).unwrap_or("".into());
                ["<SUMMARY>", &summary, "</SUMMARY>"].join("\n")
            } else {
                "".into()
            }
        } else {
            "".into()
        };

        let context = if !self.config.disable_context.unwrap_or(false) {
            if let Some(context) = llm_req_settings.memory.get("context") {
                let context = if self.config.use_full_context.unwrap_or(false) {
                    context
                        .iter()
                        .map(|c| serde_json::from_value::<String>(c.clone()).unwrap_or("".into()))
                        .collect::<Vec<String>>()
                        .join("\n\n")
                } else {
                    let context = context.last().cloned().unwrap_or_default();
                    serde_json::from_value::<String>(context.clone()).unwrap_or("".into())
                };
                ["<CONTEXT>", &context, "</CONTEXT>"].join("\n")
            } else {
                "".into()
            }
        } else {
            "".into()
        };

        let llm_output = if !self.config.disable_llm_request.unwrap_or(false) {
            let system_prompt = self.prompts.system.clone().unwrap_or("".into());

            let chat_prompt = self.prompts.chat.as_ref().unwrap_or(&"".into()).clone();

            let llm_output = if system_prompt.len() + chat_prompt.len() > 0 {
                let full_prompt =
                    [system_prompt, summary.clone(), context.clone(), chat_prompt].join("\n");
                let model = llm_req_settings.model.clone();
                let api_key = llm_req_settings.api_key.clone();
                let temperature = llm_req_settings.temperature;
                let ignore_llm_output = self.config.ignore_llm_output.unwrap_or(false);

                self.handle = Some(
                    get_llm_req_process_handle(
                        state_name.clone(),
                        tx.clone(),
                        messages.clone(),
                        full_prompt,
                        temperature,
                        ignore_llm_output,
                        (model, api_key),
                    )
                    .await,
                );

                if let Some(handle) = self.handle.take() {
                    let llm_output = tokio::join!(handle);
                    let llm_output = llm_output.0.unwrap();
                    self.set_attribute("llm_output", llm_output.clone()).await;
                    llm_output
                } else {
                    self.set_attribute("llm_output", "".into()).await;
                    "".into()
                }
            } else {
                String::new()
            };

            if self.config.save_to_summary.unwrap_or(false) {
                let _ = tx
                    .send((self.name.clone(), "summary".into(), llm_output.clone()))
                    .await;
            }

            if self.config.save_to_context.unwrap_or(false) {
                let _ = tx
                    .send((self.name.clone(), "context".into(), llm_output.clone()))
                    .await;
            }

            if self.config.extract_code.unwrap_or(false) {
                let code = extract_code(&llm_output);
                let _ = tx.send((self.name.clone(), "code".into(), code)).await;
            }
            llm_output
        } else {
            "".into()
        };

        #[derive(Deserialize, Debug)]
        struct ExecuteCode {
            run: bool,
        }

        let (stdout, stderr) = if self.config.execute_code.unwrap_or(false) {
            let code = if let Some(code) = self.config.code.clone() {
                let mut tera_context = tera::Context::new();
                let messages = escape_json_string(&json!(&messages).to_string());
                let context = escape_json_string(&json!(&context).to_string());
                let summary = escape_json_string(&json!(&summary).to_string());
                let state_name = escape_json_string(&json!(&state_name).to_string());
                let state_history =
                    escape_json_string(&json!(llm_req_settings.state_history).to_string());
                tera_context.insert("messages", &messages);
                tera_context.insert("context", &context);
                tera_context.insert("summary", &summary);
                tera_context.insert("state_name", &state_name);
                tera_context.insert("state_history", &state_history);
                Tera::one_off(&code, &tera_context, false).unwrap()
            } else {
                let code = llm_req_settings
                    .memory
                    .get("code")
                    .cloned()
                    .unwrap_or_default();
                let code = code.last().cloned().unwrap_or_default();
                serde_json::from_value::<String>(code).unwrap_or("".into())
            };

            if self.config.wait_for_msg.unwrap_or(false) {
                // execute code depending on the LLM's response
                let llm_output = self
                    .get_attribute("llm_output")
                    .await
                    .unwrap_or(String::new());
                let llm_output =
                    serde_json::from_str(&llm_output).unwrap_or(ExecuteCode { run: false });
                if llm_output.run {
                    let _ = tx
                        .send((
                            self.name.clone(),
                            "output".into(),
                            "\nconditionally, run code from the context:\n".into(),
                        ))
                        .await;
                    let (stdout, stderr) = run_code_in_docker(&code);
                    let _ = tx
                        .send((
                            self.name.clone(),
                            "output".into(),
                            format!("stdout:\n {}\n", stdout),
                        ))
                        .await;

                    let _ = tx
                        .send((
                            self.name.clone(),
                            "output".into(),
                            format!("stderr:\n {}\n", stderr),
                        ))
                        .await;
                    (stdout, stderr)
                } else {
                    let _ = tx
                        .send((
                            self.name.clone(),
                            "output".into(),
                            "code execution rejected\n".into(),
                        ))
                        .await;
                    ("".into(), "".into())
                }
            } else {
                // execute code without confirmation
                let (stdout, stderr) = run_code_in_docker(&code);
                let _ = tx
                    .send((
                        self.name.clone(),
                        "output".into(),
                        format!("stdout:\n{}\n", stdout),
                    ))
                    .await;

                let _ = tx
                    .send((
                        self.name.clone(),
                        "output".into(),
                        format!("stderr:\n{}\n", stderr),
                    ))
                    .await;
                (stdout, stderr)
            }
        } else {
            ("".into(), "".into())
        };

        if self.config.execute_code.unwrap_or(false) {
            if self.config.save_to_context.unwrap_or(false) {
                let _ = tx
                    .send((self.name.clone(), "context".into(), stdout.clone()))
                    .await;
            }

            if self.config.save_execution_output.unwrap_or(false) {
                let execution_output = serde_json::to_value(ExecutionOutput { stdout, stderr })
                    .unwrap()
                    .to_string();

                let _ = tx
                    .send((
                        self.name.clone(),
                        "execution_output".into(),
                        execution_output,
                    ))
                    .await;
            }
        }

        if let Some(fsm_code) = self.config.fsm_code.clone() {
            let mut tera_context = tera::Context::new();
            let messages = escape_json_string(&json!(&messages).to_string());
            let context = escape_json_string(&json!(&context).to_string());
            let summary = escape_json_string(&json!(&summary).to_string());
            let state_name = escape_json_string(&json!(&state_name).to_string());
            let next_states = if let Some(next_states) = next_states {
                escape_json_string(&json!(&next_states).to_string())
            } else {
                "[]".into()
            };
            let state_history =
                escape_json_string(&json!(llm_req_settings.state_history).to_string());
            tera_context.insert("messages", &messages);
            tera_context.insert("context", &context);
            tera_context.insert("summary", &summary);
            tera_context.insert("state_name", &state_name);
            tera_context.insert("next_states", &next_states);
            tera_context.insert("state_history", &state_history);
            let code = Tera::one_off(&fsm_code, &tera_context, false).unwrap();
            let (stdout, _stderr) = run_code_in_docker(&code);
            // TODO: check if the stdout is a single string contained in the next_states
            Some(stdout.trim().into())
        } else {
            // get the the FSM state
            if let Some(next_states) = next_states {
                if next_states.len() == 1 {
                    Some(next_states.first().unwrap().clone())
                } else if let Some(fsm_prompt) = self.prompts.fsm.clone() {
                    let available_transitions = next_states.join(", ");
                    let summary = llm_req_settings
                        .memory
                        .get("summary")
                        .cloned()
                        .unwrap_or_default();
                    let summary = summary.last().cloned().unwrap_or_default().to_string();
                    let last_messages = llm_req_settings
                        .messages
                        .last()
                        .cloned()
                        .unwrap_or(("".into(), "".into()));

                    let msg = format!(
                        r#"
Summary of the previous chat: 

{}

Current State: {}

Available Next States: {}

The user input: 

{}

The last response: 

{}"#,
                        summary, self.name, available_transitions, last_messages.1, llm_output
                    );
                    let fsm_prompt = [fsm_prompt, msg].join("\n");
                    let llm_client = GenaiLlmclient {
                        model: llm_req_settings.model.clone(),
                        api_key: llm_req_settings.api_key.clone(),
                    };
                    // println!("for debug: \n<FSM> {} </FSM>\n", fsm_prompt);
                    let next_state = llm_client
                        .generate(
                            &fsm_prompt,
                            &[("user".into(), "determine the next state".into())],
                            llm_req_settings.temperature,
                        )
                        .await
                        .unwrap();

                    let next_fsm_state_response = serde_json::from_str::<LlmResponse>(&next_state);
                    // println!("res: {:?}", next_fsm_state_response);
                    match next_fsm_state_response {
                        Ok(next_fsm_state_response) => next_fsm_state_response.next_state,
                        Err(e) => {
                            eprintln!("fail to parse LLM json output for next fsm state: {:?} \n LLM output: {}", e, next_state);
                            None
                        }
                    }
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

    async fn remove_attribute(&mut self, k: &str) -> Option<String> {
        self.attributes.remove(k)
    }

    async fn clone_attribute(&self, k: &str) -> Option<String> {
        self.attributes.get(k).cloned()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}