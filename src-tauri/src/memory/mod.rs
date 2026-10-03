use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryCategory {
    UserPreference,
    Goal,
    Study,
    Project,
    Important,
    Temporary,
}

impl MemoryCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserPreference => "USER_PREFERENCE",
            Self::Goal => "GOAL",
            Self::Study => "STUDY",
            Self::Project => "PROJECT",
            Self::Important => "IMPORTANT",
            Self::Temporary => "TEMPORARY",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "USER_PREFERENCE" => Some(Self::UserPreference),
            "GOAL" => Some(Self::Goal),
            "STUDY" => Some(Self::Study),
            "PROJECT" => Some(Self::Project),
            "IMPORTANT" => Some(Self::Important),
            "TEMPORARY" => Some(Self::Temporary),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub category: MemoryCategory,
    pub created_at: i64,
    pub updated_at: i64,
}

pub mod repository;
pub mod service;
pub mod commands;
