use super::{service::ProductivityService, Task, TaskPriority, TaskStatus};
use tauri::State;

#[tauri::command]
pub fn create_task(
    title: String,
    description: Option<String>,
    date: String,
    estimated_minutes: Option<i32>,
    priority: String,
    service: State<'_, ProductivityService>,
) -> Result<Task, String> {
    let p = TaskPriority::from_str(&priority).unwrap_or(TaskPriority::Medium);
    service.create_task(&title, description, &date, estimated_minutes, p)
}

#[tauri::command]
pub fn get_task(id: String, service: State<'_, ProductivityService>) -> Result<Option<Task>, String> {
    service.get_task(&id)
}

#[tauri::command]
pub fn list_tasks(date: String, service: State<'_, ProductivityService>) -> Result<Vec<Task>, String> {
    service.list_tasks(&date)
}

#[tauri::command]
pub fn update_task(
    id: String,
    title: String,
    description: Option<String>,
    date: String,
    estimated_minutes: Option<i32>,
    priority: String,
    status: String,
    service: State<'_, ProductivityService>,
) -> Result<(), String> {
    let p = TaskPriority::from_str(&priority).unwrap_or(TaskPriority::Medium);
    let s = TaskStatus::from_str(&status).unwrap_or(TaskStatus::Pending);
    service.update_task(&id, &title, description, &date, estimated_minutes, p, s)
}

#[tauri::command]
pub fn set_task_status(
    id: String,
    status: String,
    service: State<'_, ProductivityService>,
) -> Result<(), String> {
    let s = TaskStatus::from_str(&status).unwrap_or(TaskStatus::Pending);
    service.set_task_status(&id, s)
}

#[tauri::command]
pub fn delete_task(id: String, service: State<'_, ProductivityService>) -> Result<(), String> {
    service.delete_task(&id)
}

#[tauri::command]
pub fn reorder_tasks(
    date: String,
    ordered_ids: Vec<String>,
    service: State<'_, ProductivityService>,
) -> Result<(), String> {
    service.reorder_tasks(&date, ordered_ids)
}

#[tauri::command]
pub fn get_remaining_workload(
    date: String,
    service: State<'_, ProductivityService>,
) -> Result<super::RemainingWorkload, String> {
    service.get_remaining_workload(&date)
}

#[tauri::command]
pub fn get_next_recommended_task(
    date: String,
    service: State<'_, ProductivityService>,
) -> Result<super::TaskRecommendation, String> {
    service.get_next_recommended_task(&date)
}
