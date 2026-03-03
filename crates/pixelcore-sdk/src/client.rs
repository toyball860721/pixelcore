use crate::models::{ApiRequest, ApiResponse, SdkConfig};
use std::collections::HashMap;

/// SDK 客户端
#[derive(Clone)]
pub struct SdkClient {
    config: SdkConfig,
}

impl SdkClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    /// 发送 API 请求（模拟实现）
    pub async fn send_request(&self, request: ApiRequest) -> Result<ApiResponse, String> {
        // 这是一个模拟实现
        // 在实际应用中，这里会使用 reqwest 或其他 HTTP 客户端

        println!("[SDK] Sending {} request to {}", request.method, request.path);

        // 模拟响应
        let mut response = ApiResponse::new(200);
        response.body = Some(serde_json::json!({
            "success": true,
            "message": "Request processed successfully"
        }));

        Ok(response)
    }

    /// GET 请求
    pub async fn get(&self, path: &str) -> Result<ApiResponse, String> {
        let request = ApiRequest::new("GET".to_string(), path.to_string());
        self.send_request(request).await
    }

    /// POST 请求
    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<ApiResponse, String> {
        let request = ApiRequest::new("POST".to_string(), path.to_string())
            .with_body(body);
        self.send_request(request).await
    }

    /// PUT 请求
    pub async fn put(&self, path: &str, body: serde_json::Value) -> Result<ApiResponse, String> {
        let request = ApiRequest::new("PUT".to_string(), path.to_string())
            .with_body(body);
        self.send_request(request).await
    }

    /// DELETE 请求
    pub async fn delete(&self, path: &str) -> Result<ApiResponse, String> {
        let request = ApiRequest::new("DELETE".to_string(), path.to_string());
        self.send_request(request).await
    }

    /// 获取配置
    pub fn config(&self) -> &SdkConfig {
        &self.config
    }
}

/// SDK 构建器
pub struct SdkClientBuilder {
    config: SdkConfig,
}

impl SdkClientBuilder {
    pub fn new(api_endpoint: String) -> Self {
        Self {
            config: SdkConfig::new(api_endpoint),
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.config = self.config.with_api_key(api_key);
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.config = self.config.with_timeout(timeout_secs);
        self
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.config = self.config.with_retry_count(retry_count);
        self
    }

    pub fn build(self) -> SdkClient {
        SdkClient::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sdk_client_get() {
        let client = SdkClientBuilder::new("http://localhost:8080".to_string())
            .build();

        let response = client.get("/api/test").await.unwrap();
        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_sdk_client_post() {
        let client = SdkClientBuilder::new("http://localhost:8080".to_string())
            .build();

        let body = serde_json::json!({"key": "value"});
        let response = client.post("/api/test", body).await.unwrap();
        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_sdk_client_builder() {
        let client = SdkClientBuilder::new("http://localhost:8080".to_string())
            .with_api_key("test_key".to_string())
            .with_timeout(60)
            .with_retry_count(5)
            .build();

        assert_eq!(client.config().api_endpoint, "http://localhost:8080");
        assert_eq!(client.config().api_key, Some("test_key".to_string()));
        assert_eq!(client.config().timeout_secs, 60);
        assert_eq!(client.config().retry_count, 5);
    }
}
