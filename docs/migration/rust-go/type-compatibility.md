# Type Compatibility & Serialization Contract — P2-04

**Parent ledger task:** P2-04  
**Dependencies:** P1-04 (`docs/migration/rust-go/type-mapping.md`)  
**Consumers:** P2-08 (`comparison-normalization.md`), P5-03 (Pagination & sorting), P5-04 (Validation & decoding), P8 canary and parity oracles.  
**Governing principle:** Strict wire and persistence parity between Rust `apiserver` and the Go target implementation. Grounded in source code citations and executable test fixtures. Zero undocumented behavioral drifting.

---

## 1. Executive Summary & Verification Matrix

The LeeAttend API server relies on specific serialization nuances in `serde`, `serde_json`, `chrono`, `chrono_tz`, `rust_decimal`, and Diesel PostgreSQL adapters. To guarantee byte-for-byte or semantically identical wire exchanges during differential shadow testing, Go implementations must replicate these rules without deviation.

| Type Category | Rust Baseline | Wire Format (JSON / HTTP) | DB PostgreSQL Type | Go Target Type | Parity Enforcement Rule |
|---|---|---|---|---|---|
| **UUID** | `uuid::Uuid` (v4/v7) | Lowercase 36-char canonical hex string (`8-4-4-4-12`) | `uuid` | `uuid.UUID` (`github.com/google/uuid`) | Case-insensitive decode; strict lowercase hyphenated encode; v7 for generated IDs (`utils/id.rs:19`). |
| **Missing vs Null** | `Option<T>` vs `Option<Option<T>>` | Missing key vs explicit `null` vs value | `NULL` vs column default | `*T` vs `Nullable[T]` | Tri-state update detection: PATCH handlers distinguish absent from explicit null. |
| **Enums** | Rust `enum` with serde attributes | String literals (`snake_case`, `SCREAMING_SNAKE_CASE`, PascalCase) | `VARCHAR` or PG `CREATE TYPE ... AS ENUM` | `type EnumName string` with typed consts | Wire serialization matches serde string verbatim; DB writes use DB-specific naming. Unknown variants reject with 400. |
| **JSON Objects** | `serde_json::Value` | Arbitrary JSON object/array/primitive | `jsonb` | `json.RawMessage` or typed struct | Preserves raw message payload; no silent key stripping or reordering on pass-through. |
| **Decimal / Money** | `rust_decimal::Decimal` | String (e.g. `"1500000.00"`) or numeric on legacy endpoints | `numeric` (`decimal(15,2)`, `numeric(12,4)`) or `float8` | `shopspring/decimal.Decimal` (D-04) | Decodes string or number; serializes as normalized decimal string; arithmetic forbids `float64`. |
| **Timestamp (Instant)** | `chrono::DateTime<Utc>` | RFC 3339 / ISO 8601 string (`YYYY-MM-DDTHH:MM:SSZ` or with offset) | `timestamptz` | `time.Time` | Inbound accepts any RFC 3339 offset and normalizes to UTC; outbound emits RFC 3339 in UTC (`Z`). Sub-second auto. |
| **Date-Only** | `chrono::NaiveDate` | `YYYY-MM-DD` | `date` | Custom `Date` (`time.Time` formatted `2006-01-02`) | Outbound strictly `YYYY-MM-DD`. Inbound accepts fallback formats in import services only. No UTC midnight shift. |
| **Timezone & DST** | `chrono_tz::Tz` | IANA TZ identifier string (`"Asia/Ho_Chi_Minh"`) | `text` / `varchar` | `*time.Location` / string | Strict IANA validation on writes; lenient UTC fallback on reads (`parse_workspace_tz`). DST gap: +1h; DST ambiguous: first instant. |
| **Binary & Bytea** | `Vec<u8>` | Base64 string in JSON, raw octet-stream in HTTP body, or download URL | `bytea` / storage path | `[]byte` | Raw octet stream with `Content-Disposition` for downloads; Base64 only if explicitly wrapped in DTO. |
| **Arrays & Nullable Elements**| `Vec<T>`, `Option<Vec<Option<T>>>` | JSON Array `[...]` | `ARRAY` / `T[]` | `[]T`, `*[]*T` | Distinct null vs empty array (`null` vs `[]`); distinguishes nullable array elements. |
| **Pagination** | `PaginationParams`, `PaginatedResponse<T>` | Standard envelope: `{data: [], total: N, page: N, limit: N}` | SQL `LIMIT ... OFFSET ...` | Standard Go Generic Envelope | `page` floored at 1 (`page >= 1`); `limit` clamped to `[1, 3000]` (`MAX_PAGE_LIMIT`). Truncation flag where supported. |

---

## 2. UUID Compatibility Rules

### 2.1 Specification & Grounded Rust Behavior
- **Generation:** Handled centrally by `apiserver/src/utils/id.rs:1-21`. Production DB records generate time-sorted **UUIDv7** (`uuid::Uuid::now_v7()`). Test fixtures and ephemeral keys use **UUIDv4** (`Uuid::new_v4()`).
- **Wire Representation:** Serialized via `serde` to 36-character canonical lowercase string with hyphens: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (`utils/id.rs:49-55`).
- **Parsing / Inbound:** Receptive to any valid 128-bit UUID format (hyphenated, unhyphenated, uppercase, lowercase). Rejects invalid string inputs with HTTP 400 Bad Request / Validation Error (`AppError::validation(I18nKey::InvalidId)` or Axum path rejection).
- **Database Storage:** Mapped directly to native PostgreSQL `uuid` type via Diesel (`schema.rs:63,81,98,115,150,170,189`).

