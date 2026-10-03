use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopSettings {
    pub enabled: bool,
    pub grace_period_seconds: u32,
    pub cooldown_minutes: u32,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            grace_period_seconds: 60,
            cooldown_minutes: 5,
        }
    }
}

pub struct DesktopSettingsRepository {
    conn: crate::db::DbConnection,
}

impl DesktopSettingsRepository {
    pub fn new(conn: crate::db::DbConnection) -> Self {
        Self { conn }
    }

    pub fn get_settings(&self) -> Result<DesktopSettings, String> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT key, value FROM desktop_settings")
            .map_err(|e| e.to_string())?;
            
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        }).map_err(|e| e.to_string())?;

        let mut settings = DesktopSettings::default();
        
        for row in rows {
            if let Ok((key, value)) = row {
                match key.as_str() {
                    "enabled" => settings.enabled = value == "true",
                    "grace_period_seconds" => {
                        if let Ok(v) = value.parse() {
                            settings.grace_period_seconds = v;
                        }
                    }
                    "cooldown_minutes" => {
                        if let Ok(v) = value.parse() {
                            settings.cooldown_minutes = v;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &DesktopSettings) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        
        let enabled_val = if settings.enabled { "true" } else { "false" };
        let grace_val = settings.grace_period_seconds.to_string();
        let cooldown_val = settings.cooldown_minutes.to_string();

        conn.execute(
            "INSERT OR REPLACE INTO desktop_settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["enabled", enabled_val],
        ).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO desktop_settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["grace_period_seconds", grace_val],
        ).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO desktop_settings (key, value) VALUES (?1, ?2)",
            rusqlite::params!["cooldown_minutes", cooldown_val],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }
}
