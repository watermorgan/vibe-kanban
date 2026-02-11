# V2-I1 Path Fix Log

## Problem
Execution workspace could not find prompt files because they were created outside repo (`~/workspace/docs/...`).

## Fix
- Copied V2-I1 prompt and rule docs into repo:
  - `docs/starbus/prompts/*.md`
  - `docs/starbus/v2-i1-*.md`
- Updated vibe-kanban task descriptions to reference repo-local paths.

## Updated Task IDs
- `9a01b508-d520-48c1-8a12-1666983485cc` (REQ)
- `5828d9d4-42de-461a-97ee-0a4dc8a0e159` (DEV)
- `60acf2a1-31cd-4594-ab65-a235e287c1e0` (TEST)
- `9f601e87-44e5-42de-bb7b-01f5ac47f0ec` (ACCEPT)

## Result
Role agents can now resolve prompts from the same repo/worktree context.
