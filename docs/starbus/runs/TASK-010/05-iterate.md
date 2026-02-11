# 05-iterate

Status: READY

## Next minimal tasks (decision-complete)

### I1-P0-01: Canonical run output binding
- Goal: ensure every `run-role-task` execution binds to one canonical run folder and updates phase files consistently.
- Scope:
  - introduce run alias (e.g. `TASK-010`) to task state metadata
  - map role output to canonical files (`01-clarify.md` ... `06-audit.md`)
  - keep raw per-task-id folder as evidence mirror
- Acceptance:
  - one end-to-end run updates all 6 phase docs under `docs/starbus/runs/TASK-010/`
  - raw run folder path is linked from `collaborator.md`
- Owner role: `role-project-ops`
- Evidence required:
  - updated phase docs
  - `GET /api/starbus/state` snapshot showing run binding metadata

### I1-P0-02: Duplicate title protection
- Goal: avoid accidental duplicate task creation for same project + title in active states.
- Scope:
  - add backend validation in `run-role-task` and `intake/create`
  - allow override flag for intentional duplicates
- Acceptance:
  - duplicate create without override returns 4xx with clear message
  - with override=true, create succeeds
- Owner role: `role-technology`
- Evidence required:
  - API request/response samples in markdown
  - regression test commands and outputs

### I1-P0-03: Active task hygiene
- Goal: keep global active task aligned with current project execution intent.
- Scope:
  - add API to set/clear active task explicitly
  - add stale-task cleanup endpoint (soft archive or inactive mark)
- Acceptance:
  - active task can be switched deterministically via API
  - old scratch tasks no longer pollute operator view by default
- Owner role: `role-product-manager`
- Evidence required:
  - before/after `GET /api/starbus/state` snapshots
  - Kanban screenshot showing clean active context

### I1-P1-01: One-click fan-out orchestration
- Goal: create REQ/DEV/TEST/ACCEPT sub-flow from one kickoff API call.
- Scope:
  - add orchestrated sequence endpoint or background workflow state machine
  - preserve HITL gate stops
- Acceptance:
  - one kickoff creates ordered subtasks and transitions through gates
  - gate failure pauses with decision payload
- Owner role: `role-product-manager`
- Evidence required:
- event timeline excerpt
- runbook update

### I1-P0-06: In-review contract reconciliation
- Goal: make V2-I1 in-review board strictly match StarBus artifact contract.
- Scope:
  - inspect all `inreview` tasks in project `46419dc8-19dc-4d10-9930-58bd0ab3be8f`
  - move non-ready items out of `inreview` or backfill missing required files
  - keep one-to-one mapping between board state and required run artifacts
- Acceptance:
  - `node scripts/check-starbus-inreview.js` reports `fail=0`
  - audit round status can be upgraded from `NEEDS_REVISION` to `APPROVE_NEXT`
- Owner role: `role-qa-security`
- Evidence required:
  - updated `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/inreview-integrity-report.json`
  - updated `06-audit.md` round closure note
