use ai_gent_lib::{llm_agent::{LlmFsmBuilder, LlmFsmStateInit, StateConfig}, GenaiLlmclient};
use async_trait::async_trait;
use futures::StreamExt;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use ai_gent_lib::fsm::FsmState;
use anyhow::Result;
use serde::Deserialize;
use tokio::sync::mpsc::{self, Receiver, Sender};

use ai_gent_lib::llm_agent::{
    self, AgentSettings, LlmFsmAgentConfigBuilder, LlmFsmAgent, LlmClient, LlmResponse, StatePrompts,
};
use tokio::task::JoinHandle;

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
        if let Some(get_msg) = config.get_msg {
            if get_msg {
                attributes.insert("get_msg".into(), "true".into());
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
        // println!("tracing: llm_request");
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
    use std::process::Command;
    use std::io::Write;
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

        if !self.config.disable_llm_request.unwrap_or(false) {
            let summary = if !llm_req_settings.summary.is_empty() {
                ["<SUMMARY>", &llm_req_settings.summary, "</SUMMARY>"].join("\n")
            } else {
                "".into()
            };

            let context = if let Some(ref context) = llm_req_settings.context {
                ["<CONTEXT>", context, "</CONTEXT>"].join("\n")
            } else {
                "".into()
            };

            let system_prompt = self.prompts.system.clone().unwrap_or("".into());

            let chat_prompt = self.prompts.chat.as_ref().unwrap_or(&"".into()).clone();

            let state_name = self.name.clone();
            let _ = tx
                .send((state_name.clone(), "state".into(), state_name.clone()))
                .await;

            let llm_output = if system_prompt.len() + chat_prompt.len() > 0 {
                let full_prompt = [system_prompt, summary, context, chat_prompt].join("\n");
                let model = llm_req_settings.model.clone();
                let api_key = llm_req_settings.api_key.clone();
                let messages = llm_req_settings.messages.clone();
                let temperature = llm_req_settings.temperature;
                let ignore_llm_output = self.config.ignore_llm_output.unwrap_or(false);

                self.handle = Some(
                    get_llm_req_process_handle(
                        state_name,
                        tx.clone(),
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
            };

            if self.config.extract_code.unwrap_or(false) {
                let code = extract_code(&llm_output);
                let _ = tx.send((self.name.clone(), "code".into(), code)).await;
            }
        };

        #[derive(Deserialize, Debug)]
        struct ExcuteCode {
            run: bool
        }

        if self.config.execute_code.unwrap_or(false) {
            let code = llm_req_settings.context.as_ref().unwrap_or(&"".into()).clone();
            if self.config.get_msg.unwrap_or(false) {
                let llm_output = self.get_attribute("llm_output").await.unwrap_or(String::new());
                let llm_output = serde_json::from_str(&llm_output).unwrap_or(ExcuteCode{run:false});      
                if llm_output.run {
                    println!("conditionally, run code from the context:\n");
                    let (stdout, stderr) = run_code_in_docker(&code);
                    println!("stdout:\n {}\n", stdout);
                    println!("stderr:\n {}\n", stderr);
                } else {
                    println!("code execution rejected");
                }
            } else {
                println!("run code from the context\n");
                let (stdout, stderr) = run_code_in_docker(&code);
                println!("stdout:\n {}\n", stdout);
                println!("stderr:\n {}\n", stderr);
            }
        }

        {
            // get the the FSM state
            if let Some(next_states) = next_states {
                if next_states.len() == 1 {
                    Some(next_states.first().unwrap().clone())
                } else if let Some(fsm_prompt) = self.prompts.fsm.clone() {
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

                    let next_fsm_state_response: LlmResponse = serde_json::from_str(&next_state)
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

const FSM_CONFIG: &str = include_str!("../../../dev_config/fsm_config.toml");

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fsm_config = LlmFsmAgentConfigBuilder::from_toml(FSM_CONFIG)?.build()?;

    let fsm = LlmFsmBuilder::from_config::<FSMChatState>(&fsm_config, HashMap::default())?.build()?;

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
    let mut agent = LlmFsmAgent::new(fsm, llm_req_setting);

    // tracing::info!("agent config: {}", fsm_config.to_json().unwrap());

    //write_agent_config_to_file(&fsm_config);

    println!("Welcome to the LLM Agent CLI. Type 'exit' to quit.");
    let mut rl = DefaultEditor::new()?; // Use DefaultEditor instead

    loop {
        let readline = rl.readline("\n>> ");
        match readline {
            Ok(user_input) => {
                if user_input.trim().eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(user_input.as_str());
                let (tx, mut rx) = mpsc::channel::<(String, String, String)>(1);

                let t = tokio::spawn(async move {
                    let mut llm_output = Vec::<String>::new();
                    while let Some(message) = rx.recv().await {
                        match (message.0.as_str(), message.1.as_str()) {
                            (_, "state") => {
                                println!("\n--------- Agent State: {}", message.2);
                            }
                            (s, "token") if s != "MakeSummary" => {
                                print!("{}", message.2);
                            }
                            (_, "llm_output") => {
                                llm_output.push(message.2);
                                println!(); // clean rustline's buffer, or we miss the final line of the output
                            }
                            _ => {}
                        }
                    }
                    llm_output
                });
                match agent.fsm_message_service(&user_input, Some(tx), None).await {
                    Ok(_res) => {}
                    Err(err) => {
                        println!("LLM error, please retry your question. {:?}", err)
                    }
                }

                let _llm_output = tokio::join!(t).0.unwrap();
                //println!("\nout: {}", llm_output);              // if let Some(current_state) = agent.fsm.current_state() {
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
    Ok(())
}
