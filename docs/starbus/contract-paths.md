# StarBus Path Contract (V2-I1)

## Purpose
Keep role agents, vibe-kanban tasks, and workspace execution aligned to repo-local paths.

## Source of Truth
- All prompt and memory files must be read from this repository:
  - `docs/starbus/prompts/*.md`
  - `docs/starbus/v2-i1-*.md`
- Do not reference `~/workspace/docs/...` in task descriptions.

## Output Location Contract
- Iteration output root: `docs/starbus/runs/TASK-010/`
- Required phase files:
  - `01-clarify.md`
  - `02-design.md`
  - `03-dev.md`
  - `04-test.md`
  - `05-iterate.md`
  - `06-audit.md`
  - `collaborator.md`

## Evidence Location Contract
- Evidence index and command outputs:
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/README.md`
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/*`

## Validation Rule
- A task description is invalid if it references files outside this repo.
- Role agents should stop and request correction if path contract is violated.

