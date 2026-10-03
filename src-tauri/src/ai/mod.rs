use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String, // "user", "assistant", or "tool"
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub name: String,
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub messages: Vec<ConversationMessage>,
    pub system_instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub context: ConversationContext,
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub text: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIError {
    ProviderError(String),
    NetworkError(String),
    ConfigurationError(String),
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::ProviderError(e) => write!(f, "Provider Error: {}", e),
            AIError::NetworkError(e) => write!(f, "Network Error: {}", e),
            AIError::ConfigurationError(e) => write!(f, "Configuration Error: {}", e),
        }
    }
}

// Ensure AIError can be converted to a string when returning via Tauri IPC.
// Normally Tauri errors need to implement Serialize or we manually map to String.
// We already derive Serialize.

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_response(&self, request: AIRequest) -> Result<AIResponse, AIError>;
}

pub mod gemini;
pub mod conversation;
pub mod tools;
pub mod executor;
