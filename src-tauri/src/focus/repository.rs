use super::{FocusSession, SessionStatus, SessionType};
use crate::db::DbConnection;
use rusqlite::params;

pub struct FocusRepository {
    conn: DbConnection,
}

impl FocusRepository {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn create_session(&self, session: &FocusSession) -> Result<(), String> {
        self.conn.lock().unwrap().execute(
            "INSERT INTO focus_sessions (id, task_id, session_type, planned_seconds, started_at, paused_at, ended_at, status, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                session.id,
                session.task_id,
                session.session_type.as_str(),
                session.planned_seconds,
                session.started_at,
                session.paused_at,
                session.ended_at,
                session.status.as_str(),
                session.created_at,
                session.updated_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_session(&self, session: &FocusSession) -> Result<(), String> {
        self.conn.lock().unwrap().execute(
            "UPDATE focus_sessions SET task_id = ?1, session_type = ?2, planned_seconds = ?3, started_at = ?4, paused_at = ?5, ended_at = ?6, status = ?7, updated_at = ?8 WHERE id = ?9",
            params![
                session.task_id,
                session.session_type.as_str(),
                session.planned_seconds,
                session.started_at,
                session.paused_at,
                session.ended_at,
                session.status.as_str(),
                session.updated_at,
                session.id
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_active_session(&self) -> Result<Option<FocusSession>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, task_id, session_type, planned_seconds, started_at, paused_at, ended_at, status, created_at, updated_at FROM focus_sessions WHERE status IN ('RUNNING', 'PAUSED') ORDER BY created_at DESC LIMIT 1").map_err(|e| e.to_string())?;
        
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let session_type_str: String = row.get(2).map_err(|e| e.to_string())?;
            let status_str: String = row.get(7).map_err(|e| e.to_string())?;
            
            Ok(Some(FocusSession {
                id: row.get(0).map_err(|e| e.to_string())?,
                task_id: row.get(1).map_err(|e| e.to_string())?,
                session_type: SessionType::from_str(&session_type_str).unwrap_or(SessionType::Focus),
                planned_seconds: row.get(3).map_err(|e| e.to_string())?,
                started_at: row.get(4).map_err(|e| e.to_string())?,
                paused_at: row.get(5).map_err(|e| e.to_string())?,
                ended_at: row.get(6).map_err(|e| e.to_string())?,
                status: SessionStatus::from_str(&status_str).unwrap_or(SessionStatus::Running),
                created_at: row.get(8).map_err(|e| e.to_string())?,
                updated_at: row.get(9).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }
}
