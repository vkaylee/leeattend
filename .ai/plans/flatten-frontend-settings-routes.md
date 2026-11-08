# Flatten Frontend Settings Routes

Status: Completed

## Goal
Flatten frontend settings routes from `/workspace/settings/*` to `/settings/*` in TanStack Router, aligning frontend URLs with the standalone single-organization architecture.

## Constraints and Interfaces
- Pathless layout `_workspace.tsx` remains for authenticated application shell.
- Relocate `src/routes/_authenticated/_workspace/workspace/settings.tsx` -> `src/routes/_authenticated/_workspace/settings.tsx`.
- Relocate `src/routes/_authenticated/_workspace/workspace/settings/*` -> `src/routes/_authenticated/_workspace/settings/*`.
- Update all `createFileRoute` and `createLazyFileRoute` route IDs to `/_authenticated/_workspace/settings/*`.
- Retain backwards-compatibility redirect in `src/routes/_authenticated/_workspace/workspace/people.tsx` to `/settings/members`.
- Update `webdashboard/src/lib/routes.ts` `ROUTES.SETTINGS` values from `/workspace/settings/*` to `/settings/*`.
- Update all layout focus-mode checks, navigation hooks, and UI link call-sites.
- Verify with 100% passing tests via `./leedevkit test web --unit-only` and `./leedevkit test api`.

## Tasks
- [x] 1. Create plan file `.ai/plans/flatten-frontend-settings-routes.md`.
- [x] 2. Move route files from `workspace/settings` to `settings` and update `createFileRoute` / `createLazyFileRoute` definitions.
- [x] 3. Update `ROUTES.SETTINGS` in `webdashboard/src/lib/routes.ts` and UI call-sites across components, hooks, and pages.
- [x] 4. Update unit test assertions matching new `/settings/*` paths.
- [x] 5. Run `./leedevkit test web --unit-only` and `./leedevkit test api` to verify clean pass.