### 2.2 Go Implementation Contract
- Target type: `github.com/google/uuid.UUID`.
- Inbound JSON unmarshaling: `uuid.Parse(str)` accepting standard hex and hyphens.
- Outbound JSON marshaling: Canonical lowercase string representation (`uuid.UUID.String()`).
- Database binding: `pgx/v5` native UUID driver.
- Generator interface:
  ```go
  // internal/pkg/id/id.go
  package id

  import "github.com/google/uuid"

  func NewID() uuid.UUID {
      id, err := uuid.NewV7()
      if err != nil {
          return uuid.New() // Fallback to v4 only if clock fails
      }
      return id
  }
  ```

### 2.3 Fixture Vectors: UUID

| Input Wire Value | Rust Result (`from_str`) | Go Result (`uuid.Parse`) | Canonical Outbound Wire | DB Cast (`$1::uuid`) | Status / Error |
|---|---|---|---|---|---|
| `"a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"` | Valid UUID | Valid UUID | `"a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"` | `a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8` | Accept (200/201) |
| `"A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8"` | Valid UUID | Valid UUID | `"a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8"` | `a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8` | Accept (Normalized) |
| `"018d3b84-1d88-7e3f-a316-29158fb69d27"` | Valid UUIDv7 | Valid UUIDv7 | `"018d3b84-1d88-7e3f-a316-29158fb69d27"` | `018d3b84-1d88-7e3f-a316-29158fb69d27` | Accept (v7) |
| `"00000000-0000-0000-0000-000000000000"` | Valid Nil UUID | Valid Nil UUID | `"00000000-0000-0000-0000-000000000000"` | `00000000-0000-0000-0000-000000000000` | Accept (Nil) |
| `"not-a-valid-uuid"` | Error (`InvalidLength`) | Error (`invalid UUID length`) | N/A | Syntax Error | Reject (400) |
| `""` (Empty string) | Error (`InvalidLength`) | Error (`invalid UUID length`) | N/A | Syntax Error | Reject (400) |
| `12345` (Numeric) | Deserialization error | Deserialization error | N/A | Type Error | Reject (400) |

---

## 3. Nullable, Missing, and Empty Representation Rules

### 3.1 Specification & Grounded Rust Behavior
In Rust serde, there are three distinct field presence scenarios across query, create, and update requests:
1. **Required Non-Null:** Declared as `T`. Omission or explicit `null` fails JSON deserialization with 400/422.
2. **Optional Nullable (Two-State):** Declared as `Option<T>`.
   - Omitted field $\rightarrow$ `None`.
   - Explicit `null` $\rightarrow$ `None`.
   - Present value $\rightarrow$ `Some(v)`.
   - On serialization: Omitted if `#[serde(skip_serializing_if = "Option::is_none")]` (e.g. `PaginatedResponse.truncated` in `models/common.rs:199-200`, `ConfirmTerminationResponse.job_id` in `handlers/employees.rs:573`); serialized as explicit `null` otherwise.
3. **Tri-State Partial Update (`deserialize_option_option`):**
   - Source: `apiserver/src/utils/serde_helpers.rs:1-85`.
   - Attribute: `#[serde(default, deserialize_with = "crate::utils::serde_helpers::deserialize_option_option")]`.
   - Field type: `Option<Option<T>>`.
   - Used in update DTOs:
     - `UpdateAssignmentRuleRequest.start_date`, `.end_date` (`models/employee.rs:1074,1079`).
     - `UpdateEmployeeRequest.user_id` (`models/employee.rs:1164`).
     - `UpdateEmploymentRequest.primary_location_id`, `.department_id`, `.end_date`, `.manager_employment_id` (`models/employment.rs:183,188,195,202`).
     - `UpdateCustomFieldDefinitionRequest.default_value`, `.section_name`, `.validation_rules` (`models/custom_field.rs:433,443,448`).
     - `UpdateEmployerRequest.legal_address`, `.tax_code` (`models/employer.rs:50,55`).
   - Semantics:
     - **Absent in JSON:** `None` $\rightarrow$ Field was not supplied; DO NOT TOUCH existing DB value.
     - **Explicit `null` in JSON:** `Some(None)` $\rightarrow$ Explicit client instruction to clear / NULL the DB field.
     - **Value provided in JSON:** `Some(Some(v))` $\rightarrow$ Update DB field to new value `v`.
4. **Empty Strings vs Null:**
   - Text fields store `""` as empty string in Postgres, while `null` sets SQL `NULL`.
   - `normalize_employee_id` (`utils/mod.rs:37-62`): Empty or whitespace-only strings normalize to `"0"`. Non-empty numeric IDs strip leading zeros.

### 3.2 Go Implementation Contract
- **Two-State Optional:** Pointer type `*T` with struct tag `json:"field,omitempty"` or `json:"field"`.
- **Tri-State Optional (`Nullable[T]`):** Go standard pointers cannot distinguish omitted from `null`. Go must provide a dedicated generic type implementing `json.Unmarshaler`:
  ```go
  // internal/pkg/types/nullable.go
  package types

  import (
      "bytes"
      "encoding/json"
  )

  type Nullable[T any] struct {
      Defined bool // True if field was present in JSON (even if null)
      Valid   bool // True if field was present AND not null
      Value   T
  }

  func (n *Nullable[T]) UnmarshalJSON(data []byte) error {
      n.Defined = true
      if bytes.Equal(data, []byte("null")) {
          n.Valid = false
          return nil
      }
      n.Valid = true
      return json.Unmarshal(data, &n.Value)
  }

  func (n Nullable[T]) MarshalJSON() ([]byte, error) {
      if !n.Defined || !n.Valid {
          return []byte("null"), nil
      }
      return json.Marshal(n.Value)
  }
  ```

### 3.3 Fixture Vectors: Tri-State Semantics

Grounded in `apiserver/src/utils/serde_helpers.rs:26-85`:

