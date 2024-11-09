use crate::ChatRequest;
use crate::ChatResponseStream;
use crate::Error;
use futures_util::stream::TryStreamExt;
use nd_tokio_sse_codec::SseCodec;
use tokio_util::codec::FramedRead;
use tokio_util::io::StreamReader;

static USER_AGENT_STR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const CHAT_URL: &str = "https://duckduckgo.com/duckchat/v1/chat";

/// A client for duck duck go's ai features.
#[derive(Debug, Clone)]
pub struct Client {
    /// The inner http client
    pub client: reqwest::Client,
}

impl Client {
    /// Make a new client.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT_STR)
            .http1_title_case_headers()
            .build()
            .expect("failed to build client");

        Self { client }
    }

    /// Init a new chat.
    pub async fn init_chat(&self) -> Result<ChatRequest, Error> {
        let url = "https://duckduckgo.com/duckchat/v1/status";
        let response = self
            .client
            .get(url)
            .header("x-vqd-accept", "1")
            .send()
            .await?
            .error_for_status()?;
        let vqd = response
            .headers()
            .get("x-vqd-4")
            .and_then(|header| header.to_str().ok())
            .ok_or(Error::MissingVqd)?
            .to_string();
        let _text = response.text().await?;

        Ok(ChatRequest {
            messages: Vec::new(),
            model: DEFAULT_MODEL.into(),
            vqd: Some(vqd),
        })
    }

    /// Chat with an AI.
    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponseStream, Error> {
        let vqd = request.vqd.as_deref().ok_or(Error::MissingVqd)?;
        let response = self
            .client
            .post(CHAT_URL)
            .header("x-vqd-4", vqd)
            .json(request)
            .send()
            .await?
            .error_for_status()?;
        let stream = response.bytes_stream().map_err(std::io::Error::other);
        let stream_reader = StreamReader::new(stream);
        let codec = SseCodec::new();
        let reader = FramedRead::new(stream_reader, codec);

        Ok(ChatResponseStream::new(Box::pin(reader)))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
