# Data Flow & Sensitive Data Analysis Report: LeeAttend

**Report ID:** scouter-260128-1309-data-flow-sensitive-data-analysis
**Date:** 2026-01-28 13:09
**Scout Agent:** ID a028030
**Scope:** Complete data flow mapping, sensitive data identification, privacy impact assessment
**System Version:** Based on git commit d9b5df8

---

## Executive Summary

LeeAttend is distributed attendance management system with multi-layered data flows involving biometric devices, edge agents, central servers, and web dashboards. System handles highly sensitive PII (employee profiles), biometric logs (fingerprint punches), financial data (payroll, salaries), and authentication credentials. Data flows through 4 primary channels: Device→Agent→Server→Database, Dashboard→API→Database, Background Workers→Database, and Agent→SQLite Cache.

**Key Findings:**
- **Critical Privacy Concerns:** Biometric data transmitted without TLS enforcement, sensitive data unencrypted at rest
- **Multi-Tenant Isolation:** Strong path-based workspace_id filtering enforced at query level
- **365-Day Retention:** Agent implements automatic cleanup of synced logs older than 365 days
- **No External Integrations:** System does not integrate with third-party APIs (no webhooks, no outbound HTTP)
- **Authentication:** JWT-based with bcrypt password hashing, but weak default secrets identified

---

## 1. Data Flow Paths

### 1.1 Primary Flow: Device → Edge Agent → API Server → PostgreSQL

```
Biometric Device (ZKTeco)
    ↓ TCP port 5005 (proprietary protocol)
    ↓ Pull every N minutes (configurable per device)
Python Worker Sidecar (agent-zk-worker/main.py)
    ↓ JSON-RPC 2.0 over TCP localhost
    ↓ Methods: device.get_logs() → [{timestamp, user_id}]
Rust Agent Main Orchestrator (agent-main/src/main.rs)
    ↓ Producer-consumer pattern (mpsc channel, capacity 10000)
    ↓ Batch buffer: 100 records OR 200ms timeout
SQLite Local Cache (test-agent.db)
    ↓ Table: zkt_raw_attendance
    ↓ Fields: id, device_id, user_id, timestamp, status, punch, is_synced
    ↓ 365-day retention (cleanup_old_synced_logs)
WebSocket Connection (central_client.rs)
    ↓ Protocol: ws:// (NO TLS enforcement - CRITICAL)
    ↓ Message: {"type":"attendance", "device_id":"...", "data":{...}}
    ↓ Retry logic: Exponential backoff, max 10 attempts, 60s max delay
API Server WebSocket Handler (handlers/agents.rs)
    ↓ Handshake: Verify agent_id + token
    ↓ Extract workspace_id from authorized_agents table
    ↓ Call attendance_service.ingest_agent_log()
Attendance Log Service (services/attendance_log_service.rs)
    ↓ Background channel (capacity 5000, batch 100, flush 1000ms)
    ↓ Parse timestamp using workspace timezone config
PostgreSQL Database (raw_attendance_logs table)
    ↓ Immutable audit trail (no updates, only inserts)
    ↓ Fields: id, workspace_id, employee_id, punch_time, source, device_id, raw_data (JSONB)
    ↓ Indexes: workspace_id, employee_id, punch_time
Pairing Engine (triggered by cron or manual)
    ↓ Algorithm: Anchor-First (primary) or FSM
    ↓ Read raw_attendance_logs WHERE workspace_id = ?
    ↓ Pair check_in/check_out into work sessions
PostgreSQL Database (time_entries table)
    ↓ Work sessions with duration
    ↓ Fields: id, employee_id, punch_in_time, punch_out_time, duration_minutes, generation_metadata (JSONB)
Payroll Calculation Engine
    ↓ Read time_entries for period
    ↓ Apply 3-tier policy resolution (employee > department > workspace)
    ↓ Calculate regular + OT + deductions
PostgreSQL Database (payslips + payslip_segments tables)
    ↓ Frozen snapshots, status = finalized (immutable)
```

**Sensitive Data in Flow:**
- Biometric identifiers: Device user_id mapped to employee_id_number
- Punch timestamps: Exact location + time (privacy sensitive)
- Raw device payload: JSONB may contain fingerprint templates (stored but not processed)
- Employee work hours: Used for payroll calculations

### 1.2 Dashboard Flow: Web Client → API Server → PostgreSQL

