# ADR-0003: Apiserver-Owned External Attendance Log Pull

Status: Accepted for shadow migration
Date: 2026-10-30

## Context

`agent-main` currently owns device log polling and delivery. Replacing it requires durable per-device checkpoints, idempotent provider event identity, singleton-organization ownership, and a reversible shadow period. The external provider contract remains an integration prerequisite for production cutover.

## Decision

- `apiserver` owns scheduled pulls for enabled devices belonging to the implicit organization.
- Provider access uses a typed HTTP adapter with HTTPS-only configuration, bearer-token authentication, bounded response bodies, request IDs, and caller cancellation.
- `device_pull_states` stores UUID-backed lease/checkpoint state. `external_attendance_events` stores provider device IDs as `TEXT`, because provider identity is not required to be a UUID.
- Event identity is unique on `(device_id, external_event_id)`; authenticated ownership and organization-admin authorization remain explicit.
- Write mode advances cursor only in the same transaction as event metadata and attendance raw/staging writes.
- Shadow mode fetches and normalizes pages, but does not advance cursor or write attendance rows. It may acquire and release pull leases.
- Scheduler starts only when `LOG_PULL_ENABLED=true`; `LOG_PULL_MODE` defaults to `shadow`. Shutdown cancels and waits for active pulls within application shutdown context.
- Provider device identity currently maps to device UUID string. This is a contract assumption, not proof of provider behavior; production cutover requires versioned fixtures and pre-production verification.

## Rejected alternatives

- In-memory cursor or queue: loses progress on restart and cannot coordinate concurrent apiserver instances.
- Direct write cutover: gives no reconciliation window against agent delivery.
- UUID storage for provider device IDs: rejects valid non-UUID provider identities and creates unsafe rollback casts.
- Dual-write during cutover: creates duplicate ownership and ambiguous recovery.

## Consequences

Positive:

- Restart-safe checkpointing and lease ownership.
- Organization-scoped idempotency and transactional cursor advancement.
- Shadow mode can validate provider reads without attendance writes.
- Scheduler lifecycle is bounded by normal apiserver shutdown.

Risks and follow-up:

- Provider endpoint, cursor semantics, retention, identity mapping, and error contract still require versioned fixtures and pre-production evidence.
- Shadow comparison metrics and retention cleanup require completion before agent removal.
- `agent-main` remains until reconciliation and rollback-window acceptance.

## Revisit triggers

Revisit identity mapping if provider device IDs differ from device UUIDs; revisit cursor transaction boundaries if provider cursors are not stable per device; revisit scheduler ownership if multiple apiserver replicas run without shared leases.
