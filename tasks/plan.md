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
- Run `leedevkit --help` and `leedevkit test --help`.
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
- `leedevkit test api`
- `leedevkit test all`

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
- [ ] Run `leedevkit --help` and `leedevkit test --help` if baseline not already recorded.

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
- [ ] Run `leedevkit test web --unit-only --pattern 'employee|Employees'` during iteration.

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
- [ ] Run `leedevkit test api --unit-only` then `leedevkit test api`.

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
- [ ] Run `leedevkit test web </dev/null`.

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
- [ ] `leedevkit test web --unit-only` for iteration.
- [ ] `leedevkit test web </dev/null`.
- [ ] `leedevkit test api </dev/null`.
- [ ] `leedevkit test all </dev/null` when API/OpenAPI/DB contract changes.
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

# Implementation Plan: Duplicate Employee Detection After Employee Tenures

## Context

Employee creation currently accepts name, employee number, and hire date but cannot determine whether person already exists. `employee_profiles` already stores phone, date of birth, and national ID; `employee_tenures` stores historical employment periods. Current duplicate protection mostly relies on tenure date-range exclusion, so no person-level preflight exists and historical employee numbers can be reused.

Target flow: **Search existing → explicitly reuse existing employee or create new employee**. Never fuzzy-auto-merge. Reuse keeps `employee.id` and employee code, then creates new tenure/employment. Create-new creates employee/profile/first tenure atomically. No raw PII in URLs, logs, metrics, screenshots, or match responses.

## Decisions

- **Approved at checkpoint D-A (user approval, 2026-11-24) — field normalization:** name uses Unicode NFC, trims and collapses whitespace, then Unicode case-folds; punctuation remains significant. DOB parses strictly as ISO `YYYY-MM-DD`; no timezone conversion. National ID uses Unicode NFKC, trims, uppercases, and removes documented visual separators only; leading zeroes remain, and empty-after-normalization never matches. Phone must parse to E.164 with an explicit country calling code; reject extensions/ambiguous national-only input rather than infer a country. Employee code trims outer whitespace and uppercases; preserve leading zeroes, punctuation, and internal whitespace.
- **Approved hard match:** exact normalized national ID or exact E.164 phone; empty values never match; DOB alone never hard-matches. **Soft match:** normalized full name + DOB, or normalized name + partial phone; review warning only, never auto-reuse. Never fuzzy-auto-merge.
- **Approved employee-code policy:** normalized code cannot identify different employees, including across historical tenures; same employee may reuse its historical code only on non-overlapping tenures, consistent with `docs/plans/employee-tenures.md`. No `MAX()+1` allocation in V1. Existing case-sensitive overlap constraint is not sufficient for this normalized policy; enforcement design must handle normalized collisions before any uniqueness constraint is activated.
- New endpoint proposal: `POST /api/v1/employees/match`; identity PII stays in POST body, never URL/query. Reuse endpoint proposal: `POST /api/v1/employees/{employee_id}/tenures`.
- **Approved permission:** dedicated `employees:identity:match`, granted only to `admin` and `hr_manager`; API enforces org/object scope and existing PII access policy. Deny by default.
- Keep raw profile fields under existing restricted access. Lookup uses keyed HMAC over canonical field value with configured secret; never plaintext/unsalted indexes. Do not include PII, lookup hashes, or candidate identifiers in logs/metrics/audit detail. Audit actor, action, outcome tier, and aggregate candidate count only; audit write failure blocks the match response.
- **Approved rate limit:** reuse existing authenticated write limiter, 30 requests per actor per 60 seconds. Enforce bounded input/body and constant-shape responses. Do not trust caller-supplied forwarding headers as identity.
- **Approved flag policy:** default off; shadow mode may compute only aggregate tier/count metrics and must not disclose candidates or mutate data; pilot only admin/HR. Kill switch disables endpoint/UI behavior; additive schema remains until readers are retired. Owner/removal date must be set before rollout.
- Policy approval clears the D-A policy gate only. D0.2 currently covers empty disposable-schema baseline, not populated target-data prevalence. No migration, API, or UI work is started by this approval alone.

