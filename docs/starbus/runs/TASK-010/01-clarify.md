# 01-clarify

Status: READY

## Goal
Integrate Star-State-Bus V2 workflow control into vibe-kanban so one iteration can run as `REQ -> DEV -> TEST -> ACCEPT` with backend as the only state authority.

## In Scope
- Use vibe-kanban tasks as visible phase carriers.
- Use backend APIs for task updates and phase transitions.
- Enforce path contract for role prompts and phase outputs.
- Store phase artifacts under repo-local `docs/starbus/runs/TASK-010` and evidence under `artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP`.

## Out of Scope
- New state machine engine.
- Multi-project orchestration.
- New actor runtime implementation.

## Gate Definitions
### Gate0 (Clarification)
- Scope confirmed.
- Output paths and evidence paths confirmed.
- Role prompts and rules are readable in repo.

### Gate1 (Design)
- API and UI integration path defined.
- Backend SoT boundary explicit.
- Failure handling and evidence contract explicit.

### Gate2 (Implementation + Test)
- DEV output completed with component/API mapping.
- TEST output completed with reproducible commands and evidence links.

### Gate3 (Accept)
- Requirement, implementation, and test consistency checked.
- Final decision: PASS or NEEDS_REVISION with minimal iteration list.

## Role Boundaries
- REQ: defines scope, gates, acceptance.
- DEV: implements FE+BE changes under SoT constraint.
- TEST: verifies success/failure paths and evidence chain.
- ACCEPT: performs release gate decision.

## Output Contract
- REQ output: `01-clarify.md`, `02-design.md`
- DEV output: `03-dev.md`, `collaborator.md`
- TEST output: `04-test.md`
- ACCEPT output: `06-audit.md`, `05-iterate.md`

## Open Questions
- None blocking for V2-I1.
