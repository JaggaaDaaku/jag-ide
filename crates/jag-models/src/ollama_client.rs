use jag_core::errors::{JagError, Result};
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OllamaOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub num_predict: Option<i32>,
    pub num_ctx: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub total_duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub digest: String,
    pub modified_at: String,
}

#[derive(Debug, Deserialize)]
struct ListModelsResponse {
    models: Vec<OllamaModel>,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: base_url.to_string(),
        }
    }

    /// Retry execution helper with exponential backoff
    async fn with_retries<F, Fut, T>(&self, mut action: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, ReqwestError>>,
    {
        let mut retries = 0;
        let mut delay = Duration::from_secs(1);

        loop {
            match action().await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    if retries >= 3 {
                        return Err(JagError::Http(e));
                    }
                    if e.is_connect() || e.is_timeout() {
                        warn!("Ollama connection failed: {}. Retrying in {:?}...", e, delay);
                        retries += 1;
                        sleep(delay).await;
                        delay *= 2; // 1s, 2s, 4s...
                    } else {
                        // Don't retry parsing or builder errors
                        return Err(JagError::Http(e));
                    }
                }
            }
        }
    }

    /// Apply a timeout to a future and map it to JagError
    async fn with_timeout<F, T>(&self, duration: Duration, future: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match tokio::time::timeout(duration, future).await {
            Ok(res) => res,
            Err(_) => Err(JagError::Timeout(duration)),
        }
    }

    pub async fn generate(&self, request: OllamaGenerateRequest) -> Result<OllamaGenerateResponse> {
        self.with_timeout(Duration::from_secs(120), async {
            let url = format!("{}/api/generate", self.base_url);
            
            let response = self.with_retries(|| async {
                self.client.post(&url)
                    .json(&request)
                    .send()
                    .await
            }).await?;

            response.json::<OllamaGenerateResponse>().await.map_err(JagError::Http)
        }).await
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        self.with_timeout(Duration::from_secs(10), async {
            let url = format!("{}/api/tags", self.base_url);
            
            let response = self.with_retries(|| async {
                self.client.get(&url)
                    .send()
                    .await
            }).await?;

            let parsed: ListModelsResponse = response.json().await.map_err(JagError::Http)?;
            Ok(parsed.models)
        }).await
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        self.with_timeout(Duration::from_secs(300), async {
            let url = format!("{}/api/pull", self.base_url);
            let body = serde_json::json!({ "name": model_name, "stream": false });
            
            self.with_retries(|| async {
                self.client.post(&url)
                    .json(&body)
                    .send()
                    .await
            }).await?;
            
            Ok(())
        }).await
    }

    pub async fn delete_model(&self, model_name: &str) -> Result<()> {
        self.with_timeout(Duration::from_secs(10), async {
            let url = format!("{}/api/delete", self.base_url);
            let body = serde_json::json!({ "name": model_name });
            
            self.with_retries(|| async {
                self.client.delete(&url)
                    .json(&body)
                    .send()
                    .await
            }).await?;
            
            Ok(())
        }).await
    }

    pub async fn health_check(&self) -> bool {
        let url = self.base_url.clone();
        let client = self.client.clone();
        
        let future = async {
            client.get(&url).send().await.is_ok()
        };
        
        match tokio::time::timeout(Duration::from_secs(10), future).await {
            Ok(success) => success,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Spawns a background listener that implements HTTP behavior based on the `handler`
    async fn spawn_mock_server<F, Fut>(handler: F) -> SocketAddr
    where
        F: Fn(tokio::net::TcpStream) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        let handler = Arc::new(handler);
        
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let handler = Arc::clone(&handler);
                tokio::spawn(async move {
                    handler(stream).await;
                });
            }
        });
        
        addr
    }

    #[tokio::test]
    async fn test_generate_formats_request_correctly() {
        let req_received = Arc::new(tokio::sync::Mutex::new(String::new()));
        let req_clone = Arc::clone(&req_received);
        
        let addr = spawn_mock_server(move |mut stream| {
            let req_clone = Arc::clone(&req_clone);
            async move {
                let mut buf = [0; 1024];
                if let Ok(n) = stream.read(&mut buf).await {
                    let request_str = String::from_utf8_lossy(&buf[..n]).to_string();
                    let mut req_lock = req_clone.lock().await;
                    *req_lock = request_str;
                    
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n\
                    {\n\
                        \"model\": \"qwen2.5-coder:14b\",\n\
                        \"response\": \"Test response\",\n\
                        \"done\": true\n\
                    }";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            }
        }).await;

        let client = OllamaClient::new(&format!("http://{}", addr));
        let req = OllamaGenerateRequest {
            model: "qwen2.5-coder:14b".into(),
            prompt: "Test prompt".into(),
            stream: false,
            options: None,
            system: None,
        };

        let res = client.generate(req).await.expect("Generate failed");
        
        // Assert properties in response parsed correctly
        assert_eq!(res.model, "qwen2.5-coder:14b");
        assert_eq!(res.response, "Test response");
        assert!(res.done);

        // Assert request was properly formatted
        let req_str = req_received.lock().await;
        assert!(req_str.contains("POST /api/generate HTTP/1.1"));
        assert!(req_str.contains("\"model\":\"qwen2.5-coder:14b\""));
        assert!(req_str.contains("\"prompt\":\"Test prompt\""));
    }

    #[tokio::test]
    async fn test_list_models_parses_response() {
        let addr = spawn_mock_server(|mut stream| async move {
            let mut buf = [0; 1024];
            let _ = stream.read(&mut buf).await;
            
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n\
            {\n\
                \"models\": [\n\
                    {\n\
                        \"name\": \"gemma2:9b\",\n\
                        \"size\": 12345,\n\
                        \"digest\": \"sha256:abc\",\n\
                        \"modified_at\": \"2024-01-01T00:00:00Z\"\n\
                    }\n\
                ]\n\
            }";
            let _ = stream.write_all(response.as_bytes()).await;
        }).await;

        let client = OllamaClient::new(&format!("http://{}", addr));
        let models = client.list_models().await.expect("List models failed");
        
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "gemma2:9b");
        assert_eq!(models[0].size, 12345);
    }

    #[tokio::test]
    async fn test_retry_triggers_on_connection_error() {
        // Reduced default timeout config helps but for simplicity 
        // connecting to an unbound port guarantees a fast connection refused error,
        // which makes `is_connect()` true.
        let client = OllamaClient::new("http://127.0.0.1:23456");
        
        // Measure time taken. Since exponential backoff is 1s, 2s, 4s, it should take ~7 seconds
        let start = std::time::Instant::now();
        let result = client.list_models().await;
        let elapsed = start.elapsed();
        
        assert!(result.is_err());
        assert!(elapsed.as_secs() >= 7, "Elapsed time {}s is less than expected backoff", elapsed.as_secs());
    }

    #[tokio::test]
    async fn test_timeout_returns_jag_error() {
        let addr = spawn_mock_server(|mut stream| async move {
            let mut buf = [0; 1024];
            let _ = stream.read(&mut buf).await;
            
            // Hang indefinitely instead of writing response
            tokio::time::sleep(Duration::from_secs(60)).await;
        }).await;

        let client = OllamaClient::new(&format!("http://{}", addr));
        
        // We override the default timeout manually for testing by calling health_check which is 10s.
        // Actually, let's wrap it ourselves to force a small timeout just for the test to pass quickly.
        // Wait, health_check is hardcoded to 10s. That will make the test take 10s.
        // Let's test using with_timeout with 1ms.
        let result = client.with_timeout(Duration::from_millis(50), async {
            client.list_models().await
        }).await;

        match result {
            Err(JagError::Timeout(d)) => {
                assert_eq!(d, Duration::from_millis(50));
            }
            res => panic!("Expected Timeout error, got {:?}", res),
        }
    }
}
