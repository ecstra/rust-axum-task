# AI Usage

This project was built primarily with the assistance of Antigravity, an AI agent coding assistant from Google.

## What AI was used for:
- Formulating the initial implementation plan and determining the suitable Rust crates to use (`axum`, `sqlx`, `argon2`, etc.).
- Writing the SQL schema and automated sqlx migrations.
- Generating the Serde models for request validation and response formatting.
- Constructing the API endpoints: login, 2FA challenge creation and verification, task creation and assignment, and the cached task viewing logic.
- Implementing the custom Axum JWT authentication middleware.
- Developing a PowerShell script (`test_flow.ps1`) to simulate the end-to-end verification flow autonomously.

## What was manually handled/changed:
- Setting up the initial `Cargo.toml` with base dependencies and verifying the target Redis environment (IP configuration).
- Adjusting to SQLite limitations and modifying the `sqlx` configurations to bypass compilation issues in absence of the `sqlx-cli`.
- Resolving the `chrono` feature flagging to integrate smoothly with SQLite mapping logic.
- Adding the required crypto providers to `jsonwebtoken`.
