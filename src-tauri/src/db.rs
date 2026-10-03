use rusqlite::{Connection, Result as SqlResult};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type DbConnection = Arc<Mutex<Connection>>;

pub fn init_db(db_path: PathBuf) -> Result<DbConnection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    // Create memories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            category TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Create tasks table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            date TEXT NOT NULL,
            estimated_minutes INTEGER,
            priority TEXT NOT NULL,
            status TEXT NOT NULL,
            position INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Create focus_sessions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS focus_sessions (
            id TEXT PRIMARY KEY,
            task_id TEXT,
            session_type TEXT NOT NULL,
            planned_seconds INTEGER NOT NULL,
            started_at INTEGER,
            paused_at INTEGER,
            ended_at INTEGER,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS desktop_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS distraction_rules (
            app_name TEXT PRIMARY KEY,
            category TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(Arc::new(Mutex::new(conn)))
}

#[cfg(test)]
pub fn init_in_memory_db() -> Result<DbConnection, String> {
    let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            category TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            date TEXT NOT NULL,
            estimated_minutes INTEGER,
            priority TEXT NOT NULL,
            status TEXT NOT NULL,
            position INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS focus_sessions (
            id TEXT PRIMARY KEY,
            task_id TEXT,
            session_type TEXT NOT NULL,
            planned_seconds INTEGER NOT NULL,
            started_at INTEGER,
            paused_at INTEGER,
            ended_at INTEGER,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS desktop_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS distraction_rules (
            app_name TEXT PRIMARY KEY,
            category TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(Arc::new(Mutex::new(conn)))
}
