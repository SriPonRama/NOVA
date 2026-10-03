use super::{gemini::GeminiProvider, AIProvider, AIRequest, ConversationContext, ConversationMessage};
use std::sync::Mutex;
use tauri::State;

pub struct ConversationState {
    pub history: Mutex<Vec<ConversationMessage>>,
    pub provider: Box<dyn AIProvider>,
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            history: Mutex::new(Vec::new()),
            provider: Box::new(GeminiProvider::new()),
        }
    }
}

const NOVA_SYSTEM_INSTRUCTION: &str = "\
You are NOVA, a mature, calm, and friendly AI desktop companion. 
You are supportive, conversational, and concise when appropriate.
You help the user accomplish tasks.
You are currently in Phase 3 of development (Gemini integration).
Do NOT claim to have capabilities that are not yet implemented.
Available capabilities: Conversation.
Planned capabilities (DO NOT CLAIM THESE ARE ACTIVE YET): memory, drowsy detection, active-window monitoring, AI news, games.
Be helpful, professional, and clear.";

#[tauri::command]
pub async fn send_message(
    message: String,
    state: State<'_, ConversationState>,
) -> Result<String, String> {
    
    // 1. Update context
    let mut history = state.history.lock().unwrap().clone();
    
    let user_msg = ConversationMessage {
        role: "user".to_string(),
        content: message.clone(),
    };
    history.push(user_msg.clone());

    let context = ConversationContext {
        messages: history.clone(),
        system_instruction: Some(NOVA_SYSTEM_INSTRUCTION.to_string()),
    };

    let request = AIRequest { context };

    // 2. Call provider
    let response = state.provider.generate_response(request).await.map_err(|e| e.to_string())?;

    // 3. Save assistant response
    let assistant_msg = ConversationMessage {
        role: "assistant".to_string(),
        content: response.text.clone(),
    };
    
    let mut actual_history = state.history.lock().unwrap();
    actual_history.push(user_msg);
    actual_history.push(assistant_msg);

    Ok(response.text)
}

#[tauri::command]
pub async fn clear_conversation(state: State<'_, ConversationState>) -> Result<(), String> {
    let mut history = state.history.lock().unwrap();
    history.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{AIError, AIResponse};
    use async_trait::async_trait;

    struct MockProvider {
        succeed: bool,
    }

    #[async_trait]
    impl AIProvider for MockProvider {
        async fn generate_response(&self, request: AIRequest) -> Result<AIResponse, AIError> {
            if !self.succeed {
                return Err(AIError::ProviderError("Mock failure".to_string()));
            }
            if request.context.messages.is_empty() {
                return Err(AIError::ProviderError("Empty messages".to_string()));
            }
            Ok(AIResponse {
                text: "Mock response".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn test_provider_abstraction_success() {
        let provider = MockProvider { succeed: true };
        let req = AIRequest {
            context: ConversationContext {
                messages: vec![ConversationMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                }],
                system_instruction: None,
            }
        };
        let res = provider.generate_response(req).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().text, "Mock response");
    }

    #[tokio::test]
    async fn test_provider_abstraction_failure() {
        let provider = MockProvider { succeed: false };
        let req = AIRequest {
            context: ConversationContext {
                messages: vec![ConversationMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                }],
                system_instruction: None,
            }
        };
        let res = provider.generate_response(req).await;
        assert!(res.is_err());
        if let Err(AIError::ProviderError(msg)) = res {
            assert_eq!(msg, "Mock failure");
        } else {
            panic!("Wrong error type");
        }
    }
}
