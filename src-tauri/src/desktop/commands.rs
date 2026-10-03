use tauri::State;
use super::settings::DesktopSettings;
use super::service::DesktopAwarenessService;
use super::distraction::DistractionState;

#[tauri::command]
pub fn get_desktop_awareness_settings(
    service: State<'_, DesktopAwarenessService>,
) -> Result<DesktopSettings, String> {
    service.get_settings()
}

#[tauri::command]
pub fn set_desktop_awareness_enabled(
    enabled: bool,
    service: State<'_, DesktopAwarenessService>,
) -> Result<(), String> {
    let mut settings = service.get_settings()?;
    settings.enabled = enabled;
    service.save_settings(&settings)
}

#[tauri::command]
pub fn set_grace_period(
    seconds: u32,
    service: State<'_, DesktopAwarenessService>,
) -> Result<(), String> {
    let mut settings = service.get_settings()?;
    settings.grace_period_seconds = seconds;
    service.save_settings(&settings)
}

#[tauri::command]
pub fn set_cooldown(
    minutes: u32,
    service: State<'_, DesktopAwarenessService>,
) -> Result<(), String> {
    let mut settings = service.get_settings()?;
    settings.cooldown_minutes = minutes;
    service.save_settings(&settings)
}

#[tauri::command]
pub fn get_distraction_state(
    service: State<'_, DesktopAwarenessService>,
) -> Result<DistractionState, String> {
    Ok(service.get_distraction_state())
}

#[tauri::command]
pub fn return_to_focus(
    service: State<'_, DesktopAwarenessService>,
) -> Result<(), String> {
    service.return_to_focus();
    Ok(())
}

#[tauri::command]
pub fn keep_working_here(
    service: State<'_, DesktopAwarenessService>,
) -> Result<(), String> {
    service.keep_working_here();
    Ok(())
}