## Dependency graph

```text
Identity policy + privacy + permission
  -> lookup hash migration + duplicate report/backfill
    -> read-only match API/repository
      -> atomic create-new and reuse-tenure commands
        -> /employees/new preflight UI
          -> attendance staging integration + route audit
```

## Vertical tasks

### Phase 0 — Policy and baseline

#### Task D0 — Freeze matching policy and inventory duplicates

**Acceptance criteria:**
- [x] Proposed normalization rules recorded for phone, national ID, name, DOB, and employee code; D0.1 awaits human approval at D-A.
- [x] Proposed hard/soft match rules, global code reuse policy, permission, privacy, audit, rate limit, and feature flag documented; D0.1 remains pending D-A.
- [x] Disposable DB report counted duplicate profiles, duplicate codes, tenure overlaps, profile/code mismatches, orphan profiles, and reconciliation rows without PII output.

**Verification:** Applied repository schema phases to isolated disposable PostgreSQL (`leeattend-duplicate-d0-20261124`); aggregate report returned zero for all ten counters on empty schema. These are schema/bootstrap baseline counts, not production or populated-fixture data. No PII-bearing output or artifacts created. Disposable container/network removed after report.

**Dependencies:** None

**Estimated scope:** Small

### Checkpoint D-A

- [x] Human approved identity normalization, hard/soft matching, global code identity/reuse, permission, privacy, audit-failure behavior, rate limit, and feature-flag default/shadow/pilot policy on 2026-11-24. Feature-flag owner/removal date remains a pre-rollout prerequisite.
- [ ] Confirm populated disposable duplicate report is required/provisioned; current result covers empty disposable schema only, not target-data prevalence.

### Phase 1 — Lookup foundation

#### Task D1 — Add reversible identity lookup migration

**Acceptance criteria:**
- [x] Add nullable `national_id_lookup_hash` and `phone_lookup_hash` (`character(64)` + lowercase 64-hex CHECK) to `employee_profiles` (`apiserver/migrations/20261125000000_employee_identity_matching/{up,down}.sql` and `apiserver/migrations/phases/hr_core/up.sql`).
- [x] Add non-unique partial lookup indexes (`WHERE ... IS NOT NULL`); unique indexes remain deferred until populated duplicate report / collision queue is clean; NULL/empty values never collide.
- [x] Existing raw identity columns and historical rows remain unchanged.
- [x] `up.sql` and `down.sql` verified on fresh phased and upgraded numeric disposable DBs (`schema_contract.sql` + `phased_schema_contract.sql` PASS).

**Files likely touched:**
- `apiserver/migrations/20261125000000_employee_identity_matching/up.sql`
- `apiserver/migrations/20261125000000_employee_identity_matching/down.sql`
- `apiserver/migrations/phases/hr_core/up.sql`
- `apiserver/migrations/testdata/schema_contract.sql`
- `apiserver/migrations/testdata/phased_schema_contract.sql`

**Verification:** Disposable PostgreSQL verified numeric upgrade + `schema_contract.sql` + `down.sql` (0 columns left) + re-apply `up.sql` + `schema_contract.sql`, and fresh phased install + `phased_schema_contract.sql` + reverse `down.sql`.

**Dependencies:** D0

**Estimated scope:** Medium

#### Task D2 — Add bounded application backfill/reconciliation

**Acceptance criteria:**
- [x] Normalize + domain-separated HMAC-SHA256 (`apiserver/internal/domain/identity/identity.go`) and bounded keyset-paginated idempotent backfill (`apiserver/internal/repository/employee_identity_backfill.go`).
- [x] Missing or `<32`-byte `IDENTITY_LOOKUP_HMAC_SECRET` fails closed before DB access (`config.go`, `identity.NewHasher`, `BackfillIdentityLookupHashes`).
- [x] Collisions reported as aggregate group/row counts via `IdentityReadiness` / `IdentityBackfillReport`; no winner selected; `UniqueIndexReady` remains false when collisions exist.
- [x] Reports/errors/config stringers contain aggregate counts/status only, never raw PII, secrets, or lookup hashes.

