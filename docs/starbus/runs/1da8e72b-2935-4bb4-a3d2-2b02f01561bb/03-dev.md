# 03-dev

Status: DONE

## Scope implemented (P0)

1. Starbus backend routes and guards
- Added/extended endpoints in `crates/server/src/routes/starbus.rs`:
  - `GET /api/starbus/state`
  - `POST /api/starbus/intake/preflight`
  - `POST /api/starbus/intake/create`
  - `POST /api/starbus/run-role-task`
  - `GET /api/starbus/runs/{task_id}`
  - `POST /api/starbus/handoff`
  - `POST /api/starbus/state/next_action`
  - `POST /api/starbus/state/transition`
  - `POST /api/starbus/state/decision/resolve`
- Enforced tool actor completion guard: tool actors cannot set `DONE/FAILED`.
- Enforced transition guard with explicit `BLOCKED_HUMAN` gate binding.

2. Intake + task skeleton atomic creation
- `intake_create` now creates DB task + starbus scratch + skeleton files as one flow.
- On persistence failure, rollback is executed (task removal + directory cleanup).

3. Decision resolve + auto-resume
- `resolve_decision` supports:
  - explicit `resume_status`
  - automatic resume when all decisions resolved and status is `BLOCKED_HUMAN`
  - inferred resume status by `Gate0..Gate3`

4. Frontend API adapter integration
- Added starbus API client in `frontend/src/lib/api.ts` (`starbusApi`).
- Hooked project/task mutation flow for starbus-oriented create/start pathway:
  - `frontend/src/hooks/useProjectMutations.ts`
  - `frontend/src/components/projects/ProjectCard.tsx`

5. Path strategy fix (critical)
- Fixed starbus skeleton/handoff path resolution:
  - output now defaults to repo-local `docs/starbus/runs/<task_id>/`
  - avoids writing to unrelated parent workspace paths.

## Evidence
- Key backend file: `crates/server/src/routes/starbus.rs`
- Key frontend files:
  - `frontend/src/lib/api.ts`
  - `frontend/src/hooks/useProjectMutations.ts`
  - `frontend/src/components/projects/ProjectCard.tsx`

## Known limits kept for next iteration
- No built-in dedup for same-title task creation.
- No automatic phase fan-out (REQ -> DEV -> TEST -> ACCEPT) from one command.
