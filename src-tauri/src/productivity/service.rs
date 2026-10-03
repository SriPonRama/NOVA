use super::{repository::ProductivityRepository, Task, TaskPriority, TaskStatus};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct ProductivityService {
    repo: ProductivityRepository,
}

impl ProductivityService {
    pub fn new(repo: ProductivityRepository) -> Self {
        Self { repo }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    fn validate_title(title: &str) -> Result<String, String> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err("Task title cannot be empty".to_string());
        }
        if trimmed.len() > 500 {
            return Err("Task title is too long".to_string());
        }
        Ok(trimmed.to_string())
    }

    fn validate_date(date: &str) -> Result<String, String> {
        let trimmed = date.trim();
        if trimmed.is_empty() {
            return Err("Task date cannot be empty".to_string());
        }
        if trimmed.len() != 10 {
            return Err("Invalid date format, expected YYYY-MM-DD".to_string());
        }
        Ok(trimmed.to_string())
    }

    pub fn create_task(
        &self,
        title: &str,
        description: Option<String>,
        date: &str,
        estimated_minutes: Option<i32>,
        priority: TaskPriority,
    ) -> Result<Task, String> {
        let valid_title = Self::validate_title(title)?;
        let valid_date = Self::validate_date(date)?;
        
        if let Some(est) = estimated_minutes {
            if est < 0 || est > 1440 {
                return Err("Estimated minutes must be between 0 and 1440".to_string());
            }
        }

        let max_pos = self.repo.get_max_position(&valid_date)?;
        let position = max_pos + 1;
        let timestamp = Self::current_timestamp();
        
        let task = Task {
            id: Uuid::new_v4().to_string(),
            title: valid_title,
            description,
            date: valid_date,
            estimated_minutes,
            priority,
            status: TaskStatus::Pending,
            position,
            created_at: timestamp,
            updated_at: timestamp,
        };

        self.repo.create_task(&task)?;
        Ok(task)
    }

    pub fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        self.repo.get_task(id)
    }

    pub fn list_tasks(&self, date: &str) -> Result<Vec<Task>, String> {
        let valid_date = Self::validate_date(date)?;
        self.repo.list_tasks(&valid_date)
    }

    pub fn update_task(
        &self,
        id: &str,
        title: &str,
        description: Option<String>,
        date: &str,
        estimated_minutes: Option<i32>,
        priority: TaskPriority,
        status: TaskStatus,
    ) -> Result<(), String> {
        let valid_title = Self::validate_title(title)?;
        let valid_date = Self::validate_date(date)?;
        
        if let Some(est) = estimated_minutes {
            if est < 0 || est > 1440 {
                return Err("Estimated minutes must be between 0 and 1440".to_string());
            }
        }

        let mut existing = self.repo.get_task(id)?.ok_or("Task not found")?;
        
        if existing.date != valid_date {
            let max_pos = self.repo.get_max_position(&valid_date)?;
            existing.position = max_pos + 1;
        }
        
        existing.title = valid_title;
        existing.description = description;
        existing.date = valid_date;
        existing.estimated_minutes = estimated_minutes;
        existing.priority = priority;
        existing.status = status;
        existing.updated_at = Self::current_timestamp();

        self.repo.update_task(&existing)
    }

    pub fn set_task_status(&self, id: &str, status: TaskStatus) -> Result<(), String> {
        let mut existing = self.repo.get_task(id)?.ok_or("Task not found")?;
        existing.status = status;
        existing.updated_at = Self::current_timestamp();
        self.repo.update_task(&existing)
    }

    pub fn delete_task(&self, id: &str) -> Result<(), String> {
        self.repo.delete_task(id)
    }

    pub fn reorder_tasks(&self, date: &str, ordered_ids: Vec<String>) -> Result<(), String> {
        let valid_date = Self::validate_date(date)?;
        let mut current_tasks = self.repo.list_tasks(&valid_date)?;
        
        if ordered_ids.len() != current_tasks.len() {
            return Err("Ordered IDs length does not match number of tasks for date".to_string());
        }

        let mut id_set = std::collections::HashSet::new();
        for id in &ordered_ids {
            if !id_set.insert(id.clone()) {
                return Err("Duplicate task ID in reorder list".to_string());
            }
        }

        let timestamp = Self::current_timestamp();
        
        for (i, id) in ordered_ids.iter().enumerate() {
            if let Some(task) = current_tasks.iter_mut().find(|t| t.id == *id) {
                task.position = i as i32;
                task.updated_at = timestamp;
                self.repo.update_task(task)?;
            } else {
                return Err("Task ID not found in current date".to_string());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_service() -> ProductivityService {
        let conn = crate::db::init_in_memory_db().unwrap();
        let repo = ProductivityRepository::new(conn);
        ProductivityService::new(repo)
    }

    #[test]
    fn test_create_and_retrieve_task() {
        let service = setup_service();
        let task = service.create_task("Test task", None, "2026-10-03", None, TaskPriority::High).unwrap();
        
        assert_eq!(task.title, "Test task");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Pending);
        
        let retrieved = service.get_task(&task.id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Test task");
    }

    #[test]
    fn test_validation() {
        let service = setup_service();
        
        // Empty title
        assert!(service.create_task("", None, "2026-10-03", None, TaskPriority::Medium).is_err());
        
        // Invalid date
        assert!(service.create_task("T", None, "invalid", None, TaskPriority::Medium).is_err());
        
        // Invalid duration
        assert!(service.create_task("T", None, "2026-10-03", Some(-10), TaskPriority::Medium).is_err());
    }

    #[test]
    fn test_list_and_reorder() {
        let service = setup_service();
        let t1 = service.create_task("Task 1", None, "2026-10-03", None, TaskPriority::Medium).unwrap();
        let t2 = service.create_task("Task 2", None, "2026-10-03", None, TaskPriority::Medium).unwrap();
        
        let tasks = service.list_tasks("2026-10-03").unwrap();
        assert_eq!(tasks[0].id, t1.id);
        assert_eq!(tasks[1].id, t2.id);
        
        // Reorder
        service.reorder_tasks("2026-10-03", vec![t2.id.clone(), t1.id.clone()]).unwrap();
        
        let tasks = service.list_tasks("2026-10-03").unwrap();
        assert_eq!(tasks[0].id, t2.id);
        assert_eq!(tasks[1].id, t1.id);
    }
}
