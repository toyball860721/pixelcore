use reqwest::Client;
use crate::error::ClawError;
use crate::types::{LlmRequest, LlmResponse, OpenAiResponse};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const SILICONFLOW_API_URL: &str = "https://api.siliconflow.cn/v1/chat/completions";

pub enum ApiBackend {
    Anthropic,
    OpenAiCompat { base_url: String },
}

pub struct ClawClient {
    client: Client,
    api_key: String,
    backend: ApiBackend,
}

impl ClawClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            backend: ApiBackend::Anthropic,
        }
    }

    pub fn with_openai_compat(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            backend: ApiBackend::OpenAiCompat { base_url: base_url.into() },
        }
    }

    pub fn siliconflow(api_key: impl Into<String>) -> Self {
        Self::with_openai_compat(api_key, SILICONFLOW_API_URL)
    }

    pub fn from_env() -> Result<Self, ClawError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| ClawError::MissingApiKey("anthropic".to_string()))?;
        Ok(Self::new(api_key))
    }

    pub async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, ClawError> {
        match &self.backend {
            ApiBackend::Anthropic => self.complete_anthropic(request).await,
            ApiBackend::OpenAiCompat { base_url } => {
                self.complete_openai(request, base_url.clone()).await
            }
        }
    }

    async fn complete_anthropic(&self, request: LlmRequest) -> Result<LlmResponse, ClawError> {
        let response = self.client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(ClawError::RateLimited { retry_after });
        }
        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClawError::Api { status, message: body });
        }

        Ok(response.json::<LlmResponse>().await?)
    }

    async fn complete_openai(&self, request: LlmRequest, url: String) -> Result<LlmResponse, ClawError> {
        // Convert to OpenAI chat format
        let openai_req = request.to_openai();

        let response = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .json(&openai_req)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(ClawError::RateLimited { retry_after });
        }
        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClawError::Api { status, message: body });
        }

        let oai: OpenAiResponse = response.json().await?;
        Ok(oai.into_llm_response())
    }
}