```
Web Browser / Tauri Desktop App
    ↓ HTTP (Vite dev: http://localhost:${API_PORT})
    ↓ Headers: Authorization: Bearer ${JWT}, X-Workspace-Id
Vite Proxy (dev) / Reverse Proxy (prod)
    ↓ Reads ../apiserver/.api_port for dynamic port
    ↓ Forwards to API Server
API Server Handler Layer (handlers/*.rs)
    ↓ Middleware: Extract UserId from JWT
    ↓ Middleware: Extract workspace_id from path /workspaces/:ws_id/...
    ↓ Fallback: X-Workspace-Id header (security concern if path missing)
Service Layer (services/*.rs)
    ↓ Business logic orchestration
    ↓ Pass workspace_id to all repository calls
Repository Layer (repositories/*.rs)
    ↓ ALL queries filter: WHERE workspace_id = $1
    ↓ SQLx parameterized queries (no SQL injection risk)
PostgreSQL Connection Pool (max 10 connections)
    ↓ Acquire timeout: 30 seconds
    ↓ No TLS enforcement on connection (DATABASE_URL default)
PostgreSQL Database (55 tables)
    ↓ Return filtered results
API Server Handler
    ↓ Serialize to JSON
    ↓ Return HTTP 200 + JSON body
Web Dashboard
    ↓ React Query caching (staleTime: 5 minutes)
    ↓ Local state management (Context API)
```

**Sensitive Data in Flow:**
- Authentication: JWT tokens in Authorization header (short-lived: 60 min)
- Refresh tokens: Stored in database sessions table (7-day expiry)
- Employee PII: Full names, emails, phone numbers, addresses, bank details
- Payroll data: Salary amounts, bonuses, deductions
- Leave requests: Medical leave reasons (potential health data)

### 1.3 Employee Self-Service Portal Flow

```
ESS Portal (webdashboard/src/pages/ess/*)
    ↓ Same authentication as main dashboard
    ↓ Filtered by user_id from JWT (not workspace admin)
API Server ESS Endpoints (handlers/ess.rs)
    ↓ Query employees WHERE user_id = ? AND workspace_id = ?
    ↓ Returns only logged-in employee's data
    ↓ Leave requests, payslips, time entries
PostgreSQL Database
    ↓ employees, time_entries, leave_requests, payslips
ESS Portal
    ↓ Display personal data only
```

**Sensitive Data:**
- Personal payslips: Salary amounts, tax deductions
- Leave balances: Accrued vacation days
- Time tracking: Work hours and location

### 1.4 Background Workers Flow

```
Attendance Import Jobs (services/attendance_import_service.rs)
    ↓ Triggered on CSV/Excel upload
    ↓ Store file in file system (location not documented)
    ↓ Background worker processes rows
    ↓ Validate employee_id_number + timestamp
    ↓ Insert into raw_attendance_logs with source='import'
    ↓ Update job status: pending → processing → completed/failed

Pairing Engine (cron-triggered or manual)
    ↓ Read unprocessed raw_attendance_logs
    ↓ Apply Anchor-First algorithm
    ↓ Write to time_entries with generation_metadata

Cleanup Tasks
    ↓ Agent: cleanup_old_synced_logs (365 days, runs every 24 hours)
    ↓ Server: No automated cleanup documented
```

---

## 2. Sensitive Data Inventory

### 2.1 Personally Identifiable Information (PII)

**Database Tables:**

**employees table:**
- first_name, last_name (VARCHAR 100)
- employee_id_number (VARCHAR 50, UNIQUE) - payroll identifier
- hire_date, termination_date (DATE)
- user_id (UUID, nullable) - links to authentication

**employee_profiles table (assumed from migration patterns):**
- date_of_birth (DATE)
- gender (VARCHAR)
- national_id (VARCHAR) - government ID
- address (TEXT)
- phone (VARCHAR)
- emergency_contact_name (VARCHAR)
- emergency_contact_phone (VARCHAR)

**users table:**
- email (TEXT, UNIQUE)
- password_hash (TEXT) - bcrypt with cost 12
- full_name (TEXT)
- avatar_url (TEXT)

**Retention:** Indefinite unless workspace deleted (CASCADE)
**Access Control:** Multi-tenant filtered by workspace_id

### 2.2 Biometric Data

**raw_attendance_logs table:**
- punch_time (TIMESTAMPTZ) - precise location timestamp
- device_id (VARCHAR) - links to physical location
- raw_data (JSONB) - may contain device-specific biometric metadata

**Edge Agent SQLite (zkt_raw_attendance):**
- Same fields as above
- 365-day retention (cleanup_old_synced_logs deletes synced logs)
- Stored unencrypted (SQLite default, no SQLCipher)

**Biometric Identifiers:**
- Device user_id mapped to employee_id_number
- Fingerprint templates: NOT processed by LeeAttend, stored in device memory only
- Face recognition data: NOT processed (if device supports)

**Compliance Risk:** GDPR Article 9 (special category data), PDPA biometric data provisions

### 2.3 Financial Data

