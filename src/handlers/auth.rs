use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::Rng;
use serde_json::json;
use uuid::Uuid;

use crate::middleware::auth::{AppState, Claims};
use crate::models::{
    EmailLog, LoginRequest, LoginResponse, User, Verify2FaRequest, Verify2FaResponse,
};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, full_name, email, hashed_password, role, created_at, updated_at FROM users WHERE email = ?"
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into())),
    };

    let parsed_hash = PasswordHash::new(&user.hashed_password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid password hash format in db".into(),
        )
    })?;

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    // Generate 6-digit code
    let code: String = {
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen_range(100_000..=999_999);
        num.to_string()
    };

    let challenge_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(5);

    let log_id = Uuid::new_v4().to_string();

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
    let salt = SaltString::generate(&mut OsRng);
    let hashed_code = Argon2::default()
        .hash_password(code.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    sqlx::query(
        "INSERT INTO login_challenges (challenge_id, user_id, verification_code, expires_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&challenge_id)
    .bind(&user.id)
    .bind(&hashed_code)
    .bind(expires_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("INSERT INTO email_logs (id, email, verification_code) VALUES (?, ?, ?)")
        .bind(&log_id)
        .bind(&user.email)
        .bind(&code)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(LoginResponse {
        login_challenge_id: challenge_id,
        message: "2FA code generated and sent to email".into(),
    }))
}

pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(payload): Json<Verify2FaRequest>,
) -> Result<Json<Verify2FaResponse>, (StatusCode, String)> {
    let challenge = sqlx::query_as::<_, crate::models::LoginChallenge>(
        "SELECT challenge_id, user_id, verification_code, expires_at FROM login_challenges WHERE challenge_id = ?"
    )
    .bind(&payload.challenge_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let challenge = match challenge {
        Some(c) => c,
        None => return Err((StatusCode::BAD_REQUEST, "Invalid challenge ID".into())),
    };

    let parsed_hash = PasswordHash::new(&challenge.verification_code)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid hash".into()))?;

    if Argon2::default()
        .verify_password(payload.code.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err((StatusCode::BAD_REQUEST, "Invalid verification code".into()));
    }

    if Utc::now() > challenge.expires_at {
        return Err((StatusCode::BAD_REQUEST, "Verification code expired".into()));
    }

    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, full_name, email, hashed_password, role, created_at, updated_at FROM users WHERE id = ?"
    )
    .bind(&challenge.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user = user.ok_or((StatusCode::INTERNAL_SERVER_ERROR, "User not found".into()))?;

    // Delete challenge to prevent reuse
    sqlx::query("DELETE FROM login_challenges WHERE challenge_id = ?")
        .bind(&payload.challenge_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Generate JWT
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = Claims {
        sub: user.id,
        email: user.email,
        role: user.role,
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(Verify2FaResponse {
        access_token: token,
    }))
}

pub async fn get_latest_email_log(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let log = sqlx::query_as::<_, EmailLog>(
        "SELECT id, email, verification_code, created_at FROM email_logs ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match log {
        Some(l) => Ok(Json(json!({
            "email": l.email,
            "verification_code": l.verification_code
        }))),
        None => Err((StatusCode::NOT_FOUND, "No email logs found".into())),
    }
}
