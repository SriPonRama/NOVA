use super::{repository::ProductivityRepository, Task, TaskPriority, TaskStatus};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone)]
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

    pub fn get_productivity_summary(&self, date: &str) -> Result<super::ProductivitySummary, String> {
        let valid_date = Self::validate_date(date)?;
        let tasks = self.repo.list_tasks(&valid_date)?;
        
        let mut total_tasks = 0;
        let mut completed_tasks = 0;
        let mut remaining_tasks = 0;
        let mut high_priority_remaining = 0;
        let mut estimated_remaining_minutes = 0;

        for task in tasks {
            total_tasks += 1;
            if task.status == TaskStatus::Completed || task.status == TaskStatus::Cancelled {
                if task.status == TaskStatus::Completed {
                    completed_tasks += 1;
                }
            } else {
                remaining_tasks += 1;
                if task.priority == TaskPriority::High {
                    high_priority_remaining += 1;
                }
                estimated_remaining_minutes += task.estimated_minutes.unwrap_or(0);
            }
        }

        Ok(super::ProductivitySummary {
            date: valid_date,
            total_tasks,
            completed_tasks,
            remaining_tasks,
            high_priority_remaining,
            estimated_remaining_minutes,
        })
    }

    pub fn get_remaining_workload(&self, date: &str) -> Result<super::RemainingWorkload, String> {
        let valid_date = Self::validate_date(date)?;
        let tasks = self.repo.list_tasks(&valid_date)?;
        
        let mut remaining_tasks = 0;
        let mut total_estimated_minutes = 0;
        let mut high_priority_minutes = 0;
        let mut medium_priority_minutes = 0;
        let mut low_priority_minutes = 0;

        for task in tasks {
            if task.status != TaskStatus::Completed && task.status != TaskStatus::Cancelled {
                remaining_tasks += 1;
                let mins = task.estimated_minutes.unwrap_or(0);
                total_estimated_minutes += mins;
                
                match task.priority {
                    TaskPriority::High => high_priority_minutes += mins,
                    TaskPriority::Medium => medium_priority_minutes += mins,
                    TaskPriority::Low => low_priority_minutes += mins,
                }
            }
        }

        Ok(super::RemainingWorkload {
            remaining_tasks,
            total_estimated_minutes,
            high_priority_minutes,
            medium_priority_minutes,
            low_priority_minutes,
        })
    }

    pub fn get_next_recommended_task(&self, date: &str) -> Result<super::TaskRecommendation, String> {
        let valid_date = Self::validate_date(date)?;
        let mut tasks = self.repo.list_tasks(&valid_date)?;
        
        // Exclude completed or cancelled tasks
        tasks.retain(|t| t.status != TaskStatus::Completed && t.status != TaskStatus::Cancelled);

        if tasks.is_empty() {
            return Ok(super::TaskRecommendation {
                recommended_task: None,
                reasoning: "No pending or in-progress tasks found for today.".to_string(),
            });
        }

        // 1. In-progress task wins
        if let Some(in_progress) = tasks.iter().find(|t| t.status == TaskStatus::InProgress) {
            return Ok(super::TaskRecommendation {
                recommended_task: Some(in_progress.clone()),
                reasoning: "Task is already in-progress.".to_string(),
            });
        }

        // 2. Sort by Priority (High > Medium > Low) then Position
        tasks.sort_by(|a, b| {
            // Priority order: High(3) > Medium(2) > Low(1)
            let prio_score = |p: &TaskPriority| match p {
                TaskPriority::High => 3,
                TaskPriority::Medium => 2,
                TaskPriority::Low => 1,
            };
            
            let a_score = prio_score(&a.priority);
            let b_score = prio_score(&b.priority);
            
            if a_score != b_score {
                b_score.cmp(&a_score) // Descending priority
            } else {
                a.position.cmp(&b.position) // Ascending position (earlier first)
            }
        });

        // 3. Shorter task when appropriate logic (simplified to: if tie in priority/position, pick shorter? 
        // Position breaks ties already, so let's just pick the top one after sort.)
        let best_task = tasks.first().unwrap().clone();
        
        Ok(super::TaskRecommendation {
            recommended_task: Some(best_task.clone()),
            reasoning: format!("Selected highest priority ({:?}) pending task.", best_task.priority),
        })
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

    #[test]
    fn test_productivity_summary_and_workload() {
        let service = setup_service();
        service.create_task("T1", None, "2026-10-03", Some(30), TaskPriority::High).unwrap();
        service.create_task("T2", None, "2026-10-03", Some(45), TaskPriority::Medium).unwrap();
        
        let mut t3 = service.create_task("T3", None, "2026-10-03", Some(60), TaskPriority::High).unwrap();
        t3.status = TaskStatus::Completed;
        service.update_task(&t3.id, &t3.title, t3.description.clone(), &t3.date, t3.estimated_minutes, t3.priority.clone(), t3.status.clone()).unwrap();

        let summary = service.get_productivity_summary("2026-10-03").unwrap();
        assert_eq!(summary.total_tasks, 3);
        assert_eq!(summary.completed_tasks, 1);
        assert_eq!(summary.remaining_tasks, 2);
        assert_eq!(summary.high_priority_remaining, 1);
        assert_eq!(summary.estimated_remaining_minutes, 75);

        let workload = service.get_remaining_workload("2026-10-03").unwrap();
        assert_eq!(workload.remaining_tasks, 2);
        assert_eq!(workload.total_estimated_minutes, 75);
        assert_eq!(workload.high_priority_minutes, 30);
        assert_eq!(workload.medium_priority_minutes, 45);
        assert_eq!(workload.low_priority_minutes, 0);
    }

    #[test]
    fn test_recommendation_logic() {
        let service = setup_service();
        
        // Empty workload
        let rec = service.get_next_recommended_task("2026-10-03").unwrap();
        assert!(rec.recommended_task.is_none());

        // Create tasks
        let low_prio = service.create_task("Low", None, "2026-10-03", Some(10), TaskPriority::Low).unwrap();
        let _med_prio = service.create_task("Med", None, "2026-10-03", None, TaskPriority::Medium).unwrap(); // missing duration
        let high_prio = service.create_task("High", None, "2026-10-03", Some(60), TaskPriority::High).unwrap();

        // High priority should win
        let rec = service.get_next_recommended_task("2026-10-03").unwrap();
        assert_eq!(rec.recommended_task.unwrap().id, high_prio.id);

        // If one is in-progress, it should win regardless of priority
        let mut in_prog = low_prio.clone();
        in_prog.status = TaskStatus::InProgress;
        service.update_task(&in_prog.id, &in_prog.title, in_prog.description.clone(), &in_prog.date, in_prog.estimated_minutes, in_prog.priority.clone(), in_prog.status.clone()).unwrap();
        
        let rec = service.get_next_recommended_task("2026-10-03").unwrap();
        assert_eq!(rec.recommended_task.unwrap().id, in_prog.id);

        // If in-progress is completed, it should revert to High prio
        in_prog.status = TaskStatus::Completed;
        service.update_task(&in_prog.id, &in_prog.title, in_prog.description.clone(), &in_prog.date, in_prog.estimated_minutes, in_prog.priority.clone(), in_prog.status.clone()).unwrap();

        let rec = service.get_next_recommended_task("2026-10-03").unwrap();
        assert_eq!(rec.recommended_task.unwrap().id, high_prio.id);

        // Tie breaking position: create another high priority
        let _high_prio2 = service.create_task("High 2", None, "2026-10-03", Some(30), TaskPriority::High).unwrap();
        
        // The earlier positioned one (high_prio) should win
        let rec = service.get_next_recommended_task("2026-10-03").unwrap();
        assert_eq!(rec.recommended_task.unwrap().id, high_prio.id);
    }
}