**payslips table:**
- gross_salary (DECIMAL)
- net_salary (DECIMAL)
- payroll_period_start/end (DATE)
- status (pending, finalized, paid)

**payslip_segments table:**
- segment_type (basic, ot, bonus, deduction)
- amount (DECIMAL)
- rate (DECIMAL) - overtime multiplier

**employee_payroll_configs table:**
- base_salary (DECIMAL)
- hourly_rate (DECIMAL)
- incentive_structure (JSONB)

**department_payroll_configs table:**
- overtime_multiplier (DECIMAL)
- bonus_structure (JSONB)

**Encryption Status:** None (plaintext in database)
**Access Control:** Filtered by workspace_id, role-based (admin/manager only)

### 2.4 Authentication & Authorization Data

**users table:**
- password_hash (TEXT) - bcrypt cost 12, salted

**sessions table:**
- token (UUID) - refresh token
- user_id (UUID)
- expires_at (TIMESTAMPTZ) - 7 days from creation
- revoked (BOOLEAN)

**authorized_agents table:**
- token (TEXT) - static agent authentication token
- workspace_id (UUID)

**JWT Tokens:**
- Secret: JWT_SECRET environment variable (default: "dev_secret_key_leeattend_2026")
- Algorithm: HS256
- Expiry: 60 minutes
- Claims: user_id, workspace_id

**Security Concerns:**
- Weak default JWT secret (V004)
- Agent tokens stored plaintext in .env (V034)
- No token rotation policy documented

### 2.5 Organizational Data

**workspaces table:**
- name (TEXT) - organization name
- timezone (TEXT)
- date_format (VARCHAR)
- time_format (VARCHAR)

**departments table:**
- name (TEXT)
- code (TEXT)
- manager_id (UUID)

**devices table:**
- name (TEXT) - location identifier (e.g., "Main Gate")
- ip_address (TEXT) - network topology
- location (TEXT)

---

## 3. Data Storage Locations

### 3.1 PostgreSQL Database (Central Authority)

**Connection:**
- URL: DATABASE_URL environment variable
- Pool size: Max 10 connections
- Timeout: 30 seconds
- TLS: NOT enforced (no sslmode=require) - V005

**55 Tables Total (from migrations):**

**Core Tables:**
- workspaces, users, workspace_members
- employees, departments
- devices, authorized_agents

**Attendance Tables:**
- raw_attendance_logs (immutable)
- time_entries (paired sessions)
- attendance_adjustments (manual corrections)

**Shift Tables:**
- shifts, shift_rosters
- working_patterns (recurring schedules)
- assignment_rules (auto-scheduling)

**Leave Tables:**
- leave_types, leave_requests, leave_balances

**Payroll Tables:**
- payslips, payslip_segments
- payroll_settings, department_payroll_configs, employee_payroll_configs
- overtime_policies

**Encryption at Rest:** NOT IMPLEMENTED (PostgreSQL default, no TDE) - V030
**Backup Strategy:** NOT DOCUMENTED
**Access Control:** Role-based (postgres user), no field-level encryption

### 3.2 SQLite Database (Edge Agents)

**Location:** `test-agent.db` (filename hardcoded)
**Mode:** WAL (Write-Ahead Logging)
**Pool:** Max 5 connections
**Timeout:** 30 seconds (busy_timeout)
**Cache:** 64MB in-memory (cache_size=-64000)

**Tables:**
- zkt_raw_attendance: Local punch cache
- devices: Device configuration
- parsing_errors: Failed log entries
- agent_config: Runtime settings (key-value)

**Encryption:** NOT IMPLEMENTED (SQLite default, no SQLCipher) - V035
**Retention:** 365 days for synced logs (cleanup_old_synced_logs)
**Physical Security Risk:** Gateway device theft exposes logs

### 3.3 File System Storage

**Attendance Import Files:**
- Uploaded CSV/Excel files
- Location: NOT DOCUMENTED in codebase
- Cleanup: NOT DOCUMENTED

**API Port File:**
- Location: /home/elt1541/leeattend/apiserver/.api_port
- Content: Dynamic port number (plaintext)
- Cleanup: Deleted on shutdown (PortFileGuard)

**Logs:**
- Format: stdout/stderr (tracing crate)
- Destination: System logs (systemd journal, Docker logs)
- Retention: NOT DOCUMENTED
- Risk: Sensitive data in logs (V006 - agent tokens, V041 - password hashes)

### 3.4 Memory Caches

**Agent Batch Buffer (mpsc channel):**
- Capacity: 10,000 messages
- Contents: Unpersisted attendance logs (device_id, user_id, timestamp)
- Risk: Process crash loses unwritten logs (mitigated by persistence in SQLite first)

