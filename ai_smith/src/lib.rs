use pyo3::{prelude::*, types::PyModule};
// use pyo3::types::{PyDict, PyString};
use std::{collections::HashMap, time::Duration};

use ai_gent_lib::{
    fsm_chat_state::FsmAgentState,
    llm_agent::{LlmFsmAgent, LlmFsmAgentConfigBuilder, LlmFsmBuilder},
};

use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::sleep,
};

#[pyclass]
#[derive(Clone)]
struct AgentSettings {
    model: String,
    api_key: String,
    total_state_transition_limit: Option<u32>,
}

#[pymethods]
impl AgentSettings {
    #[new]
    #[pyo3(signature = (model, api_key, total_state_transition_limit=None))]
    fn new(model: String, api_key: String, total_state_transition_limit: Option<u32>) -> Self {
        AgentSettings {
            model,
            api_key,
            total_state_transition_limit,
        }
    }

    #[getter]
    fn get_model(&self) -> PyResult<String> {
        Ok(self.model.clone())
    }

    #[getter]
    fn get_api_key(&self) -> PyResult<String> {
        Ok(self.api_key.clone())
    }

    #[getter]
    fn get_total_state_transition_limit(&self) -> PyResult<Option<u32>> {
        Ok(self.total_state_transition_limit)
    }
}

#[pyclass]
struct Agent {
    fsm_config: String,
    agent_settings: AgentSettings,
    handle: Option<JoinHandle<Result<(), anyhow::Error>>>,
}

#[pymethods]
impl Agent {
    #[new]
    fn new(fsm_config: &str, agent_settings: AgentSettings) -> Self {
        Self {
            fsm_config: fsm_config.to_string(),
            agent_settings,
            handle: None,
        }
    }

    #[pyo3(signature = (agent_command_rx, agent_service_tx, temperature=None))]
    fn agent_message_service(
        &mut self,
        _py: Python<'_>,
        agent_command_rx: PyObject,
        agent_service_tx: PyObject,
        temperature: Option<f32>,
    ) -> PyResult<()> {
        let rx = receive_from_py_queue(agent_command_rx)?;
        let tx = send_to_py_queue(agent_service_tx)?;

        // Specify the correct type including Mutex
        let config = LlmFsmAgentConfigBuilder::from_toml(&self.fsm_config)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        // Build FSM
        let fsm = LlmFsmBuilder::from_config::<FsmAgentState>(&config, HashMap::default())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let settings = ai_gent_lib::llm_agent::AgentSettings {
            sys_prompt: config.system_prompt,
            fsm_prompt: config.fsm_prompt,
            summary_prompt: config.summary_prompt,
            fsm_initial_state: config.initial_state,
            model: self.agent_settings.model.clone(),
            api_key: self.agent_settings.api_key.clone(),
            tools: config.tools,
            total_state_transition_limit: self.agent_settings.total_state_transition_limit,
        };

        let mut agent = LlmFsmAgent::new(fsm, settings);

        self.handle = Some(pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
            // let mut agent_guard = agent.
            agent.agent_message_service(rx, tx, temperature).await
        }));
        Ok(())
    }

    fn stop_agent_message_service(&mut self, _py: Python<'_>) {
        let handle = self.handle.take();
        if let Some(handle) = handle {
            let _ = pyo3_async_runtimes::tokio::get_runtime()
                .block_on(async move { tokio::join!(handle) });
        }
    }

    fn abort_agent_message_service(&mut self, _py: Python<'_>) {
        let handle = self.handle.take();
        if let Some(handle) = handle {
            pyo3_async_runtimes::tokio::get_runtime().block_on(async move {
                handle.abort();
            });
        }
    }
}

fn receive_from_py_queue(queue: PyObject) -> PyResult<Receiver<(String, String)>> {
    let (tx, rx) = mpsc::channel(1); // Create channel with buffer size 100

    // Spawn a task that continuously gets items from Python queue
    let rt = pyo3_async_runtimes::tokio::get_runtime();
    rt.spawn(async move {
        loop {
            let msg = Python::with_gil(|py| -> Option<(String, String)> {
                let queue_clone = queue.clone_ref(py);
                // Call get() method on Python queue
                if let Ok(item) = queue_clone.call_method0(py, "get_nowait") {
                    // Convert PyObject to String
                    let msg = item.extract::<(String, String)>(py).unwrap();
                    Some(msg)
                } else {
                    None
                }
            });
            match msg {
                Some(msg) => {
                    if tx.send(msg).await.is_err() {
                        break; // Channel closed
                    }
                }
                None => {
                    sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    Ok(rx)
}

fn send_to_py_queue(queue: PyObject) -> PyResult<Sender<(String, String, String)>> {
    let (tx, mut rx) = mpsc::channel(1); // Create channel with buffer size 100

    // Spawn a task that sends items to Python queue
    let rt = pyo3_async_runtimes::tokio::get_runtime();
    rt.spawn(async move {
        while let Some(msg) = rx.recv().await {
            Python::with_gil(|py| {
                let queue_clone = queue.clone_ref(py);
                queue_clone.call_method1(py, "put", (msg,)).unwrap();
            })
        }
    });

    Ok(tx)
}

#[pymodule]
fn ai_smith(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AgentSettings>()?;
    m.add_class::<Agent>()?;
    Ok(())
}
