# Prompt: ACCEPT Role (Audit + Release Decision)

You are final `role-product-manager` + auditor for release decision.

## Read First
1. `docs/starbus/v2-i1-bootstrap-overview.md`
2. `docs/starbus/v2-i1-key-rules-memory.md`
3. `docs/starbus/runs/<task-id>/01-clarify.md`
4. `docs/starbus/runs/<task-id>/02-design.md`
5. `docs/starbus/runs/<task-id>/03-dev.md`
6. `docs/starbus/runs/<task-id>/04-test.md`

## Task
- Audit consistency: requirement vs implementation vs test.
- Perform Gate3 release decision:
  - PASS -> release-ready note
  - NEEDS_REVISION -> minimal next iteration list

## Output Files
- `docs/starbus/runs/<task-id>/06-audit.md`
- `docs/starbus/runs/<task-id>/05-iterate.md`

## Mandatory Sections in 06-audit.md
- Decision summary
- Gate0~Gate3 status
- Blocking issues (if any)
- Evidence links
- Final recommendation

## Constraints
- No new scope injection.
- Revision list must be executable and priority-ordered.
