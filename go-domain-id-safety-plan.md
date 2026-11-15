# Go domain ID safety: compiler-guided agent changes

Status: Implemented (pilot)
Scope: Plan only. Pilot work-policy writes, then expand only after verified results.

## Goal and evidence
Reduce UUID mix-ups and agent context requirements using explicit Go types, small operation-specific inputs, and compiler feedback; preserve business behavior and external contracts. No runtime-speed claim.

Verified hotspots: `apiserver/internal/repository/work_policy.go:215,226,273,337` accept several interchangeable UUIDs; `apiserver/internal/api/work_policy.go:95` exposes raw UUIDs in the consumer-owned interface. Existing repository tests use `fakeDBTX` (`work_policy_test.go:20`), so they are not proof of real PostgreSQL compatibility. Repository currently imports API request/response types; do not disguise a full layering rewrite as an ID refactor.

## Design decisions
- Use distinct Go defined types (`type GroupID uuid.UUID`), not aliases, public embedding, a generic `ID[T]` framework, reflection, or constructor/getter boilerplate. Value types remain comparable and suitable as map keys; zero value remains possible and must be rejected where required.
- Policy-owned IDs live in `internal/domain/workpolicy/ids.go`: GroupID, VersionID, WorkCreditPolicyID, PayBasisPolicyID, MembershipID. Cross-domain references need one canonical owner: reuse existing neutral identity types if discovered; otherwise a small leaf `internal/domain/id/ids.go` contains only UserID, EmploymentID, AttendancePolicyID, WorkingPatternID, OvertimePolicyID, RateComponentDefinitionID needed by this slice. No service, DB, transport imports or unrelated IDs. Confirm imports before adding files.
- Actor is a role of UserID, not a new entity type. Same-type roles use named fields (e.g. Source/Target); do not invent a distinct type for every argument.
- Keep `context.Context` first. Use operation-specific input structs with keyed literals where IDs/roles are easy to swap; keep short, clear signatures positional. Required input stays explicit, not in context, `any`, maps, variadics or functional options.
- Preserve existing consumer-owned repository interface; update methods and stubs together. No new service/interface layer. Put shared typed business inputs in workpolicy only where needed; retain wire DTOs and current response ownership during pilot.
- Decode HTTP into existing DTOs, validate, then map once to typed inputs. Actor comes only from authenticated context. Generate IDs into typed variables immediately. Keep types through helpers; convert explicitly to/from uuid.UUID only at SQL/wire/legacy integration boundaries. No casts between semantic ID types as compiler-error workarounds.
- SQL `Exec(...any)` and positional `Scan` erase type protection. Retain local SQL adapters and test exact column/argument mapping with distinct fixture IDs. Do not claim compiler safety for SQL placeholders or same-type field swaps.
- Keep UUID JSON strings, null/omitted semantics, schema, SQL constraints, transactions, authorization, errors and calculations unchanged. Prefer raw UUID scan locals and explicit conversion over custom pgx codecs; add methods only for demonstrated consumers. No custom JSON methods needed while DTOs remain raw UUIDs.

## Ordered work (after approval)
1. [ ] **Baseline and ownership:** refresh working-tree status; preserve concurrent edits; inspect all callers, domain struct consumers, test fixtures and existing ID conventions. Record impacted files and baseline failures. Verify dependency graph has no new cycle; final pilot scope is bounded before editing.
2. [ ] **Protect behavior:** characterize create group, create version and membership with distinguishable IDs; cover HTTP DTO mapping, trusted actor provenance, nullable references and existing error paths. Reuse current API/repository test files. Verify assertions detect swapped SQL parameters, not merely successful responses.
3. [ ] **Introduce types:** add only consumed types and minimal typed input structs. Preserve zero/nil validation semantics. Add one small stdlib-based type-check test or existing negative-compilation mechanism proving real distinct types cannot be assigned/passed interchangeably; include a valid control and avoid exact compiler-message matching. No new test framework/script.
4. [ ] **Migrate one vertical write slice:** create group, create version, membership; update interface, handlers, repository helpers and stubs together. Replace long helper parameter lists with named typed inputs, reusing existing domain values where suitable. Compile after each coherent operation; fix callers without `any`, aliases or blind casts. Convert only required read-side helpers/consumers, not payroll wholesale.
5. [ ] **Record adoption rule:** after approval, add one ADR under `docs/adr/` with ownership, boundary exceptions, alternatives, rollback and status. Add a short agent-facing convention to existing coding rules only with explicit policy-edit approval. Use existing gofmt/vet/staticcheck gates; no custom linter initially. Future modules adopt per complete operation, not by global search/replace.
6. [ ] **Verification (last):** focused hermetic Go checks, then `./leedevkit test api --json` without skip/unit-only flags. Extend existing integration harness with real PostgreSQL create/version/membership/read-back checks and rollback/conflict cases. Confirm integration cases actually execute; skipped DB tests are not a pass. Review individual gate logs, API compatibility and diff boundaries before reporting completion.

## Test impact / acceptance
- Type boundary: GroupID vs VersionID, EmploymentID vs UserID rejected; valid calls compile. Same-type swaps and explicit conversions remain review/test responsibilities.
- Input/security: malformed/missing/zero UUID, optional nil reference, unknown/deleted reference, unauthorized caller; actor cannot be supplied by request. Preserve existing error/status contracts and absence of writes on rejection.
- Persistence/state: distinct IDs reach correct columns; audit actor preserved; create/read-back, version linkage, membership linkage, exclusive effective-date boundaries, conflicts and transactional rollback preserved. DB failure/cancellation propagates without partial writes.
- Compatibility/operations: exact JSON UUID strings and null/omitted behavior unchanged; no migrations, frontend changes, extra queries, new dependencies or PII logging. Existing decimal/payroll calculations unchanged; relevant regression tests retained.
- Completion: no positional raw UUID ambiguity in migrated business/helper signatures; one canonical type per identity; all affected consumers compile; lint/race/unit/integration evidence recorded. Measure migrated signatures and remaining raw-ID boundary exceptions, not speculative agent productivity percentages.

## Constraints and rollback
Separate from ongoing business/migration changes; no commits/pushes without request. Never edit `.leedevkit/`. Revert only this bounded source refactor if needed; no data rollback required. Defer cross-backend conversion, generic infrastructure, full transport/domain separation and custom analyzers until concrete need appears.

Rules: Coding Standards §2/§6 (KISS, no speculative code), Project Structure §5 (dependency direction), Architecture Governance §2/§5 (ADR and migration), Development Workflow §2/§6 (test impact and refactor evidence), Testing Standards §1/§4 (full applicable gate), Execution Wrappers §1 (hermetic execution).

Planning evidence: source reads, caller search and graph inbound trace verified pilot hotspots; graph search endpoint rejected query/semantic_query parameters, so discovery used source fallback rather than claiming a complete graph audit. No runtime tests executed for this documentation-only plan.
