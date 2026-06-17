# Task Management API

A Rust backend API for a task management workflow. Features authentication (JWT), two-factor login via email verification code, role-based permissions (Admin/Staff), task assignment, and per-user caching.

## Tech Stack
- **Language:** Rust (Stable edition)
- **Web Framework:** Axum
- **Database:** SQLite (using `sqlx`)
- **Caching:** Redis
- **Auth:** JWT (`jsonwebtoken`) and password hashing (`argon2`)

## Prerequisites
- Rust and Cargo installed
- A running Redis server
  - **Windows Users**: You can host Redis on WSL2 or using Docker Desktop (`docker run --name my-redis -p 6379:6379 -d redis`). If you are running Redis in a VM or external Linux environment, take note of its IP address.
  - **Linux/macOS Users**: You can install Redis directly or via Docker.

## Setup
1. Clone the repository and navigate into it.
2. Initialize the environment:
   ```sh
   cp .env.example .env
   ```
   Ensure `REDIS_URL` in `.env` points to your running Redis instance. For example, if hosted locally via Docker, `redis://127.0.0.1:6379`. If hosted on a VM, change it to `redis://<VM_IP_ADDRESS>:6379`.
3. Create the SQLite database file:
   ```powershell
   New-Item -Path .\sqlite.db -ItemType File
   ```

## Running the Server
```sh
cargo run
```
The server will automatically run the SQLite migrations on startup and bind to `http://localhost:3000`.

## Testing & Validation Workflow
The project includes a robust end-to-end integration test written in PowerShell (`test_flow.ps1`) that sequentially executes the exact core workflow specified in the assignment. Additionally, `cargo test` runs unit-level assertions.

To perform the required validation flow locally using PowerShell:

1. **Seed Users**
   ```powershell
   Invoke-RestMethod -Uri "http://localhost:3000/seed/users" -Method Post
   ```
   *Creates `admin@example.com` and `jamesbond@example.com` with password `password123`.*

2. **Login and 2FA**
   ```powershell
   # Login to receive a challenge ID
   $login = Invoke-RestMethod -Uri "http://localhost:3000/auth/login" -Method Post -Body '{"email":"admin@example.com","password":"password123"}' -ContentType "application/json"
   
   # Retrieve code from dev logs
   $log = Invoke-RestMethod -Uri "http://localhost:3000/dev/email-logs/latest" -Method Get
   
   # Verify code and receive JWT
   Invoke-RestMethod -Uri "http://localhost:3000/auth/verify-2fa" -Method Post -Body (@{challenge_id=$login.login_challenge_id; code=$log.verification_code} | ConvertTo-Json) -ContentType "application/json"
   ```

3. **Validation Output**
   Follow the same steps for `jamesbond@example.com` to log in and get a JWT token. Then, fetch assigned tasks:
   ```powershell
   Invoke-RestMethod -Uri "http://localhost:3000/tasks/view-my-tasks" -Method Get -Headers @{ Authorization = "Bearer JAMES_BOND_TOKEN" }
   ```

### Final Validation Response:

```json
{
  "user": {
    "email": "jamesbond@example.com",
    "role": "staff"
  },
  "tasks": [
    {
      "id": "91e9d8b2-e247-4e08-869d-a9184b740fa8",
      "title": "Task 1",
      "status": "todo",
      "priority": "high",
      "assigned_to": "jamesbond@example.com"
    },
    {
      "id": "adb3c20c-f5c0-4202-aff7-c176f1f1a4d5",
      "title": "Task 2",
      "status": "todo",
      "priority": "medium",
      "assigned_to": "jamesbond@example.com"
    },
    {
      "id": "cec98f00-92e0-4923-aad3-bc717b76c2da",
      "title": "Task 3",
      "status": "todo",
      "priority": "low",
      "assigned_to": "jamesbond@example.com"
    }
  ],
  "summary": {
    "total_assigned_tasks": 3
  },
  "cache": {
    "hit": true
  }
}
```
