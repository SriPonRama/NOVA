use tauri::{AppHandle, State};
use super::service::FocusService;
use super::{FocusSession, TimerState};
use crate::AppMode;
use std::sync::Mutex;

#[tauri::command]
pub fn start_focus_session(
    task_id: String, 
    duration_seconds: i32, 
    state: State<'_, FocusService>,
    mode: State<'_, Mutex<AppMode>>
) -> Result<FocusSession, String> {
    let current_mode = mode.lock().unwrap().clone();
    if current_mode == AppMode::Assessment {
        return Err("Cannot start focus sessions in Assessment Mode".to_string());
    }
    state.start_focus_session(&task_id, duration_seconds)
}

#[tauri::command]
pub fn start_break(
    duration_seconds: i32, 
    state: State<'_, FocusService>,
    mode: State<'_, Mutex<AppMode>>
) -> Result<FocusSession, String> {
    let current_mode = mode.lock().unwrap().clone();
    if current_mode == AppMode::Assessment {
        return Err("Cannot start breaks in Assessment Mode".to_string());
    }
    state.start_break(duration_seconds)
}

#[tauri::command]
pub fn pause_focus_session(
    state: State<'_, FocusService>,
    mode: State<'_, Mutex<AppMode>>
) -> Result<(), String> {
    let current_mode = mode.lock().unwrap().clone();
    if current_mode == AppMode::Assessment {
        return Err("Cannot modify sessions in Assessment Mode".to_string());
    }
    state.pause_session()
}

#[tauri::command]
pub fn resume_focus_session(
    state: State<'_, FocusService>,
    mode: State<'_, Mutex<AppMode>>
) -> Result<(), String> {
    let current_mode = mode.lock().unwrap().clone();
    if current_mode == AppMode::Assessment {
        return Err("Cannot modify sessions in Assessment Mode".to_string());
    }
    state.resume_session()
}

#[tauri::command]
pub fn finish_focus_session(
    state: State<'_, FocusService>,
    mode: State<'_, Mutex<AppMode>>
) -> Result<(), String> {
    let current_mode = mode.lock().unwrap().clone();
    if current_mode == AppMode::Assessment {
        return Err("Cannot modify sessions in Assessment Mode".to_string());
    }
    state.finish_session()
}

#[tauri::command]
pub fn cancel_focus_session(
    state: State<'_, FocusService>,
    mode: State<'_, Mutex<AppMode>>
) -> Result<(), String> {
    let current_mode = mode.lock().unwrap().clone();
    if current_mode == AppMode::Assessment {
        return Err("Cannot modify sessions in Assessment Mode".to_string());
    }
    state.cancel_session()
}

#[tauri::command]
pub fn get_timer_state(app: AppHandle, state: State<'_, FocusService>) -> Result<TimerState, String> {
    state.get_timer_state(Some(app))
}
