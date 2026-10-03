use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "PENDING",
            TaskStatus::InProgress => "IN_PROGRESS",
            TaskStatus::Completed => "COMPLETED",
            TaskStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PENDING" => Some(TaskStatus::Pending),
            "IN_PROGRESS" => Some(TaskStatus::InProgress),
            "COMPLETED" => Some(TaskStatus::Completed),
            "CANCELLED" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "LOW",
            TaskPriority::Medium => "MEDIUM",
            TaskPriority::High => "HIGH",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "LOW" => Some(TaskPriority::Low),
            "MEDIUM" => Some(TaskPriority::Medium),
            "HIGH" => Some(TaskPriority::High),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub date: String,
    pub estimated_minutes: Option<i32>,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub position: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

pub mod repository;
pub mod service;
pub mod commands;
