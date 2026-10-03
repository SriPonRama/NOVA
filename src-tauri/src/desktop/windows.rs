use super::activity::{DesktopActivityProvider, ForegroundActivity};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::Win32::Foundation::{HANDLE, MAX_PATH};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW, PROCESS_NAME_WIN32};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub struct WindowsDesktopActivityProvider;

impl WindowsDesktopActivityProvider {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopActivityProvider for WindowsDesktopActivityProvider {
    fn get_current_activity(&self) -> Result<ForegroundActivity, String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == std::ptr::null_mut() {
                return Err("No foreground window".to_string());
            }

            let mut process_id = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            if process_id == 0 {
                return Err("Failed to get process ID".to_string());
            }

            let process_handle = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION,
                false,
                process_id,
            );

            let mut process_name = String::new();
            if let Ok(handle) = process_handle {
                let mut buffer = [0u16; MAX_PATH as usize];
                let mut size = MAX_PATH;
                
                if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(buffer.as_mut_ptr()), &mut size).is_ok() {
                    let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
                    if let Some(name) = std::path::Path::new(&full_path).file_name() {
                        process_name = name.to_string_lossy().into_owned();
                    }
                }
                let _ = windows::Win32::Foundation::CloseHandle(handle);
            }

            if process_name.is_empty() {
                return Err("Failed to get process name".to_string());
            }

            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            Ok(ForegroundActivity {
                process_name: process_name.clone(),
                application_name: process_name, // Can be improved later if we query FileDescription
                window_title_optional: None, // Purposely not capturing for privacy
                timestamp: current_time,
            })
        }
    }
}