| Field Payload (e.g. `end_date`) | Rust `Option<Option<T>>` | Go `Nullable[T]` State | Intended SQL Operation | Test Verification |
|---|---|---|---|---|
| `{}` (Key omitted) | `None` | `Defined=false, Valid=false` | Column omitted from `UPDATE ... SET` | Existing column unchanged |
| `{"end_date": null}` | `Some(None)` | `Defined=true, Valid=false` | `SET end_date = NULL` | Column set to SQL `NULL` |
| `{"end_date": "2026-12-31"}` | `Some(Some(2026-12-31))` | `Defined=true, Valid=true, Value="2026-12-31"` | `SET end_date = '2026-12-31'` | Column updated |
| `{"department_id": null}` | `Some(None)` | `Defined=true, Valid=false` | `SET department_id = NULL` | FK cleared |
| `{"user_id": null}` | `Some(None)` | `Defined=true, Valid=false` | `SET user_id = NULL` | Unlinked from user account |

---

## 4. Enum Representation & Serialization

### 4.1 Specification & Grounded Rust Behavior
LeeAttend enums use varying casing strategies depending on the subsystem. Casing rules are governed by serde and strum attributes across `apiserver/src/models/enums.rs`, `models/attendance.rs`, `handlers/worker_monitor.rs`, and `models/dashboard_metrics.rs`:

1. **`snake_case` (Default API standard):**
   - `ContractType`: `probation`, `training`, `fixed_term`, `indefinite` (`models/enums.rs:19-25`).
   - `ContractStatus`: `draft`, `active`, `expired`, `terminated` (`models/enums.rs:43-49`).
   - `LeaveStatus`: `pending`, `approved`, `rejected`, `cancelled`, `pre_approved` (`models/enums.rs:66-73`).
   - `LeaveDurationType`: `full_day`, `first_half`, `second_half`, `custom_hours` (`models/enums.rs:88-95`).
   - `AdjustmentStatus`: `pending`, `approved`, `rejected` (`models/enums.rs:111-116`).
   - `LogSource`: `device`, `import`, `manual` (`models/enums.rs:132-137`).
   - `ImportJobStatus`: `pending`, `processing`, `partially_failed`, `completed`, `failed` (`models/enums.rs:154-162`).
   - `ImportTargetType`: `log`, `adjustment`, `employee`, `leave_import` (`models/enums.rs:176-183`).
   - `LeaveTransactionType`: `accrual`, `usage`, `adjustment`, `carry_over` (`models/enums.rs:188-194`).
   - `EarlyArrivalStrategy`: `allow`, `block`, `clamp` (`models/enums.rs:208-214`).
   - `RoundingPolicy`: `floor`, `round`, `ceil`, `none` (`models/attendance.rs:472-479`). Tolerant fallback: unknown strings deserialize to `None` in Rust.
   - `DashboardMetricUnit`: `count`, `rate`, `hours`, `currency`, `duration`, `percent`, `percentage_point_delta` (`models/dashboard_metrics.rs:190-198`).
   - `DashboardMetricStatus`: `available`, `not_applicable`, `data_gap`, `restricted` (`models/dashboard_metrics.rs:202-208`).
   - `DashboardMetricHealth`: `ok`, `warning`, `critical`, `not_applicable`, `data_gap` (`models/dashboard_metrics.rs:211-218`).
   - `DashboardMetricDirection`: `higher_is_better`, `lower_is_better`, `neutral` (`models/dashboard_metrics.rs:221-226`).
   - `DashboardComparisonBasis`: `none`, `previous_period`, `same_period_last_year`, `target`, `budget`, `benchmark` (`models/dashboard_metrics.rs:272-280`).
   - `DashboardInsightType`: `trend`, `threshold`, `concentration`, `data_quality`, `workflow` (`models/dashboard_metrics.rs:655-662`).
   - `DashboardInsightSeverity`: `info`, `watch`, `high`, `critical` (`models/dashboard_metrics.rs:665-671`).
   - `ShiftSwapStatus`: `pending_target`, `pending_admin`, `approved`, `rejected`, `cancelled` (`models/shift_swap.rs:10-17`).
   - `SignatureStatus`: `pending`, `signed`, `declined`, `canceled` (`models/signature.rs:7-13`).
   - `WorkerLiveness`: `live`, `stale`, `offline` (`handlers/worker_monitor.rs:81-86`).
   - `MonitorInfraJobState`: `dispatching`, `queued`, `processing`, `completed`, `failed`, `retryable`, `dead_letter` (`handlers/worker_monitor.rs:246-253`).
   - `MonitorSourceStatus`: `available`, `degraded`, `unavailable` (`handlers/worker_monitor.rs:280-284`).
   - `MonitorDataQuality`: `complete`, `partial`, `degraded`, `unavailable` (`handlers/worker_monitor.rs:288-293`).
   - `MonitorFreshness`: `fresh`, `aging`, `stale`, `unknown` (`handlers/worker_monitor.rs:297-302`).
   - `ViolationType`: `overlap`, `rest_period`, `weekly_hours`, `monthly_overtime` (`models/compliance.rs:46-56`).
   - `ViolationSeverity`: `hard`, `soft` (`models/compliance.rs:59-65`).
   - `ChecklistCategory`: `hr_admin`, `it_setup`, `documentation`, `training`, `equipment` (`models/lifecycle_checklist.rs:48-55`).
   - `ChecklistAssigneeType`: `hr_manager`, `direct_manager`, `it_admin`, `employee`, `specific_user` (`models/lifecycle_checklist.rs:85-92`).
   - `ChecklistStatus`: `in_progress`, `completed`, `cancelled` (`models/lifecycle_checklist.rs:122-127`).
   - `ChecklistItemStatus`: `pending`, `completed`, `skipped`, `overdue` (`models/lifecycle_checklist.rs:153-159`).

