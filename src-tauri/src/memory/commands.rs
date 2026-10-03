use super::{service::MemoryService, Memory, MemoryCategory};
use tauri::State;

#[tauri::command]
pub fn create_memory(
    content: String,
    category: MemoryCategory,
    service: State<'_, MemoryService>,
) -> Result<Memory, String> {
    service.create_memory(&content, category)
}

#[tauri::command]
pub fn get_memory(id: String, service: State<'_, MemoryService>) -> Result<Option<Memory>, String> {
    service.get_memory(&id)
}

#[tauri::command]
pub fn list_memories(service: State<'_, MemoryService>) -> Result<Vec<Memory>, String> {
    service.list_memories()
}

#[tauri::command]
pub fn search_memories(query: String, service: State<'_, MemoryService>) -> Result<Vec<Memory>, String> {
    service.search_memories(&query)
}

#[tauri::command]
pub fn update_memory(
    id: String,
    content: String,
    category: MemoryCategory,
    service: State<'_, MemoryService>,
) -> Result<(), String> {
    service.update_memory(&id, &content, category)
}

#[tauri::command]
pub fn delete_memory(id: String, service: State<'_, MemoryService>) -> Result<(), String> {
    service.delete_memory(&id)
}

#[tauri::command]
pub fn clear_all_memories(service: State<'_, MemoryService>) -> Result<(), String> {
    service.clear_all_memories()
}
