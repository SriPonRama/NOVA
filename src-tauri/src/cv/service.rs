use serde::{Deserialize, Serialize};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Write};
use std::thread;
use tauri::{AppHandle, Emitter};
use super::state::{DrowsinessStateMachine, DrowsinessState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DrowsinessSignal {
    pub face_detected: bool,
    pub eye_measurement: f64,
    pub confidence: f64,
    pub timestamp: u64,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DrowsinessIntervention {
    pub reason: String,
    pub timestamp: u64,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CVSettings {
    pub enabled: bool,
}

pub struct CVService {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    pub settings: Arc<Mutex<CVSettings>>,
    latest_signal: Arc<Mutex<Option<DrowsinessSignal>>>,
    pub state_machine: Arc<Mutex<DrowsinessStateMachine>>,
}

impl CVService {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            settings: Arc::new(Mutex::new(CVSettings { enabled: false })),
            latest_signal: Arc::new(Mutex::new(None)),
            state_machine: Arc::new(Mutex::new(DrowsinessStateMachine::new())),
        }
    }

    pub fn is_running(&self) -> bool {
        self.process.lock().unwrap().is_some()
    }

    pub fn start(&self, app_handle: Option<AppHandle>) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();
        if process_guard.is_some() {
            return Ok(());
        }

        // Only start if enabled
        {
            let settings = self.settings.lock().unwrap();
            if !settings.enabled {
                return Err("CV is disabled by user".to_string());
            }
        }

        // Launch Python process
        let mut child = match Command::new("python")
            .arg("cv/src/main.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() 
        {
            Ok(c) => c,
            Err(e) => return Err(format!("Failed to start CV process: {}", e)),
        };

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        
        *self.stdin.lock().unwrap() = Some(stdin);

        // Send START command
        if let Some(mut stdin_ref) = self.stdin.lock().unwrap().as_mut() {
            let _ = writeln!(stdin_ref, "{{\"type\": \"START\"}}");
        }

        let latest_signal = self.latest_signal.clone();
        let sm = self.state_machine.clone();
        
        // Output reading thread
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line_str) = line {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line_str) {
                        if value.get("type").and_then(|t| t.as_str()) == Some("drowsiness_signal") {
                            if let Ok(signal) = serde_json::from_value::<DrowsinessSignal>(value) {
                                *latest_signal.lock().unwrap() = Some(signal.clone());
                                
                                let new_state = {
                                    let mut state_machine = sm.lock().unwrap();
                                    let prev_state = state_machine.state.clone();
                                    let current = state_machine.process_signal(&signal);
                                    
                                    if prev_state != current {
                                        if let Some(app) = &app_handle {
                                            let _ = app.emit("drowsiness_state_changed", current.clone());
                                            
                                            if current == DrowsinessState::Intervention {
                                                let intervention = DrowsinessIntervention {
                                                    reason: "sustained_eye_closure".to_string(),
                                                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                                                    severity: "medium".to_string(),
                                                };
                                                let _ = app.emit("drowsiness_intervention", intervention);
                                            }
                                        }
                                    }
                                    current
                                };
                            }
                        }
                    }
                }
            }
        });

        *process_guard = Some(child);
        Ok(())
    }

    pub fn stop(&self) {
        // Send STOP command gracefully
        if let Some(mut stdin) = self.stdin.lock().unwrap().take() {
            let _ = writeln!(stdin, "{{\"type\": \"STOP\"}}");
        }
        
        // Wait a bit, then kill
        if let Some(mut process) = self.process.lock().unwrap().take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        
        // Reset state
        self.state_machine.lock().unwrap().reset();
    }

    pub fn get_latest_signal(&self) -> Option<DrowsinessSignal> {
        self.latest_signal.lock().unwrap().clone()
    }

    pub fn set_enabled(&self, enabled: bool, app_handle: Option<AppHandle>) {
        self.settings.lock().unwrap().enabled = enabled;
        if !enabled {
            self.stop();
        } else if let Some(app) = app_handle {
            let _ = self.start(Some(app));
        }
    }

    pub fn get_settings(&self) -> CVSettings {
        let s = self.settings.lock().unwrap();
        CVSettings { enabled: s.enabled }
    }

    pub fn dismiss_intervention(&self) {
        let mut sm = self.state_machine.lock().unwrap();
        if sm.state == DrowsinessState::Intervention || sm.state == DrowsinessState::Drowsy {
            sm.set_cooldown();
        }
    }

    pub fn take_a_break(&self) {
        let mut sm = self.state_machine.lock().unwrap();
        sm.set_cooldown();
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cv_signal_parsing() {
        let json = r#"{"type": "drowsiness_signal", "face_detected": true, "eye_measurement": 0.25, "confidence": 0.9, "timestamp": 12345, "status": "FACE_DETECTED"}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let signal: DrowsinessSignal = serde_json::from_value(value).unwrap();
        assert!(signal.face_detected);
        assert_eq!(signal.eye_measurement, 0.25);
    }
    
    #[test]
    fn test_cv_requires_explicit_enable() {
        let service = CVService::new();
        // Disabled by default
        assert!(service.start(None).is_err());
        
        service.set_enabled(true, None);
        // It would fail to find python in CI potentially, so we just check it doesn't return the disabled error
        let res = service.start(None);
        if let Err(e) = res {
            assert!(e.starts_with("Failed to start CV process") || e == "Failed to get stdin");
        }
    }
}