**Files likely touched:**
- `apiserver/internal/domain/identity/identity.go`, `identity_test.go`
- `apiserver/internal/config/config.go`, `config_test.go`
- `apiserver/internal/repository/employee_identity_backfill.go`, `employee_identity_backfill_test.go`, `employee_identity_backfill_integration_test.go`

**Verification:** `scripts/_go-gates.sh --lint-only` (`gofmt`, `go vet`, `staticcheck`, `golangci-lint --new-from-rev`, `govulncheck`, `go mod verify`), `--unit-only ./...` (`-race`), and `--integration` (`TestIntegrationEmployeeIdentityBackfillIdempotentAndCollisionAware`, `TestIntegrationEmployeeTenures*`) all passed.

**Dependencies:** D1

**Estimated scope:** Medium

### Checkpoint D-B

- [x] Lookup columns, non-unique partial indexes, bounded idempotent backfill, and collision-readiness queries verified on disposable DB.
- [x] No unique index activated while unresolved collisions exist (asserted by both schema contracts and `UniqueIndexReady` check).

### Phase 2 — Match API

#### Task D3 — Implement redacted read-only match endpoint

**Acceptance criteria:**
- [x] `POST /api/v1/employees/match` validates typed request and rejects unknown fields (`apiserver/internal/api/employee_match.go`, 4 KiB body cap returning 413, rejects non-`lang` query params, strict `internal/domain/identity` normalization).
- [x] Exact hash matching produces hard candidates (`national_id_exact`, `phone_exact`); normalized name+DOB (`name_dob_review`) or name+partial-phone (`name_phone_review`) produces bounded soft review candidates (`apiserver/internal/repository/employee_identity_match.go`).
- [x] Response contains only candidate ID, sanitized display name, employee code, status (`active`/`terminated`), tier, and allowlisted non-sensitive reason labels.
- [x] No raw phone, national ID, DOB, lookup hash, or PII query string appears in response/log/audit details.
- [x] 401/403 fail closed with `employees:identity:match` (`admin` + `hr_manager` only); missing/short `IDENTITY_LOOKUP_HMAC_SECRET` fails closed (503); audit emits actor/action/outcome tier/candidate count only and blocks response on failure; existing 30 req/actor/60s write limiter resolves Bearer and HttpOnly cookie actors and ignores spoofed forwarding headers.

**Files likely touched:**
- `apiserver/internal/auth/permissions.go`, `permissions_test.go`
- `apiserver/internal/standalone/catalog.go`, `bootstrap_test.go`
- `apiserver/internal/domain/identity/identity.go`, `identity_test.go`
- `apiserver/internal/infra/http/rate_limit.go`, `rate_limit_test.go`
- `apiserver/internal/api/employee_match.go`, `employee_match_test.go`, `employees.go`, `validation.go`
- `apiserver/internal/repository/employee_identity_match.go`, `employee_identity_match_integration_test.go`
- `apiserver/routes.go`, `routes_employees.go`, `router_test.go`
- `apiserver/openapi.yaml`, `scripts/generate_openapi.py`

**Verification:** `scripts/validate_openapi_contract.py`, `scripts/_go-gates.sh --lint-only --base-ref HEAD` on touched packages (plus `--build-tags=integration` check), `scripts/_go-gates.sh --unit-only` (`-race`), and `scripts/_go-gates.sh --integration` on disposable DB (`TestIntegrationEmployeeIdentity*`, `TestIntegrationEmployeeTenures*`) all passed.

**Dependencies:** D2

**Estimated scope:** Large; keep endpoint and repository matcher in one slice.

