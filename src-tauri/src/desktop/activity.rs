use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityCategory {
    Study,
    Productive,
    Neutral,
    Distracting,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForegroundActivity {
    pub process_name: String,
    pub application_name: String,
    pub window_title_optional: Option<String>,
    pub timestamp: i64,
}

pub trait DesktopActivityProvider: Send + Sync {
    fn get_current_activity(&self) -> Result<ForegroundActivity, String>;
}
