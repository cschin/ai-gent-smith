use std::collections::HashMap;

use async_trait::async_trait;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use tera::Tera;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    fsm::FsmState,
    llm_agent::{self, *},
    GenaiLlmclient,
};

type Messages = Vec<(String, String)>;
#[derive(Default, Debug)]
struct FSMChatStateData {
    messages: Messages,
    summary: String,
    task: String,
    context: String,
    tools: String,
    memory: HashMap<String, String>,
}

#[derive(Default)]
pub struct FSMChatState {
    name: String,
    attributes: HashMap<String, String>,
    prompts: StatePrompts,
    config: StateConfig,
    handle: Option<JoinHandle<String>>,
    state_data: FSMChatStateData,
    llm_req_setting: LlmReqSetting,
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
    fsm_tx: Sender<(String, String, String)>,
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
        let _ = fsm_tx
            .send((
                state_name.clone(),
                "message".into(),
                "LLM request sent, waiting for response\n".into(),
            ))
            .await;
        let mut llm_output = String::default();
        //println!(" --- state: {}; full prompt: {}", state_name, full_prompt);
        let mut llm_stream = llm_client
            .generate_stream(&full_prompt, &messages, temperature)
            .await;
        while let Some(result) = llm_stream.next().await {
            match result {
                Ok(output) => {
                    if let Some(output) = output {
                        llm_output.push_str(&output);
                        if !ignore_llm_output {
                            if !fsm_tx.is_closed() {
                                match fsm_tx
                                    .send((state_name.clone(), "token".into(), output))
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(_) => {
                                        break;
                                    }
                                }
                            } else {
                                break;
                            }
                        };
                    }
                }
                Err(e) => {
                    if !fsm_tx.is_closed() {
                        match fsm_tx
                            .send((
                                state_name.clone(),
                                "error".into(),
                                format!("LLM API call error: {}", e),
                            ))
                            .await
                        {
                            Ok(_) => {}
                            Err(_) => {
                                break;
                            }
                        };
                    } else {
                        break;
                    }
                }
            };
        }
        if !ignore_llm_output && !fsm_tx.is_closed() {
            let _ = fsm_tx
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

fn _format_messages(messages: &[(String, String)]) -> String {
    messages
        .iter()
        .map(|(tag, msg)| match tag.as_str() {
            "user" => format!("user: {}", msg),
            "bot" => format!("assistant: {}", msg),
            other => format!("{}: {}", other, msg),
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[async_trait]
impl FsmState for FSMChatState {
    async fn start_service(
        &mut self,
        fsm_tx: Sender<(String, String, String)>,
        _rx: Option<Receiver<(String, String, String)>>,
        next_states: Option<Vec<String>>,
    ) -> Option<String> {
        let llm_req_setting = self.llm_req_setting.clone();
        let _ = fsm_tx
            .send((self.name.clone(), "state".into(), self.name.clone()))
            .await;

        self.state_data = self.prepare_context(&llm_req_setting).await;

        let llm_output = match self.handle_llm_output(&llm_req_setting, &fsm_tx).await {
            Ok(llm_output) => llm_output,
            Err(e) => {
                let _ = fsm_tx
                    .send((
                        self.name.clone(),
                        "error".into(),
                        format!("error in generate LLM output: {:?}", e),
                    ))
                    .await;
                return None;
            }
        };

        let (stdout, stderr) = match self.execute_code(&llm_req_setting, &fsm_tx).await {
            Ok((stdout, stderr)) => (stdout, stderr),
            Err(e) => {
                let _ = fsm_tx
                    .send((
                        self.name.clone(),
                        "error".into(),
                        format!("error in execute_code {:?}", e),
                    ))
                    .await;
                return None;
            }
        };

        self.save_execution_output(&fsm_tx, &stdout, &stderr).await;

        let next_state = match self
            .determine_next_state(&llm_req_setting, &fsm_tx, &next_states, &llm_output)
            .await
        {
            Ok(next_state) => next_state,
            Err(e) => {
                let _ = fsm_tx
                    .send((
                        self.name.clone(),
                        "error".into(),
                        format!("error in determining the next step {:?}", e),
                    ))
                    .await;
                None
            }
        };
        #[allow(clippy::let_and_return)]
        next_state
    }

    async fn set_service_context(&mut self, context: Value) {
        self.llm_req_setting = serde_json::from_value(context).unwrap();
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

impl FSMChatState {
    async fn prepare_context(
        &self,
        llm_req_settings: &llm_agent::LlmReqSetting,
    ) -> FSMChatStateData {
        let messages = if self.config.use_only_last_message.unwrap_or(false) {
            vec![llm_req_settings
                .messages
                .last()
                .cloned()
                .unwrap_or_default()]
        } else {
            llm_req_settings.messages.clone()
        };

        let summary = if let Some(summary) = llm_req_settings.memory.get("summary") {
            let summary = summary.last().cloned().unwrap_or_default();
            serde_json::from_value::<String>(summary.clone()).unwrap_or("".into())
        } else {
            "".into()
        };

        let task = llm_req_settings.task.clone().unwrap_or_default();

        let context = if let Some(context) = llm_req_settings.memory.get("context") {
            if self.config.use_full_context.unwrap_or(false) {
                context
                    .iter()
                    .map(|c| serde_json::from_value::<String>(c.clone()).unwrap_or("".into()))
                    .collect::<Vec<String>>()
                    .join("\n\n")
            } else {
                let context = context.last().cloned().unwrap_or_default();
                serde_json::from_value::<String>(context.clone()).unwrap_or("".into())
            }
        } else {
            "".into()
        };

        let tools = if let Some(ref tools) = llm_req_settings.tools {
            tools.iter().map(|(tool_name, tool)| {
                format!("\n<tool>\nName:: {}\nDescription:: {}\nTake input:: {}\nReturn an output of type:: {}\n<tool>\n", 
                tool_name, tool.description, tool.arguments, tool.output_type)
            }).collect::<Vec<_>>().join("\n")
        } else {
            "".into()
        };

        let memory: HashMap<String, String> = if let Some(ref use_memory) = self.config.use_memory {
            use_memory
                .iter()
                .flat_map(|(slot_name, n)| {
                    if let Some(vec) = llm_req_settings.memory.get(slot_name) {
                        let start = if vec.len() > *n { vec.len() - *n } else { 0 };
                        let m = vec[start..]
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join("\n\n");
                        Some((slot_name.clone(), m))
                    } else {
                        None
                    }
                })
                .collect::<_>()
        } else {
            HashMap::default()
        };

        FSMChatStateData {
            messages,
            summary,
            task,
            context,
            tools,
            memory,
        }
    }

    async fn handle_llm_output(
        &mut self,
        llm_req_settings: &llm_agent::LlmReqSetting,
        fsm_tx: &Sender<(String, String, String)>,
    ) -> Result<String, anyhow::Error> {
        let llm_output = if !self.config.disable_llm_request.unwrap_or(false) {
            let system_prompt = self.prompts.system.clone().unwrap_or("".into());
            let chat_prompt = self.prompts.chat.as_ref().unwrap_or(&"".into()).clone();

            if system_prompt.len() + chat_prompt.len() > 0 {
                let mut tera_context = tera::Context::new();
                tera_context.insert("context", &self.state_data.context);
                tera_context.insert("summary", &self.state_data.summary);
                tera_context.insert("task", &self.state_data.task);
                tera_context.insert("tools", &self.state_data.tools);

                self.state_data.memory.iter().for_each(|(slot_name, m)| {
                    tera_context.insert(slot_name, m);
                });

                let full_prompt = [system_prompt, chat_prompt].join("\n");
                let full_prompt = Tera::one_off(&full_prompt, &tera_context, false)?;

                let model = llm_req_settings.model.clone();
                let api_key = llm_req_settings.api_key.clone();
                let temperature = llm_req_settings.temperature;
                let ignore_llm_output = self.config.ignore_llm_output.unwrap_or(false);
                let messages = if self.config.ignore_messages.unwrap_or(false) {
                    vec![]
                } else {
                    self.state_data.messages.clone()
                };
                self.handle = Some(
                    get_llm_req_process_handle(
                        self.name.clone(),
                        fsm_tx.clone(),
                        messages,
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
            }
        } else {
            "".into()
        };

        if self.config.save_to_summary.unwrap_or(false) {
            let _ = fsm_tx
                .send((self.name.clone(), "summary".into(), llm_output.clone()))
                .await;
        }

        if self.config.save_to_context.unwrap_or(false) {
            let _ = fsm_tx
                .send((self.name.clone(), "context".into(), llm_output.clone()))
                .await;
        }

        if self.config.extract_code.unwrap_or(false) {
            let code = extract_code(&llm_output);
            let _ = fsm_tx.send((self.name.clone(), "code".into(), code)).await;
        }

        if let Some(ref memory_slots) = self.config.save_to {
            for slot in memory_slots.iter() {
                let _ = fsm_tx
                    .send((
                        self.name.clone(),
                        format!("save_to:{}", slot),
                        llm_output.clone(),
                    ))
                    .await;
            }
        }
        Ok(llm_output)
    }

    async fn execute_code(
        &self,
        llm_req_settings: &llm_agent::LlmReqSetting,
        tx: &Sender<(String, String, String)>,
    ) -> Result<(String, String), anyhow::Error> {
        #[derive(Deserialize, Debug)]
        struct ExecuteCode {
            run: bool,
        }

        if self.config.execute_code.unwrap_or(false) {
            let code = if let Some(code) = self.config.code.clone() {
                self.wrap_code(llm_req_settings, None, None, code)?
            } else {
                let code = llm_req_settings
                    .memory
                    .get("code")
                    .cloned()
                    .unwrap_or_default();
                let code = code.last().cloned().unwrap_or_default();
                serde_json::from_value::<String>(code).unwrap_or("".into())
            };

            match self.config.wait_for_msg.unwrap_or(false) {
                true => {
                    // if wait for msg and let LLM infer if the user want to continue
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
                                "exec_output".into(),
                                "\nconditionally, run code from the context:\n".into(),
                            ))
                            .await;
                        let (stdout, stderr) = run_code_in_docker(&code);
                        let _ = tx
                            .send((
                                self.name.clone(),
                                "exec_output".into(),
                                format!("stdout:\n {}\n", stdout),
                            ))
                            .await;
                        let _ = tx
                            .send((
                                self.name.clone(),
                                "exec_output".into(),
                                format!("stderr:\n {}\n", stderr),
                            ))
                            .await;
                        Ok((stdout, stderr))
                    } else {
                        let _ = tx
                            .send((
                                self.name.clone(),
                                "exec_output".into(),
                                "code execution rejected\n".into(),
                            ))
                            .await;
                        Ok(("".into(), "".into()))
                    }
                }
                false => {
                    // just execute the code without a user input
                    let (stdout, stderr) = run_code_in_docker(&code);
                    let _ = tx
                        .send((
                            self.name.clone(),
                            "exec_output".into(),
                            format!("stdout:\n{}\n", stdout),
                        ))
                        .await;
                    let _ = tx
                        .send((
                            self.name.clone(),
                            "exec_output".into(),
                            format!("stderr:\n{}\n", stderr),
                        ))
                        .await;
                    Ok((stdout, stderr))
                }
            }
        } else {
            Ok(("".into(), "".into()))
        }
    }

    async fn save_execution_output(
        &self,
        tx: &Sender<(String, String, String)>,
        stdout: &str,
        stderr: &str,
    ) {
        if self.config.execute_code.unwrap_or(false) {
            if self.config.save_to_context.unwrap_or(false) {
                let _ = tx
                    .send((self.name.clone(), "context".into(), stdout.into()))
                    .await;
            }

            if self.config.save_execution_output.unwrap_or(false) {
                let execution_output = serde_json::to_value(ExecutionOutput {
                    stdout: stdout.into(),
                    stderr: stderr.into(),
                })
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

            if let Some(ref memory_slots) = self.config.save_to {
                for slot in memory_slots.iter() {
                    let _ = tx
                        .send((
                            self.name.clone(),
                            format!("save_to:{}", slot),
                            stdout.to_string(),
                        ))
                        .await;
                }
            }
        }
    }

    async fn determine_next_state(
        &self,
        llm_req_settings: &llm_agent::LlmReqSetting,
        tx: &Sender<(String, String, String)>,
        next_states: &Option<Vec<String>>,
        llm_output: &String,
    ) -> Result<Option<String>, anyhow::Error> {
        if let Some(fsm_code) = self.config.fsm_code.clone() {
            let code = self.wrap_code(
                llm_req_settings,
                next_states.as_ref(),
                Some(llm_output),
                fsm_code,
            )?;
            let (stdout, stderr) = run_code_in_docker(&code);
            let _ = tx
                .send((
                    self.name.clone(),
                    "fsm_exec_output".into(),
                    format!("stdout:\n{}\n", stdout),
                ))
                .await;
            let _ = tx
                .send((
                    self.name.clone(),
                    "fsm_exec_output".into(),
                    format!("stderr:\n{}\n", stderr),
                ))
                .await;
            Ok(Some(stdout.trim().into()))
        } else if let Some(next_states) = next_states {
            if next_states.len() == 1 {
                Ok(Some(next_states.first().unwrap().clone()))
            } else if let Some(fsm_prompt) = self.prompts.fsm.clone() {
                let available_transitions = next_states.join(", ");
                let msg = format!(
                    r#"
Given these information, you need to determine the next state following the instructions below:

Current State: {}

Available Next States: {}

Make sure the output is just a simple valid JSON string in 
the format following the instruction above: `{{"next_state": SOME_NEXT_STATE}}`. 
The "SOME_NEXT_STATE" is one of the Available Next States.
"#,
                    self.name, available_transitions
                );
                let fsm_prompt = [msg, fsm_prompt].join("\n");
                let mut tera_context = tera::Context::new();
                tera_context.insert("task", &self.state_data.task);
                tera_context.insert("messages", &self.state_data.messages);
                tera_context.insert("summary", &self.state_data.summary);
                tera_context.insert("context", &self.state_data.context);
                tera_context.insert("summary", &self.state_data.summary);
                tera_context.insert("response", &llm_output);

                self.state_data.memory.iter().for_each(|(slot_name, m)| {
                    tera_context.insert(slot_name, m);
                });

                let fsm_prompt = Tera::one_off(&fsm_prompt, &tera_context, false)?;

                let llm_client = GenaiLlmclient {
                    model: llm_req_settings.model.clone(),
                    api_key: llm_req_settings.api_key.clone(),
                };

                let next_state = llm_client
                    .generate(
                        &fsm_prompt,
                        &[("user".into(), "determine the next state".into())],
                        llm_req_settings.temperature,
                    )
                    .await?;
                // println!("\nllm nextstep raw response: {}", next_state );
                let next_fsm_state_response =
                    serde_json::from_str::<LlmResponse>(next_state.trim());
                // println!("\nllm next_fsm_state_response: {:?}", next_fsm_state_response );
                match next_fsm_state_response {
                    Ok(next_fsm_state_response) => Ok(next_fsm_state_response.next_state),
                    Err(e) => Err(anyhow::anyhow!(
                        "fail to parse LLM json output for next fsm state: {:?} \n LLM output: {}",
                        e,
                        next_state
                    )),
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn wrap_code(
        &self,
        llm_req_settings: &LlmReqSetting,
        next_states: Option<&Vec<String>>,
        llm_output: Option<&String>,
        fsm_code: String,
    ) -> Result<String, anyhow::Error> {
        let mut tera_context = tera::Context::new();
        let messages = escape_json_string(&json!(&self.state_data.messages).to_string());
        let context = escape_json_string(&json!(&self.state_data.context).to_string());
        let summary = escape_json_string(&json!(&self.state_data.summary).to_string());
        let state_name = escape_json_string(&json!(&self.name).to_string());
        let next_states = if let Some(next_states) = next_states {
            escape_json_string(&json!(&next_states).to_string())
        } else {
            "[]".into()
        };
        let state_history = escape_json_string(&json!(llm_req_settings.state_history).to_string());
        let task = escape_json_string(&json!(llm_req_settings.task).to_string());
        tera_context.insert("messages", &messages);
        tera_context.insert("context", &context);
        tera_context.insert("summary", &summary);
        tera_context.insert("state_name", &state_name);
        tera_context.insert("next_states", &next_states);
        tera_context.insert("state_history", &state_history);
        tera_context.insert("task", &task);
        tera_context.insert("response", &llm_output);
        self.state_data.memory.iter().for_each(|(slot_name, m)| {
            tera_context.insert(slot_name, &escape_json_string(&json!(m).to_string()));
        });
        let output = Tera::one_off(&fsm_code, &tera_context, false)?;
        Ok(output)
    }
}