### Phase 3 — Atomic write paths

#### Task D4 — Harden create-new with profile and first tenure

**Acceptance criteria:**
- [ ] Create-new writes employee, profile, first tenure, and employment relationship in one transaction.
- [ ] Identity and employee-code uniqueness rechecked under transaction lock.
- [ ] Concurrent same identity/code requests cannot create duplicates; races return deterministic 409.
- [ ] Old clients remain compatible while feature flag is off.

**Verification:** DB integration success, rollback, duplicate identity/code, concurrent race, retry tests.

**Dependencies:** D3

**Estimated scope:** Large

#### Task D5 — Add explicit reuse-tenure command

**Acceptance criteria:**
- [ ] `POST /api/v1/employees/{employee_id}/tenures` locks and revalidates employee candidate.
- [ ] Reuse keeps employee ID/code and creates only new tenure/employment.
- [ ] Active employee reuse, stale candidate, date overlap, and invalid scope return deterministic errors.
- [ ] Tenure history remains immutable and no partial writes survive failure.

**Verification:** terminated employee reuse, active rejection, overlap, employer transfer, stale candidate, concurrent reuse.

**Dependencies:** D4

**Estimated scope:** Large

### Checkpoint D-C

- [ ] New person creates profile + first tenure atomically.
- [ ] Existing person creates tenure only.
- [ ] ESS, attendance, reports, employee detail resolve correct employee/tenure.

### Phase 4 — UX integration

#### Task D6 — Add preflight identity step to `/employees/new`

**Acceptance criteria:**
- [ ] Form collects phone, DOB, and national ID under role policy.
- [ ] Debounced match displays loading, none, hard, soft, stale, conflict, and error states.
- [ ] Hard candidate requires explicit `Reuse existing` or safe correction; no silent reuse.
- [ ] Soft candidate warns and permits explicit `Not this person — continue create`; never auto-reuses.
- [ ] Reuse calls tenure endpoint; new create calls create-new path; duplicate click blocked.
- [ ] Keyboard/focus/aria, vi/en/zh strings, responsive layout, and PII-safe artifacts pass.

**Files likely touched:**
- `webdashboard/src/pages/employees/employee-create-page.tsx`
- `webdashboard/src/components/features/employees/employee-form-inputs.tsx`
- `webdashboard/src/lib/api/employees.ts`
- locale employee files
- page/component tests

**Verification:** hard/soft/none, reuse/create, stale result, API failure, keyboard/a11y, route tests.

**Dependencies:** D5

**Estimated scope:** Large

#### Task D7 — Integrate attendance staging

**Acceptance criteria:**
- [ ] Staging unknown log uses same preflight path.
- [ ] Existing employee can be assigned; create-new uses safe enrollment prefill.
- [ ] `raw_user_id` is never treated as proof of identity.
- [ ] Staging row identity and refresh/resolve behavior remain intact.

**Files likely touched:**
- `webdashboard/src/components/features/attendance/staging-logs-table.tsx`
- `webdashboard/src/pages/attendance/hooks/use-attendance-logs-page.ts`
- staging tests

**Verification:** staging component/hook tests and disposable E2E flow.

**Dependencies:** D6

**Estimated scope:** Medium

### Checkpoint D-D

- [ ] Search-before-create works from employee page and attendance staging.
- [ ] Hard match cannot accidentally duplicate; soft match never auto-reuses.

### Phase 5 — Rollout and final verification

#### Task D8 — Shadow/pilot rollout and observability

**Acceptance criteria:**
- [ ] Feature flag defaults off and has clear owner/removal date.
- [ ] Pilot enables admin/HR only.
- [ ] Metrics include aggregate match tier, conflicts, latency, and rollback counts only.
- [ ] Rollback disables behavior without destructive schema changes.

#### Task D9 — Full verification