**API Server Batch Buffer (AttendanceLogService):**
- Capacity: 5,000 messages
- Contents: Parsed attendance logs before DB write
- Flush: 100 records OR 1000ms timeout

**React Query Cache:**
- Duration: 5 minutes (staleTime)
- Contents: Employee lists, payslips, leave balances
- Location: Browser memory (cleared on page refresh)

---

## 4. Data Transmission Security

### 4.1 WebSocket (Agent ↔ Server)

**Protocol:** ws:// (NOT wss://)
**Encryption:** NONE - V037 (Critical)
**Authentication:** Handshake with agent_id + token
**Port:** Dynamic (API server port)

**Message Format:**
```json
{
  "type": "attendance",
  "msg_id": "12345",
  "agent_id": "agent-001",
  "device_id": "device-uuid",
  "data": {
    "user_id": "EMP001",
    "timestamp": "2026-01-28 09:00:00",
    "status": 1,
    "punch": 1
  }
}
```

**Ack Response:**
```json
{
  "type": "ack",
  "msg_id": "12345",
  "status": "success",
  "timestamp": "2026-01-28T09:00:01Z"
}
```

**Risk:** Network sniffing exposes employee attendance logs, device IDs, timestamps
**Impact:** GDPR breach, biometric data exposed in transit

### 4.2 HTTP/HTTPS (Dashboard ↔ API)

**Protocol:** HTTP (dev), HTTPS (prod - depends on reverse proxy)
**Encryption:** TLS 1.2+ (if reverse proxy configured)
**Authentication:** JWT in Authorization header

**Headers:**
- Authorization: Bearer ${jwt_token}
- X-Workspace-Id: ${workspace_id} (fallback if path missing)

**CORS Configuration:**
- Dev mode: allow_origin(Any) - V012 (Critical if debug build deployed)
- Prod mode: Whitelist from ALLOWED_ORIGINS env var

**Risk:** If reverse proxy misconfigured, HTTP traffic exposes JWT tokens

### 4.3 Database Connections

**PostgreSQL:**
- Default: Unencrypted (no sslmode=require) - V005
- Pool: SQLx with max 10 connections
- Risk: Man-in-the-middle on database traffic

**SQLite:**
- Local file access only (no network transmission)

### 4.4 Inter-Process Communication

**Agent Main ↔ Worker Sidecars:**
- Protocol: JSON-RPC 2.0 over TCP localhost
- Port: Dynamic (OS-assigned)
- Encryption: None (localhost only, no network exposure)

---

## 5. Data Processing Stages

### 5.1 Data Collection (Device Punch)

**Input:** Biometric device fingerprint scan
**Processing:**
1. Device authenticates user via fingerprint match
2. Device logs timestamp + user_id + punch type
3. Worker sidecar polls device (pull model)
4. Worker returns logs via JSON-RPC

**Consent:** No consent tracking documented
**Minimization:** Full biometric logs retained (not just summary)

### 5.2 Data Ingestion (Agent → Server)

**Processing:**
1. Agent buffers logs (batch 100 or 200ms)
2. Agent inserts into SQLite (local persistence)
3. Sync loop reads unsynced logs (FIFO order, id ASC)
4. WebSocket transmission to server
5. Server acks with msg_id
6. Agent marks as synced (is_synced = 1)

**Data Transformation:**
- Timestamp parsing with workspace timezone
- Employee ID mapping (device user_id → employee_id_number)

**Error Handling:**
- Parse errors stored in parsing_errors table
- Retry logic: Exponential backoff, max 10 attempts
- Stale in-flight logs reset after timeout

### 5.3 Data Pairing (Raw Logs → Time Entries)

**Algorithm: Anchor-First**
1. Read raw_attendance_logs for date range
2. Sort by employee_id, punch_time
3. Pair first check_in with first check_out
4. Create time_entry with duration_minutes
5. Store source log IDs in generation_metadata JSONB

**Algorithm: FSM (Finite State Machine)**
- Explicit state transitions
- Handles complex shift patterns

**Audit Trail:**
- generation_metadata: {"algorithm":"anchor-first", "source_logs":[uuid1, uuid2]}
- Allows re-pairing with new logic (immutable raw logs preserved)

### 5.4 Payroll Calculation

**3-Tier Policy Resolution:**
1. Query employee_payroll_configs (highest priority)
2. Query department_payroll_configs (medium priority)
3. Query payroll_settings (workspace default)
4. Apply temporal filter: effective_from <= target_date AND (effective_to IS NULL OR effective_to >= target_date)

