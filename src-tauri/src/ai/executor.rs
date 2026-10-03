use crate::memory::service::MemoryService;
use crate::memory::MemoryCategory;
use super::{ToolCall, ToolResult};
use serde_json::json;

pub struct ToolExecutor;

impl ToolExecutor {
    pub fn execute(call: &ToolCall, memory_service: &MemoryService) -> ToolResult {
        let result = match call.name.as_str() {
            "create_memory" => Self::execute_create_memory(&call.arguments, memory_service),
            "search_memory" => Self::execute_search_memory(&call.arguments, memory_service),
            "get_memory" => Self::execute_get_memory(&call.arguments, memory_service),
            "update_memory" => Self::execute_update_memory(&call.arguments, memory_service),
            "delete_memory" => Self::execute_delete_memory(&call.arguments),
            _ => json!({ "error": format!("UNKNOWN_TOOL: {}", call.name) }),
        };

        ToolResult {
            id: call.id.clone(),
            name: call.name.clone(),
            result,
        }
    }

    fn execute_create_memory(args: &serde_json::Value, memory_service: &MemoryService) -> serde_json::Value {
        let content = args["content"].as_str().unwrap_or_default();
        let category_str = args["category"].as_str().unwrap_or_default();
        
        let category = MemoryCategory::from_str(category_str).unwrap_or(MemoryCategory::Temporary);
        
        match memory_service.create_memory(content, category) {
            Ok(mem) => json!({ "status": "success", "memory": mem }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_search_memory(args: &serde_json::Value, memory_service: &MemoryService) -> serde_json::Value {
        let query = args["query"].as_str().unwrap_or_default();
        if query.is_empty() {
            return json!({ "status": "error", "message": "Query cannot be empty" });
        }
        
        match memory_service.search_memories(query) {
            Ok(mut memories) => {
                // Limit to 10 max
                memories.truncate(10);
                json!({ "status": "success", "memories": memories })
            },
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_get_memory(args: &serde_json::Value, memory_service: &MemoryService) -> serde_json::Value {
        let id = args["id"].as_str().unwrap_or_default();
        
        match memory_service.get_memory(id) {
            Ok(Some(mem)) => json!({ "status": "success", "memory": mem }),
            Ok(None) => json!({ "status": "error", "message": "Memory not found" }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_update_memory(args: &serde_json::Value, memory_service: &MemoryService) -> serde_json::Value {
        let id = args["id"].as_str().unwrap_or_default();
        let content = args["content"].as_str().unwrap_or_default();
        let category_str = args["category"].as_str().unwrap_or_default();
        
        let category = MemoryCategory::from_str(category_str).unwrap_or(MemoryCategory::Temporary);
        
        match memory_service.update_memory(id, content, category) {
            Ok(_) => match memory_service.get_memory(id) {
                Ok(Some(mem)) => json!({ "status": "success", "memory": mem }),
                _ => json!({ "status": "success" }),
            },
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_delete_memory(args: &serde_json::Value) -> serde_json::Value {
        let id = args["id"].as_str().unwrap_or_default();
        // Return a pending UI state so the frontend knows it must confirm.
        // The AI receives this result and understands it was queued.
        json!({
            "status": "pending_confirmation",
            "message": "Deletion requires user confirmation. The UI has been instructed to prompt the user. Do not attempt to delete again.",
            "requested_id": id
        })
    }
}
