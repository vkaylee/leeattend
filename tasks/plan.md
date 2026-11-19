# Remove obsolete SaaS workspace surfaces

## Context

LeeAttend now uses one implicit organization, not SaaS workspaces. Active schema/runtime already preserve `organization_settings` and valid employer/department/employment scopes. Cleanup must remove stale workspace/tenant selectors, docs, tooling, and contradictory contracts without deleting required async infrastructure.

## Scope

Remove:

- Active workspace/tenant selectors, routes, claims, headers, browser state, SQL predicates, and compatibility aliases.
- Stale tenant migration/tooling/config references.
- Contradictory workspace language in infrastructure job contracts.
- Stale Rust/distributed-worker documentation that does not describe current Go runtime.
- Misleading UI/docs wording such as workspace selection and old `/api/v1/workspaces/:id/...` routes.

Preserve:

- Singleton `organization_settings` and organization/employer/department/employment authorization/config scopes.
- PostgreSQL `outbox_jobs`, lease/fencing, retry/dead-letter behavior, and `ScheduleOfficeWorker`.
- Redis auth token/revocation/limiter services, report Pub/Sub/transport, and required import/report producers.
- Worker monitor, heartbeats, durable job ledger, idempotency, and negative tests proving workspace rejection/non-exposure.
- Historical ADRs, migration evidence, fixtures, and tracked backups unless separately approved for deletion.

## Dependency graph

```text
ADR-013 / ADR-0005 + schema contract
  -> single-org schema/bootstrap truth
  -> active API, RBAC, repository SQL, frontend

schedule lifecycle
  -> PostgreSQL outbox_jobs
  -> ScheduleOfficeWorker
  -> schedule publication/evidence

report/import APIs
  -> Redis transport + metadata/PubSub
  -> external or future worker contract

Redis
  -> auth token/revocation/limiters
  -> report/import transport
  -> monitor/heartbeat signals
```

## Vertical tasks

### T0 — Inventory and baseline

- Capture current git status; preserve existing user changes.
- Run `./leedevkit --help` and `./leedevkit test --help`.
- Inventory tracked active, historical, generated, and untracked workspace references.
- Classify each match: remove, rewrite, preserve valid scope, preserve historical, or ignore technical `/workspace` mounts/WebSocket `ws`.

Acceptance: inventory complete; queue/Redis not marked removable merely because workspaces are gone.

Checkpoint: review classification before edits.

### T1 — Contract and schema truth

- Verify `apiserver/migrations/testdata/schema_contract.sql` and baseline catalog already reject workspace objects.
- Verify `infra_job_records`/`infra_job_events` are workspace-free.
- Update `docs/decisions/ADR-018-infrastructure-job-truth-and-reconciliation.md` to remove workspace ID and tenant-scoped requirements while preserving durable PostgreSQL ledger, leases, retries, reconciliation, idempotency, and fencing.
- Do not alter schema or queue behavior in this slice.

Acceptance: schema catalog unchanged; docs match singleton organization architecture.

### T2 — Active runtime/API cleanup

- Search Go routes, DTOs, auth claims, middleware, browser state, and repository SQL.
- Remove confirmed client-selectable workspace scope only.
- Preserve flat `/api/v1/...` routes, organization bootstrap, valid business scopes, fail-closed authorization, and old-route rejection tests.
- Re-check report worker path before touching it; graph data may be stale.

Acceptance: no active client workspace selector/header/query/JWT/storage contract; canonical routes and authorization remain intact.

### T3 — Queue boundary decision

- Prove call sites for `apiserver/internal/queue.Worker`.
- Remove it only if repository/build/CI checks prove no runtime or external contract depends on it; update tests/docs together.
- Do not remove `outbox.go`, `ScheduleOfficeWorker`, Redis client/keys, report/import producers, or monitor contracts.
- Treat absent Redis consumers as a separate behavior migration, not workspace cleanup.