**Verification:**
- [ ] Regenerate OpenAPI TypeScript artifacts through repository wrapper; never hand-edit generated output.
- [ ] Run `leedevkit test web --unit-only`, then full web/API/all targets as applicable.
- [ ] Read `.test_logs/` per phase.
- [ ] Browser audit all five roles at `1920x1080`, then mobile smoke; verify direct URL, refresh, match states, 403/409, console/network, and no PII artifacts.

**Dependencies:** D8

### Final checkpoint

- [ ] Search existing → explicit reuse/create works.
- [ ] New employee has profile + first tenure atomically.
- [ ] Reuse adds tenure only.
- [ ] Concurrent identity/code races are safe.
- [ ] No PII leaks; rollback evidence recorded.

## Duplicate detection test-impact matrix

| Dimension | Protection |
|---|---|
| Changed behavior | Search existing before create; explicit reuse/create; reuse adds tenure only. |
| Existing behavior | List/detail, rehire, staging, ESS, attendance, reports, historical tenure remain. |
| Inputs | Empty/normalized phone/national ID, DOB-only, name variants, duplicate code, malformed date/UUID, unknown fields. |
| State | None/hard/soft match, stale candidate, active/terminated employee, dirty/pending/retry. |
| Failures | 401, 403, 404, 409, 422, 5xx, timeout, rollback, race, duplicate submit. |
| Security | Exact-only T2 lookup, HMAC secret failure, scope checks, rate limit, no URL/log/artifact PII. |
| Compatibility | Existing create/rehire while flag off, old profile rows, generated contracts, migration down. |
| Operations | Audit lookup/write, aggregate metrics, collision queue, bounded backfill, flag rollback. |

## Verification commands

```bash
leedevkit test web --unit-only
leedevkit test web </dev/null
leedevkit test api </dev/null
leedevkit test all </dev/null
```

Browser: read `docs/dev-accounts.md`, cover all five roles, set and verify `1920x1080` first, then mobile; never store credentials/tokens/raw PII.

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
leedevkit test api --unit-only
leedevkit test api
leedevkit test all
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

---

# Plan: Optimize `/shifts/new` UX for HR operations

## Context

Replace technical interval language with clear work-period language. Keep one normal work period simple (`07:30–16:30` plus break); reveal split-shift configuration only when explicitly requested. Preserve inline interval API payload, edit hydration, overnight behavior, save/cancel, dirty blocker, version history, and all unrelated worktree changes. Frontend only; no backend schema or API change.

## Dependency graph

```text
useShiftEditor state/defaults/hydration/payload
  -> shift-editor-page.tsx
  -> IntervalEditorCard
  -> IntervalDraftEditor
  -> BreakConfigSection
  -> IntervalDraft[]
  -> getShiftCardMetrics + ShiftTimelinePreview
  -> MetricsGrid / ShiftDetailsList / TimelineBar

attendance.json (en/vi/zh) -> all visible copy and validation
Tests: metrics -> hook payload -> components -> preview -> E2E
```

## Vertical tasks

### Phase 0 — Baseline and contract freeze

- Capture current diff and preserve unrelated changes.
- Verify `/shifts/new`, inline `CreateIntervalRequest[]`, locale parity, and focused test targets.
- Confirm no backend/API/schema files enter scope.

Acceptance: existing changes preserved; current payload and non-goals documented.
Verification: `git status --short`; `leedevkit --help`; `leedevkit test --help`; inspect existing shift tests.

**Checkpoint A:** contract and scope confirmed.

### Phase 1 — Operational metrics

Files: `webdashboard/src/lib/shift-card-metrics.ts`, `webdashboard/src/lib/shift-timeline-math.ts` only if needed, metric tests.

- Reuse `getShiftCardMetrics` for all-interval duration, break, net work, sorting, and midnight wrapping.
- Add smallest missing expected-punch summary value.
- Cover single period, split period, duration/fixed break, overnight, crossing-midnight break, empty/incomplete input.

Acceptance: `07:30–16:30` + 60-minute break reports 9h presence, 1h break, 8h net; split shift aggregates both blocks; incomplete drafts do not crash.

