# V2 Iteration-1 Key Rules Memory

## Core Rules
1. `vibe-kanban` is the execution surface, not the state authority.
2. Backend APIs own transitions and validation.
3. Role agents write markdown deliverables, not orchestration JSON.
4. Evidence must be file-based and referenceable.
5. Gate model is mandatory:
   - Gate0: requirement clarified
   - Gate1: design executable
   - Gate2: implementation and tests valid
   - Gate3: audit and release decision

## State and Routing
- Allowed actor/role progression is controlled by backend route policy.
- If a transition is invalid, do not force bypass; create a revision action.
- `BLOCKED_HUMAN` must include a clear decision question and options.

## Quality Bars
- REQ must define scope, out-of-scope, acceptance, dependencies.
- DEV must include API/UI mapping and SoT boundary proof.
- TEST must include failure-path validation and reproducible commands.
- ACCEPT must include consistency check: requirement vs code vs tests.

## Disallowed Practices
- No direct edits of runtime state files from frontend.
- No completion claims without evidence links.
- No silent assumption changes between phases.