**Calculation Steps:**
1. Fetch time_entries for payroll period
2. Calculate regular hours (up to overtime_threshold)
3. Calculate overtime hours (above threshold)
4. Apply multipliers (overtime_multiplier, weekend_multiplier)
5. Calculate bonuses from incentive_structure JSONB
6. Calculate deductions (tax, benefits)
7. Generate payslip + payslip_segments
8. Mark payslip as finalized (immutable)

**Financial Data Integrity:**
- No encryption on amounts - V031
- No audit log for payslip modifications (if editable)

### 5.5 Leave Accrual & Consumption

**Workflow:**
1. Employee submits leave_request
2. Status: pending → approved (by manager) or rejected
3. On approval: leave_balances.consumed += duration_days
4. Fiscal year rollover: carry_forward applied if allowed
5. Annual accrual: accrued += annual_entitlement / 12 (monthly)

**Compliance:** No right-to-deletion workflow (GDPR Article 17)

---

## 6. Data Lifecycle

### 6.1 Collection Points

**Primary Sources:**
- Biometric devices (fingerprint scanners)
- Manual entry by managers (time_entries with source='manual')
- CSV/Excel imports (attendance_import_jobs)
- Employee self-registration (users, employees)

**Consent Capture:** NOT DOCUMENTED
**Purpose Limitation:** Attendance tracking, payroll calculation

### 6.2 Processing Stages

1. **Collection:** Device → Agent SQLite
2. **Transmission:** Agent WebSocket → Server
3. **Storage:** PostgreSQL raw_attendance_logs
4. **Transformation:** Pairing engine → time_entries
5. **Aggregation:** Payroll engine → payslips
6. **Access:** Dashboard queries → JSON response

### 6.3 Storage Duration

**Agent SQLite:**
- Synced logs: 365 days (cleanup_old_synced_logs)
- Unsynced logs: Retained until sync successful
- Parsing errors: Indefinite (no cleanup)

**PostgreSQL:**
- raw_attendance_logs: Indefinite (immutable audit trail)
- time_entries: Indefinite
- payslips: Indefinite (legal requirement for payroll records)
- employees: Until workspace deleted (CASCADE)
- users: Until account deleted

**Compliance Gap:** No data retention policy documented (V033)
**GDPR Issue:** Right to erasure not implemented

### 6.4 Archival & Deletion

**Automated Deletion:**
- Agent: cleanup_old_synced_logs (365 days)
- Server: NONE documented

**Manual Deletion:**
- Workspace deletion: ON DELETE CASCADE removes all tenant data
- Employee termination: termination_date set, is_active = FALSE (data retained)

**Backup Encryption:** NOT DOCUMENTED (V025)

### 6.5 Data Export

**Employee Data Portability (GDPR Article 20):**
- NOT IMPLEMENTED
- No export API endpoint found

**Payroll Reports:**
- Dashboard UI export (format not documented)

---

## 7. Integration Points

### 7.1 Internal Integrations

**Agent ↔ Server:**
- Protocol: WebSocket
- Authentication: Agent token (static, stored in .env)
- Data: Attendance logs, config updates

**Dashboard ↔ API:**
- Protocol: HTTP/HTTPS
- Authentication: JWT (60 min expiry)
- Data: Full CRUD operations on all entities

**Background Workers ↔ Database:**
- Direct PostgreSQL connection pool
- Attendance import processing
- Pairing engine execution

### 7.2 External Integrations

**Analysis Result:** NONE FOUND

**No HTTP Client Libraries:**
- Grep search for "reqwest", "http.*client", "webhook" returned no results
- No outbound HTTP calls in codebase

**No Third-Party APIs:**
- No payment gateways
- No email services (SMTP not configured)
- No cloud storage (S3, Azure Blob)
- No analytics (Google Analytics, Mixpanel)

**No Webhooks:**
- No webhook endpoints for external systems

**Biometric Devices:**
- Inbound only (agent pulls logs)
- Proprietary protocol via pyzk2 library

### 7.3 Data Import/Export

**Import:**
- CSV/Excel attendance logs (attendance_import_jobs)
- No API for bulk employee import

**Export:**
- Dashboard UI downloads (implementation not verified)
- No documented API export endpoints

---

## 8. Privacy Impact Assessment

### 8.1 Data Minimization

**Compliance Status:** PARTIAL

**Excessive Data Collection:**
- raw_data JSONB field stores full device payload (may contain unnecessary biometric metadata)
- Indefinite retention of raw_attendance_logs (not time-limited)

**Necessary Data:**
- Employee ID, timestamp, location for attendance tracking ✓
- Payroll calculations require work hours ✓

**Recommendation:** Limit raw_data to essential fields, implement data retention policy

### 8.2 Purpose Limitation

**Declared Purposes (inferred from system):**
- Attendance tracking
- Payroll calculation
- Leave management
- Workforce planning