2. **`SCREAMING_SNAKE_CASE`:**
   - `WorkerStatus`: `IDLE`, `PROCESSING` (`handlers/worker_monitor.rs:73-76`).
   - `AttendanceStatusType` (legacy report): `models/enums.rs:408`.

3. **PascalCase / Exact Renames:**
   - `WorkingConditionType`: `EarlyLeave`, `LateIn`, `Breastfeeding` (`models/enums.rs:217-221`).
   - `ConditionEarlyArrivalStrategy`: `Allow`, `Block`, `Clamp` (`models/condition_type.rs:64-68`).
   - `PayoutType`: `Fixed`, `Percentage` (`models/enums.rs:224-227`).
   - `HolidayOvernightStrategy`: `SplitAtMidnight`, `ByPunchInDate`, `ByMajorityHours` (`models/holiday.rs:161-170`).
   - `DashboardTargetOperator`: `lt`, `lte`, `eq`, `gte`, `gt` (`models/dashboard_metrics.rs:283-294`).

4. **Complex / External Tagged Enums:**
   - `ConfigValue`: `{"Int": 1}`, `{"Decimal": "1.5"}`, `{"Text": "..."}`, `{"Bool": true}`, `{"Json": {...}}` (`models/workspace_config.rs:9-15`).
   - `HolidayLeaveResult`: `Paid`, `{"LeaveDeducted": {...}}`, `{"Unpaid": {...}}` (`repositories/holiday_repository.rs:11-23`).
   - `FileDownloadResponse`: `ProxyBytes(Vec<u8>)`, `PresignedUrl(String)` (`models/file.rs:235-238`).

### 4.2 Go Implementation Contract
- Represent each enum as a Go string type:
  ```go
  type LeaveStatus string

  const (
      LeaveStatusPending     LeaveStatus = "pending"
      LeaveStatusApproved    LeaveStatus = "approved"
      LeaveStatusRejected    LeaveStatus = "rejected"
      LeaveStatusCancelled   LeaveStatus = "cancelled"
      LeaveStatusPreApproved LeaveStatus = "pre_approved"
  )
  ```
- Reject unknown enum strings with `400 Bad Request` on unmarshal. Do not accept arbitrary strings.

### 4.3 Fixture Vectors: Enum JSON Parity

| Rust Enum Type | Source Citation | Variant | Expected Wire JSON | Inbound Go Action |
|---|---|---|---|---|
| `LeaveStatus` | `models/enums.rs:67` | `PreApproved` | `"pre_approved"` | Parse `LeaveStatusPreApproved` |
| `AdjustmentStatus` | `models/enums.rs:112` | `Approved` | `"approved"` | Parse `AdjustmentStatusApproved` |
| `EarlyArrivalStrategy` | `models/enums.rs:209` | `Block` | `"block"` | Parse `EarlyArrivalStrategyBlock` |
| `WorkerStatus` | `handlers/worker_monitor.rs:73`| `Idle` | `"IDLE"` | Parse `WorkerStatusIdle` |
| `WorkingConditionType` | `models/enums.rs:217` | `EarlyLeave` | `"EarlyLeave"` | Parse `WorkingConditionTypeEarlyLeave` |
| `DashboardTargetOperator`| `models/dashboard_metrics.rs:283`| `Gte` | `"gte"` | Parse `DashboardTargetOperatorGte` |
| `RoundingPolicy` | `models/attendance.rs:473` | `Floor` | `"floor"` | Parse `RoundingPolicyFloor` |
| `RoundingPolicy` | `models/attendance.rs:470` | (Unknown: `"round_down"`)| N/A (inbound) | Deserializes to default `RoundingPolicyNone` |

---

## 5. JSON Field Compatibility & Untagged / Flattened Objects

### 5.1 Specification & Grounded Rust Behavior
1. **`serde_json::Value` (Free-form / Dynamic JSON):**
   - Mapped to PostgreSQL `jsonb`.
   - Examples:
     - `approval_workflow_instances.steps_snapshot` (`schema.rs:90`, `handlers/approval_workflow.rs:60`).
     - `approval_workflow_steps.approver_config` (`schema.rs:105`).
     - `assignment_rules.conditions` (`schema.rs:156`).
     - `attendance_policies.rules` (`schema.rs:280`).
     - `custom_field_values.value_json` (`schema.rs:862`).
     - `dashboard_preferences.visible_widgets` (`handlers/dashboard_preferences.rs:21`).
     - `audit_logs.details` (`domain/audit.rs:16`).
   - Must be preserved verbatim: no property re-ordering or key stripping on write paths.
2. **`#[serde(flatten)]`:**
   - Inlined child struct fields into parent JSON object.
   - Examples:
     - `FullPayslipResponse`: flattens `Payslip` fields at top level alongside `segments`, `incentives`, etc. (`handlers/payroll/payslips.rs:35`).
     - `PayslipSegmentResponse`: flattens `PayslipSegment` alongside `reason` (`handlers/payroll/payslips.rs:28`).
     - `EmployeeContractResponse`: flattens contract metadata (`models/contract.rs:132`).
3. **`#[serde(untagged)]`:**
   - Polymorphic deserialization without discriminator tag.
   - Example: `serde_decimal_as_str` (`models/payroll.rs:498-511`): Deserializes either a string (`"22.5"`) or a number (`22.5`) into `Decimal`.

### 5.2 Go Implementation Contract
- `serde_json::Value` $\rightarrow$ `json.RawMessage` (or `map[string]any` if validated).
- Flattened structs in Go:
  - If read-only DTO: Embed the inner struct as an anonymous field (`struct { Payslip; Segments []... }`).
  - If dual read/write: Provide custom `MarshalJSON` and `UnmarshalJSON` to ensure embedded fields are merged correctly without conflicting keys.
