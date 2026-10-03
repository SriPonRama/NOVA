use crate::memory::service::MemoryService;
use crate::memory::MemoryCategory;
use crate::productivity::service::ProductivityService;
use crate::productivity::{TaskPriority, TaskStatus};
use crate::focus::service::FocusService;
use super::{ToolCall, ToolResult};
use serde_json::json;
use std::str::FromStr;

pub struct ToolExecutor;

impl ToolExecutor {
    pub fn execute(
        call: &ToolCall,
        memory_service: &MemoryService,
        productivity_service: &ProductivityService,
        focus_service: &FocusService,
    ) -> ToolResult {
        let result = match call.name.as_str() {
            "create_memory" => Self::execute_create_memory(&call.arguments, memory_service),
            "search_memory" => Self::execute_search_memory(&call.arguments, memory_service),
            "get_memory" => Self::execute_get_memory(&call.arguments, memory_service),
            "update_memory" => Self::execute_update_memory(&call.arguments, memory_service),
            "delete_memory" => Self::execute_delete_memory(&call.arguments),
            "create_task" => Self::execute_create_task(&call.arguments, productivity_service),
            "get_task" => Self::execute_get_task(&call.arguments, productivity_service),
            "list_tasks" => Self::execute_list_tasks(&call.arguments, productivity_service),
            "get_today_tasks" => Self::execute_get_today_tasks(productivity_service),
            "update_task" => Self::execute_update_task(&call.arguments, productivity_service),
            "delete_task" => Self::execute_delete_task(&call.arguments),
            "get_current_focus" => Self::execute_get_current_focus(focus_service),
            "get_timer_state" => Self::execute_get_timer_state(focus_service),
            "get_productivity_summary" => Self::execute_get_productivity_summary(productivity_service, focus_service),
            "get_remaining_workload" => Self::execute_get_remaining_workload(productivity_service),
            "get_next_recommended_task" => Self::execute_get_next_recommended_task(productivity_service),
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
            "requested_id": id,
            "type": "memory"
        })
    }

    fn execute_create_task(args: &serde_json::Value, productivity_service: &ProductivityService) -> serde_json::Value {
        let title = args["title"].as_str().unwrap_or_default();
        let date = args["date"].as_str().unwrap_or_default();
        let priority_str = args["priority"].as_str().unwrap_or_default();
        
        let priority = TaskPriority::from_str(priority_str).unwrap_or(TaskPriority::Medium);
        
        let description = args["description"].as_str().map(|s| s.to_string());
        let estimated_minutes = args["estimated_minutes"].as_i64().map(|v| v as i32);
        
        match productivity_service.create_task(title, description, date, estimated_minutes, priority) {
            Ok(task) => json!({ "status": "success", "task": task }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_get_task(args: &serde_json::Value, productivity_service: &ProductivityService) -> serde_json::Value {
        let id = args["id"].as_str().unwrap_or_default();
        
        match productivity_service.get_task(id) {
            Ok(Some(task)) => json!({ "status": "success", "task": task }),
            Ok(None) => json!({ "status": "error", "message": "Task not found" }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_list_tasks(args: &serde_json::Value, productivity_service: &ProductivityService) -> serde_json::Value {
        let date = args["date"].as_str().unwrap_or_default();
        
        match productivity_service.list_tasks(date) {
            Ok(tasks) => json!({ "status": "success", "tasks": tasks }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_get_today_tasks(productivity_service: &ProductivityService) -> serde_json::Value {
        // Find today\'s date string in YYYY-MM-DD
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        
        match productivity_service.list_tasks(&today) {
            Ok(tasks) => json!({ "status": "success", "date": today, "tasks": tasks }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_update_task(args: &serde_json::Value, productivity_service: &ProductivityService) -> serde_json::Value {
        let id = args["id"].as_str().unwrap_or_default();
        
        // Get existing task to avoid overwriting with defaults if not provided
        let existing_task = match productivity_service.get_task(id) {
            Ok(Some(t)) => t,
            _ => return json!({ "status": "error", "message": "Task not found" }),
        };

        let title = args["title"].as_str().unwrap_or(&existing_task.title);
        let date = args["date"].as_str().unwrap_or(&existing_task.date);
        
        let priority = if let Some(p) = args["priority"].as_str() {
            TaskPriority::from_str(p).unwrap_or(existing_task.priority)
        } else {
            existing_task.priority
        };

        let status = if let Some(s) = args["status"].as_str() {
            TaskStatus::from_str(s).unwrap_or(existing_task.status)
        } else {
            existing_task.status
        };

        let description = if args.get("description").is_some() {
            args["description"].as_str().map(|s| s.to_string())
        } else {
            existing_task.description
        };

        let estimated_minutes = if args.get("estimated_minutes").is_some() {
            args["estimated_minutes"].as_i64().map(|v| v as i32)
        } else {
            existing_task.estimated_minutes
        };
        
        match productivity_service.update_task(id, title, description, date, estimated_minutes, priority, status) {
            Ok(_) => match productivity_service.get_task(id) {
                Ok(Some(task)) => json!({ "status": "success", "task": task }),
                _ => json!({ "status": "success" }),
            },
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_delete_task(args: &serde_json::Value) -> serde_json::Value {
        let id = args["id"].as_str().unwrap_or_default();
        json!({
            "status": "pending_confirmation",
            "message": "Deletion requires user confirmation. The UI has been instructed to prompt the user. Do not attempt to delete again.",
            "requested_id": id,
            "type": "task"
        })
    }

    fn execute_get_current_focus(focus_service: &FocusService) -> serde_json::Value {
        match focus_service.get_timer_state(None) {
            Ok(state) => match state.active_session {
                Some(session) => json!({ "status": "success", "session": session }),
                None => json!({ "status": "success", "session": null, "message": "No active focus session." }),
            },
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_get_timer_state(focus_service: &FocusService) -> serde_json::Value {
        match focus_service.get_timer_state(None) {
            Ok(state) => json!({ "status": "success", "timer_state": state }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_get_productivity_summary(productivity_service: &ProductivityService, focus_service: &FocusService) -> serde_json::Value {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        
        let summary = match productivity_service.get_productivity_summary(&today) {
            Ok(s) => s,
            Err(e) => return json!({ "status": "error", "message": e }),
        };
        
        let timer_state = match focus_service.get_timer_state(None) {
            Ok(s) => json!(s),
            Err(_) => json!(null),
        };

        json!({
            "status": "success",
            "summary": summary,
            "timer_state": timer_state
        })
    }

    fn execute_get_remaining_workload(productivity_service: &ProductivityService) -> serde_json::Value {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        match productivity_service.get_remaining_workload(&today) {
            Ok(workload) => json!({ "status": "success", "date": today, "workload": workload }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }

    fn execute_get_next_recommended_task(productivity_service: &ProductivityService) -> serde_json::Value {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        match productivity_service.get_next_recommended_task(&today) {
            Ok(recommendation) => json!({ "status": "success", "date": today, "recommendation": recommendation }),
            Err(e) => json!({ "status": "error", "message": e }),
        }
    }
}
