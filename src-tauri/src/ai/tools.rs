use super::ToolDefinition;
use serde_json::json;

pub fn get_ai_tools() -> Vec<ToolDefinition> {
    vec![
        // Memory Tools
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
        // Productivity Tools
        ToolDefinition {
            name: "create_task".to_string(),
            description: "Create a new task. Only do this if the user clearly requests to add a task. Priority must be Low, Medium, or High. Date must be YYYY-MM-DD.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "title": { "type": "STRING", "description": "The title of the task." },
                    "description": { "type": "STRING", "description": "Optional details about the task." },
                    "date": { "type": "STRING", "description": "Date in YYYY-MM-DD format." },
                    "estimated_minutes": { "type": "INTEGER", "description": "Estimated time in minutes." },
                    "priority": { "type": "STRING", "description": "Priority: Low, Medium, or High." }
                },
                "required": ["title", "date", "priority"]
            }),
        },
        ToolDefinition {
            name: "get_task".to_string(),
            description: "Retrieve a specific task by its unique ID.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "id": { "type": "STRING", "description": "The UUID of the task." }
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "list_tasks".to_string(),
            description: "Retrieve tasks for a specified date.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "date": { "type": "STRING", "description": "Date in YYYY-MM-DD format." }
                },
                "required": ["date"]
            }),
        },
        ToolDefinition {
            name: "get_today_tasks".to_string(),
            description: "Retrieve today's tasks. The system automatically uses the correct local date.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "update_task".to_string(),
            description: "Modify an existing task. Status must be Pending, InProgress, Completed, or Cancelled.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "id": { "type": "STRING", "description": "The UUID of the task." },
                    "title": { "type": "STRING", "description": "The title of the task." },
                    "description": { "type": "STRING", "description": "Optional details about the task." },
                    "date": { "type": "STRING", "description": "Date in YYYY-MM-DD format." },
                    "estimated_minutes": { "type": "INTEGER", "description": "Estimated time in minutes." },
                    "priority": { "type": "STRING", "description": "Priority: Low, Medium, or High." },
                    "status": { "type": "STRING", "description": "Status: Pending, InProgress, Completed, or Cancelled." }
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "delete_task".to_string(),
            description: "Request to delete a task. This requires user confirmation.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {
                    "id": { "type": "STRING", "description": "The UUID of the task to delete." }
                },
                "required": ["id"]
            }),
        },
        // Focus Tools
        ToolDefinition {
            name: "get_current_focus".to_string(),
            description: "Return the current focus-session state, including whether it is active, paused, or a break.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "get_timer_state".to_string(),
            description: "Return the authoritative timer state, including remaining seconds and active session info.".to_string(),
            parameters: json!({
                "type": "OBJECT",
                "properties": {}
            }),
        }
    ]
}