- Untagged types: Provide explicit custom unmarshaler that tests string first, then number, then errors.

---

## 6. Decimal & Float Precision Rules

### 6.1 Specification & Grounded Rust Behavior
- **Governing ADR:** Decision D-04 (`docs/migration/rust-go/decisions.md:15`) strictly prohibits `float64` in money, salary, and rate computations.
- **Rust Implementation:** Uses `rust_decimal::Decimal` (128-bit fixed-point decimal, scale up to 28 digits).
- **Wire Representation:**
  - Standard `rust_decimal`: Serializes as a JSON string (e.g. `"95"`, `"33.33"`, `"1.2"`, `"1.29"`, confirmed by tests in `models/common.rs:231-243` and `models/dashboard_metrics.rs:831-836`).
  - Payroll configs: `serde_decimal_as_str` (`models/payroll.rs:482-512`) serializes with `value.normalize().to_string()` (omits trailing zeros: `Decimal("22.50")` $\rightarrow$ `"22.5"`). Inbound accepts both string and number.
- **Legacy Float Anomaly (Decision D-12):**
  - PostgreSQL columns in payroll: `payroll_incentive_policies.base_amount`, `payroll_incentive_tiers.payout_amount`, `payslip_incentives.amount`, `payslip_incentives.standard_amount` use `Float8` (`schema.rs:1579,1594,1649,1652`).
  - Rust models read them as `f64`. Go MUST read these as `decimal.Decimal` at the database boundary (via string conversion or SQL cast `amount::numeric`) and execute all arithmetic in `decimal.Decimal`.

### 6.2 Go Implementation Contract
- Target type: `github.com/shopspring/decimal.Decimal`.
- Database mapping: `pgx` numeric adapter (`github.com/jackc/pgx/v5/pgtype.Numeric`) or `decimal.Decimal` directly.
- Serialization rule:
  - Default: Always format as JSON string.
  - Inbound decoding: Accept both string (`"123.45"`) and numeric JSON token (`123.45`).
  - Outbound formatting: Call `.String()` or `.Truncate(scale).String()`. For endpoints matching `serde_decimal_as_str`, call `.Normalize().String()`.

### 6.3 Fixture Vectors: Decimal & Money

| Input JSON Wire | Target Rust Type | Decoded Rust Value | Expected Go `decimal.Decimal` | Outbound JSON Wire | DB Persistence |
|---|---|---|---|---|---|
| `"1500000"` | `Decimal` | `1500000` | `decimal.NewFromInt(1500000)` | `"1500000"` | `1500000.00` |
| `"22.5"` | `Decimal` (payroll) | `22.5` | `decimal.NewFromFloat(22.5)` | `"22.5"` | `22.50` |
| `22.5` (Numeric) | `Decimal` (payroll) | `22.5` | `decimal.NewFromFloat(22.5)` | `"22.5"` | `22.50` |
| `"33.33"` | `RateQuantity.value` | `33.33` | `decimal.NewFromFloat(33.33)` | `"33.33"` | N/A (Derived) |
| `null` | `Option<Decimal>` | `None` | `*Decimal = nil` | `null` | SQL `NULL` |
| `"invalid"` | `Decimal` | Deserialization Err | Parse Error | N/A | Reject (400) |

---

## 7. Timestamp & Instant Compatibility Rules

### 7.1 Specification & Grounded Rust Behavior
- **Type:** `chrono::DateTime<Utc>`.
- **Wire Format:** RFC 3339 / ISO 8601 string.
  - Deserialization (`chrono::datetime::serde`): Accepts any RFC 3339 string with valid offset (e.g. `2026-01-16T10:00:00Z`, `2026-01-16T17:00:00+07:00`, `2025-12-31T19:00:00-05:00`) and converts to UTC instant.
  - Serialization: Formatted as RFC 3339 in UTC with `Z` or `+00:00` offset (`write_rfc3339(f, naive, offset, SecondsFormat::AutoSi, true)`). Sub-seconds are omitted if zero, or serialized up to nanosecond precision (`.000` / `.123456`).
- **Database Mapping:** PostgreSQL `timestamptz` (`timestamp with time zone`). Mapped via Diesel (`schema.rs:75,91,109,125,143,172`).
- **Storage Rule:** All timestamps stored in database and Redis MUST be normalized to UTC. Local timezone offsets are NEVER stored in instant columns.

### 7.2 Go Implementation Contract
- Target type: `time.Time`.
- Inbound: Standard `time.RFC3339` and `time.RFC3339Nano` parsing. Immediately normalize using `.UTC()`.
- Outbound: Formatted with `.Format(time.RFC3339Nano)` or `.Format(time.RFC3339)`.

### 7.3 Fixture Vectors: Timestamp Serialization & Normalization

| Input Timestamp String | Workspace TZ | Parsed Instant (UTC) | Outbound JSON Wire | Verification Method |
|---|---|---|---|---|
| `"2026-01-16T10:00:00Z"` | Any | `2026-01-16 10:00:00 UTC` | `"2026-01-16T10:00:00Z"` | Exact match (`utils/datetime.rs:198-202`) |
| `"2026-01-16T17:00:00+07:00"` | Any | `2026-01-16 10:00:00 UTC` | `"2026-01-16T10:00:00Z"` | Offset converted to UTC |
| `"2026-01-16T10:00:00.123Z"` | Any | `2026-01-16 10:00:00.123 UTC` | `"2026-01-16T10:00:00.123Z"` | Millisecond preservation |
| `"2026-01-16T05:00:00-05:00"` | Any | `2026-01-16 10:00:00 UTC` | `"2026-01-16T10:00:00Z"` | Negative offset converted |
| `"invalid-date-format"` | Any | Parse Error | N/A | Reject (400) |

---

## 8. Date-Only Representation Rules