### Phase 2 — Hook and payload

Files: `webdashboard/src/pages/shifts/hooks/use-shift-editor.ts` and its tests.

- Keep one default draft, optional `intervalName`, edit hydration, add/remove/update, overnight detection, seconds padding, and inline intervals array.
- Ensure duration mode sends clock fields `null`; fixed mode sends both clock fields.
- Block or clearly flag incomplete fixed-break boundaries before submit.

Acceptance: existing create/edit payload remains compatible; all intervals serialize; save/correction behavior remains unchanged.

**Checkpoint B:** metrics/hook tests and typecheck pass; API contract unchanged.

### Phase 3 — Basic interval editor

Files: `interval-editor-card.tsx`, `interval-draft-editor.tsx`, `break-config-section.tsx`, focused component tests.

- Replace visible technical wording with translated `Khoảng làm việc` / locale equivalents.
- Hide interval name in default flow; expose optional name through accessible advanced disclosure.
- Explain that `07:30–16:30` is one work period; add another only for separated work.
- Use explicit break modes: duration vs fixed clock times; show only relevant fields.
- Add accessible labels, descriptions, inline validation, and accessible remove action.

Acceptance: normal HR flow needs only name, start/end, and break; second period appears only after explicit add; fixed mode requires both times.

### Phase 4 — Operational preview

Files: `shift-timeline-preview.tsx`, `metrics-grid.tsx`, `shift-details-list.tsx`, `types.ts`, preview tests.

- Stop using `intervals[0]` only. Feed all drafts through reusable metrics.
- Show total presence, total break, net work, expected punches, and useful per-period details.
- Preserve overnight timeline and empty/incomplete safety.

Acceptance: single and split summaries reconcile; summary updates immediately; no technical/blank interval name in normal flow.

**Checkpoint C:** component tests pass; no first-period-only calculation remains.

### Phase 5 — Page wiring and i18n

Files: `shift-editor-page.tsx`, `webdashboard/src/i18n/locales/{en,vi,zh}/attendance.json`, locale tests.

- Add matching keys for work-period explanation, split guidance, break modes, summary, expected punches, and validation.
- Remove hardcoded changed copy/placeholders.
- Preserve loading/error, save/cancel, dirty blocker, edit, and history behavior.

Acceptance: locale trees remain equal; Vietnamese consistently uses `Khoảng làm việc`, `Nghỉ giữa ca`, `Công thực tế`; no new hardcoded user-facing copy.

### Phase 6 — Browser proof

File: `webdashboard/tests/shifts-flow.spec.ts`.

- Cover basic form, `07:30–16:30` + duration break, split shift, fixed break, overnight, request payload, and cancel/dirty navigation.
- Use documented least-privileged account, route × role matrix, and 1920×1080 viewport before UI interaction.
- Capture console/network evidence without credentials, tokens, or PII.

Acceptance: basic, split, break-mode, overnight, save/cancel flows pass; request preserves existing inline contract; no unexpected 404/5xx/crash.

## Test impact matrix

| Dimension | Protection |
|---|---|
| Changed behavior | Work-period terms, break-mode choice, split disclosure, aggregate preview |
| Existing behavior | Payload, edit hydration, overnight, save/cancel, dirty blocker, history |
| Boundaries | Empty/incomplete fields, fixed-break boundary, midnight, multiple periods |
| Failures | Validation, API error, loading/error state |
| Compatibility | No backend/schema/API field changes; old interval names preserved |
| Operations | No PII/secrets in logs or browser artifacts |

## Verification

Run focused unit/component tests first, then locale parity, typecheck, lint, build, focused E2E, and applicable full LeeDevKit web/all targets. Inspect `.test_logs/` per phase; do not rely on trailing green summary.

## Non-goals

No backend redesign, interval CRUD flow, new dependency, schedule lifecycle change, or attendance policy change.
