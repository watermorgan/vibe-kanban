# TASK-010 Iteration Plan (Post-Gate3)

Date: 2026-02-09  
Gate: Gate3 (Verification & Release)  
Decision dependency: `docs/starbus/runs/TASK-010/06-audit.md`

## Why Iteration Is Required

The Gate3 audit is currently **NO-GO** because the ACCEPT rubric is missing and the available TASK-010 run history shows a state jump from `DESIGNING` to `DONE` without Gate2/Gate3 verification and release artifacts.

## Required Iteration Work (P0)

1. Restore/define ACCEPT rubric source.
- Add `docs/starbus/prompts/v2-i1-accept-prompt.md` (or document the canonical replacement path) so Gate3 is auditable against an explicit ACCEPT contract.

2. Restore canonical run artifacts into this worktree.
- Add or regenerate deterministic outputs under `docs/starbus/runs/TASK-010/` (at minimum `summary.md`, `result.json`, `events.ndjson`).
- Ensure file naming is stable so gate automation can consume them.

3. Provide Gate3 verification evidence bundle.
- E2E results (command, environment, pass/fail summary, artifact location).
- Quality checks (e.g., `pnpm run check`, `pnpm run lint`, `pnpm run backend:check`, `cargo test --workspace`) with pass/fail summaries.
- Performance benchmark results with baseline/threshold comparison.
- Security review/audit summary (e.g., dependency and vulnerability scan outcomes).

4. Provide release execution record.
- Deployment record (target environment, version/commit SHA, timestamps, outcome).
- Rollback plan and a tested rollback procedure.
- Post-deploy monitoring/alerting validation window and outcome.

5. Fix gate/state transition history.
- Ensure `events.ndjson` reflects valid transitions:
  `QUEUED -> DESIGNING -> AUDITING -> EXECUTING -> VERIFYING -> DONE`
- Include actor/role and rationale metadata for each transition (in-line or adjacent artifacts).

## Exit Criteria For Re-Audit

- ACCEPT prompt exists and is referenced by the Gate3 audit.
- Gate3 baseline checks are evidenced and pass.
- Transition chain is complete and consistent with allowed transitions.
- `06-audit.md` can issue **GO** with no unresolved P0 gaps.