**No Policy Documents Found:**
- No privacy policy in codebase
- No terms of service
- No consent forms

**Risk:** Cannot demonstrate GDPR Article 5(1)(b) compliance

### 8.3 Consent Management

**Status:** NOT IMPLEMENTED

**Missing Features:**
- No consent capture during employee registration
- No consent withdrawal mechanism
- No granular consent (e.g., biometric vs. manual attendance)

**GDPR Article 7:** Cannot demonstrate valid consent

### 8.4 Right to Deletion (GDPR Article 17)

**Status:** NOT IMPLEMENTED (V033)

**Missing Workflows:**
- No "Delete My Data" API endpoint
- No cascade deletion beyond workspace removal
- No anonymization process

**Current Deletion:**
- Workspace deletion: Cascades to all tenant data (employees, logs, payslips)
- Employee termination: Marks is_active = FALSE but retains data

**Recommendation:** Implement pseudonymization after termination_date + 2 years

### 8.5 Data Portability (GDPR Article 20)

**Status:** NOT IMPLEMENTED

**Missing Features:**
- No employee data export API
- No structured data format (JSON, CSV)

**Recommendation:** Build /api/employees/:id/export endpoint

### 8.6 Right to Rectification (GDPR Article 16)

**Status:** PARTIAL

**Implemented:**
- Employees can update profile via ESS portal
- Managers can adjust time_entries

**Missing:**
- No audit log for data corrections
- No notification to affected parties

### 8.7 Data Breach Notification

**Status:** NOT IMPLEMENTED

**Missing:**
- No incident response plan documented
- No breach detection mechanisms
- No 72-hour notification workflow (GDPR Article 33)

**Recommendation:** Implement security event logging (V026), alerting (V027)

---

## 9. Multi-Tenant Data Isolation

### 9.1 Isolation Enforcement Layers

**Layer 1: Path Parameter Extraction**
- URL pattern: `/api/workspaces/:ws_id/...`
- Extracted by Axum router at handler entry
- Type: Uuid (compile-time validation)

**Layer 2: Header Fallback (SECURITY RISK)**
- Fallback: X-Workspace-Id header if path missing - V003
- Risk: Ambient authority if route misconfigured

**Layer 3: Query Filtering**
- ALL repository queries: `WHERE workspace_id = $1`
- Verified in 21 repository files
- SQLx parameterized queries prevent SQL injection

**Layer 4: Schema Foreign Keys**
- ON DELETE CASCADE for workspace removal
- Ensures no orphaned data

### 9.2 Verified Isolation

**Tables with workspace_id filter (55 total):**
- employees, departments, shifts
- raw_attendance_logs, time_entries
- payslips, leave_requests
- devices, authorized_agents

**Testing:**
- schema_isolation_tests.rs validates all tables (mentioned in docs)

### 9.3 Isolation Weaknesses

**Potential Leakage Points:**
- Debug endpoints if enabled in production (V013)
- Header fallback without path validation (V003)
- Verbose error messages expose schema (V015)

---

## 10. Compliance Requirements

### 10.1 GDPR Gaps

| Requirement | Status | Gap |
|-------------|--------|-----|
| Lawful Basis (Art 6) | ❌ | No documented legal basis |
| Consent (Art 7) | ❌ | No consent capture mechanism |
| Data Minimization (Art 5.1c) | ⚠️ | Excessive retention (indefinite) |
| Purpose Limitation (Art 5.1b) | ❌ | No privacy policy |
| Storage Limitation (Art 5.1e) | ❌ | No retention policy (V033) |
| Integrity & Confidentiality (Art 5.1f) | ❌ | No encryption at rest (V030), no TLS (V005, V037) |
| Right to Erasure (Art 17) | ❌ | No deletion workflow |
| Data Portability (Art 20) | ❌ | No export API |
| Breach Notification (Art 33) | ❌ | No incident plan |

### 10.2 PDPA (Thailand) Considerations

**Sensitive Personal Data (Section 26):**
- Biometric data (fingerprint logs) requires explicit consent
- Health data (sick leave reasons) requires consent

**Data Retention:**
- Must be justified and proportionate
- 365-day agent retention acceptable
- Indefinite server retention questionable

### 10.3 SOC 2 Compliance

| Control | Status | Gap |
|---------|--------|-----|
| CC6.1 Logical Access | ⚠️ | No MFA (V019) |
| CC6.6 Encryption | ❌ | No encryption at rest (V030) |
| CC6.7 Transmission Security | ❌ | No TLS enforcement (V005, V037) |
| CC7.2 Monitoring | ❌ | No centralized logging (V026) |
| CC7.3 Alerting | ❌ | No security alerts (V027) |
| CC8.1 Change Management | ⚠️ | Minimal audit trail |

