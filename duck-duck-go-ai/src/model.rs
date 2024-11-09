use crate::Error;
use nd_tokio_sse_codec::SseCodecError;
use nd_tokio_sse_codec::SseEvent;
use std::pin::Pin;
use std::task::ready;
use std::task::Context;
use std::task::Poll;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

/// A chat request
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ChatRequest {
    /// Chat Messages
    pub messages: Vec<ChatMessage>,

    /// The model.
    ///
    /// Valid Choices:
    /// * "claude-3-haiku-20240307"
    /// * "claude-3-sonnet-20240229"
    /// * "claude-3-5-sonnet-20240620"
    /// * "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo"
    /// * "mistralai/Mixtral-8x7B-Instruct-v0.1"
    /// * "gpt-4o-mini"
    /// * "gpt-4o"
    ///
    /// These choices were valid in the past,
    /// but seem to no longer work:
    /// * "meta-llama/Llama-3-70b-chat-hf"
    /// * "gpt-3.5-turbo-0125"
    /// * "gpt-4"
    pub model: String,

    /// A vqd token.
    /// Needed to make requests,
    /// but is not a part of the JSON.
    #[serde(skip)]
    pub vqd: Option<String>,
}

/// A chat message, for a chat request
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ChatMessage {
    /// The role.
    ///
    /// Valid Choices:
    /// * "user"
    /// * "assistant"
    pub role: String,

    /// The message content.
    pub content: String,
}

/// A chat response message
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ChatResponseMessage {
    /// The role.
    pub role: Option<String>,

    /// The message part.
    pub message: Option<String>,

    /// The time the message was created?
    pub created: u64,

    /// ?
    pub id: String,

    /// ?
    pub action: String,

    /// The model that generated the model.
    pub model: String,
}

/// A response stream for a chat.
pub struct ChatResponseStream {
    stream: Pin<Box<dyn Stream<Item = Result<SseEvent, SseCodecError>> + Send>>,
    done: bool,
}

impl ChatResponseStream {
    /// Create a new [`ChatResponseStream`].
    pub(crate) fn new(
        stream: Pin<Box<dyn Stream<Item = Result<SseEvent, SseCodecError>> + Send>>,
    ) -> Self {
        Self {
            stream,
            done: false,
        }
    }

    /// Consume this stream and get the new chat message.
    pub async fn collect_into_chat_message(&mut self) -> Result<ChatMessage, Error> {
        let mut role = None;
        let mut content = String::new();

        while let Some(message) = self.next().await {
            let message = message?;

            if let Some(message_role) = message.role {
                role = Some(message_role);
            }

            if let Some(message) = message.message {
                content.push_str(&message);
            }
        }

        // TODO: Throw error if not done?

        Ok(ChatMessage {
            role: role.ok_or(Error::StreamEmpty)?,
            content,
        })
    }
}

impl std::fmt::Debug for ChatResponseStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatResponseStream")
            .field("done", &self.done)
            .finish()
    }
}

impl Stream for ChatResponseStream {
    type Item = Result<ChatResponseMessage, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        let event = match ready!(self.stream.as_mut().poll_next(cx)) {
            Some(event) => event,
            None => {
                return Poll::Ready(None);
            }
        };
        let data = event
            .map_err(Error::InvalidSseEvent)?
            .data
            .ok_or(Error::SseEventMissingData)?;

        if data == "[DONE]" {
            self.done = true;
            return Poll::Ready(None);
        }

        Poll::Ready(Some(
            serde_json::from_str(&data).map_err(Error::InvalidSseEventData),
        ))
    }
}
