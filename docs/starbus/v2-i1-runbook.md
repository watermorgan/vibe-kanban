# V2-I1 Runbook

## Scope
Run one full iteration for Star-State-Bus integration on `vibe-kanban-v2`.

## Preconditions
1. `vibe-kanban` backend/frontend are running and API is reachable.
2. Project binding points to `/Users/weitao/workspace/apps/vibe-kanban`.
3. Tasks exist:
   - REQ
   - DEV
   - TEST
   - ACCEPT
4. Task descriptions reference `docs/starbus/prompts/*`.

## Execution Order
1. REQ phase:
   - Input: `docs/starbus/prompts/v2-i1-req-prompt.md`
   - Output: `docs/starbus/runs/TASK-010/01-clarify.md`, `02-design.md`
2. DEV phase:
   - Input: `docs/starbus/prompts/v2-i1-dev-prompt.md`
   - Output: `docs/starbus/runs/TASK-010/03-dev.md`, `collaborator.md`
3. TEST phase:
   - Input: `docs/starbus/prompts/v2-i1-test-prompt.md`
   - Output: `docs/starbus/runs/TASK-010/04-test.md`
4. ACCEPT phase:
   - Input: `docs/starbus/prompts/v2-i1-accept-prompt.md`
   - Output: `docs/starbus/runs/TASK-010/06-audit.md`, `05-iterate.md`

## Gate Checks
- Gate0: Clarification complete and scoped.
- Gate1: Design executable with API/UI boundaries.
- Gate2: Tests with evidence chain.
- Gate3: Audit decision PASS or NEEDS_REVISION.

## Dispatch References
- Task matrix: `docs/starbus/v2-i1-task-matrix.md`
- Role prompt pack: `docs/starbus/prompts/v2-i1-role-prompt-pack.md`

## Auto Dispatch API
Use backend auto-dispatch to avoid manual role/prompt wiring:

```bash
curl -X POST http://127.0.0.1:3005/api/starbus/dispatch \
  -H "Content-Type: application/json" \
  -d '{
    "task_id": "<task-uuid>",
    "actor": "ACTOR_CLAUDE",
    "set_active": true,
    "auto_start": true
  }'
```

Expected result:
- `prompt_path` points to `docs/starbus/runs/<task-id>/dispatch-prompt.md`
- StarBus state has `next_action` with mapped role/action
- Task status moves to mapped phase (`DESIGNING`/`EXECUTING`/`AUDITING`/`VERIFYING`)

## Failure Handling
- If any required output file is missing, do not proceed to next phase.
- If evidence is not reproducible, mark TEST as NEEDS_REVISION.