### 8.1 Specification & Grounded Rust Behavior
- **Type:** `chrono::NaiveDate`.
- **Wire Format:** ISO 8601 calendar date `YYYY-MM-DD`.
  - Serialized via `serde` directly as `"YYYY-MM-DD"`.
  - Deserialized strictly as calendar date.
- **Inbound Import Tolerance (`parse_date` in `utils/datetime.rs:99-191`):**
  - Interactive API endpoints require strict `YYYY-MM-DD`.
  - Attendance excel/file import handlers (`utils/datetime.rs`) accept localized formats:
    - `DD/MM/YYYY`, `DD-MM-YYYY`, `DD.MM.YYYY`
    - `MM/DD/YYYY`, `MM-DD-YYYY`, `MM.DD.YYYY`
    - `YYYY/MM/DD`, `YYYY.MM.DD`
- **Database Mapping:** PostgreSQL `date` (`schema.rs:191`).
- **Critical Policy:** NEVER store `NaiveDate` as UTC midnight `timestamptz`. Doing so causes calendar date shifts across timezones (e.g. `2026-01-16 00:00:00 UTC` is `2026-01-15 19:00:00 EST`).

### 8.2 Go Implementation Contract
- Custom `Date` type wrapping `time.Time`:
  ```go
  // internal/pkg/types/date.go
  package types

  import (
      "fmt"
      "strings"
      "time"
  )

  type Date struct {
      time.Time
  }

  const DateFormat = "2006-01-02"

  func (d *Date) UnmarshalJSON(b []byte) error {
      s := strings.Trim(string(b), "\"")
      if s == "null" || s == "" {
          return nil
      }
      t, err := time.Parse(DateFormat, s)
      if err != nil {
          return fmt.Errorf("invalid date format: %w", err)
      }
      d.Time = t
      return nil
  }

  func (d Date) MarshalJSON() ([]byte, error) {
      if d.IsZero() {
          return []byte("null"), nil
      }
      return []byte(fmt.Sprintf("\"%s\"", d.Format(DateFormat))), nil
  }
  ```

---

## 9. Timezone & Daylight Saving Time (DST) Rules

### 9.1 Specification & Grounded Rust Behavior
Unified in `apiserver/src/utils/tz.rs:1-120` and `apiserver/src/utils/datetime.rs:1-87`:

1. **Workspace Timezone Parsing (`parse_workspace_tz` in `tz.rs:23-32`):**
   - Read path (lenient): If timezone string is empty or invalid, logs warning and falls back to `UTC` (or `chrono_tz::UTC`).
   - Write path (`validate_tz` in `tz.rs:47-52`): Strict IANA timezone validation (`workspace_tz.parse::<Tz>()`). Rejects invalid timezone names with `AppError::validation(I18nKey::InvalidTimezone)`.
2. **Local to UTC Conversion (`local_to_utc` in `tz.rs:69-92`):**
   - Converts local wall-clock date/time `(date, h, m, s)` to `DateTime<Utc>`.
   - **Single Result:** Direct mapping.
   - **Ambiguous Result (DST Fall-back):** Occurs when the clock is turned backward (e.g. 01:30 repeats). Strategy: **Pick the first occurrence** (pre-transition / standard time). Grounded in test `datetime.rs:278-296` (`America/New_York` `2026-11-01 01:30:00` $\rightarrow$ `05:30 UTC`).
   - **Missing Result / Gap (DST Spring-forward):** Occurs when the clock jumps forward (e.g. 02:00 jumps to 03:00, 02:30 does not exist). Strategy: **Shift forward by 1 hour** (`naive + Duration::hours(1)`), resolve to post-gap instant. Grounded in test `datetime.rs:253-269` (`America/New_York` `2026-03-08 02:30:00` shifted to `03:30 EDT` $\rightarrow$ `07:30 UTC`).
3. **Offset Priority:**
   - If incoming string has an explicit offset (ISO 8601 with `Z` or `+HH:MM`), parse directly. Workspace timezone is **ignored** (`datetime.rs:328-336`).

### 9.2 Go Implementation Contract
- Target timezone provider: Go `time.LoadLocation` (using IANA tzdb).
- DST handling algorithm in Go:
  ```go
  // internal/pkg/tz/tz.go
  package tz

  import (
      "fmt"
      "time"
  )

  func LocalToUTC(loc *time.Location, year int, month time.Month, day, hour, min, sec int) (time.Time, error) {
      // Construct naive representation
      localTime := time.Date(year, month, day, hour, min, sec, 0, loc)
      
      // Check for gap (Spring forward): if local components don't match, time was invalid in that zone
      if localTime.Hour() != hour {
          // Gap occurred: Shift forward by 1 hour
          shifted := time.Date(year, month, day, hour+1, min, sec, 0, loc)
          return shifted.UTC(), nil
      }

      // In Go, time.Date on an ambiguous fall-back automatically selects standard/pre-transition
      return localTime.UTC(), nil
  }
  ```

### 9.3 Fixture Vectors: Timezone & DST

