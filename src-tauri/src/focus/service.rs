use super::{repository::FocusRepository, FocusSession, SessionStatus, SessionType, TimerState};
use crate::productivity::service::ProductivityService;
use crate::productivity::TaskStatus;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use tauri::AppHandle;

pub struct FocusService {
    repo: FocusRepository,
    productivity_service: ProductivityService,
}

impl FocusService {
    pub fn new(repo: FocusRepository, productivity_service: ProductivityService) -> Self {
        Self { repo, productivity_service }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    fn calculate_remaining_time(session: &FocusSession, current_time: i64) -> i32 {
        if let Some(start) = session.started_at {
            let elapsed = match session.status {
                SessionStatus::Running => current_time - start,
                SessionStatus::Paused => {
                    if let Some(pause_time) = session.paused_at {
                        pause_time - start
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            
            let remaining = session.planned_seconds - (elapsed as i32);
            if remaining < 0 { 0 } else { remaining }
        } else {
            session.planned_seconds
        }
    }

    fn check_completion(&self, mut session: FocusSession, current_time: i64, app: &Option<AppHandle>) -> Result<FocusSession, String> {
        if session.status == SessionStatus::Running {
            let remaining = Self::calculate_remaining_time(&session, current_time);
            if remaining <= 0 {
                session.status = SessionStatus::Completed;
                session.ended_at = Some(current_time);
                session.updated_at = current_time;
                
                self.repo.update_session(&session)?;

                if let Some(app_handle) = app {
                    use tauri::Manager;
                    use tauri_plugin_notification::NotificationExt;
                    
                    let mut should_notify = true;
                    if let Some(state) = app_handle.try_state::<tauri::State<'_, std::sync::Mutex<crate::AppMode>>>() {
                        let current_mode = state.lock().unwrap().clone();
                        if current_mode == crate::AppMode::Disabled || current_mode == crate::AppMode::Assessment {
                            should_notify = false;
                        }
                    }

                    if should_notify {
                        let title = match session.session_type {
                            SessionType::Focus => "Focus Session Completed",
                            SessionType::Break => "Break Completed",
                        };
                        let body = match session.session_type {
                            SessionType::Focus => "Great job! Your focus session is complete.",
                            SessionType::Break => "Time to get back to work!",
                        };
                        
                        let _ = app_handle.notification()
                            .builder()
                            .title(title)
                            .body(body)
                            .show();
                    }
                }
            }
        }
        Ok(session)
    }

    pub fn get_timer_state(&self, app: Option<AppHandle>) -> Result<TimerState, String> {
        let current_time = Self::current_timestamp();
        
        if let Some(session) = self.repo.get_active_session()? {
            let updated_session = self.check_completion(session, current_time, &app)?;
            let remaining = Self::calculate_remaining_time(&updated_session, current_time);
            
            Ok(TimerState {
                active_session: Some(updated_session),
                remaining_seconds: remaining,
            })
        } else {
            Ok(TimerState {
                active_session: None,
                remaining_seconds: 0,
            })
        }
    }

    pub fn start_focus_session(&self, task_id: Option<&str>, duration_seconds: i32) -> Result<FocusSession, String> {
        if let Some(active) = self.repo.get_active_session()? {
            if active.status == SessionStatus::Running || active.status == SessionStatus::Paused {
                return Err("Finish or cancel the current focus session first.".to_string());
            }
        }

        if duration_seconds < 60 || duration_seconds > 10800 { // 1 min to 180 min
            return Err("Focus duration must be between 1 and 180 minutes.".to_string());
        }

        if let Some(tid) = task_id {
            let task = self.productivity_service.get_task(tid)?
                .ok_or_else(|| "Task not found".to_string())?;

            // Update task status if Pending
            if task.status == TaskStatus::Pending {
                self.productivity_service.set_task_status(tid, TaskStatus::InProgress)?;
            }
        }

        let timestamp = Self::current_timestamp();
        
        let session = FocusSession {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.map(|s| s.to_string()),
            session_type: SessionType::Focus,
            planned_seconds: duration_seconds,
            started_at: Some(timestamp),
            paused_at: None,
            ended_at: None,
            status: SessionStatus::Running,
            created_at: timestamp,
            updated_at: timestamp,
        };

        self.repo.create_session(&session)?;
        Ok(session)
    }

    pub fn start_break(&self, duration_seconds: i32) -> Result<FocusSession, String> {
        if let Some(active) = self.repo.get_active_session()? {
            if active.status == SessionStatus::Running || active.status == SessionStatus::Paused {
                return Err("Finish or cancel the current focus session first.".to_string());
            }
        }

        if duration_seconds < 60 || duration_seconds > 3600 {
            return Err("Break duration must be between 1 and 60 minutes.".to_string());
        }

        let timestamp = Self::current_timestamp();
        
        let session = FocusSession {
            id: Uuid::new_v4().to_string(),
            task_id: None,
            session_type: SessionType::Break,
            planned_seconds: duration_seconds,
            started_at: Some(timestamp),
            paused_at: None,
            ended_at: None,
            status: SessionStatus::Running,
            created_at: timestamp,
            updated_at: timestamp,
        };

        self.repo.create_session(&session)?;
        Ok(session)
    }

    pub fn pause_session(&self) -> Result<(), String> {
        let current_time = Self::current_timestamp();
        let mut session = self.repo.get_active_session()?.ok_or_else(|| "No active session".to_string())?;
        
        if session.status != SessionStatus::Running {
            return Err("Session is not running".to_string());
        }

        session.status = SessionStatus::Paused;
        session.paused_at = Some(current_time);
        session.updated_at = current_time;

        self.repo.update_session(&session)?;
        Ok(())
    }

    pub fn resume_session(&self) -> Result<(), String> {
        let current_time = Self::current_timestamp();
        let mut session = self.repo.get_active_session()?.ok_or_else(|| "No active session".to_string())?;
        
        if session.status != SessionStatus::Paused {
            return Err("Session is not paused".to_string());
        }

        if let (Some(start), Some(pause)) = (session.started_at, session.paused_at) {
            // Shift the start time forward by the paused duration
            let paused_duration = current_time - pause;
            session.started_at = Some(start + paused_duration);
        }

        session.status = SessionStatus::Running;
        session.paused_at = None;
        session.updated_at = current_time;

        self.repo.update_session(&session)?;
        Ok(())
    }

    pub fn finish_session(&self) -> Result<(), String> {
        let current_time = Self::current_timestamp();
        let mut session = self.repo.get_active_session()?.ok_or_else(|| "No active session".to_string())?;
        
        session.status = SessionStatus::Completed;
        session.ended_at = Some(current_time);
        session.updated_at = current_time;

        self.repo.update_session(&session)?;
        Ok(())
    }

    pub fn cancel_session(&self) -> Result<(), String> {
        let current_time = Self::current_timestamp();
        let mut session = self.repo.get_active_session()?.ok_or_else(|| "No active session".to_string())?;
        
        session.status = SessionStatus::Cancelled;
        session.ended_at = Some(current_time);
        session.updated_at = current_time;

        self.repo.update_session(&session)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::productivity::repository::ProductivityRepository;
    use crate::productivity::TaskPriority;

    fn setup_service() -> FocusService {
        let conn = crate::db::init_in_memory_db().unwrap();
        
        let prod_repo = ProductivityRepository::new(conn.clone());
        let prod_service = ProductivityService::new(prod_repo);
        
        // create dummy task
        let _ = prod_service.create_task("Test task", None, "2026-10-03", None, TaskPriority::Medium);
        
        let focus_repo = FocusRepository::new(conn);
        FocusService::new(focus_repo, prod_service)
    }

    #[test]
    fn test_start_and_get_session() {
        let service = setup_service();
        
        // Assuming task id is created first, we need to list it to get its ID
        let tasks = service.productivity_service.list_tasks("2026-10-03").unwrap();
        let task_id = &tasks[0].id;
        
        let _session = service.start_focus_session(Some(task_id), 1500).unwrap();
        
        let state = service.get_timer_state(None).unwrap();
        assert!(state.active_session.is_some());
        let active = state.active_session.unwrap();
        assert_eq!(active.status, SessionStatus::Running);
        assert_eq!(active.planned_seconds, 1500);
        
        // Test multiple sessions blocked
        let res = service.start_focus_session(Some(task_id), 1500);
        assert!(res.is_err());
    }

    #[test]
    fn test_pause_resume() {
        let service = setup_service();
        let tasks = service.productivity_service.list_tasks("2026-10-03").unwrap();
        let task_id = &tasks[0].id;
        
        service.start_focus_session(Some(task_id), 1500).unwrap();
        
        service.pause_session().unwrap();
        let state = service.get_timer_state(None).unwrap();
        assert_eq!(state.active_session.unwrap().status, SessionStatus::Paused);
        
        service.resume_session().unwrap();
        let state2 = service.get_timer_state(None).unwrap();
        assert_eq!(state2.active_session.unwrap().status, SessionStatus::Running);
    }
    
    #[test]
    fn test_finish_cancel() {
        let service = setup_service();
        let tasks = service.productivity_service.list_tasks("2026-10-03").unwrap();
        let task_id = &tasks[0].id;
        
        service.start_focus_session(Some(task_id), 1500).unwrap();
        service.finish_session().unwrap();
        
        let state = service.get_timer_state(None).unwrap();
        // active_session is still returned because check_completion mutates it to Completed, but it's completed
        // Wait, get_active_session only queries RUNNING or PAUSED. 
        // So finishing it directly will make get_timer_state return None active_session
        assert!(state.active_session.is_none());
    }
}