Acceptance: no dead generic queue code, or explicit evidence explains why it remains.

### T4 — Documentation and config cleanup

- Rewrite `docs/operations/background-worker-runner.md` to current Go/PostgreSQL/Redis behavior, or clearly mark obsolete material historical.
- Update `docs/features/ot-claim-workflow.md` to flat routes.
- Replace workspace-configured timezone wording in `docs/features/time-tracking-refactor-phase-1.md` with organization settings.
- Remove obsolete `.coveragerc` path only after proving migration script is absent/unused.
- Update `scripts/security_audit.sh` only after active-path inventory; retain auth, revocation, sensitive-log, and forbidden-selector checks.
- Keep historical migration Rust/workspace fixtures and tracked tenant backups unless separately approved.

Acceptance: active docs/tooling do not instruct workspace or obsolete Rust architecture; historical records remain clearly historical.

### T5 — Generated artifacts and frontend wording

- Remove misleading selectable/create workspace UI copy while preserving organization-wide semantics.
- Do not hand-edit generated route cache; remove/regenerate tracked `.tanstack/tmp` only after reference/build proof.
- Preserve negative tests and accessibility/i18n behavior.

Acceptance: no active UI workspace selector flow; generated output matches source.

### T6 — Verification

Run focused checks, then full gates:

- Schema/migration contract and disposable DB checks.
- Outbox claim/lease/retry/dead-letter/fencing tests.
- Schedule worker tests.
- Report/import/Redis transport, auth revocation, login limiter, and rate-limit tests.
- Monitor tests and security audit.
- Active-source grep with documented allowlist.
- `./leedevkit test api`
- `./leedevkit test all`

Inspect per-phase logs. Do not rely on trailing green summary.

## Test impact matrix

| Dimension | Required protection |
|---|---|
| Changed behavior | Workspace selectors/routes disappear; flat routes and singleton organization remain usable. |
| Existing behavior | RBAC, organization config, schedule publication, report export, Redis auth services remain. |
| Boundaries | Old route, forged selector, stale DB, duplicate org row, valid scoped config. |
| State | Fresh DB, stale/partial markers, active/expired lease, completed/dead-letter job. |
| Failures | Redis outage, publisher failure, lease loss, retry exhaustion, migration failure. |
| Security | No client scope selection; fail-closed; no PII/secrets in job docs/payloads/logs. |
| Compatibility | Historical evidence preserved; active consumers updated together; no old-schema upgrade assumed. |
| Operations | Heartbeats, monitor freshness, queue recovery, migration recovery messages. |

## Non-goals

- Do not remove PostgreSQL outbox, Redis, schedule/report worker semantics, retries, leases, heartbeats, or dead-letter handling.
- Do not remove organization/employer/department/employment scope.
- Do not rewrite Git history or silently delete backups/historical fixtures.
- Do not modify `.leedevkit/`.

---

# Implementation Plan: Employee Create Page and User-Side Linking

## Context

Current employee creation opens `EmployeeFormModal` from `/employees`. Form also loads organization members and sends `user_id`, mixing HR master data with account identity. Current backend create contract does not support `user_id`; update supports link/unlink from employee API. This creates a misleading UI and contract drift.

Target: dedicated `/employees/new` page for HR employee creation, one page without wizard or user selector. Account linking becomes User/Member-side lifecycle. Existing employee links remain intact; no mass unlink. Non-create dialogs remain dialogs.

Existing `modal-forms-to-pages-plan.md` excludes employee work. This plan adds employee scope only; do not silently expand to contracts, leave, or attendance-adjustment migrations.

## Locked architecture decisions

