$ErrorActionPreference = "Stop"

# 1. Seed users
$seedResult = Invoke-RestMethod -Uri "http://localhost:3000/seed/users" -Method Post
Write-Host "Seeded users."
$jamesBondId = $seedResult.james_bond_id

# 2. Login Admin
$adminLogin = Invoke-RestMethod -Uri "http://localhost:3000/auth/login" -Method Post -Body '{"email":"admin@example.com","password":"password123"}' -ContentType "application/json"
$adminChallengeId = $adminLogin.login_challenge_id

# 3. Get Admin 2FA
$adminEmailLog = Invoke-RestMethod -Uri "http://localhost:3000/dev/email-logs/latest" -Method Get
$adminCode = $adminEmailLog.verification_code

# 4. Verify Admin
$adminVerify = Invoke-RestMethod -Uri "http://localhost:3000/auth/verify-2fa" -Method Post -Body (@{challenge_id=$adminChallengeId; code=$adminCode} | ConvertTo-Json) -ContentType "application/json"
$adminToken = $adminVerify.access_token
$adminHeaders = @{ Authorization = "Bearer $adminToken" }
Write-Host "Admin logged in."

# 5. Create 5 Tasks with varying priorities
$priorities = @("high", "medium", "low", "high", "medium")
$taskIds = @()
for ($i = 1; $i -le 5; $i++) {
    $task = Invoke-RestMethod -Uri "http://localhost:3000/tasks" -Method Post -Body (@{title="Task $i"; description="Desc $i"; priority=$priorities[$i - 1]} | ConvertTo-Json) -ContentType "application/json" -Headers $adminHeaders
    $taskIds += $task.id
}
Write-Host "Created 5 tasks."

# 6. Assign 3 to James Bond
$assignedTaskIds = $taskIds[0..2]
Invoke-RestMethod -Uri "http://localhost:3000/tasks/assign" -Method Post -Body (@{task_ids=$assignedTaskIds; assigned_to_id=$jamesBondId} | ConvertTo-Json) -ContentType "application/json" -Headers $adminHeaders | Out-Null
Write-Host "Assigned 3 tasks to James Bond."

# 7. Login James Bond
$jamesLogin = Invoke-RestMethod -Uri "http://localhost:3000/auth/login" -Method Post -Body '{"email":"jamesbond@example.com","password":"password123"}' -ContentType "application/json"
$jamesChallengeId = $jamesLogin.login_challenge_id

$jamesEmailLog = Invoke-RestMethod -Uri "http://localhost:3000/dev/email-logs/latest" -Method Get
$jamesCode = $jamesEmailLog.verification_code

$jamesVerify = Invoke-RestMethod -Uri "http://localhost:3000/auth/verify-2fa" -Method Post -Body (@{challenge_id=$jamesChallengeId; code=$jamesCode} | ConvertTo-Json) -ContentType "application/json"
$jamesToken = $jamesVerify.access_token
$jamesHeaders = @{ Authorization = "Bearer $jamesToken" }
Write-Host "James Bond logged in."

# 8. James Bond attempts to create task (Should fail with 403 Forbidden)
try {
    Invoke-RestMethod -Uri "http://localhost:3000/tasks" -Method Post -Body (@{title="Secret Mission"} | ConvertTo-Json) -ContentType "application/json" -Headers $jamesHeaders
    Write-Host "ERROR: James Bond was able to create a task!"
} catch {
    Write-Host "Success: James Bond was forbidden from creating a task."
}

# 9. View my tasks (First time) - cache hit false
$view1 = Invoke-RestMethod -Uri "http://localhost:3000/tasks/view-my-tasks" -Method Get -Headers $jamesHeaders
Write-Host "First view-my-tasks cache hit: $($view1.cache.hit)"

# 10. View my tasks (Second time) - cache hit true
$view2 = Invoke-RestMethod -Uri "http://localhost:3000/tasks/view-my-tasks" -Method Get -Headers $jamesHeaders
Write-Host "Second view-my-tasks cache hit: $($view2.cache.hit)"

Write-Host "`nFinal Validation Response:`n"
$view2 | ConvertTo-Json -Depth 10 | Out-File "validation_output.json"
Write-Host (Get-Content "validation_output.json")
