use super::activity::ActivityCategory;
use crate::db::DbConnection;

pub struct ActivityClassifier {
    conn: DbConnection,
}

impl ActivityClassifier {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn normalize_process_name(name: &str) -> String {
        name.trim()
            .to_lowercase()
            .strip_suffix(".exe")
            .unwrap_or(name.trim().to_lowercase().as_str())
            .to_string()
    }

    pub fn get_default_category(normalized_name: &str) -> ActivityCategory {
        match normalized_name {
            "code" | "devenv" | "idea64" | "pycharm64" | "webstorm64" | "rider64" | "terminal" | "windowsterminal" | "powershell" | "cmd" | "rustrover64" => ActivityCategory::Study,
            "winword" | "excel" | "powerpnt" | "acrobat" | "foxitreader" | "obsidian" | "notion" => ActivityCategory::Productive,
            "explorer" | "taskmgr" | "systemsettings" => ActivityCategory::Neutral,
            // Example conservative defaults for distracting. We leave most unknown.
            "discord" | "slack" | "spotify" | "steam" | "epicgameslauncher" => ActivityCategory::Distracting,
            _ => ActivityCategory::Unknown,
        }
    }

    pub fn classify(&self, process_name: &str) -> Result<ActivityCategory, String> {
        let normalized = Self::normalize_process_name(process_name);
        
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT category FROM distraction_rules WHERE app_name = ?1")
            .map_err(|e| e.to_string())?;
            
        let mut rows = stmt.query(rusqlite::params![normalized])
            .map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let cat_str: String = row.get(0).map_err(|e| e.to_string())?;
            return match cat_str.as_str() {
                "Study" => Ok(ActivityCategory::Study),
                "Productive" => Ok(ActivityCategory::Productive),
                "Neutral" => Ok(ActivityCategory::Neutral),
                "Distracting" => Ok(ActivityCategory::Distracting),
                _ => Ok(ActivityCategory::Unknown),
            };
        }

        Ok(Self::get_default_category(&normalized))
    }
}
