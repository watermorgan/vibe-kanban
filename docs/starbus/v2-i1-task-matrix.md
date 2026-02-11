# V2-I1 Task Matrix (REQ/FE/BE/TEST)

## Scope
This matrix is the execution baseline for `TASK-010-STARBUS-VIBE-INTEGRATION-MVP`.
It focuses on the first iteration after P0 bootstrap pass.

## Priority and Ownership
| ID | Priority | Domain | Task | Owner Role | Depends On | Status |
|---|---|---|---|---|---|---|
| I1-P0-01 | P0 | REQ+BE | Canonical run output binding (`TASK-010` alias -> stable output path) | role-product-manager + role-technology | None | TODO |
| I1-P0-02 | P0 | BE | Duplicate title protection for starbus task creation | role-technology | None | TODO |
| I1-P0-03 | P0 | BE | Active task hygiene (single active + deterministic resume) | role-technology | I1-P0-01 | TODO |
| I1-P0-04 | P0 | FE | Control room state rendering consistency (actor/role/status source) | role-project-ops | I1-P0-01 | TODO |
| I1-P0-05 | P0 | TEST | Runtime evidence chain for decision resolve -> auto resume | role-qa-security | I1-P0-02, I1-P0-03, I1-P0-04 | TODO |
| I1-P1-01 | P1 | FE+BE | One-click fan-out orchestration (REQ->DEV->TEST->ACCEPT) | role-technology + role-project-ops | I1-P0-05 | TODO |
| I1-P1-02 | P1 | FE | Evidence wall aggregation for run outputs and artifacts | role-project-ops | I1-P0-05 | TODO |
| I1-P1-03 | P1 | TEST | End-to-end contract tests for role-run pipelines | role-qa-security | I1-P1-01 | TODO |
| I1-P2-01 | P2 | BE | SQLx workspace prepare performance investigation | role-technology | None | TODO |

## Parallelization Plan
1. Parallel group A: `I1-P0-01`, `I1-P0-02`
2. After A: `I1-P0-03`, `I1-P0-04` in parallel
3. After A+B: `I1-P0-05`
4. Then P1 sequence: `I1-P1-01` -> (`I1-P1-02` in parallel with `I1-P1-03`)

## Gate Mapping
- Gate0: Requirement clarified and task boundaries frozen (`I1-P0-01` scope fixed)
- Gate1: Design executable with interface contract (`I1-P0-02`, `I1-P0-03` accepted design)
- Gate2: Implementation + tests pass with evidence chain (`I1-P0-04`, `I1-P0-05`)
- Gate3: Audit decision (`PASS` or `NEEDS_REVISION`) with next iteration list

## Required Outputs Per Task
- REQ tasks:
  - `docs/starbus/runs/TASK-010/01-clarify.md`
  - `docs/starbus/runs/TASK-010/02-design.md`
- DEV tasks:
  - `docs/starbus/runs/TASK-010/03-dev.md`
  - `docs/starbus/runs/TASK-010/collaborator.md`
- TEST tasks:
  - `docs/starbus/runs/TASK-010/04-test.md`
  - `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/README.md`
- ACCEPT tasks:
  - `docs/starbus/runs/TASK-010/06-audit.md`
  - `docs/starbus/runs/TASK-010/05-iterate.md`

## Definition of Done (Iteration)
1. No path mismatch between prompts, run outputs, and evidence files.
2. State transitions are valid and deterministic.
3. Control room shows the same status source as backend state.
4. Audit report includes explicit PASS/NEEDS_REVISION and references evidence files.
