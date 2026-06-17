pub mod cache;
pub mod handlers;
pub mod middleware;
pub mod models;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use dotenvy::dotenv;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::cache::Cache;
use crate::middleware::auth::{AppState, require_auth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize env and tracing
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Connect to SQLite
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;

    // Connect to Redis
    let cache = Cache::new(&redis_url)?;

    let app_state = AppState {
        db,
        cache,
        jwt_secret,
    };

    let api_routes = app(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, api_routes).await?;

    Ok(())
}

pub fn app(app_state: AppState) -> Router {
    Router::new()
        .route("/seed/users", post(handlers::seed::seed_users))
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/verify-2fa", post(handlers::auth::verify_2fa))
        .route(
            "/dev/email-logs/latest",
            get(handlers::auth::get_latest_email_log),
        )
        .route(
            "/tasks",
            post(handlers::tasks::create_task)
                .route_layer(from_fn_with_state(app_state.clone(), require_auth)),
        )
        .route(
            "/tasks/assign",
            post(handlers::tasks::assign_tasks)
                .route_layer(from_fn_with_state(app_state.clone(), require_auth)),
        )
        .route(
            "/tasks/view-my-tasks",
            get(handlers::tasks::view_my_tasks)
                .route_layer(from_fn_with_state(app_state.clone(), require_auth)),
        )
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
