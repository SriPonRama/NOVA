mod ai;
mod memory;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};
use dotenvy::dotenv;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum AppMode {
    Active,
    Focus,
    Break,
    Assessment,
    Paused,
}

struct NovaState {
    mode: Mutex<AppMode>,
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
        .manage(NovaState {
            mode: Mutex::new(AppMode::Active),
        })
        .manage(ai::conversation::ConversationState::new())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).unwrap();
            let db_path = app_dir.join("nova.db");
            
            let repo = memory::repository::MemoryRepository::new(db_path).expect("Failed to init DB");
            let memory_service = memory::service::MemoryService::new(repo);
            app.manage(memory_service);

            let show_i = MenuItem::with_id(app, "show", "Show NOVA", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide NOVA", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit NOVA", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
