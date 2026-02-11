# 04-test

Status: PASS (P0 baseline)

## Test matrix

1. Backend compile path
- `cargo check -p server` -> PASS
- `SQLX_OFFLINE=true cargo check -p server` -> PASS

2. Frontend compile path
- `pnpm frontend:check` -> PASS
- `pnpm build` -> PASS

3. API smoke checks
- `GET /api/starbus/state` -> PASS
- `POST /api/starbus/intake/preflight` with invalid payload -> PASS
  - expected validation errors returned (`title`, `priority`, `include_recommended_deps`, overflow reason)
- `POST /api/starbus/run-role-task` -> PASS
  - task created and execution started
- `GET /api/starbus/runs/{task_id}` -> PASS
  - workspace and running process returned

4. Path strategy regression check
- Created `TASK-010-PATH-VERIFY-V2` via `run-role-task`.
- Verified skeleton files exist at:
  - `docs/starbus/runs/58fb3e5a-67ad-4360-a0d1-6e59ff80b4f0/task.md`
  - `docs/starbus/runs/58fb3e5a-67ad-4360-a0d1-6e59ff80b4f0/context.md`
  - `docs/starbus/runs/58fb3e5a-67ad-4360-a0d1-6e59ff80b4f0/playbook.md`

## Risk observations (non-blocking for this gate)

- Existing starbus global state still contains historical scratch tasks from earlier runs.
- Duplicate title tasks are possible; no dedup policy yet.
- End-to-end role output ingestion into canonical phase docs is still workflow-driven, not fully automated.

## Conclusion
- P0 integration baseline is now operational.
- Move to acceptance audit with explicit next-iteration backlog for orchestration hardening.
