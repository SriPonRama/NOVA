use super::{gemini::GeminiProvider, AIProvider, AIRequest, ConversationContext, ConversationMessage, ToolResult};
use crate::ai::tools::get_ai_tools;
use crate::ai::executor::ToolExecutor;
use std::sync::Mutex;
use tauri::State;
use serde_json::json;

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
You are currently in Phase 8 of development, where you have access to local memory, productivity tools, and AI productivity intelligence.
When asked to remember something, explicitly use the `create_memory` tool.
When asked to recall, use `search_memory` or `get_memory`.
If asked to forget, use `delete_memory`.
Only store information deliberately when the user intends for it to be persistent. Do not store every message.
You can also manage tasks:
- `create_task` to add a new task (only when clearly requested).
- `get_task`, `list_tasks`, `get_today_tasks` to view tasks.
- `update_task` to modify tasks.
- `delete_task` to delete tasks (requires confirmation).
- `get_current_focus` and `get_timer_state` to check focus sessions.
Productivity Intelligence & Planning:
- Use `get_productivity_summary`, `get_remaining_workload`, and `get_next_recommended_task` to understand the user's workload, plan their day, or recommend the next task.
- NEVER hallucinate statistics, remaining workload, estimated duration, or priority. Always use the data provided by the tools.
- Do not automatically create memories or modify tasks during planning unless the user explicitly requests it.
If a task name is ambiguous, ask the user to clarify before updating.
Do NOT claim to have capabilities that are not yet implemented.
Available capabilities: Conversation, Persistent Local Memory, Task Management, Focus Sessions, AI Productivity Planning.
Planned capabilities (DO NOT CLAIM THESE ARE ACTIVE YET): drowsy detection, active-window monitoring, AI news, games.
Be helpful, professional, and clear.";

#[tauri::command]
pub async fn send_message(
    message: String,
    state: State<'_, ConversationState>,
    nova_state: State<'_, crate::NovaState>,
    memory_service: State<'_, crate::memory::service::MemoryService>,
    productivity_service: State<'_, crate::productivity::service::ProductivityService>,
    focus_service: State<'_, crate::focus::service::FocusService>,
) -> Result<serde_json::Value, String> {
    
    if let crate::AppMode::Assessment = *nova_state.mode.lock().unwrap() {
        return Err("AI and memory operations are blocked during Assessment Mode.".to_string());
    }

    let mut history = state.history.lock().unwrap().clone();
    
    let user_msg = ConversationMessage {
        role: "user".to_string(),
        content: message.clone(),
        tool_calls: None,
        tool_results: None,
    };
    history.push(user_msg.clone());

    let mut pending_deletion_id: Option<String> = None;
    let mut pending_deletion_type: Option<String> = None;
    let mut round_count = 0;
    const MAX_ROUNDS: usize = 5;
    let mut final_text = String::new();

    loop {
        if round_count >= MAX_ROUNDS {
            final_text.push_str("\n[System: Maximum tool execution depth reached.]");
            break;
        }
        round_count += 1;

        let context = ConversationContext {
            messages: history.clone(),
            system_instruction: Some(NOVA_SYSTEM_INSTRUCTION.to_string()),
        };

        let request = AIRequest { 
            context,
            tools: Some(get_ai_tools()),
        };

        let response = state.provider.generate_response(request).await.map_err(|e| e.to_string())?;

        let assistant_msg = ConversationMessage {
            role: "assistant".to_string(),
            content: response.text.clone().unwrap_or_default(),
            tool_calls: response.tool_calls.clone(),
            tool_results: None,
        };
        history.push(assistant_msg.clone());
        
        if let Some(text) = response.text {
            if !text.is_empty() {
                final_text.push_str(&text);
                final_text.push('\n');
            }
        }

        if let Some(tool_calls) = response.tool_calls {
            let mut results = Vec::new();
            for call in tool_calls {
                let res = ToolExecutor::execute(&call, &memory_service, &productivity_service, &focus_service);
                
                if call.name == "delete_memory" || call.name == "delete_task" {
                    if let Some(id) = res.result.get("requested_id").and_then(|id| id.as_str()) {
                        pending_deletion_id = Some(id.to_string());
                        pending_deletion_type = Some(if call.name == "delete_memory" { "memory".to_string() } else { "task".to_string() });
                    }
                }
                results.push(res);
            }
            
            let tool_msg = ConversationMessage {
                role: "tool".to_string(),
                content: "".to_string(),
                tool_calls: None,
                tool_results: Some(results),
            };
            history.push(tool_msg);
        } else {
            break; // No more tool calls
        }
    }

    *state.history.lock().unwrap() = history;

    Ok(json!({
        "text": final_text.trim(),
        "pending_deletion": pending_deletion_id,
        "pending_deletion_type": pending_deletion_type,
    }))
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
                text: Some("Mock response".to_string()),
                tool_calls: None,
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
                    tool_calls: None,
                    tool_results: None,
                }],
                system_instruction: None,
            },
            tools: None,
        };
        let res = provider.generate_response(req).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().text.unwrap(), "Mock response");
    }

    #[tokio::test]
    async fn test_provider_abstraction_failure() {
        let provider = MockProvider { succeed: false };
        let req = AIRequest {
            context: ConversationContext {
                messages: vec![ConversationMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                    tool_calls: None,
                    tool_results: None,
                }],
                system_instruction: None,
            },
            tools: None,
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
