mod client;
pub mod model;

pub use self::client::Client;
pub use self::model::ChatMessage;
pub use self::model::ChatRequest;
pub use self::model::ChatResponseStream;

/// The library error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A http error occured
    #[error("http error")]
    Reqwest(#[from] reqwest::Error),

    /// Failed to join a tokio task
    #[error("bad tokio join")]
    TokioJoin(#[from] tokio::task::JoinError),

    /// An sse event was invalid
    #[error("invalid sse event")]
    InvalidSseEvent(#[from] nd_tokio_sse_codec::SseCodecError),

    /// An sse event is missing data
    #[error("sse event missing data")]
    SseEventMissingData,

    /// An sse event had an invalid json payload
    #[error("invalid sse json data")]
    InvalidSseEventData(#[source] serde_json::Error),

    /// The stream was empty
    #[error("stream empty")]
    StreamEmpty,

    /// Missing Vqd
    #[error("missing vqd")]
    MissingVqd,
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let client = Client::new();
        let mut request = client.init_chat().await.expect("failed to init chat");
        request.messages.push(ChatMessage {
            role: "user".into(),
            content: "Hello! How are you today?".into(),
        });
        request.model = "mistralai/Mixtral-8x7B-Instruct-v0.1".into();

        let mut stream = client
            .chat(&mut request)
            .await
            .expect("failed to send chat request");
        let message = stream
            .collect_into_chat_message()
            .await
            .expect("failed to collect message");
        dbg!(message);
    }
}