- **Create UX:** `/employees/new`, one responsive page, no modal, no wizard, no draft persistence in first slice.
- **Employee form data:** name, employee number, hire date, employer, location, department where supported, active state. No organization-member query or `user_id` field.
- **Account lifecycle:** Employee page shows linked-account status read-only. Link/unlink starts from Users/Members UI.
- **Server boundary:** Remove identity-link mutation from employee create/update contract. Add member-scoped link/unlink API with explicit `members:employee:link` permission, organization-scope validation, conflict handling, transaction, and audit evidence.
- **Conflict policy:** No force-transfer endpoint in first release. If employee or user already has another link, return `409`; operator explicitly unlinks old association, then links new one. This prevents silent account theft and keeps audit history clear.
- **Compatibility:** Preserve existing `employees.user_id` values and read responses. Never unlink existing accounts during migration. Existing clients sending removed mutation fields receive validation failure; current frontend is migrated in same release. Keep terminate, rehire, transfer, import, and detail-drawer dialogs unchanged.
- **Authorization:** Client route guards improve UX only. API remains authoritative; fail closed. Add route × role evidence for `employee`, `team_lead`, `shift_manager`, `hr_manager`, and `admin`.
- **Lifecycle:** Employee create remains immediately `active` under current backend behavior. Do not introduce draft/pending status in this change.

## Dependency graph

```text
API contract + permission decision
  ├── employee create/update server contract and tests
  │     └── /employees/new page + form + route tests
  └── member link/unlink contract + transaction + tests
        └── Users/Members link UI + cache invalidation

Existing employee list/detail
  └── navigation to /employees/new and read-only account status

Generated OpenAPI/types + TanStack route tree
  └── frontend typecheck/build/tests
```

## Vertical task slices

### Phase 0: Baseline and contract freeze

#### Task 0 — Capture baseline and route/role matrix

**Description:** Record current tree, focused test baseline, create/edit modal call sites, generated API contract, existing member permissions, and direct-route behavior. Freeze return context and authorization expectations before code changes.

**Acceptance criteria:**
- [ ] Matrix covers `/employees`, `/employees/new`, employee detail, Members page, allowed/denied actions for all five documented roles.
- [ ] Matrix distinguishes expected redirect/403 from unexpected 404/5xx/crash/wrong navigation.
- [ ] Return context is allowlisted (`/employees` or employee detail), never arbitrary URL or PII-bearing form state.
- [ ] Decision recorded: no user selector on employee create; member-side link/unlink is separate contract.

**Verification:**
- [ ] Inspect current diff and preserve unrelated worktree changes.
- [ ] Run focused existing web employee/member tests and record result.
- [ ] Run `./leedevkit --help` and `./leedevkit test --help` if baseline not already recorded.

**Dependencies:** None

**Files likely touched:**
- `tasks/employee-create-route-role-matrix.md`
- Existing employee/member test files only for baseline notes

**Estimated scope:** Small

### Checkpoint A — Contract approval

- [ ] Human reviews route names, permission names, link conflict behavior, and return-context contract.
- [ ] No UI implementation starts while `user_id` ownership remains ambiguous.

#### Task 1 — Close employee identity boundary in API

**Description:** Make employee create/update HR-only. Remove `user_id` and `force_transfer` from employee mutation DTOs and generated contract, reject or safely handle stale clients according to API compatibility policy, and extract identity reassignment into the member-side service boundary. Preserve nullable `user_id` in read responses.

**Acceptance criteria:**
- [ ] Employee create payload contains no `user_id`; employee update cannot link, unlink, or force-transfer an account.
- [ ] Existing employee records and read endpoints still expose account-link status without raw email/PII leakage.
- [ ] Stale/unknown identity mutation behavior is explicit and tested; no silent dropped field.
- [ ] OpenAPI, Go DTOs, generated TypeScript types, and frontend request aliases agree.

**Verification:**
- [ ] API tests cover create/update authorization, payload decoding, and absence of identity side effects.
- [ ] Contract generation/parity and frontend typecheck pass.
- [ ] Repository tests prove existing `user_id` is unchanged by employee HR updates.

**Dependencies:** Task 0

