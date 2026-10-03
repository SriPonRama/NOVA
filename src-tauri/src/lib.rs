mod ai;
mod memory;
mod db;
mod productivity;
mod focus;
mod desktop;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use dotenvy::dotenv;

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AppMode {
    Active,
    Focus,
    Break,
    Assessment,
    Paused,
    Disabled,
}

pub struct NovaState {
    pub mode: Mutex<AppMode>,
}

#[tauri::command]
fn set_app_mode(mode: AppMode, state: State<'_, NovaState>, app: tauri::AppHandle) {
    let mut current_mode = state.mode.lock().unwrap();
    *current_mode = mode.clone();

    // Side-effects based on mode
    match mode {
        AppMode::Assessment => {
            // Hide the UI completely during Assessment Mode
            if let Some(window) = app.get_webview_window("main") {
                window.hide().unwrap();
            }
        }
        _ => {}
    }
}

#[tauri::command]
fn get_app_mode(state: State<'_, NovaState>) -> AppMode {
    state.mode.lock().unwrap().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_notification::init())
        .manage(NovaState {
            mode: Mutex::new(AppMode::Active),
        })
        .manage(ai::conversation::ConversationState::new())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).unwrap();
            let db_path = app_dir.join("nova.db");
            
            let db_conn = db::init_db(db_path).expect("Failed to init DB");
            
            let memory_repo = memory::repository::MemoryRepository::new(db_conn.clone());
            let memory_service = memory::service::MemoryService::new(memory_repo);
            app.manage(memory_service);
            
            let productivity_repo = productivity::repository::ProductivityRepository::new(db_conn.clone());
            let productivity_service = productivity::service::ProductivityService::new(productivity_repo);
            app.manage(productivity_service.clone());
            
            let focus_repo = focus::repository::FocusRepository::new(db_conn.clone());
            let focus_service = focus::service::FocusService::new(focus_repo, productivity_service);
            app.manage(focus_service);
            
            let desktop_service = desktop::service::DesktopAwarenessService::new(db_conn.clone());
            app.manage(desktop_service);
            desktop::service::DesktopAwarenessService::start_polling(app.handle().clone());

            let show_i = MenuItem::with_id(app, "show", "Show NOVA", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide NOVA", true, None::<&str>)?;
            let today_i = MenuItem::with_id(app, "today", "Open Today", true, None::<&str>)?;
            let focus_i = MenuItem::with_id(app, "focus", "Start Focus", true, None::<&str>)?;
            let break_i = MenuItem::with_id(app, "break", "Start Break", true, None::<&str>)?;
            let assessment_i = MenuItem::with_id(app, "assessment", "Assessment Mode", true, None::<&str>)?;
            let disable_i = MenuItem::with_id(app, "disable", "Pause/Disable NOVA", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit NOVA", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &hide_i, &today_i, &focus_i, &break_i, &assessment_i, &disable_i, &quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.hide().unwrap();
                        }
                    }
                    "today" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                            let _ = window.emit("navigate", "Today");
                        }
                    }
                    "focus" => {
                        let focus_service = app.state::<focus::service::FocusService>();
                        let _ = focus_service.start_focus_session(None, 25 * 60);
                    }
                    "break" => {
                        let focus_service = app.state::<focus::service::FocusService>();
                        let _ = focus_service.start_break(5 * 60);
                    }
                    "assessment" => {
                        let state = app.state::<NovaState>();
                        let mut current_mode = state.mode.lock().unwrap();
                        *current_mode = AppMode::Assessment;
                        if let Some(window) = app.get_webview_window("main") {
                            window.hide().unwrap();
                        }
                    }
                    "disable" => {
                        let state = app.state::<NovaState>();
                        let mut current_mode = state.mode.lock().unwrap();
                        if *current_mode == AppMode::Disabled {
                            *current_mode = AppMode::Active;
                        } else if *current_mode != AppMode::Assessment {
                            *current_mode = AppMode::Disabled;
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let state: State<NovaState> = app.state();
                            let mode = state.mode.lock().unwrap().clone();
                            
                            // Don't show window if in Assessment Mode
                            if let AppMode::Assessment = mode {
                                return;
                            }

                            if window.is_visible().unwrap_or(false) {
                                window.hide().unwrap();
                            } else {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            set_app_mode, 
            get_app_mode,
            ai::conversation::send_message,
            ai::conversation::clear_conversation,
            memory::commands::create_memory,
            memory::commands::get_memory,
            memory::commands::list_memories,
            memory::commands::search_memories,
            memory::commands::update_memory,
            memory::commands::delete_memory,
            memory::commands::clear_all_memories,
            productivity::commands::create_task,
            productivity::commands::get_task,
            productivity::commands::list_tasks,
            productivity::commands::update_task,
            productivity::commands::set_task_status,
            productivity::commands::delete_task,
            productivity::commands::reorder_tasks,
            productivity::commands::get_remaining_workload,
            productivity::commands::get_next_recommended_task,
            focus::commands::start_focus_session,
            focus::commands::start_break,
            focus::commands::pause_focus_session,
            focus::commands::resume_focus_session,
            focus::commands::finish_focus_session,
            focus::commands::cancel_focus_session,
            focus::commands::get_timer_state,
            desktop::commands::get_desktop_awareness_settings,
            desktop::commands::set_desktop_awareness_enabled,
            desktop::commands::set_grace_period,
            desktop::commands::set_cooldown,
            desktop::commands::get_distraction_state,
            desktop::commands::return_to_focus,
            desktop::commands::keep_working_here,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
