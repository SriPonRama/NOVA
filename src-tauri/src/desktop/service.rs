use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, Emitter};

use super::activity::{DesktopActivityProvider, ForegroundActivity, ActivityCategory};
use super::classifier::ActivityClassifier;
use super::distraction::{DistractionEngine, DistractionState};
use super::settings::{DesktopSettings, DesktopSettingsRepository};
use super::windows::WindowsDesktopActivityProvider;
use crate::AppMode;

pub struct DesktopAwarenessService {
    provider: Box<dyn DesktopActivityProvider>,
    classifier: ActivityClassifier,
    engine: Arc<Mutex<DistractionEngine>>,
    settings_repo: DesktopSettingsRepository,
}

impl DesktopAwarenessService {
    pub fn new(conn: crate::db::DbConnection) -> Self {
        Self {
            provider: Box::new(WindowsDesktopActivityProvider::new()),
            classifier: ActivityClassifier::new(conn.clone()),
            engine: Arc::new(Mutex::new(DistractionEngine::new())),
            settings_repo: DesktopSettingsRepository::new(conn),
        }
    }

    pub fn get_settings(&self) -> Result<DesktopSettings, String> {
        self.settings_repo.get_settings()
    }

    pub fn save_settings(&self, settings: &DesktopSettings) -> Result<(), String> {
        self.settings_repo.save_settings(settings)
    }

    pub fn get_distraction_state(&self) -> DistractionState {
        self.engine.lock().unwrap().current_state.clone()
    }

    pub fn return_to_focus(&self) {
        self.engine.lock().unwrap().return_to_focus();
    }

    pub fn keep_working_here(&self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.engine.lock().unwrap().keep_working_here(current_time);
    }

    pub fn poll_activity(&self, app_handle: &AppHandle) {
        // Enforce Assessment Mode
        let app_mode = {
            if let Some(state) = app_handle.try_state::<tauri::State<Mutex<AppMode>>>() {
                state.lock().unwrap().clone()
            } else {
                AppMode::Active
            }
        };

        if app_mode == AppMode::Assessment || app_mode == AppMode::Disabled {
            self.engine.lock().unwrap().reset();
            return;
        }

        // Check Focus Session
        let is_focus_active = {
            if let Some(focus_service) = app_handle.try_state::<tauri::State<crate::focus::service::FocusService>>() {
                if let Ok(state) = focus_service.get_timer_state(Some(app_handle.clone())) {
                    if let Some(session) = state.active_session {
                        session.status == crate::focus::SessionStatus::Running && session.session_type == crate::focus::SessionType::Focus
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        let settings = self.get_settings().unwrap_or_default();

        if !settings.enabled || !is_focus_active {
            self.engine.lock().unwrap().reset();
            return;
        }

        // Get Current Activity
        let current_activity = match self.provider.get_current_activity() {
            Ok(act) => act,
            Err(_) => return, // Failed to get activity (e.g. locked screen), just return
        };

        let category = self.classifier.classify(&current_activity.process_name).unwrap_or(ActivityCategory::Unknown);
        
        let mut engine = self.engine.lock().unwrap();
        let new_state = engine.update(
            is_focus_active,
            app_mode == AppMode::Assessment,
            settings.enabled,
            &category,
            current_activity.timestamp,
            settings.grace_period_seconds,
            settings.cooldown_minutes,
        );

        if new_state == DistractionState::DistractionConfirmed {
            engine.set_intervention_shown();
            // Trigger Intervention UI
            let _ = app_handle.emit("intervention_required", ());
        }
    }

    pub fn start_polling(app: AppHandle) {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(2)); // Poll every 2 seconds
                
                if let Some(service) = app.try_state::<tauri::State<DesktopAwarenessService>>() {
                    service.poll_activity(&app);
                }
            }
        });
    }
}
