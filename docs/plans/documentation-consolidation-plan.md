# Documentation Consolidation

Status: Completed

## Goal
Create one navigable, low-noise documentation system where active runtime truth, business docs, decisions, plans, and archive have clear ownership and AI loads only current context.

## Rules
- `project-structure.md`: preserve repository conventions and update affected consumers together.
- `coding-standards.md`: remove or archive replaced documentation; no stale active instructions.
- `change-management.md`: documentation/configuration changes stay reviewable and do not alter runtime contracts silently.
- `development-workflow.md`: verify links, commands, and behavior after cross-cutting changes.

## Scope
- Keep `README.md`, `AGENTS.md`, `CLAUDE.md`, and runtime configuration at root.
- Consolidate active technical, business, audit, operations, decisions, features, and plans under `docs/`.
- Retain historical material only under explicit `docs/archive/`; delete obsolete scratch/evidence with no current consumer.
- Preserve document content unless it is stale, contradictory, duplicated, or points to removed runtime.
- Do not change Go/API/UI behavior.

## Tasks
- [x] Inventory every tracked Markdown, text, and CSV file; classify as active truth, active plan, decision, business, audit, generated/reference, completed, superseded, or obsolete. Verify: classification table covers every tracked documentation file.
- [x] Define final tree and source-of-truth map in `docs/README.md`, `docs/source-of-truth.md`, and `docs/documentation-policy.md`. Verify: each domain has one canonical document and one navigation path.
- [x] Fix active contradictions first: invalid commands, stale paths, old runtime descriptions, duplicate architecture claims, and outdated timestamps. Verify: referenced paths/commands exist or are explicitly conditional.
- [x] Move `docs/business/`, `docs/audit/`, root plans/specs, and `.ai` active docs into approved `docs/` destinations. Update inbound links in one change. Verify: link scan finds no broken moved-path references.
- [x] Archive completed/superseded plans and ADRs under `docs/archive/` with status and replacement pointer; delete only confirmed obsolete scratch material. Verify: active tree contains no completed plan without archive status.
- [x] Add `docs/ai-context.md` as compact AI entry point: runtime stack, source-of-truth links, active plans, verification commands, and excluded archive paths. Verify: new agent can reach current architecture without scanning root plans.
- [x] Run documentation quality checks and review diff for accidental runtime/config changes. Verify: Markdown links, command/path checks, `git diff --check`, and repository status pass.

## Done When
- [x] One canonical documentation tree exists under `docs/`.
- [x] Root no longer contains scattered active plans/specs except approved project entry/config files.
- [x] `docs/domain/` and `.ai/` have explicit retained purpose or are migrated.
- [x] Active docs describe Go + React/Vite runtime only.
- [x] Archive content is clearly non-authoritative.
- [x] All moved links and verification commands resolve.
- [x] No application source, API contract, database schema, or deployment behavior changes.

## Approval Gate
Do not execute tasks until maintainer approves scope, especially moving `docs/domain/`, relocating `.ai/` content, and deleting obsolete documentation.