**Files likely touched:**
- `apiserver/internal/api/employees.go`
- `apiserver/internal/repository/employees.go`
- `apiserver/internal/api/employees_test.go`
- `apiserver/openapi.yaml`
- `webdashboard/src/types/api-generated.ts` (regenerated, never hand-edited)

**Estimated scope:** Medium

### Phase 1: Employee create page

#### Task 2 — Ship `/employees/new` complete create path

**Description:** Extract create form state/mutation from `EmployeeFormModal` into a page-level form using existing `EmployeeFormInputs` or a smaller shared form component. Add TanStack route/lazy pair, back/cancel actions, loading/error states, dirty navigation blocker, inline validation, and success redirect. Remove member lookup and `user_id` payload from create.

**Acceptance criteria:**
- [ ] Add button navigates to `/employees/new`; no create modal opens.
- [ ] Direct open and refresh work for authorized users; unauthorized users fail closed.
- [ ] Required fields, duplicate-submit prevention, pending state, API error preservation, and accessible labels/focus states work.
- [ ] Successful create invalidates employee queries and returns to `/employees` with supported list context preserved.
- [ ] `enrollment_id` prefill remains supported without putting full form data or PII in URL.

**Verification:**
- [ ] Page tests cover render, validation, submit success, submit failure, pending, cancel, direct route, and dirty Stay/Leave.
- [ ] Employee list test asserts navigation rather than modal visibility.
- [ ] Run `./leedevkit test web --unit-only --pattern 'employee|Employees'` during iteration.

**Dependencies:** Task 1

**Files likely touched:**
- `webdashboard/src/routes/_authenticated/_organization/employees.new.tsx`
- `webdashboard/src/routes/_authenticated/_organization/employees.new.lazy.tsx`
- `webdashboard/src/pages/employees/employee-create-page.tsx`
- `webdashboard/src/components/features/employees/employee-form-inputs.tsx`
- `webdashboard/src/pages/employees/components/employees-page-header.tsx`
- `webdashboard/src/pages/__tests__/employees-page.test.tsx`
- New page/form tests

**Estimated scope:** Large; split implementation if form extraction exceeds one focused session.

#### Task 3 — Preserve employee detail account status without mutation

**Description:** Remove account selector from employee edit/create presentation and show linked-account state read-only in employee detail. Keep current edit behavior for HR fields unless separately approved; no employee screen may call member-link mutation.

**Acceptance criteria:**
- [ ] Employee create/edit UI has no user/member combobox or link/unlink action.
- [ ] Linked/unlinked status is visible, accessible, and does not expose unauthorized email/PII.
- [ ] Existing detail drawer, terminate, rehire, transfer, and import actions remain unchanged.

**Verification:**
- [ ] Update form/detail tests to assert selector absence and read-only status.
- [ ] Search confirms no employee UI calls organization-member mutation for linking.
- [ ] Run focused employee tests.

**Dependencies:** Task 2

**Files likely touched:**
- `webdashboard/src/components/features/employees/employee-form-modal.tsx` (only if edit remains temporarily modal-backed)
- `webdashboard/src/components/features/employees/employee-form-inputs.tsx`
- `webdashboard/src/pages/employees/employee-detail-page.tsx`
- Employee overlay/form tests

**Estimated scope:** Medium

### Checkpoint B — Employee path

- [ ] `/employees/new` works by direct URL and from Add button.
- [ ] Employee create has no account-link controls or side effects.
- [ ] Existing list/detail and non-create dialogs regressions pass.
- [ ] Human reviews page UX before member-link work.

### Phase 2: User-side account linking

#### Task 4 — Add member-scoped link/unlink API

**Description:** Add explicit User/Member lifecycle endpoints: `PUT /api/v1/members/{user_id}/employee` with `{ employee_id }`, and `DELETE /api/v1/members/{user_id}/employee`. Use dedicated `members:employee:link` permission. No force-transfer in first release: any existing link returns `409`, requiring explicit unlink before relink. Candidate search/list stays bounded and organization-scoped.

