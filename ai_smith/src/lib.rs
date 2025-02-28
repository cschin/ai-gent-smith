use pyo3::prelude::*;
// use pyo3::types::{PyDict, PyString};
use pyo3_asyncio::tokio::get_runtime;
use std::{collections::HashMap, time::Duration};

use ai_gent_lib::{
    fsm_chat_state::FsmAgentState,
    llm_agent::{
        AgentSettings, LlmFsmAgent, LlmFsmAgentConfig, LlmFsmAgentConfigBuilder, LlmFsmBuilder,
    },
};

use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle, time::sleep,
};

#[pyclass]
#[derive(Clone)]
struct PyAgentSettings {
    model: String,
    api_key: String,
    total_state_transition_limit: Option<u32>,
}

#[pymethods]
impl PyAgentSettings {
    #[new]
    fn new(model: String, api_key: String, total_state_transition_limit: Option<u32>) -> Self {
        PyAgentSettings {
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
    handle: Option<JoinHandle<Result<(), anyhow::Error>>>,
}

#[pymethods]
impl Agent {

    #[new]
    fn new() -> Self {
        Self {
            handle: None
        }
    }

    fn agent_message_service(
        &mut self,
        _py: Python<'_>,
        fsm_config: &str,
        agent_settings: PyAgentSettings,
        agent_command_rx: PyObject,
        agent_service_tx: PyObject,
        temperature: Option<f32>,
    ) -> PyResult<()> {
        let rx = receive_from_py_queue(agent_command_rx)?;
        let tx = send_to_py_queue(agent_service_tx)?;

        // Specify the correct type including Mutex
        let config = LlmFsmAgentConfigBuilder::from_toml(fsm_config)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        // Build FSM
        let fsm = LlmFsmBuilder::from_config::<FsmAgentState>(&config, HashMap::default())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let settings = AgentSettings {
            sys_prompt: config.system_prompt,
            fsm_prompt: config.fsm_prompt,
            summary_prompt: config.summary_prompt,
            fsm_initial_state: config.initial_state,
            model: agent_settings.model,
            api_key: agent_settings.api_key,
            tools: config.tools,
            total_state_transition_limit: agent_settings.total_state_transition_limit,
        };

        let mut agent = LlmFsmAgent::new(fsm, settings);

        self.handle = Some(get_runtime().spawn(async move {
            // let mut agent_guard = agent.
            agent.agent_message_service(rx, tx, temperature).await
        }));
        Ok(())
    }

    fn stop_agent_message_service(&mut self, _py: Python<'_>) {
        let handle = self.handle.take();
        if let Some(handle) = handle {
            let _ = get_runtime().block_on(async move { tokio::join!(handle) });
        }
    }
}

fn receive_from_py_queue(queue: PyObject) -> PyResult<Receiver<(String, String)>> {
    let (tx, rx) = mpsc::channel(1); // Create channel with buffer size 100

    // Spawn a task that continuously gets items from Python queue
    let queue_clone = queue.clone();
    let rt = get_runtime();
    rt.spawn(async move {
        loop {
            let msg = Python::with_gil(|py| -> Option<(String, String)> {
                // Call get() method on Python queue
                let item = queue_clone.call_method0(py, "get_nowait").unwrap();

                // Convert PyObject to String
                let msg = item.extract::<(String, String)>(py).unwrap();
                Some(msg)
                // Send through channel
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
    let queue_clone = queue.clone();
    let rt = get_runtime();
    rt.spawn(async move {
        while let Some(msg) = rx.recv().await {
            Python::with_gil(|py| {
                queue_clone.call_method1(py, "put", (msg,)).unwrap();
            })
        }
    });

    Ok(tx)
}

#[pyclass]
struct PyLlmFsmAgentConfig {
    inner: LlmFsmAgentConfig,
}

#[pymethods]
impl PyLlmFsmAgentConfig {
    #[new]
    fn new() -> Self {
        PyLlmFsmAgentConfig {
            inner: LlmFsmAgentConfig::default(),
        }
    }

    #[staticmethod]
    fn from_json(json_str: &str) -> PyResult<Self> {
        let config = LlmFsmAgentConfig::from_json(json_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(PyLlmFsmAgentConfig { inner: config })
    }

    #[staticmethod]
    fn from_toml(toml_str: &str) -> PyResult<Self> {
        let config = LlmFsmAgentConfigBuilder::from_toml(toml_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(PyLlmFsmAgentConfig { inner: config })
    }

    fn to_json(&self) -> PyResult<String> {
        self.inner
            .to_json()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn to_json_pretty(&self) -> PyResult<String> {
        self.inner
            .to_json_pretty()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
}

/// Python module definition
#[pymodule]
fn ai_smith(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyLlmFsmAgentConfig>()?;
    m.add_class::<PyAgentSettings>()?;
    m.add_class::<Agent>()?;
    Ok(())
}
