# LeeAttend — Biometric Enrollment API Contract

> **Scope:** Design + data-model round. This document specifies the REST contract
> and data model for managing which attendance devices an employee is enrolled on.
> Backend handlers, agent worker methods, `rustzk` wiring, and the React UI are
> **out of scope** for this round — they are implemented against this contract later.

## Problem & Motivation

HR (office-based) needs to control which attendance devices an employee can clock
in on — including picking **one specific device** out of ~10 at the same location.

Today the pipeline is **pull-only** (device → agent → server, attendance logs
only). There is no record of "employee X is present on device Y", no server-side
biometric store, and the agent never writes users/templates to devices. This
contract introduces the data model and API surface to close that gap.

Key decisions (confirmed with product owner):
1. **Server is the source of truth for fingerprint templates.** Enroll once on any
   device → server pulls the template into an encrypted store → push to any target
   device. HR office can do everything remotely, no one at the machine.
2. **Multi-device membership.** An employee may be enrolled on many devices at once;
   add/remove is independent per device.
3. **Fingerprint only.** `rustzk` supports fingerprint template read/write; it has
   **no face-template API**. Face-only employees cannot be moved remotely — the UI
   must warn and treat that as out of automated scope.
4. **Two entry points for HR:** employee-centric (profile tab) and device-centric
   (connectivity screen).

## Two decoupled concepts

| Concept | Table | Role |
|---------|-------|------|
| **Template store** | `employee_biometric_templates` | Source of truth. Encrypted fingerprint templates. Every push reads from here. |
| **Device enrollment** | `device_enrollments` | Membership + sync state: which employees are (to be) on which devices. **What HR manipulates directly.** |

Work location (`devices.location_id`) is **only** a filter / quick-select hint for
bulk operations. It is never the source of truth for enrollment — HR always keeps
the ability to target a single device.

**Identity note:** a template's device-side identity is a per-device `uid: u16`, not
the alphanumeric `user_id`. Moving a person to a device = ensure a user exists on the
target (it assigns a uid there) → push the template under that uid. The
uid↔employee↔device mapping lives in `device_enrollments.device_uid` (per row).

## Data Model

Migration: `apiserver/migrations/tenant/20260719000000_biometric_enrollment/`
(tenant-scoped; `workspace_id` is a logical scoping column, no FK — database-per-tenant).

### `employee_biometric_templates`
| Column | Type | Notes |
|--------|------|-------|
| `id` | uuid PK | |
| `workspace_id` | uuid | logical scope |
| `employee_id` | uuid FK→employees ON DELETE CASCADE | |
| `finger_id` | smallint | 0–9 (CHECK) |
| `template_encrypted` | text | AES-256-GCM `{key_ver}:{nonce_b64}:{ct_b64}`. **PII T1, never returned raw.** |
| `encryption_key_version` | varchar(50) | |
| `source_device_id` | uuid FK→devices ON DELETE SET NULL | provenance |
| `quality` | smallint NULL | |
| `valid` | boolean DEFAULT true | |
| `created_at`/`updated_at` | timestamptz | |

UNIQUE `(employee_id, finger_id)` · INDEX `(workspace_id, employee_id)`

### `device_enrollments`
| Column | Type | Notes |
|--------|------|-------|
| `id` | uuid PK | |
| `workspace_id` | uuid | logical scope |
| `employee_id` | uuid FK→employees ON DELETE CASCADE | |
| `device_id` | uuid FK→devices ON DELETE CASCADE | |
| `device_uid` | integer NULL | u16 the device assigned; null until first push |
| `sync_status` | varchar(30) | state machine (CHECK) |
| `last_synced_at` | timestamptz NULL | |
| `last_error` | text NULL | |
| `created_at`/`updated_at` | timestamptz | |

UNIQUE `(employee_id, device_id)` · INDEX `(workspace_id, device_id, sync_status)` · INDEX `(employee_id)`

**`sync_status` state machine:**
```
pending_push ──► synced ──► pending_removal ──► removed
     │             │              │
     └─────────────┴──────────────┴──► failed  (any state on error)
```

### `devices.location_id` (new column)
Nullable uuid FK→work_locations ON DELETE SET NULL. Filter/quick-select hint only.

## REST API

All endpoints are tenant-scoped under `/workspaces/{ws}` and require `MemberContext`
+ RBAC (mirrors existing handlers: `ctx: MemberContext`, `ctx.auth_token`).
Fingerprint templates are **PII T1** — the raw bytes are never serialized to clients.