**Acceptance criteria:**
- [ ] Server verifies actor, member, employee, and organization scope.
- [ ] Link, unlink, already-linked conflict, missing employee, deleted user, and cross-organization IDs have deterministic error shapes.
- [ ] Link/unlink and tenure identity update are atomic; no ignored transaction errors; concurrent reassignment cannot create two active links.
- [ ] Audit records identify actor/action without raw PII. Existing links remain untouched until explicit mutation.

**Verification:**
- [ ] API/repository integration tests cover success, 401/403, 404, 409, rollback, idempotent repeat, concurrent conflict, and cross-scope rejection.
- [ ] Database constraint/transaction evidence proves one active employee link per user and one user per employee.
- [ ] Run `./leedevkit test api --unit-only` then `./leedevkit test api`.

**Dependencies:** Task 1

**Files likely touched:**
- `apiserver/internal/api/organization_rbac.go` or dedicated member-link handler
- `apiserver/internal/repository/organization_rbac.go` or dedicated employee-link repository
- `apiserver/routes_organization.go`
- `apiserver/openapi.yaml`
- API/repository integration tests
- Additive migration only if current constraints cannot enforce invariant

**Estimated scope:** Large; keep contract and transaction implementation in one cohesive slice.

#### Task 5 — Add link/unlink UI on Members page

**Description:** Add member-side action to search/select an eligible employee, link, unlink, and explicitly confirm force transfer. Do not use employee page state or member mutation endpoints for hidden side effects.

**Acceptance criteria:**
- [ ] Users/Members is the only UI entry point for link/unlink.
- [ ] Current employee association, unlinked state, candidate loading/empty/error, conflict, and pending states are visible and accessible.
- [ ] Success invalidates member and employee queries; failure preserves selection and never silently steals a link.
- [ ] `members:employee:link` gates link/unlink action; unauthorized roles cannot use UI or API.

**Verification:**
- [ ] Member table/page tests cover link, unlink, conflict confirmation, API failure, and unauthorized UI.
- [ ] Contract tests assert exact endpoint/method/payload and no raw PII in logs/artifacts.
- [ ] Browser smoke checks Members page at 1920x1080 and mobile viewport after reading dev-account rules.

**Dependencies:** Task 4

**Files likely touched:**
- `webdashboard/src/pages/members-page.tsx`
- `webdashboard/src/components/members/member-table.tsx`
- `webdashboard/src/components/members/employee-link-control.tsx`
- `webdashboard/src/lib/api/organization.ts`
- `webdashboard/src/lib/api/employees.ts`
- `webdashboard/src/types/organization.ts`
- Member tests and i18n locale files

**Estimated scope:** Medium

### Checkpoint C — Identity path

- [ ] Link/unlink works only from Members UI.
- [ ] Conflict requires explicit user action; no silent reassignment.
- [ ] Self-service/attendance identity consumers still resolve linked employee correctly.
- [ ] All API negative/security tests pass.

### Phase 3: Cleanup and release verification

#### Task 6 — Remove obsolete create/link wiring

**Description:** Delete create-modal state/imports and member lookup code after exhaustive call-site search. Keep only dialogs still used for non-create actions. Update translations, tests, route tree through generator, and source plan scope.

**Acceptance criteria:**
- [ ] No Add Employee path opens `EmployeeFormModal`.
- [ ] No employee form loads organization members solely for linking.
- [ ] No dead imports, obsolete modal state, or stale `user_id` create payload remain.
- [ ] Generated route tree/types match source; no hand-edited generated artifacts.

**Verification:**
- [ ] Search `EmployeeFormModal`, `isModalOpen`, `setIsModalOpen`, `fetchOrganizationMembers`, and `user_id` call sites; every remaining hit has documented purpose.
- [ ] Run frontend lint/typecheck and focused tests.
- [ ] Run `./leedevkit test web </dev/null`.

**Dependencies:** Tasks 2–5

