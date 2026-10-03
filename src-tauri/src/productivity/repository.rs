use super::{Task, TaskPriority, TaskStatus};
use crate::db::DbConnection;
use rusqlite::params;

#[derive(Clone)]
pub struct ProductivityRepository {
    conn: DbConnection,
}

impl ProductivityRepository {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn create_task(&self, task: &Task) -> Result<(), String> {
        self.conn.lock().unwrap().execute(
            "INSERT INTO tasks (id, title, description, date, estimated_minutes, priority, status, position, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                task.id,
                task.title,
                task.description,
                task.date,
                task.estimated_minutes,
                task.priority.as_str(),
                task.status.as_str(),
                task.position,
                task.created_at,
                task.updated_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, description, date, estimated_minutes, priority, status, position, created_at, updated_at FROM tasks WHERE id = ?1").map_err(|e| e.to_string())?;
        
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let priority_str: String = row.get(5).map_err(|e| e.to_string())?;
            let status_str: String = row.get(6).map_err(|e| e.to_string())?;
            
            Ok(Some(Task {
                id: row.get(0).map_err(|e| e.to_string())?,
                title: row.get(1).map_err(|e| e.to_string())?,
                description: row.get(2).map_err(|e| e.to_string())?,
                date: row.get(3).map_err(|e| e.to_string())?,
                estimated_minutes: row.get(4).map_err(|e| e.to_string())?,
                priority: TaskPriority::from_str(&priority_str).unwrap_or(TaskPriority::Medium),
                status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
                position: row.get(7).map_err(|e| e.to_string())?,
                created_at: row.get(8).map_err(|e| e.to_string())?,
                updated_at: row.get(9).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_tasks(&self, date: &str) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, description, date, estimated_minutes, priority, status, position, created_at, updated_at FROM tasks WHERE date = ?1 ORDER BY position ASC, created_at ASC").map_err(|e| e.to_string())?;
        
        let rows = stmt.query_map(params![date], |row| {
            let priority_str: String = row.get(5)?;
            let status_str: String = row.get(6)?;
            
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                date: row.get(3)?,
                estimated_minutes: row.get(4)?,
                priority: TaskPriority::from_str(&priority_str).unwrap_or(TaskPriority::Medium),
                status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
                position: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        }).map_err(|e| e.to_string())?;
        
        let mut tasks = Vec::new();
        for task in rows {
            tasks.push(task.map_err(|e| e.to_string())?);
        }
        Ok(tasks)
    }

    pub fn update_task(&self, task: &Task) -> Result<(), String> {
        self.conn.lock().unwrap().execute(
            "UPDATE tasks SET title = ?1, description = ?2, date = ?3, estimated_minutes = ?4, priority = ?5, status = ?6, position = ?7, updated_at = ?8 WHERE id = ?9",
            params![
                task.title,
                task.description,
                task.date,
                task.estimated_minutes,
                task.priority.as_str(),
                task.status.as_str(),
                task.position,
                task.updated_at,
                task.id
            ]
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<(), String> {
        self.conn.lock().unwrap().execute("DELETE FROM tasks WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_max_position(&self, date: &str) -> Result<i32, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT MAX(position) FROM tasks WHERE date = ?1").map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![date]).map_err(|e| e.to_string())?;
        
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let max_pos: Option<i32> = row.get(0).map_err(|e| e.to_string())?;
            Ok(max_pos.unwrap_or(-1))
        } else {
            Ok(-1)
        }
    }
}
