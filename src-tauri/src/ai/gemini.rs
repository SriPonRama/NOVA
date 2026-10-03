use super::{AIError, AIProvider, AIRequest, AIResponse, ConversationMessage, ToolCall};
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
            "tool" => "user", // Function responses are sent from the user role in Gemini
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
            let mut parts = Vec::new();
            
            if !msg.content.is_empty() {
                parts.push(json!({"text": msg.content}));
            }
            
            if let Some(tool_calls) = msg.tool_calls {
                for tc in tool_calls {
                    parts.push(json!({
                        "functionCall": {
                            "name": tc.name,
                            "args": tc.arguments
                        }
                    }));
                }
            }
            
            if let Some(tool_results) = msg.tool_results {
                for tr in tool_results {
                    parts.push(json!({
                        "functionResponse": {
                            "name": tr.name,
                            "response": tr.result
                        }
                    }));
                }
            }
            
            contents.push(json!({
                "role": Self::map_role(&msg.role),
                "parts": parts
            }));
        }

        let mut body = json!({
            "contents": contents,
        });

        if let Some(tools) = request.tools {
            if !tools.is_empty() {
                let declarations: Vec<_> = tools.into_iter().map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters
                    })
                }).collect();
                
                body["tools"] = json!([{
                    "functionDeclarations": declarations
                }]);
            }
        }

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

        let candidates = json_res["candidates"].as_array()
            .ok_or_else(|| AIError::ProviderError("Malformed response: missing candidates".to_string()))?;
            
        if candidates.is_empty() {
            return Err(AIError::ProviderError("No candidates returned".to_string()));
        }
        
        let parts = candidates[0]["content"]["parts"].as_array()
            .ok_or_else(|| AIError::ProviderError("Malformed response: missing parts".to_string()))?;

        let mut text = None;
        let mut tool_calls = None;

        for part in parts {
            if let Some(t) = part["text"].as_str() {
                if !t.trim().is_empty() {
                    let mut current_text = text.unwrap_or_else(String::new);
                    current_text.push_str(t);
                    text = Some(current_text);
                }
            }
            
            if let Some(fc) = part.get("functionCall") {
                let name = fc["name"].as_str().unwrap_or_default().to_string();
                let args = fc["args"].clone();
                // Gemini doesn't provide a unique call ID natively in the same way OpenAI does,
                // so we generate a random one or just use a dummy one since we only execute serially.
                let tc = ToolCall {
                    id: uuid::Uuid::new_v4().to_string(),
                    name,
                    arguments: args,
                };
                
                let mut current_calls = tool_calls.unwrap_or_else(Vec::new);
                current_calls.push(tc);
                tool_calls = Some(current_calls);
            }
        }

        Ok(AIResponse {
            text,
            tool_calls,
        })
    }
}