**Files likely touched:**
- `webdashboard/src/pages/employees/components/employee-overlays.tsx`
- `webdashboard/src/pages/employees/hooks/use-employees-modals.ts`
- `webdashboard/src/pages/employees/components/employee-modals.tsx`
- `webdashboard/src/components/features/employees/employee-form-modal.tsx` (delete only after call-site proof)
- Existing plan/spec and locale files

**Estimated scope:** Medium

#### Task 7 — Full verification and route audit

**Description:** Verify API, frontend, integration, and UI behavior against project gates. Record evidence, failures, skipped checks, and remaining risks.

**Acceptance criteria:**
- [ ] Focused tests, full web target, API target, and applicable all-target gate run without unit-only substitution.
- [ ] Route × role evidence covers allowed/denied create and member-link actions, direct URL, refresh, Back/Cancel, dirty form, validation, 403/404/409/5xx, console, and network.
- [ ] Desktop-large viewport uses 1920x1080; mobile smoke covers responsive layout.
- [ ] No credentials, tokens, raw emails, phone numbers, or screenshots with PII enter logs/artifacts.

**Verification:**
- [ ] `./leedevkit test web --unit-only` for iteration.
- [ ] `./leedevkit test web </dev/null`.
- [ ] `./leedevkit test api </dev/null`.
- [ ] `./leedevkit test all </dev/null` when API/OpenAPI/DB contract changes.
- [ ] Read per-phase `.test_logs/` results; do not trust trailing green summary alone.

**Dependencies:** Task 6

**Files likely touched:**
- `tasks/employee-create-route-role-matrix.md`
- `.test_logs/` outputs (do not commit generated logs unless repository policy requires)

**Estimated scope:** Medium

### Checkpoint D — Definition of Done

- [ ] Employee create is page-based, one page, no user selector.
- [ ] User linking is user-side, explicit, permissioned, atomic, audited.
- [ ] Existing employee links and non-create flows preserved.
- [ ] Full applicable test and UI evidence recorded.

## Test impact matrix

| Dimension | Required protection |
|---|---|
| Changed behavior | Add Employee navigates to page; account linking moves to Members. |
| Existing behavior | Employee list/detail, HR edits, imports, terminate/rehire/transfer, self-service identity remain usable. |
| Inputs | Empty/trimmed fields, duplicate employee number, invalid UUID/date, absent employer/location, stale enrollment prefill, empty candidate search. |
| State | Initial, loading, dirty, pending, stale refetch, existing link, unlinked, conflict, concurrent reassignment. |
| Failures | 401, 403, 404, 409, 422, 5xx, network timeout, transaction rollback, duplicate submit. |
| Security | Cross-organization employee/user IDs, unauthorized route/API, forced transfer, PII in URL/logs/artifacts. |
| Compatibility | Existing `employees.user_id` rows, generated OpenAPI/types, old `/employees` bookmarks, current modal-only non-create actions. |
| Operations | Audit event, structured error, cache invalidation, no silent retry that can duplicate/create or relink. |

## Open questions for Checkpoint A

- Confirm exact permission name for member-side link (`members:employee:link` recommended) versus existing member-management permission.
- Confirm forced transfer policy: reject conflict always, or allow only explicit permission + confirmation.
- Confirm whether employee create should remain immediately `active` or introduce `draft/pending` status in a separate HRM lifecycle project. This plan does not add draft status.

---

# Implementation Plan: Typed UserID Internal Boundary

## Context

Authenticated user identity currently crosses JWT, HTTP context, authorization, handlers, repositories, and tests as raw `uuid.UUID`. This permits accidental mixing with employee, employment, or policy IDs. Existing ADR `docs/decisions/2026-11-15-go-domain-ids.md` approves distinct Go defined ID types while preserving UUID JSON, JWT, and PostgreSQL contracts.

Current worktree has unrelated work-policy/payroll changes. Preserve them.

## Decision

Use existing `apiserver/internal/domain/ids.UserID`:

```go
type UserID uuid.UUID
```

