use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionType {
    Focus,
    Break,
}

impl SessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionType::Focus => "FOCUS",
            SessionType::Break => "BREAK",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "FOCUS" => Some(SessionType::Focus),
            "BREAK" => Some(SessionType::Break),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Running,
    Paused,
    Completed,
    Cancelled,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Running => "RUNNING",
            SessionStatus::Paused => "PAUSED",
            SessionStatus::Completed => "COMPLETED",
            SessionStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "RUNNING" => Some(SessionStatus::Running),
            "PAUSED" => Some(SessionStatus::Paused),
            "COMPLETED" => Some(SessionStatus::Completed),
            "CANCELLED" => Some(SessionStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub id: String,
    pub task_id: Option<String>,
    pub session_type: SessionType,
    pub planned_seconds: i32,
    pub started_at: Option<i64>,
    pub paused_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub status: SessionStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub active_session: Option<FocusSession>,
    pub remaining_seconds: i32,
}

pub mod repository;
pub mod service;
pub mod commands;