### Enrollment (membership) — what HR manipulates

#### `GET /devices/{device_id}/enrollments`
Who is on this device (device-centric view).
```jsonc
// 200
[{ "id": "uuid", "employee_id": "uuid", "device_id": "uuid",
   "device_uid": 42, "sync_status": "synced",
   "last_synced_at": "2026-07-19T…", "last_error": null,
   "created_at": "…", "updated_at": "…", "workspace_id": "uuid" }]
```

#### `GET /employees/{employee_id}/enrollments`
Which devices this employee is on (employee-centric view). Same `DeviceEnrollment` shape.

#### `POST /employees/{employee_id}/enrollments`
Enroll onto one or more **specific** devices (exact targets, even a single device).
```jsonc
// request
{ "device_ids": ["uuid", "uuid"] }
// 201 → created/updated enrollments as DeviceEnrollment[], each sync_status = "pending_push"
```
Idempotent per `(employee_id, device_id)`: an existing `removed`/`failed` row is
re-armed to `pending_push` rather than duplicated (UNIQUE constraint).

#### `DELETE /employees/{employee_id}/enrollments/{device_id}`
Transition the enrollment to `pending_removal` (soft — actual device removal happens
on the sync path later). `204`.

#### `POST /enrollments/preview`
Compute a **Sync Plan** without writing. Powers the confirm-before-apply UX.
```jsonc
// request — proposed changes
{ "add":    [{ "employee_id": "uuid", "device_ids": ["uuid"] }],
  "remove": [{ "employee_id": "uuid", "device_ids": ["uuid"] }] }
// 200
{ "push_count": 12, "remove_count": 3,
  "warnings": [
    { "employee_id": "uuid", "code": "no_fingerprint_template",
      "message": "Employee has no stored fingerprint; push will fail until enrolled at a device." }
  ] }
```

### Template store

#### `GET /employees/{employee_id}/biometric-templates`
Metadata only — which fingers exist, quality, provenance. **Never** the raw template.
```jsonc
// 200 — BiometricTemplateInfo[]
[{ "id": "uuid", "employee_id": "uuid", "finger_id": 1,
   "quality": 80, "valid": true, "source_device_id": "uuid",
   "created_at": "…", "updated_at": "…" }]
```

## Rust types (already added this round)

| Layer | File | Types |
|-------|------|-------|
| Domain DTO | `apiserver/src/models/device_enrollment.rs` | `SyncStatus`, `DeviceEnrollment`, `CreateEnrollmentsRequest` |
| Domain DTO | `apiserver/src/models/biometric_template.rs` | `BiometricTemplateInfo` |
| Diesel | `apiserver/src/models/diesel/device_enrollment.rs` | `DeviceEnrollmentEntity`, `NewDeviceEnrollment`, `UpdateDeviceEnrollment` |
| Diesel | `apiserver/src/models/diesel/biometric_template.rs` | `BiometricTemplateEntity`, `NewBiometricTemplate` |
| Device | `apiserver/src/models/agent.rs`, `apiserver/src/models/diesel/agent.rs` | `location_id` added to `Device`, `CreateDeviceRequest`, `UpdateDeviceRequest`, `DeviceEntity`, `UpdateDieselDevice` |

`sync_status` is stored as `VARCHAR` + DB CHECK (not a native PG enum), so the entity
keeps it as `String` and `SyncStatus` is applied at the API boundary via
`strum` `Display`/`FromStr`. Encryption reuses `CryptoService`
(`apiserver/src/crypto.rs`), same pattern as `national_id` in
`repositories/diesel_employee_repository/ess.rs`.

## Out of scope this round (next rounds)

- Handlers/routes/repositories implementing the endpoints above.
- Agent `Worker` trait methods `pull_user_template` / `push_user_template` /
  `delete_user`, and the WS command dispatch (`message_handler.rs`) to invoke them.
- Wiring `rustzk` (`get_user_template` / `save_user_template` / `set_user` /
  `delete_user`) into `agent-main`.
- The template pull-back flow that populates `employee_biometric_templates` after a
  first enrollment at a device.
- React UI: employee-detail devices tab, device-centric bulk add/remove, Sync Plan
  preview, per-device progress; i18n keys in `employees.json` / `settings.json` (en/vi/zh).
- Face templates — not extractable via `rustzk`; remote move unsupported by design.
