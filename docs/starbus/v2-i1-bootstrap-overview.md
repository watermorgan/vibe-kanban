# V2 Iteration-1 Bootstrap Overview

## Goal
Run the first Star-State-Bus V2 integration loop on top of `vibe-kanban` using tool-bootstrapping:
`REQ -> DEV -> TEST -> ACCEPT`.

## Project
- Project name: `vibe-kanban-v2`
- Project id: `46419dc8-19dc-4d10-9930-58bd0ab3be8f`
- Repo binding: `/Users/weitao/workspace/apps/vibe-kanban`

## Active Task Set (P0)
1. `9a01b508-d520-48c1-8a12-1666983485cc` - `V2-I1-P0 REQ Gate Definition`
2. `5828d9d4-42de-461a-97ee-0a4dc8a0e159` - `V2-I1-P0 DEV FE+BE Integration`
3. `60acf2a1-31cd-4594-ab65-a235e287c1e0` - `V2-I1-P0 TEST Evidence Chain`
4. `9f601e87-44e5-42de-bb7b-01f5ac47f0ec` - `V2-I1-P0 ACCEPT Audit & Release Gate`

## Collaboration Contract
- Single source of truth for state transitions: backend API + DB.
- No direct writes to `state.json` from UI or role agents.
- Evidence must be stored to artifacts and linked in markdown reports.
- All role outputs must be markdown-first; no mandatory structured JSON except API payloads.
- Each phase produces one handoff file before moving to next phase.

## Phase Outputs
- REQ: `docs/starbus/runs/TASK-010/01-clarify.md`, `docs/starbus/runs/TASK-010/02-design.md`
- DEV: `docs/starbus/runs/TASK-010/03-dev.md`
- TEST: `docs/starbus/runs/TASK-010/04-test.md`
- ACCEPT: `docs/starbus/runs/TASK-010/06-audit.md`, `docs/starbus/runs/TASK-010/05-iterate.md`

## Pass Criteria
- P0 API flow works end-to-end from vibe-kanban UI:
  - task creation
  - decision resolve
  - auto-resume
  - events visibility
- Evidence chain is reproducible and linked.
- ACCEPT outputs either:
  - `PASS` with release note, or
  - `NEEDS_REVISION` with a minimal, executable revision list.