| Local Wall-Clock String | Timezone | Transition Scenario | Resolved UTC Instant | Source Grounding |
|---|---|---|---|---|
| `"16/01/2026 08:00"` | `"Asia/Ho_Chi_Minh"` | Normal (UTC+7, no DST) | `2026-01-16T01:00:00Z` | `datetime.rs:205-215` |
| `"12/31/2025 11:00 PM"` | `"America/New_York"` | Normal (UTC-5, EST) | `2026-01-01T04:00:00Z` | `datetime.rs:218-228` |
| `"08/03/2026 02:30:00"` | `"America/New_York"` | DST Gap (Spring Forward: 02:00 $\rightarrow$ 03:00) | `2026-03-08T07:30:00Z` (03:30 EDT) | `datetime.rs:253-269` |
| `"01/11/2026 01:30:00"` | `"America/New_York"` | DST Ambiguous (Fall Back: 01:30 twice) | `2026-11-01T05:30:00Z` (1st EDT) | `datetime.rs:278-296` |
| `"16/01/2026 00:00:00"` | `"Asia/Ho_Chi_Minh"` | Midnight boundary | `2026-01-15T17:00:00Z` | `datetime.rs:314-325` |
| `"2026-01-16T10:00:00+02:00"` | `"Asia/Ho_Chi_Minh"` | String has offset | `2026-01-16T08:00:00Z` (TZ ignored) | `datetime.rs:328-336` |
| `"16/01/2026 08:00"` | `"Invalid/Timezone"` | Invalid Zone | Error: `InvalidTimezone` | `datetime.rs:244-250` |

---

## 10. Binary Data & Byte Arrays

### 10.1 Specification & Grounded Rust Behavior
1. **File Downloads:**
   - Handled via `FileDownloadResponse` (`models/file.rs:235-238`, `handlers/files.rs:260-288`).
   - `ProxyBytes(Vec<u8>)`: Transmitted as HTTP body with `Content-Type: <mime_type>` and `Content-Disposition: attachment; filename="..."`. NEVER base64-encoded in a JSON wrapper.
   - `PresignedUrl(String)`: Returns HTTP 307 Temporary Redirect to object storage URL.
2. **Template & Error Downloads:**
   - `download_import_error_file` (`handlers/attendance/import.rs:323,360-371`): Emits raw XLSX bytes with MIME `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`.
3. **Encrypted Biometric Payloads (PII T1 Guard):**
   - Source: `models/diesel/biometric_template.rs:7-94`.
   - `BiometricTemplateEntity.template_encrypted` is stored in PostgreSQL as `String` (Base64/Ciphertext) / `bytea`.
   - **Inviolable Privacy Rule:** `template_encrypted` and `encryption_key_version` are strictly dropped when mapping to `BiometricTemplateInfo`. They MUST NEVER be serialized to API clients (`models/diesel/biometric_template.rs:88-93`).

### 10.2 Go Implementation Contract
- Target type: `[]byte`.
- File download responses: Write raw `[]byte` directly to `http.ResponseWriter` with appropriate headers.
- If an internal API or message queue requires binary in JSON: Use standard RFC 4648 Base64 string encoding (`encoding/base64.StdEncoding`).

---

## 11. Array Representations & Nullable Elements

### 11.1 Specification & Grounded Rust Behavior
1. **Empty vs Null Arrays:**
   - In JSON, `[]` (empty list) is distinct from `null` or omitted field.
   - In Rust, `Vec<T>` serializes as `[]`. `Option<Vec<T>>` serializes as `null` or omitted.
   - Test evidence: `CachedMemberContext.permissions` (`cache/redis_cache.rs:289-305`): Empty permissions serializes as `[]`, while `managed_department_ids` serializes as `null` when `None`, and `["..."]` when present.
2. **Arrays with Nullable Elements:**
   - PostgreSQL allows arrays with NULL elements: `Array<Nullable<Uuid>>`.
   - Rust representation: `Option<Vec<Option<Uuid>>>` (e.g. `attendance_adjustment_requests.attachment_file_ids` in `schema.rs:180`, `models/diesel/attendance_adjustment.rs:50`, `models/diesel/leave.rs:66`).
   - Rust model mapping: Entity carries `Option<Vec<Option<Uuid>>>`, but public domain DTO filters out nulls to `Option<Vec<Uuid>>` (`models/attendance.rs:127`, `models/leave.rs:183`).

### 11.2 Go Implementation Contract
- Non-nullable array: `[]T`. Serializes to `[]` when non-nil empty slice (`make([]T, 0)`), or `null` if nil. To preserve Rust `[]`, initialize slices as non-nil.
- Nullable array: `*[]T`.
- Array of nullable elements: `[]*T` or `*[]*T`.
- Public DTOs must sanitize and drop internal null items where Rust domain models do.

---

## 12. Pagination Envelope & Query Parameter Rules

### 12.1 Specification & Grounded Rust Behavior
- **Query DTO:** `PaginationParams` (`models/common.rs:173-184`):
  - `page: Option<i64>`
  - `limit: Option<i64>`
- **Clamping Engine (`SafePagination` in `handlers/common.rs:430-509`):**
  - `page`: Default is 1; floored at 1: `page.unwrap_or(1).max(1)`. Values $\le 0$ are silently adjusted to 1 (`handlers/common.rs:1240-1246`).
  - `limit`: Clamped to `1..=MAX_PAGE_LIMIT` (`MAX_PAGE_LIMIT = 3000`): `limit.unwrap_or(default).clamp(1, 3000)` (`handlers/common.rs:438,489,1224-1238`).
  - Zero, negative, and oversized limits: `0` $\rightarrow$ `1`; `-1` $\rightarrow$ `1`; `1_000_000` $\rightarrow$ `3000`.
  - Default limits:
    - Codebase standard: `50` (or `DEFAULT_PAGE_LIMIT`).
    - Reports & Attendance Logs: `20` (`DEFAULT_REPORT_PAGE_LIMIT = 20`, `handlers/attendance/logs.rs:110`, `handlers/common.rs:435`).
    - Staging / Monitoring: `50` (clamped to `100`, `handlers/attendance/staging.rs:84`, `handlers/workspaces/audit.rs:56`).