---

## 11. Data Flow Diagrams (Textual)

### 11.1 Complete End-to-End Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         BIOMETRIC DEVICE LAYER                       │
├─────────────────────────────────────────────────────────────────────┤
│  ZKTeco Fingerprint Scanner                                         │
│    ↓ Fingerprint match → Device User ID + Timestamp                │
│    ↓ TCP port 5005 (proprietary protocol, NOT HTTP)                │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│                         EDGE AGENT LAYER                            │
├─────────────────────────────────────────────────────────────────────┤
│  Python Worker Sidecar (agent-zk-worker)                           │
│    ↓ JSON-RPC: device.get_logs() → [{user_id, timestamp}]         │
│  Rust Agent Main (agent-main)                                      │
│    ↓ Batch buffer: 100 records OR 200ms timeout                   │
│  SQLite (test-agent.db)                                            │
│    ↓ zkt_raw_attendance table                                     │
│    ↓ 365-day retention (cleanup_old_synced_logs)                  │
│  Sync Loop                                                         │
│    ↓ WebSocket ws://server:port (NO TLS - CRITICAL)               │
│    ↓ {"type":"attendance", "data":{...}}                          │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│                       API SERVER LAYER                              │
├─────────────────────────────────────────────────────────────────────┤
│  WebSocket Handler (handlers/agents.rs)                            │
│    ↓ Handshake: Verify agent_id + token                           │
│    ↓ Extract workspace_id from authorized_agents                  │
│  Attendance Service (services/attendance_log_service.rs)           │
│    ↓ Background channel: 5000 capacity, batch 100, flush 1000ms   │
│    ↓ Parse timestamp with workspace timezone                      │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│                     DATABASE LAYER (PostgreSQL)                     │
├─────────────────────────────────────────────────────────────────────┤
│  raw_attendance_logs (immutable)                                    │
│    ↓ Fields: workspace_id, employee_id, punch_time, raw_data      │
│    ↓ NO ENCRYPTION AT REST (V030)                                 │
│  Pairing Engine (cron or manual trigger)                          │
│    ↓ Anchor-First algorithm                                       │
│  time_entries (paired work sessions)                               │
│    ↓ Fields: punch_in_time, punch_out_time, duration_minutes      │
│  Payroll Engine (scheduled or manual)                             │
│    ↓ 3-tier policy resolution                                     │
│  payslips + payslip_segments (finalized)                          │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│                       WEB DASHBOARD LAYER                           │
├─────────────────────────────────────────────────────────────────────┤
│  React App (webdashboard)                                          │
│    ↓ HTTP GET /api/workspaces/:id/payslips                        │
│    ↓ Authorization: Bearer JWT                                     │
│  API Server Handler                                                │
│    ↓ Extract workspace_id from path                               │
│    ↓ Query: WHERE workspace_id = $1                               │
│  Dashboard Display                                                 │
│    ↓ Payslip amounts, employee names, work hours                  │
└─────────────────────────────────────────────────────────────────────┘
```

### 11.2 Authentication Flow

```
User Login
    ↓ POST /api/auth/login {email, password}
API Server (handlers/auth.rs)
    ↓ Query users WHERE email = $1
    ↓ bcrypt::verify(password, password_hash)
    ↓ Generate JWT (jsonwebtoken crate)
    ↓   Claims: {user_id, exp: 60min}
    ↓   Secret: JWT_SECRET env var (default weak - V004)
    ↓ Create session (sessions table)
    ↓   Token: UUIDv4
    ↓   Expires: 7 days
Response
    ↓ {access_token: JWT, refresh_token: UUID}
Dashboard
    ↓ Store tokens in memory (React state)
    ↓ Include in Authorization header
API Request
    ↓ Middleware: Extract JWT
    ↓ Verify signature with JWT_SECRET
    ↓ Extract user_id from claims
    ↓ Pass to handler as UserId extractor
```

### 11.3 Multi-Tenant Isolation Flow

```
Dashboard Request
    ↓ GET /api/workspaces/abc-123/employees
API Server Router
    ↓ Extract workspace_id from path: abc-123
    ↓ MemberContext extractor
    ↓   Query workspace_members WHERE user_id = ? AND workspace_id = ?
    ↓   Verify user has access to workspace abc-123
Handler
    ↓ Pass workspace_id to service
Service
    ↓ Pass workspace_id to repository
Repository
    ↓ sqlx::query_as::<_, Employee>(
    ↓   "SELECT * FROM employees WHERE workspace_id = $1"
    ↓ ).bind(workspace_id)
PostgreSQL
    ↓ Filter results by workspace_id
    ↓ Return only tenant's employees
