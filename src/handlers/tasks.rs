use axum::{Extension, Json, extract::State, http::StatusCode};
use serde_json::json;
use uuid::Uuid;

use crate::middleware::auth::{AppState, Claims};
use crate::models::{
    AssignTaskRequest, CacheMetadata, CreateTaskRequest, Task, TaskResponse, TaskSummary, UserInfo,
    ViewMyTasksResponse,
};

pub async fn create_task(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Task>, (StatusCode, String)> {
    if claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Only admins can create tasks".into()));
    }

    let task_id = Uuid::new_v4().to_string();
    let status = payload.status.unwrap_or_else(|| "todo".into());
    let priority = payload.priority.unwrap_or_else(|| "medium".into());

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO tasks (id, title, description, status, priority, created_by_id) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&task_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&status)
    .bind(&priority)
    .bind(&claims.sub)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let task = sqlx::query_as::<_, Task>(
        "SELECT id, title, description, status, priority, created_by_id, assigned_to_id, created_at, updated_at FROM tasks WHERE id = ?"
    )
    .bind(&task_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(task))
}

pub async fn assign_tasks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AssignTaskRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Only admins can assign tasks".into()));
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for task_id in &payload.task_ids {
        sqlx::query(
            "UPDATE tasks SET assigned_to_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(&payload.assigned_to_id)
        .bind(task_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Invalidate cache for the user assigned
    state
        .cache
        .invalidate_user_tasks(&payload.assigned_to_id)
        .await;

    Ok(Json(json!({
        "message": format!("Successfully assigned {} tasks", payload.task_ids.len())
    })))
}

pub async fn view_my_tasks(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ViewMyTasksResponse>, (StatusCode, String)> {
    // Check cache first
    let cached: Option<String> = state.cache.get_cached_tasks(claims.sub.as_str()).await;
    if let Some(cached_json) = cached
        && let Ok(mut response) = serde_json::from_str::<ViewMyTasksResponse>(cached_json.as_str())
    {
        response.cache.hit = true;
        return Ok(Json(response));
    }

    // Cache miss, hit DB — join with users to resolve assigned_to email
    let rows = sqlx::query_as::<_, Task>(
        "SELECT t.id, t.title, t.description, t.status, t.priority, t.created_by_id, t.assigned_to_id, t.created_at, t.updated_at FROM tasks t WHERE t.assigned_to_id = ?"
    )
    .bind(&claims.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let task_responses: Vec<TaskResponse> = rows
        .into_iter()
        .map(|t| TaskResponse {
            id: t.id,
            title: t.title,
            status: t.status,
            priority: t.priority,
            assigned_to: claims.email.clone(),
        })
        .collect();

    let total_assigned = task_responses.len();

    let response = ViewMyTasksResponse {
        user: UserInfo {
            email: claims.email.clone(),
            role: claims.role.clone(),
        },
        tasks: task_responses,
        summary: TaskSummary {
            total_assigned_tasks: total_assigned,
        },
        cache: CacheMetadata { hit: false }, // Cache miss this time
    };

    // Store in cache
    if let Ok(json_str) = serde_json::to_string(&response) {
        state.cache.set_cached_tasks(&claims.sub, &json_str).await;
    }

    Ok(Json(response))
}
