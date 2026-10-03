use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub messages: Vec<ConversationMessage>,
    pub system_instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub context: ConversationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub text: String,
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
