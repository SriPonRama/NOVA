use super::{AIError, AIProvider, AIRequest, AIResponse, ConversationMessage};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::env;

pub struct GeminiProvider {
    client: Client,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn map_role(role: &str) -> &str {
        match role {
            "user" => "user",
            "assistant" => "model",
            _ => "user",
        }
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn generate_response(&self, request: AIRequest) -> Result<AIResponse, AIError> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| AIError::ConfigurationError("GEMINI_API_KEY not found in environment".to_string()))?;

        if api_key.is_empty() || api_key == "your_gemini_api_key_here" {
            return Err(AIError::ConfigurationError("GEMINI_API_KEY is not configured correctly".to_string()));
        }

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );

        let mut contents = Vec::new();
        for msg in request.context.messages {
            contents.push(json!({
                "role": Self::map_role(&msg.role),
                "parts": [{"text": msg.content}]
            }));
        }

        let mut body = json!({
            "contents": contents,
        });

        if let Some(sys_inst) = request.context.system_instruction {
            body["system_instruction"] = json!({
                "parts": [{"text": sys_inst}]
            });
        }

        let res = self.client.post(&url).json(&body).send().await.map_err(|e| {
            AIError::NetworkError(format!("Failed to send request: {}", e))
        })?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(AIError::ProviderError(format!(
                "Gemini API Error {}: {}",
                status, text
            )));
        }

        let json_res: serde_json::Value = res.json().await.map_err(|e| {
            AIError::ProviderError(format!("Failed to parse response JSON: {}", e))
        })?;

        // Extract the text from the response
        // Format: candidates[0].content.parts[0].text
        let text = json_res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| AIError::ProviderError("Malformed response: missing text".to_string()))?;

        if text.trim().is_empty() {
            return Err(AIError::ProviderError("Received empty response".to_string()));
        }

        Ok(AIResponse {
            text: text.to_string(),
        })
    }
}
