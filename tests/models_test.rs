#[cfg(test)]
mod tests {
    use chrono::Utc;
    use rust_axum_todo::models::{Task, User};
    use uuid::Uuid;

    #[test]
    fn test_user_creation() {
        let user = User {
            id: Uuid::new_v4().to_string(),
            full_name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            hashed_password: "hash".to_string(),
            role: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(user.role, "admin");
    }

    #[test]
    fn test_task_creation() {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            title: "Test Task".to_string(),
            description: Some("Desc".to_string()),
            status: "todo".to_string(),
            priority: "high".to_string(),
            created_by_id: Uuid::new_v4().to_string(),
            assigned_to_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(task.status, "todo");
        assert_eq!(task.priority, "high");
    }
}