Do not add wrapper struct, generic ID framework, migration, or public wire-format change. Keep `uuid.UUID` in DTOs, JWT subject text, SQL scan locals, and infrastructure adapters. Convert once at boundaries.

## Dependency graph

```text
ids.UserID
  -> JWT auth boundary
  -> HTTP auth context
  -> authorization and handlers
  -> work-policy/payroll actor inputs and repositories
  -> tests and full API verification
```

## Vertical tasks

### Phase 1 — Foundation and authentication

1. Add minimal `UserID` parse/string/UUID conversion behavior only if current callers need it; test valid, malformed, zero, and round-trip values.
2. Change `auth.GenerateToken`, `GenerateTokenWithTimes`, and `VerifyToken` to use `ids.UserID` internally. Keep JWT subject UUID string and existing errors/lifetimes.
3. Change HTTP context storage, `UserIDFromContext`, and `WithUserIDForTest` to `ids.UserID`. Keep request IDs and SQL/Redis adapter values as UUID where they are infrastructure contracts.

**Acceptance:** authenticated context returns typed `ids.UserID`; malformed/expired/revoked tokens retain 401 behavior; JWT output and revocation keys remain unchanged.

**Verification:** focused auth and middleware tests; no handler receives raw UUID from `UserIDFromContext`.

**Checkpoint A:** auth and context path passes before business call-site migration.

### Phase 2 — One complete business path

1. Update work-policy, payroll, and organization handler actor values to use typed context values without casts.
2. Update authorization interfaces/mocks and work-policy/payroll repository/domain actor parameters to use `ids.UserID`.
3. Convert to `uuid.UUID` only at SQL/audit persistence boundaries; preserve existing entity ID types and API DTOs.

**Acceptance:** actor ID cannot be confused with employment/employee/policy IDs; persistence and audit UUID values remain identical; 401/403 behavior remains unchanged; no migration required.

**Verification:** API and repository tests for success, missing identity, forbidden access, persistence, conflict, rollback, and audit actor output.

**Checkpoint B:** HTTP-to-SQL work-policy/payroll actor path passes.

### Phase 3 — Remaining authenticated-user consumers

Migrate only parameters whose semantic meaning is authenticated user/actor. Leave transport DTOs, path IDs, unrelated IDs, map keys, SQL locals, and refresh-token storage fields unchanged unless caller analysis proves they are domain boundaries.

**Acceptance:** `Authorizer` actor parameters consistently use `ids.UserID`; remaining raw user UUIDs have documented boundary reasons; no broad UUID replacement.

**Verification:** scoped API tests, repository wrappers for format/vet, and search review of all `UserIDFromContext` callers.

**Checkpoint C:** remaining raw UUID uses classified as transport, SQL, or infrastructure boundaries.

### Phase 4 — Full verification

Run:

```bash
./leedevkit test api --unit-only
./leedevkit test api
./leedevkit test all
```

Inspect per-phase `.test_logs/`; report pass/fail/skip/warnings. Confirm unrelated worktree paths remain unchanged and unstaged.

## Test impact matrix

| Dimension | Protection |
|---|---|
| Changed behavior | Internal signatures distinguish user/actor IDs from other IDs. |
| Existing behavior | JWT, auth status, audit text, JSON, and SQL values unchanged. |
| Boundaries | Valid, malformed, missing, zero, expired, revoked, forbidden identities. |
| Security | Actor comes only from verified middleware context; authorization remains fail-closed. |
| Compatibility | No migration, URL, DTO, or UUID wire-format change. |

## Risks

- Defined type loses UUID methods at call sites: use explicit conversion at boundaries.
- Partial migration leaves mixed context signatures: migrate auth, middleware, then one complete business path.
- Existing worktree changes get overwritten: no reset, checkout, stash, or broad formatting.

**Ponytail:** skip wrapper structs and whole-repository UUID migration; add only when compile-time domain safety needs behavior beyond defined types.
