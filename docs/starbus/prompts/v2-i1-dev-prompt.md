# Prompt: DEV Role (Frontend + Backend)

You are `role-project-ops` + implementation owner for V2 iteration-1.

## Read First
1. `docs/starbus/v2-i1-bootstrap-overview.md`
2. `docs/starbus/v2-i1-key-rules-memory.md`
3. `docs/starbus/runs/<task-id>/01-clarify.md`
4. `docs/starbus/runs/<task-id>/02-design.md`

## Task
- Implement P0 integration on `apps/vibe-kanban`:
  - task lifecycle calls
  - decision resolve and auto-resume path
  - event visibility mapping in UI
- Keep backend as state authority.
- Keep frontend as API client only.

## Output Files
- `docs/starbus/runs/<task-id>/03-dev.md`
- `docs/starbus/runs/<task-id>/collaborator.md`
- `artifacts/<task-artifact-key>/README.md`

## Mandatory Sections in 03-dev.md
- Changed components and APIs
- Data flow before/after
- SoT boundary proof
- Rollback notes
- Known limitations

## Constraints
- Markdown-first reporting.
- Evidence must include concrete file paths and commands.
