use super::ToolDefinition;
use serde_json::json;

pub fn get_memory_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "create_memory".to_string(),
            description: "Store a new memory. Use this only when the user explicitly asks to remember something or when it's clearly intentional persistent memory. Supported categories: USER_PREFERENCE, GOAL, STUDY, PROJECT, IMPORTANT, TEMPORARY.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "content": { "type": "STRING", "description": "The memory content to store." },
                    "category": { "type": "STRING", "description": "The category of the memory (e.g., USER_PREFERENCE, PROJECT)." }
                },
                "required": ["content", "category"]
            }),
        },
        ToolDefinition {
            name: "search_memory".to_string(),
            description: "Search local memories by text query.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "query": { "type": "STRING", "description": "The search query." }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "get_memory".to_string(),
            description: "Retrieve a specific memory by its unique ID.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "id": { "type": "STRING", "description": "The UUID of the memory." }
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "update_memory".to_string(),
            description: "Update the content and category of an existing memory.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "id": { "type": "STRING", "description": "The UUID of the memory." },
                    "content": { "type": "STRING", "description": "The new content." },
                    "category": { "type": "STRING", "description": "The new category." }
                },
                "required": ["id", "content", "category"]
            }),
        },
        ToolDefinition {
            name: "delete_memory".to_string(),
            description: "Request to delete a memory. This will NOT delete immediately but will trigger a user confirmation prompt in the UI. Return a conversational message indicating you asked for confirmation.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "id": { "type": "STRING", "description": "The UUID of the memory to delete." }
                },
                "required": ["id"]
            }),
        },
    ]
}
