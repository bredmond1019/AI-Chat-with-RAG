use futures::stream::Stream;
use log::{error, info};
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponseStream},
    Ollama,
};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::error::Error as StdError;
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct AIModel {
    ollama: Ollama,
    py_wrapper: Arc<Mutex<PyObject>>,
}

impl AIModel {
    pub fn new() -> PyResult<Self> {
        info!("Initializing new AIModel with Ollama and Python integration");
        let ollama = Ollama::new_default_with_history(30);

        let py_wrapper = Python::with_gil(|py| -> PyResult<Arc<Mutex<PyObject>>> {
            py.import_bound("sys")?
                .getattr("path")?
                .call_method1("append", ("./python_app",))?;
            let py_module = py.import_bound("python_wrapper")?;
            let wrapper = py_module.getattr("wrapper")?.to_object(py);
            Ok(Arc::new(Mutex::new(wrapper)))
        })?;

        Ok(Self { ollama, py_wrapper })
    }

    pub async fn generate_response(
        &mut self,
        input: String,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<String, Box<dyn std::error::Error>>> + Send>>,
        Box<dyn std::error::Error>,
    > {
        info!("Generating AI response for input: {}", input);
        let stream: ChatMessageResponseStream = self
            .ollama
            .send_chat_messages_with_history_stream(
                ChatMessageRequest::new(
                    "llama3.1:latest".to_string(),
                    vec![ChatMessage::user(input.clone())],
                ),
                "user".to_string(),
            )
            .await
            .map_err(|e| {
                error!("Failed to send chat message: {}", e);
                AIModelError::RequestError(e.to_string())
            })?;

        info!("Successfully initiated chat message stream");

        Ok(Box::pin(stream.map(|res| match res {
            Ok(chunk) => {
                if let Some(assistant_message) = chunk.message {
                    info!("Received chunk of AI response");
                    Ok(assistant_message.content)
                } else {
                    Ok(String::new())
                }
            }
            Err(e) => {
                error!("Error while streaming response: {:?}", e);
                Err(Box::new(AIModelError::StreamingError(format!("{:?}", e)))
                    as Box<dyn std::error::Error>)
            }
        })))
    }

    pub async fn ingest_documents(&self) -> Result<(), Box<dyn StdError>> {
        info!("Ingesting documents");
        let wrapper = self.py_wrapper.clone();
        tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| wrapper.blocking_lock().call_method0(py, "ingest_documents"))
        })
        .await??;
        Ok(())
    }

    pub async fn query_documents(&self, input: String) -> Result<String, Box<dyn StdError>> {
        info!("Querying documents using Python for input: {}", input);
        let wrapper = self.py_wrapper.clone();
        let response = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                wrapper
                    .blocking_lock()
                    .call_method1(py, "query_documents", (input,))
            })
        })
        .await??;

        let response_str = Python::with_gil(|py| response.extract::<String>(py))?;
        Ok(response_str)
    }
}

#[derive(Debug)]
enum AIModelError {
    RequestError(String),
    StreamingError(String),
    EmptyResponse,
    PythonError(String),
}

impl fmt::Display for AIModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AIModelError::RequestError(e) => write!(f, "Request error: {}", e),
            AIModelError::StreamingError(e) => write!(f, "Streaming error: {}", e),
            AIModelError::EmptyResponse => write!(f, "Empty response generated"),
            AIModelError::PythonError(e) => write!(f, "Python error: {}", e),
        }
    }
}

impl StdError for AIModelError {}
