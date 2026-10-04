use tauri::State;
use std::sync::Arc;
use super::service::{CVService, CVSettings, DrowsinessSignal};

#[tauri::command]
pub fn toggle_cv_monitoring(state: State<'_, Arc<CVService>>, enabled: bool) -> Result<(), String> {
    state.set_enabled(enabled);
    
    // Attempt to start if enabled
    if enabled {
        if let Err(e) = state.start() {
            // Note: we might fail if python isn't available, but we still set it as enabled in settings
            // The frontend will see the error.
            return Err(e);
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_cv_settings(state: State<'_, Arc<CVService>>) -> Result<CVSettings, String> {
    Ok(state.get_settings())
}

#[tauri::command]
pub fn get_latest_cv_signal(state: State<'_, Arc<CVService>>) -> Result<Option<DrowsinessSignal>, String> {
    Ok(state.get_latest_signal())
}
