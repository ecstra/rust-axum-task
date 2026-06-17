use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub full_name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub hashed_password: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_by_id: String,
    pub assigned_to_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct LoginChallenge {
    pub challenge_id: String,
    pub user_id: String,
    pub verification_code: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct EmailLog {
    pub id: String,
    pub email: String,
    pub verification_code: String,
    pub created_at: DateTime<Utc>,
}

// Request and Response Models
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub login_challenge_id: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct Verify2FaRequest {
    pub challenge_id: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct Verify2FaResponse {
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Deserialize)]
pub struct AssignTaskRequest {
    pub task_ids: Vec<String>,
    pub assigned_to_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ViewMyTasksResponse {
    pub user: UserInfo,
    pub tasks: Vec<TaskResponse>,
    pub summary: TaskSummary,
    pub cache: CacheMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: String,
}

#[derive(Serialize, Deserialize)]
pub struct TaskSummary {
    pub total_assigned_tasks: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CacheMetadata {
    pub hit: bool,
}
