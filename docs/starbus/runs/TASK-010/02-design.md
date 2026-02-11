# 02-design

Status: READY

## Architecture Decision
Use vibe-kanban task records for phase visibility while keeping transition validity in backend APIs.

## Data Flow
1. Role agent reads prompt from `docs/starbus/prompts/*`.
2. Role agent writes markdown output to `docs/starbus/runs/TASK-010/*`.
3. Role agent updates task status via API (`/api/tasks/{id}`) when phase is complete.
4. Evidence is indexed in `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/README.md`.
5. ACCEPT phase produces final decision and iteration plan.

## API Usage
- `GET /api/tasks?project_id=...` for board status.
- `PUT /api/tasks/{task_id}` for phase description/status updates.
- `POST /api/tasks` and `DELETE /api/tasks/{task_id}` for board maintenance.

## SoT Boundary
- Allowed write surface: backend API only.
- Forbidden: direct writes from UI to runtime state files.

## Failure Modes
- Missing output files: block next phase.
- Missing evidence links: TEST cannot pass.
- Path outside repo: reject and request correction.

## Implementation Mapping
- REQ task id: `9a01b508-d520-48c1-8a12-1666983485cc`
- DEV task id: `5828d9d4-42de-461a-97ee-0a4dc8a0e159`
- TEST task id: `60acf2a1-31cd-4594-ab65-a235e287c1e0`
- ACCEPT task id: `9f601e87-44e5-42de-bb7b-01f5ac47f0ec`

## Acceptance Baseline
- P0 phase loop completes once without path mismatch.
- All phase outputs exist and contain actionable content.
- Final ACCEPT output contains PASS or NEEDS_REVISION with concrete items.
