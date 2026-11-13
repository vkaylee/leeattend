# Salary Setup Page Tasks

- [ ] Task 1: Permission, scope, and API contract gate
  - [ ] Confirm salary T2 exception and write grant.
  - [ ] Define employment-scoped detail/history/write/stop contract.
  - [ ] Decide whether schema migration is required.

- [ ] Task 2: Employment-scoped read/write backend path
  - [ ] Implement repository transaction/validation/scope/audit.
  - [ ] Add handlers/routes and structured errors.
  - [ ] Add unit and PostgreSQL integration tests.

- [ ] Checkpoint 1: Backend contract
  - [ ] Contract and grant gates approved.
  - [ ] API integration tests pass.

- [ ] Task 3: Guarded edit route and list entry links
  - [ ] Add `salary-management.$employmentId.edit.tsx` and lazy sibling.
  - [ ] Add permission guard and row action.
  - [ ] Preserve typed list search context.
  - [ ] Regenerate route tree.

- [ ] Task 4: Employment context and setup page shell
  - [ ] Load explicit employment context.
  - [ ] Add loading/error/empty/missing states.
  - [ ] Prevent empty/invalid employment queries.

- [ ] Checkpoint 2: Route/context
  - [ ] HR list → setup → back flow works.
  - [ ] Denied roles blocked.

- [ ] Task 5: Canonical assignment/edit/stop workflow
  - [ ] Reuse/refactor salary component history UI.
  - [ ] Add validation and temporal statuses.
  - [ ] Handle pending/error/409 and query invalidation.
  - [ ] Add focused tests.

- [ ] Task 6: Explicit unconfigured worklist/search
  - [ ] Add search/select path for missing salary configuration.
  - [ ] Resolve selected employee to explicit employment.
  - [ ] Keep default table assignment-only.

- [ ] Checkpoint 3: Complete setup flow
  - [ ] Add/change/stop, refresh, and history verified.
  - [ ] Failed save and 409 behavior verified.

- [ ] Task 7: Employee-detail summary/link consolidation
  - [ ] Remove duplicate salary write path after call-site review.
  - [ ] Preserve compensation history and payroll-config behavior.
  - [ ] Add return-context link.

- [ ] Task 8: i18n/accessibility/responsive/operational polish
  - [ ] Complete en/vi/zh translations.
  - [ ] Verify keyboard/focus/mobile/dark/error states.
  - [ ] Verify no sensitive data in URL/storage/logs.

- [ ] Checkpoint 4: Final acceptance
  - [ ] `./leedevkit test api`
  - [ ] `./leedevkit test web`
  - [ ] `./leedevkit test all`
  - [ ] `git diff --check`
  - [ ] Browser route × role matrix complete.
