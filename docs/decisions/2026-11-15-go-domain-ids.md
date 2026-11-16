# ADR: Go Domain IDs at Internal Business Boundaries

| Property | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-15 |
| Owner | LeeAttend backend |
| Scope | Go internal business and repository interfaces; payroll/work-policy rollout |

## Context

LeeAttend uses `uuid.UUID` for database and HTTP identifiers. Go treats every UUID value as the same type, so a caller can pass an employee ID where a policy, batch, rule, employer, or actor ID is expected. Long positional signatures amplify this failure mode for humans and AI agents. JSON and PostgreSQL contracts must remain UUID-compatible.

## Decision

Use distinct Go defined types for semantic IDs at internal business boundaries:

- Shared identity types live in `apiserver/internal/domain/ids`.
- Domain-owned types live in the owning domain package.
- Use named operation input structs when multiple IDs, versions, dates, or same-kind roles can be confused; fields use keyed literals.
- Keep `context.Context` first and required business inputs explicit.
- Keep transport DTOs and API responses as `uuid.UUID`; convert once at HTTP boundary.
- Keep PostgreSQL arguments and scan locals as `uuid.UUID`; convert once at repository/SQL boundary.
- Never replace type safety with aliases, `any`, reflection, generic ID frameworks, or blind casts.
- Do not create a distinct type for every parameter role when one semantic type is sufficient.
- Runtime validation, authorization, transaction boundaries, and database constraints remain mandatory. A typed ID does not prove existence or permission.

## Rollout

First rollout covers work-policy and payroll boundaries, including employee, employment, employer, user/actor, policy, payroll input, formula, rule, tier, assignment, incentive, period, and salary-management filter IDs. Operation input struct `PayrollPeriodMutation` protects period state transitions from positional argument swaps.

Future modules adopt types only when a complete boundary has a concrete ambiguity risk. Each slice must update interface, callers, adapters, tests, and SQL conversions together.

## Alternatives considered

### Keep raw UUIDs everywhere

Rejected. Compile-time checks cannot distinguish semantic IDs and reviewers/tests must catch avoidable swaps.

### Use UUID aliases

Rejected. An alias preserves the original type and provides no protection.

### Use wrapper structs or generic `ID[T]`

Deferred. They add constructors, methods, serialization, and framework surface without current need. Defined types provide required compile-time separation at lower cost.

### Change HTTP or database representation

Rejected. External contracts and persistence already use UUIDs; internal safety does not justify compatibility risk.

### Add a custom linter first

Deferred. Compiler errors and reviewable signatures cover current risk. Add a bounded analyzer only after repeated violations demonstrate need.

## Consequences

### Positive

- Compiler rejects cross-entity ID swaps at internal call sites.
- Operation signatures communicate business meaning to humans and agents.
- Named inputs reduce same-type positional ambiguity.
- HTTP and SQL compatibility remains unchanged.
- Rollout stays incremental and reviewable.

### Costs and limits

- Explicit conversions remain at HTTP, SQL, and legacy calculation boundaries.
- `Exec(...any)` and positional `Scan` still require mapping tests.
- Same semantic type used twice can still be swapped; use named input fields and behavior tests there.
- Zero values remain possible; boundary validation must reject invalid IDs where required.
- More type declarations and mock updates increase local diff size.

## Security and operations

Typed IDs do not replace handler authorization. Authorization must run before business logic and actor IDs must come from authenticated context, never request payloads. SQL remains parameterized. No raw PII logging or new telemetry is introduced. No migration or runtime deployment change is required.

## Migration and rollback

Adoption is source-only and backward-compatible at transport and persistence boundaries. Roll back one cohesive module commit if needed; no data rollback is required. Do not remove legacy conversions until all consumers in a boundary use the semantic type. Do not add compatibility adapters without an owner and removal condition.

## Acceptance and review gate

A slice is accepted when:

- No ambiguous raw-UUID business signature remains in its scoped boundary.
- Compiler, unit/race, integration, format, and vet checks pass.
- JSON and SQL behavior remain unchanged.
- Negative authorization, validation, persistence, conflict, and rollback behavior remains covered.
- Unrelated working-tree changes are not staged.

Review typed-ID coverage by boundary risk, not by raw UUID count. Stop rollout when remaining raw UUID is DTO/output, SQL adapter, scan local, or a clear single-semantic helper with no meaningful swap risk.