- **Response Envelope (`PaginatedResponse<T>` in `models/common.rs:192-201`):**
  ```json
  {
    "data": [...],
    "total": 128,
    "page": 1,
    "limit": 50,
    "truncated": true
  }
  ```
  - `data: Vec<T>`: Required JSON array. Empty result is `[]`, never `null`.
  - `total: usize`: Total count across all pages (maps to integer).
  - `page: i64`: Current page number ($\ge 1$).
  - `limit: i64`: Applied page limit ($\ge 1$).
  - `truncated: Option<bool>`: Serialized only when `Some(true)`; omitted when `None` via `#[serde(skip_serializing_if = "Option::is_none")]`.

### 12.2 Go Implementation Contract
- Target Generic Envelope:
  ```go
  // internal/pkg/types/pagination.go
  package types

  type PaginatedResponse[T any] struct {
      Data      []T   `json:"data"`
      Total     int64 `json:"total"`
      Page      int64 `json:"page"`
      Limit     int64 `json:"limit"`
      Truncated *bool `json:"truncated,omitempty"`
  }

  func ClampPagination(reqPage, reqLimit *int64, defaultLimit, maxLimit int64) (page, limit int64) {
      page = 1
      if reqPage != nil && *reqPage > 1 {
          page = *reqPage
      }
      limit = defaultLimit
      if reqLimit != nil {
          limit = *reqLimit
      }
      if limit < 1 {
          limit = 1
      } else if limit > maxLimit {
          limit = maxLimit
      }
      return page, limit
  }
  ```

### 12.3 Fixture Vectors: Pagination Clamping

| Query `?page=` | Query `?limit=` | Endpoint Class | Default Limit | Resolved `page` | Resolved `limit` | Test Grounding |
|---|---|---|---|---|---|---|
| Absent | Absent | Standard List | 50 | 1 | 50 | `handlers/common.rs:1266` |
| `0` | Absent | Standard List | 50 | 1 | 50 | `handlers/common.rs:1240` |
| `-5` | `0` | Standard List | 50 | 1 | 1 | `handlers/common.rs:1231,1240` |
| `2` | `1000000` | Standard List | 50 | 2 | 3000 | `handlers/common.rs:1224` |
| Absent | Absent | Attendance Report | 20 | 1 | 20 | `handlers/common.rs:1272` |
| `1` | `500` | Attendance Logs | 20 | 1 | 100 (Capped at 100) | `handlers/attendance/logs.rs:110` |
| `3` | `100` | Standard List | 50 | 3 | 100 | `handlers/common.rs:1248` |

---

## 13. Unresolved Ambiguities & Contract Decisions

The following edge cases represent potential behavioral divergence between Rust and Go. Each has been grounded in repository evidence and assigned a concrete compatibility rule:

| # | Ambiguity Area | Evidence in Codebase | Fixed Migration Rule | Unresolved Decision / Escalation |
|---|---|---|---|---|
| **A-01** | Float8 in Payroll Incentives | `payroll_incentive_policies.base_amount` is `Float8` in `schema.rs:1579`, but D-04 prohibits float arithmetic. | Go DB driver reads `Float8` as `decimal.Decimal` via string/numeric conversion; all internal math uses `shopspring/decimal`. | Logged in `decisions.md:23` (Decision D-12: Approved/Escalated). No schema migration during initial cutover. |
| **A-02** | Decimal String vs Number on Wire | `serde_decimal_as_str` (`models/payroll.rs:498-511`) accepts string or number, whereas `models/common.rs:238` outputs decimal string. | Go unmarshaler accepts string or number for any decimal field; outbound serialization always produces normalized string (`"22.5"`). | Resolved. Parity oracle tests with both representations. |
| **A-03** | Tolerant Enum Fallback vs Strict Reject | `RoundingPolicy` (`models/attendance.rs:470`) falls back to `None` on unknown variant; other enums reject with 400. | Replicate exact Rust behavior: `RoundingPolicy` defaults to `none` on unknown string; all other enums fail validation with HTTP 400. | Grounded in `models/attendance.rs:470`. Resolved. |
| **A-04** | DST Ambiguous Wall-Clock Hour | `America/New_York` fall-back (01:30 twice) picks first instant in `tz.rs:80`. | Go must pick first instant (EDT, UTC-4), not second instant. | Verified by unit test in `utils/datetime.rs:278-296`. Resolved. |
| **A-05** | Empty Array vs Null Array in SQL Projections | Diesel sometimes returns `NULL` for empty PostgreSQL arrays. | Go repository layer normalizes SQL `NULL` array columns to empty non-nil slices (`[]T{}`) if public DTO field is non-optional. | Grounded in `cache/redis_cache.rs:290-305`. Resolved. |
| **A-06** | Leading Zeros in Employee IDs | `normalize_employee_id` in `utils/mod.rs:37-62` strips leading zeros from trailing numeric segment (`"EMP001"` $\rightarrow$ `"1"`, `"000123"` $\rightarrow$ `"123"`). | Go port of `NormalizeEmployeeID` must execute the exact right-to-left reverse digit parse and `u64` conversion. | Grounded in unit tests in `utils/mod.rs:77-103`. Resolved. |

---

## 14. Verification Method & Golden Test Plan

1. **Unit Test Suite (`internal/pkg/types`):**
   - Implement Go unit tests mirroring `apiserver/src/utils/serde_helpers.rs:26-85` for `Nullable[T]`.
   - Implement Go unit tests mirroring `apiserver/src/utils/datetime.rs:194-340` for DST gaps, ambiguities, and formats.
   - Implement Go unit tests mirroring `apiserver/src/utils/mod.rs:77-103` for `NormalizeEmployeeID`.
   - Implement Go unit tests mirroring `apiserver/src/handlers/common.rs:1221-1280` for `SafePagination`.
2. **Golden File Parity Gate:**
   - Under Phase 3 (`P3-01`), execute test matrix with serialized JSON vectors from both Rust and Go runtimes.
   - Assert zero divergence on JSON AST comparison (using `nsf/jsondiff` or custom comparator following `comparison-normalization.md`).
