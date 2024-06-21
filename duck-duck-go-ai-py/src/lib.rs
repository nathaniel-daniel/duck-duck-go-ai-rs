use anyhow::Context;
use duck_duck_go_ai::ChatMessage;
use duck_duck_go_ai::ChatRequest;
use duck_duck_go_ai::Client;
use once_cell::sync::Lazy;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use tokio::sync::OwnedMutexGuard;
use tokio_stream::StreamExt;

static TOKIO_RUNTIME: Lazy<std::io::Result<tokio::runtime::Runtime>> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
});

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

/// A chat with an AI
#[pyclass(sequence)]
struct Chat {
    chat_request: Arc<Mutex<ChatRequest>>,
}

impl Chat {
    /// Get the chat request, if it is not streaming.
    fn get_chat_request(&self) -> Option<MutexGuard<'_, ChatRequest>> {
        self.chat_request.try_lock().ok()
    }
}

#[pymethods]
impl Chat {
    /// Create a new chat.
    #[staticmethod]
    pub fn init() -> PyResult<Self> {
        let tokio_rt = TOKIO_RUNTIME
            .as_ref()
            .context("failed to init tokio runtime")?;

        let chat_request = tokio_rt
            .block_on(CLIENT.init_chat())
            .context("failed to init chat")?;

        Ok(Self {
            chat_request: Arc::new(Mutex::new(chat_request)),
        })
    }

    /// Get the model.
    pub fn get_model<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyString>> {
        let chat_request = self.get_chat_request().context("chat is busy")?;
        Ok(PyString::new_bound(py, chat_request.model.as_str()))
    }

    /// Set the model.
    pub fn set_model(&mut self, model: &str) -> PyResult<()> {
        let mut chat_request = self.get_chat_request().context("chat is busy")?;
        if chat_request.messages.len() >= 2 {
            return Err(Into::into(anyhow::Error::msg(
                "cannot change model of in-progress chat",
            )));
        }

        chat_request.model = model.into();
        Ok(())
    }

    /// Get the number of messages.
    pub fn __len__(&self) -> PyResult<usize> {
        let chat_request = self.get_chat_request().context("chat is busy")?;
        Ok(chat_request.messages.len())
    }

    /// Get the chat message at the given index.
    pub fn __getitem__<'a>(
        &self,
        py: Python<'a>,
        index: usize,
    ) -> PyResult<Option<(Bound<'a, PyString>, Bound<'a, PyString>)>> {
        let chat_request = self.get_chat_request().context("chat is busy")?;
        let message = match chat_request.messages.get(index) {
            Some(message) => message,
            None => {
                return Err(PyIndexError::new_err("message index out of range"));
            }
        };

        let role = PyString::new_bound(py, message.role.as_str());
        let content = PyString::new_bound(py, message.content.as_str());

        Ok(Some((role, content)))
    }

    /// Create a user message and get the response.
    pub fn send_message(&self, content: &str) -> PyResult<ChatResponseStream> {
        struct ChatRequestGuard {
            chat_request: OwnedMutexGuard<ChatRequest>,
            pop_last: bool,
        }

        impl ChatRequestGuard {
            fn new(chat_request: OwnedMutexGuard<ChatRequest>) -> Self {
                Self {
                    chat_request,
                    pop_last: true,
                }
            }

            /// Add the response message from the server, diffusing the drop guard.
            fn push_response(&mut self, response: ChatMessage) {
                assert!(self.pop_last);

                self.chat_request.messages.push(response);
                self.pop_last = false;
            }
        }

        impl Drop for ChatRequestGuard {
            fn drop(&mut self) {
                if self.pop_last {
                    self.chat_request.messages.pop();
                }
            }
        }

        let tokio_rt = TOKIO_RUNTIME
            .as_ref()
            .context("failed to init tokio runtime")?;

        let mut chat_request = self
            .chat_request
            .clone()
            .try_lock_owned()
            .ok()
            .context("chat is busy")?;

        chat_request.messages.push(ChatMessage {
            role: "user".into(),
            content: content.into(),
        });

        let mut chat_request = ChatRequestGuard::new(chat_request);

        let rx = tokio_rt.block_on(async move {
            let mut stream = CLIENT
                .chat(&chat_request.chat_request)
                .await
                .context("failed to send chat request")?;

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(async move {
                let mut role = None;
                let mut content = String::new();
                let mut stream_error = None;

                while let Some(message_result) = stream.next().await {
                    let message_result = message_result.context("failed to parse event");
                    match message_result {
                        Ok(message) => {
                            role = Some(message.role);
                            content.push_str(&message.message);

                            // We need to keep processing even if we got cancelled.
                            let _ = tx.send(Ok(message.message)).is_ok();
                        }
                        Err(error) => {
                            // The error might be fatal.
                            // We are forced to exit here.
                            // Note that an error here means the user
                            // should recreate the entire chat from scratch.
                            stream_error = Some(error);
                            break;
                        }
                    }
                }

                match stream_error {
                    Some(error) => {
                        // Doesn't matter if nobody cares that we failed.
                        let _ = tx.send(Err(error)).is_ok();
                        drop(chat_request);
                    }
                    None => {
                        match role.context("missing role") {
                            Ok(role) => {
                                let new_message = ChatMessage { role, content };
                                chat_request.push_response(new_message);
                                drop(chat_request);
                            }
                            Err(error) => {
                                // Doesn't matter if nobody cares that we failed.
                                let _ = tx.send(Err(error)).is_ok();
                            }
                        };
                    }
                }
            });

            anyhow::Ok(rx)
        })?;

        Ok(ChatResponseStream { rx })
    }
}

/// A streaming chat response.
#[pyclass]
pub struct ChatResponseStream {
    rx: tokio::sync::mpsc::UnboundedReceiver<anyhow::Result<String>>,
}

#[pymethods]
impl ChatResponseStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'a>(
        mut slf: PyRefMut<'_, Self>,
        py: Python<'a>,
    ) -> PyResult<Option<Bound<'a, PyString>>> {
        slf.rx
            .blocking_recv()
            .transpose()
            .map(|token| token.map(|token| PyString::new_bound(py, &token)))
            .map_err(Into::into)
    }
}

/// A pyo3 module for Duck Duck Go's AI chat.
#[pymodule]
fn duck_duck_go_ai_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Chat>()?;
    m.add_class::<ChatResponseStream>()?;
    Ok(())
}
