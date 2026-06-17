use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::StatusCode};
use serde_json::json;
use uuid::Uuid;

use crate::middleware::auth::AppState;
use crate::models::User;

pub async fn seed_users(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let admin_id = Uuid::new_v4().to_string();
    let james_id = Uuid::new_v4().to_string();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // Using a simple password for seed users
    let hashed_password = argon2
        .hash_password("password123".as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    // Check if admin exists
    let admin_exists: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, full_name, email, hashed_password, role, created_at, updated_at FROM users WHERE email = 'admin@example.com'"
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if admin_exists.is_none() {
        sqlx::query(
            "INSERT INTO users (id, full_name, email, hashed_password, role) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&admin_id)
        .bind("Admin User")
        .bind("admin@example.com")
        .bind(&hashed_password)
        .bind("admin")
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let james_exists: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, full_name, email, hashed_password, role, created_at, updated_at FROM users WHERE email = 'jamesbond@example.com'"
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if james_exists.is_none() {
        sqlx::query(
            "INSERT INTO users (id, full_name, email, hashed_password, role) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&james_id)
        .bind("James Bond")
        .bind("jamesbond@example.com")
        .bind(&hashed_password)
        .bind("staff")
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Fetch James' ID if he existed
    let james_id_final = if let Some(j) = james_exists {
        j.id
    } else {
        james_id
    };

    Ok(Json(json!({
        "message": "Users seeded successfully. Default password is 'password123'.",
        "admin_id": admin_id,
        "james_bond_id": james_id_final
    })))
}
