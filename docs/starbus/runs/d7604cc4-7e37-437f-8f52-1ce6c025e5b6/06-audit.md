# 06-audit

Status: APPROVE_NEXT (with iteration backlog)

## Audit summary

- REQ gate artifacts exist and are coherent:
  - `01-clarify.md`
  - `02-design.md`
- DEV implementation covers P0 route surface and core guards.
- TEST baseline passed for compile and API smoke.
- Critical path fix (repo-local run output directory) has been implemented and verified.

## Quality checks

1. Contract consistency
- PASS: starbus endpoints are exposed under `/api/starbus/*`.
- PASS: frontend adapter uses those endpoints.

2. Safety/guard rails
- PASS: invalid transitions blocked by `is_valid_transition`.
- PASS: tool actors blocked from terminal status writes.
- PASS: blocked->resume behavior requires explicit or inferred gate path.

3. Traceability
- PASS: role prompts and runbook are present under `docs/starbus/`.
- PARTIAL: automatic phase-to-file binding still requires improvement (tracked in `05-iterate.md`).

## Decision

- `APPROVE_NEXT` for V2-I1 P0 baseline.
- Continue immediately with items in `05-iterate.md` (I1-P0-01, I1-P0-02, I1-P0-03).

---

## Round 2 audit (requirements vs code vs board)

Status: NEEDS_REVISION

### Scope

- Verify current V2-I1 board state against StarBus file contract.
- Validate that task dispatch/prompt contract is deterministic per task id.
- Check requirement-code consistency for mandatory phase artifacts.

### Evidence

- Integrity report:
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/inreview-integrity-report.json`
- Runtime check command:
  - `node scripts/check-starbus-inreview.js`
- Contract implementation:
  - `crates/server/src/routes/starbus.rs`
- Prompt pack and role prompts:
  - `docs/starbus/prompts/v2-i1-role-prompt-pack.md`
  - `docs/starbus/prompts/v2-i1-req-prompt.md`
  - `docs/starbus/prompts/v2-i1-dev-prompt.md`
  - `docs/starbus/prompts/v2-i1-test-prompt.md`
  - `docs/starbus/prompts/v2-i1-accept-prompt.md`

### Findings

1. In-review artifact completeness is not yet met for the current V2-I1 set.
   - Current check result: `inreview=13 pass=1 fail=12`.
   - Multiple in-review tasks still miss one or more required files:
     - `task.md`, `context.md`, `playbook.md`, `03-dev.md`, `04-test.md`, `06-audit.md`, `handoff.md`.

2. Dispatch contract is corrected for new runs.
   - Backend dispatch prompt now binds mandatory outputs to task-scoped run path.
   - Prompt templates now use placeholders (`<task-id>`, `<task-artifact-key>`) instead of hardcoded `TASK-010`.

3. Existing in-review tasks created before the contract fix still violate the new contract and require cleanup/backfill.

### Decision

- `NEEDS_REVISION` for this review round.

### Required actions (gate to close this round)

1. Reconcile all V2-I1 in-review tasks:
   - Either move non-ready tasks out of `inreview`, or backfill all required artifacts.
2. Re-run integrity checker until:
   - `fail=0` for V2-I1 in-review tasks.
3. Append one final PASS snapshot to artifacts:
   - Updated `inreview-integrity-report.json` with `pass_count == total_inreview`.

### Round 2 closure

Status: APPROVE_NEXT

- Reconciliation applied:
  - non-ready tasks were moved out of `inreview`
  - active execution task kept in `inprogress`
- Latest integrity result:
  - `inreview=1 pass=1 fail=0`
- Evidence:
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/inreview-integrity-report.json`