```

---

## 12. Unresolved Questions

1. **File Storage:** Where are imported CSV/Excel files stored? Are they encrypted? What is cleanup policy?
2. **Email Service:** How are leave approval notifications sent? SMTP configured? Email addresses exposed?
3. **Backup Encryption:** Are PostgreSQL backups encrypted? (V025) Where stored? Who has access?
4. **Log Retention:** What is production log retention policy? Are logs rotated? Archived to external storage?
5. **Biometric Templates:** Are fingerprint templates stored in raw_data JSONB? If yes, this is critical PII.
6. **Device Photos:** Do devices capture photos on punch? If yes, where stored? Photo retention policy?
7. **External Auth:** Any SSO/OAuth integration planned? Currently only local auth with bcrypt.
8. **Data Export:** Is there a documented process for employee data export requests (GDPR Article 20)?
9. **Anonymization:** After employee termination, is data pseudonymized or deleted? Retention policy?
10. **Incident Response:** Who is notified in case of data breach? Is there a 72-hour GDPR breach response plan?
11. **Penetration Testing:** Has external security assessment been performed? Last audit date?
12. **Compliance Documentation:** Where are privacy policies, terms of service, consent forms stored? Are they presented to users?

---

## 13. Recommendations (Prioritized)

### Critical (Fix Immediately)

1. **V037 - Enforce WebSocket TLS (wss://)**
   - Impact: Biometric data transmitted unencrypted
   - Fix: Update agent config to require wss://, reject ws:// connections
   - Effort: 8 hours

2. **V004 - Remove Default JWT Secret**
   - Impact: Attackers can forge tokens if default used
   - Fix: Panic on startup if JWT_SECRET unset or weak
   - Effort: 2 hours

3. **V030 - Implement Database Encryption at Rest**
   - Impact: Physical breach exposes all PII
   - Fix: Enable PostgreSQL TDE or LUKS filesystem encryption
   - Effort: 16 hours

### High Priority (Fix Within 30 Days)

4. **V005 - Enforce TLS on Database Connections**
   - Fix: Add `?sslmode=require` to DATABASE_URL
   - Effort: 4 hours

5. **V026 - Centralized Security Event Logging**
   - Fix: Implement syslog or ELK integration
   - Effort: 16 hours

6. **V033 - Implement Data Retention Policy**
   - Fix: Document retention periods, implement automated cleanup
   - Effort: 8 hours

7. **V019 - Implement MFA for Admin Roles**
   - Fix: TOTP-based 2FA
   - Effort: 40 hours

### Medium Priority (Fix Within 90 Days)

8. **GDPR Consent Management**
   - Fix: Build consent capture during registration, consent withdrawal API
   - Effort: 24 hours

9. **Right to Deletion Workflow**
   - Fix: Implement /api/employees/:id/delete-my-data endpoint
   - Effort: 16 hours

10. **Data Portability API**
    - Fix: Implement /api/employees/:id/export endpoint (JSON format)
    - Effort: 8 hours

11. **Field-Level Encryption**
    - Fix: Encrypt salary amounts, bank account numbers at application level
    - Effort: 24 hours

### Low Priority (Fix Within 180 Days)

12. **Privacy Policy Documentation**
    - Fix: Create privacy policy, terms of service, consent forms
    - Effort: 16 hours

13. **Incident Response Plan**
    - Fix: Document breach detection, notification, remediation workflows
    - Effort: 16 hours

14. **Backup Encryption**
    - Fix: Implement AES-256 encryption for PostgreSQL backups
    - Effort: 16 hours

---

## Conclusion

LeeAttend handles highly sensitive data (biometric, PII, financial) with strong multi-tenant isolation but **critical gaps in encryption, compliance, and privacy workflows**. Most urgent issues:

1. **Biometric data transmitted without TLS** (V037) - IMMEDIATE risk of GDPR breach
2. **No encryption at rest** (V030) - Physical security breach exposes all data
3. **No GDPR compliance workflows** - Right to deletion, consent management, data portability missing
4. **Weak default secrets** (V004, V034) - Authentication bypass risk

**Estimated Remediation Effort:**
- Critical: 26 hours (~1 week)
- High: 68 hours (~2 weeks)
- Medium: 72 hours (~2 weeks)
- Total: 166 hours (~4 weeks with one engineer)

**Next Steps:**
1. Implement TLS enforcement for WebSocket (V037)
2. Remove weak default JWT secret (V004)
3. Enable database encryption at rest (V030)
4. Conduct GDPR compliance gap analysis
5. Implement incident response plan
6. Schedule external penetration testing

---

**Report Prepared By:** Scout Agent (ID: a028030)
**Next Review Date:** 2026-02-28
**Distribution:** Security Engineer, Tech Lead, Privacy Officer, DevOps Engineer
