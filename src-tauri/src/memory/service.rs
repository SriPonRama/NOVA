use super::{repository::MemoryRepository, Memory, MemoryCategory};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct MemoryService {
    repo: MemoryRepository,
}

impl MemoryService {
    pub fn new(repo: MemoryRepository) -> Self {
        Self { repo }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    pub fn create_memory(&self, content: &str, category: MemoryCategory) -> Result<Memory, String> {
        let trimmed_content = content.trim();
        if trimmed_content.is_empty() {
            return Err("Memory content cannot be empty".to_string());
        }
        if trimmed_content.len() > 10000 {
            return Err("Memory content is too large".to_string());
        }

        let id = Uuid::new_v4().to_string();
        let timestamp = Self::current_timestamp();

        let memory = Memory {
            id,
            content: trimmed_content.to_string(),
            category,
            created_at: timestamp,
            updated_at: timestamp,
        };

        self.repo.create_memory(&memory)?;
        Ok(memory)
    }

    pub fn get_memory(&self, id: &str) -> Result<Option<Memory>, String> {
        self.repo.get_memory(id)
    }

    pub fn list_memories(&self) -> Result<Vec<Memory>, String> {
        self.repo.list_memories()
    }

    pub fn search_memories(&self, query: &str) -> Result<Vec<Memory>, String> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() {
            return self.repo.list_memories();
        }
        self.repo.search_memories(trimmed_query)
    }

    pub fn update_memory(&self, id: &str, content: &str, category: MemoryCategory) -> Result<(), String> {
        let trimmed_content = content.trim();
        if trimmed_content.is_empty() {
            return Err("Memory content cannot be empty".to_string());
        }

        let existing = self.repo.get_memory(id)?;
        if existing.is_none() {
            return Err("Memory not found".to_string());
        }

        let timestamp = Self::current_timestamp();
        self.repo.update_memory(id, trimmed_content, &category, timestamp)?;
        Ok(())
    }

    pub fn delete_memory(&self, id: &str) -> Result<(), String> {
        self.repo.delete_memory(id)
    }

    pub fn clear_all_memories(&self) -> Result<(), String> {
        self.repo.clear_all_memories()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_service() -> MemoryService {
        let repo = MemoryRepository::new_in_memory().unwrap();
        MemoryService::new(repo)
    }

    #[test]
    fn test_create_and_retrieve_memory() {
        let service = setup_service();
        let mem = service.create_memory("Test memory", MemoryCategory::Goal).unwrap();
        assert_eq!(mem.content, "Test memory");
        assert_eq!(mem.category, MemoryCategory::Goal);

        let retrieved = service.get_memory(&mem.id).unwrap().unwrap();
        assert_eq!(retrieved.content, "Test memory");
    }

    #[test]
    fn test_validation() {
        let service = setup_service();
        assert!(service.create_memory("", MemoryCategory::Temporary).is_err());
        assert!(service.create_memory("   ", MemoryCategory::Temporary).is_err());
    }

    #[test]
    fn test_search() {
        let service = setup_service();
        service.create_memory("Apple pie recipe", MemoryCategory::Study).unwrap();
        service.create_memory("Banana bread", MemoryCategory::Study).unwrap();
        
        let results = service.search_memories("pie").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Apple pie recipe");
    }

    #[test]
    fn test_update() {
        let service = setup_service();
        let mem = service.create_memory("Old", MemoryCategory::Temporary).unwrap();
        
        service.update_memory(&mem.id, "New", MemoryCategory::Important).unwrap();
        let retrieved = service.get_memory(&mem.id).unwrap().unwrap();
        assert_eq!(retrieved.content, "New");
        assert_eq!(retrieved.category, MemoryCategory::Important);
    }
    
    #[test]
    fn test_delete() {
        let service = setup_service();
        let mem = service.create_memory("To delete", MemoryCategory::Temporary).unwrap();
        service.delete_memory(&mem.id).unwrap();
        let retrieved = service.get_memory(&mem.id).unwrap();
        assert!(retrieved.is_none());
    }
}
