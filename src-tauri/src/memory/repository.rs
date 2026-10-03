use super::{Memory, MemoryCategory};
use rusqlite::params;
use crate::db::DbConnection;

pub struct MemoryRepository {
    conn: DbConnection,
}

impl MemoryRepository {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn create_memory(&self, memory: &Memory) -> Result<(), String> {
        self.conn.lock().unwrap().execute(
            "INSERT INTO memories (id, content, category, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                memory.id,
                memory.content,
                memory.category.as_str(),
                memory.created_at,
                memory.updated_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_memory(&self, id: &str) -> Result<Option<Memory>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, content, category, created_at, updated_at FROM memories WHERE id = ?1").map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let category_str: String = row.get(2).map_err(|e| e.to_string())?;
            let category = MemoryCategory::from_str(&category_str).unwrap_or(MemoryCategory::Temporary);
            
            Ok(Some(Memory {
                id: row.get(0).map_err(|e| e.to_string())?,
                content: row.get(1).map_err(|e| e.to_string())?,
                category,
                created_at: row.get(3).map_err(|e| e.to_string())?,
                updated_at: row.get(4).map_err(|e| e.to_string())?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_memories(&self) -> Result<Vec<Memory>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, content, category, created_at, updated_at FROM memories ORDER BY created_at DESC").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            let category_str: String = row.get(2)?;
            let category = MemoryCategory::from_str(&category_str).unwrap_or(MemoryCategory::Temporary);
            
            Ok(Memory {
                id: row.get(0)?,
                content: row.get(1)?,
                category,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut memories = Vec::new();
        for mem in rows {
            memories.push(mem.map_err(|e| e.to_string())?);
        }
        Ok(memories)
    }

    pub fn search_memories(&self, query: &str) -> Result<Vec<Memory>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, content, category, created_at, updated_at FROM memories WHERE content LIKE ?1 ORDER BY created_at DESC").map_err(|e| e.to_string())?;
        let like_query = format!("%{}%", query);
        let rows = stmt.query_map(params![like_query], |row| {
            let category_str: String = row.get(2)?;
            let category = MemoryCategory::from_str(&category_str).unwrap_or(MemoryCategory::Temporary);
            
            Ok(Memory {
                id: row.get(0)?,
                content: row.get(1)?,
                category,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut memories = Vec::new();
        for mem in rows {
            memories.push(mem.map_err(|e| e.to_string())?);
        }
        Ok(memories)
    }

    pub fn update_memory(&self, id: &str, content: &str, category: &MemoryCategory, updated_at: i64) -> Result<(), String> {
        self.conn.lock().unwrap().execute(
            "UPDATE memories SET content = ?1, category = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                content,
                category.as_str(),
                updated_at,
                id
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_memory(&self, id: &str) -> Result<(), String> {
        self.conn.lock().unwrap().execute("DELETE FROM memories WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_all_memories(&self) -> Result<(), String> {
        self.conn.lock().unwrap().execute("DELETE FROM memories", []).map_err(|e| e.to_string())?;
        Ok(())
    }
}
