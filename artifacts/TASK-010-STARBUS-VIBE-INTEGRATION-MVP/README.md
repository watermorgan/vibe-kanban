# TASK-010 Evidence Index

## Build and compile checks

- Backend:
  - `cargo check -p server` -> PASS
  - `SQLX_OFFLINE=true cargo check -p server` -> PASS
- Frontend:
  - `pnpm frontend:check` -> PASS
  - `pnpm build` -> PASS

## API smoke checks

- `GET /api/starbus/state` -> PASS
- `POST /api/starbus/intake/preflight` with invalid payload -> PASS (returns expected validation errors)
- `POST /api/starbus/run-role-task` -> PASS (task/workspace started)
- `GET /api/starbus/runs/{task_id}` -> PASS

## Runtime IDs

- Integration task id: `944481b5-4f7b-4584-a57a-88883ba7341c`
- Integration workspace id: `67ae1cd9-a5f7-473a-9419-1769edb68da6`
- Path verification task id: `58fb3e5a-67ad-4360-a0d1-6e59ff80b4f0`
- Path verification workspace id: `bf422564-e11b-498b-9a5f-092b98619f8f`

## Key output files

- `docs/starbus/runs/TASK-010/03-dev.md`
- `docs/starbus/runs/TASK-010/04-test.md`
- `docs/starbus/runs/TASK-010/05-iterate.md`
- `docs/starbus/runs/TASK-010/06-audit.md`
- `docs/starbus/runs/TASK-010/collaborator.md`
