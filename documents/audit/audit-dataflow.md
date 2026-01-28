# Data Flow & Privacy Audit Report: LeeAttend

**Audit Type:** Data Flow Analysis & Privacy Impact Assessment
**Report Date:** 2026-01-28
**System:** LeeAttend Attendance Management System
**Audit Scope:** Complete data flow mapping, sensitive data inventory, privacy impact
**Auditor:** Privacy & Data Flow Review Team

---

## Executive Summary

LeeAttend is a distributed attendance management system with 4 primary data flows involving biometric devices, edge agents, central API servers, and web dashboards. System processes highly sensitive data: employee PII, biometric logs (fingerprint punch records), financial information (payroll), and authentication credentials. Data flows are partially secure with strong multi-tenant isolation but lack encryption at rest and in transit.

### Data Classification

| Data Type | Sensitivity | Volume | Retention |
|-----------|-------------|--------|-----------|
| Biometric (punch logs) | CRITICAL | ~1000 records/day/agent | 365 days (agent), indefinite (server) |
| Employee PII | CRITICAL | ~100-10000 records/workspace | Indefinite |
| Financial (payslips) | CRITICAL | ~100 records/month/workspace | Indefinite (legal requirement) |
| Authentication | CRITICAL | Continuous | 7-60 days (sessions/tokens) |
| Organizational | SENSITIVE | ~1000 records/workspace | Indefinite |

---

## Data Flow Path 1: Device → Agent → Server → Database

### Flow Diagram

```
ZKTeco Biometric Device
    ↓ TCP 5005 (proprietary protocol)
    ↓ Pull interval: configurable
Python Worker Sidecar (agent-zk-worker)
    ↓ JSON-RPC 2.0 localhost
    ↓ Methods: device.get_logs()
Rust Agent Orchestrator (agent-main)
    ↓ Batch buffer (mpsc, capacity 10,000)
    ↓ Batch: 100 records OR 200ms timeout
SQLite Local Cache (test-agent.db)
    ↓ Table: zkt_raw_attendance
    ↓ Fields: id, device_id, user_id, timestamp, is_synced
    ↓ Retention: 365 days (cleanup_old_synced_logs)
WebSocket Sync Loop
    ↓ Protocol: ws:// (NO TLS - CRITICAL FINDING)
    ↓ Format: {"type":"attendance", "data":{...}}
    ↓ Retry: Exponential backoff, max 10 attempts
API Server WebSocket Handler
    ↓ Verify agent_id + token
    ↓ Extract workspace_id
Attendance Service
    ↓ Batch buffer (5000 capacity, 100 records, 1000ms flush)
PostgreSQL raw_attendance_logs
    ↓ Immutable append-only table
    ↓ 55 total tables in database
```

### Sensitive Data in Path 1

**Data Elements:**
- Device user_id mapping to employee_id_number
- Punch timestamp (exact attendance time and location)
- Device ID (reveals physical location)
- Raw device payload (JSONB with biometric metadata)

**Security Issues:**
- **CRITICAL:** WebSocket transmitted without TLS encryption
- **HIGH:** SQLite database on edge agent unencrypted
- **MEDIUM:** Agent token stored plaintext in .env
- **MEDIUM:** Device authentication uses static token only

**Exposure Points:**
- Network sniffing of WebSocket traffic
- Physical access to gateway device
- SQLite file theft
- Stolen agent tokens

---

## Data Flow Path 2: Dashboard → API → Database

### Flow Diagram

```
Web Browser / Tauri Desktop
    ↓ HTTP (dev: localhost:${API_PORT})
    ↓ Headers: Authorization: Bearer JWT
Reverse Proxy / Vite Proxy
    ↓ Routes to API Server
API Server Handler Layer
    ↓ Middleware: Extract UserId from JWT
    ↓ Middleware: Extract workspace_id from path
    ↓ Fallback: X-Workspace-Id header (security risk)
Service Layer
    ↓ Business logic with workspace context
Repository Layer
    ↓ ALL queries: WHERE workspace_id = $1
    ↓ SQLx parameterized (no SQL injection)
PostgreSQL Connection Pool
    ↓ No TLS enforcement (DATABASE_URL lacks sslmode=require)
    ↓ Max 10 connections
PostgreSQL Database
    ↓ 55 tables, multi-tenant via workspace_id
Response
    ↓ JSON serialization
    ↓ HTTP 200 OK
Web Dashboard
    ↓ React Query cache (5 minute staleTime)
    ↓ Context API state
```

### Sensitive Data in Path 2

**Data Elements:**
- Employee personal data (names, emails, phones, addresses)
- Bank account information
- Salary amounts
- Leave reasons (medical, personal)
- Payslips and financial history
- Work hours and location

**Security Issues:**
- **HIGH:** Insufficient TLS enforcement (reverse proxy dependent)
- **CRITICAL:** CORS misconfiguration in debug mode
- **MEDIUM:** Verbose error messages leak schema
- **MEDIUM:** JWT tokens in Authorization header (60 min expiry)

**Exposure Points:**
- Man-in-the-middle if reverse proxy misconfigured
- XSS via stored data rendering
- CSRF on state-changing operations
- Session hijacking if token compromised

---

## Data Flow Path 3: Background Workers & Pairing Engine

### Flow Diagram

```
CSV/Excel Upload
    ↓ Attendance Import Job
    ↓ File stored (location NOT documented)
Background Worker Process
    ↓ Validate employee_id_number + timestamp
    ↓ Insert into raw_attendance_logs with source='import'
    ↓ Status: pending → processing → completed
Raw Attendance Logs
    ↓ Immutable table (no updates)
Pairing Engine (cron or manual)
    ↓ Algorithm: Anchor-First OR FSM
    ↓ Read unprocessed logs by workspace
    ↓ Pair check_in/check_out into work sessions
    ↓ Write generation_metadata JSONB (algorithm + source logs)
Time Entries Table
    ↓ Work sessions with duration_minutes
Payroll Engine
    ↓ 3-tier policy resolution
    ↓ (employee > department > workspace configs)
    ↓ Calculate regular + OT + bonuses + deductions
    ↓ Generate payslips (status: finalized = immutable)
Payslips & Segments
    ↓ Financial freeze (readonly after finalization)
```

### Sensitive Data in Path 3

**Data Elements:**
- Imported attendance records (timestamps, user IDs)
- Payroll calculations (hours, rates, multipliers)
- Salary amounts
- Policy configurations
- Algorithm metadata

**Security Issues:**
- **CRITICAL:** Imported files not encrypted
- **HIGH:** No file cleanup documented (retention risk)
- **MEDIUM:** No integrity checks on import
- **MEDIUM:** Payslip amounts plaintext (no field-level encryption)

---

## Data Flow Path 4: Employee Self-Service Portal (ESS)

### Flow Diagram

```
ESS Portal (webdashboard/src/pages/ess/*)
    ↓ Same authentication as main dashboard
    ↓ Filtered by user_id from JWT
API Server ESS Endpoints
    ↓ Query employees WHERE user_id = ? AND workspace_id = ?
    ↓ Returns only logged-in employee's data
    ↓ Leave requests, payslips, time entries
PostgreSQL Database
    ↓ Filtered query results
ESS Portal Display
    ↓ Personal payslips only
    ↓ Leave balance and requests
    ↓ Time entries and work hours
```

### Sensitive Data in Path 4

**Data Elements:**
- Personal payslips (salary, tax, deductions)
- Leave balances and history
- Work hours and attendance
- Personal profile information

**Access Control:**
- ✅ STRONG: Only employee's own data visible
- ⚠️ MEDIUM: No withdrawal of data access rights
- ⚠️ MEDIUM: No audit log of what employee accessed

---

## Sensitive Data Inventory

### Personally Identifiable Information (PII)

**Database Storage:**

**employees table:**
- first_name, last_name (VARCHAR 100)
- employee_id_number (VARCHAR 50, UNIQUE)
- hire_date, termination_date (DATE)
- user_id (UUID, nullable - links to authentication)

**employee_profiles table:**
- date_of_birth (DATE) - Special category data
- gender (VARCHAR)
- national_id (VARCHAR) - Government ID
- address (TEXT)
- phone (VARCHAR)
- emergency_contact_name, emergency_contact_phone (VARCHAR)

**users table:**
- email (TEXT, UNIQUE)
- password_hash (TEXT) - bcrypt with cost 12
- full_name (TEXT)
- avatar_url (TEXT)

**Sensitivity Level:** CRITICAL
**Encryption:** NONE (plaintext)
**Retention:** Indefinite (unless workspace deleted)

### Biometric Data

**Storage Locations:**
- raw_attendance_logs table (PostgreSQL) - primary
- zkt_raw_attendance table (SQLite agent) - secondary cache

**Fields:**
- punch_time (TIMESTAMPTZ) - timestamp at device location
- device_id (VARCHAR) - location identifier
- raw_data (JSONB) - may contain device-specific biometric templates

**Classification:** SPECIAL CATEGORY DATA (GDPR Article 9)
**Encryption:** NONE
**Retention:**
  - Agent: 365 days (cleanup_old_synced_logs)
  - Server: Indefinite (no policy documented)

### Financial Data

**payslips table:**
- gross_salary (DECIMAL)
- net_salary (DECIMAL)
- payroll_period_start/end (DATE)
- status (pending, finalized, paid)

**payslip_segments table:**
- segment_type (basic, ot, bonus, deduction)
- amount (DECIMAL)
- rate (DECIMAL)

**employee_payroll_configs table:**
- base_salary (DECIMAL)
- hourly_rate (DECIMAL)
- incentive_structure (JSONB)

**Financial Classification:** CRITICAL
**Encryption:** NONE (plaintext)
**Access Control:** workspace_id filter, role-based (admin/manager only)

### Authentication & Authorization Data

**users table:**
- password_hash (TEXT) - bcrypt salted
- **Issue V004:** JWT_SECRET defaults to weak value

**sessions table:**
- token (UUID) - refresh token
- expires_at (TIMESTAMPTZ) - 7 days
- revoked (BOOLEAN)

**authorized_agents table:**
- token (TEXT) - static agent authentication
- **Issue V034:** Stored plaintext in .env

**JWT Tokens:**
- Secret: JWT_SECRET environment variable
- Algorithm: HS256
- Expiry: 60 minutes (access), 7 days (refresh)
- Claims: user_id, workspace_id

---

## Data Storage Security Analysis

### PostgreSQL Database

**Connection Security:**
- URL: DATABASE_URL environment variable
- Pool: Max 10 connections
- Timeout: 30 seconds
- **Issue V005:** No TLS enforcement (no sslmode=require)

**Encryption Status:**
- **At Rest:** ❌ NOT IMPLEMENTED
- **Issue V030:** PostgreSQL data files plaintext
- **Issue V031:** Salary amounts plaintext (no field-level encryption)

**Access Control:**
- Role-based (postgres user only)
- No field-level encryption
- Multi-tenant isolation via workspace_id filtering

**Tables:** 55 total
- Immutable audit logs (raw_attendance_logs)
- Time tracking (time_entries)
- Financial (payslips, payslip_segments)
- Personnel (employees, employee_profiles, users)
- Configuration (workspaces, departments, devices)

### SQLite Edge Agent Database

**Location:** test-agent.db
**Mode:** WAL (Write-Ahead Logging)
**Pool:** Max 5 connections
**Cache:** 64MB in-memory

**Encryption Status:**
- **❌ NOT IMPLEMENTED**
- **Issue V035:** No SQLCipher encryption
- **Risk:** Physical access to gateway exposes local cache

**Tables:**
- zkt_raw_attendance (local punch cache)
- devices (device configuration)
- parsing_errors (failed log entries)
- agent_config (runtime settings)

**Retention:** 365-day cleanup for synced logs

### File System Storage

**Imported Files:**
- Location: NOT DOCUMENTED
- Cleanup: NOT DOCUMENTED
- Encryption: UNKNOWN
- **Risk:** Untracked storage location

**API Port File:**
- Location: /home/elt1541/leeattend/apiserver/.api_port
- Content: Dynamic port number (plaintext)
- Lifecycle: Deleted on shutdown

**Logs:**
- Format: stdout/stderr via tracing
- Destination: systemd journal or Docker logs
- **Issue V006:** Sensitive data in WebSocket errors
- **Issue V041:** Password hashes in debug logs

### Memory Caches

**Agent Batch Buffer (mpsc channel):**
- Capacity: 10,000 messages
- Contents: Unpersisted attendance logs
- Risk: Process crash loses unwritten data

**API Server Batch Buffer:**
- Capacity: 5,000 messages
- Flush: 100 records OR 1000ms
- Contents: Parsed logs before persistence

**React Query Cache:**
- Duration: 5 minutes (staleTime)
- Contents: Employee lists, payslips, balances
- Location: Browser memory

---

## Data Transmission Security

### WebSocket (Agent ↔ Server)

**Protocol:** ws:// (NOT wss://)
**Encryption:** NONE - **CRITICAL ISSUE V037**
**Authentication:** Handshake with agent_id + token

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

**Exposure Risk:** Network sniffing reveals employee attendance logs, device locations, timestamps

### HTTP/HTTPS (Dashboard ↔ API)

**Protocol:** HTTP (development), HTTPS (production - reverse proxy dependent)
**Encryption:** TLS 1.2+ (if reverse proxy configured)
**Authentication:** JWT in Authorization header

**Headers:**
- Authorization: Bearer ${jwt_token}
- X-Workspace-Id: ${workspace_id} (fallback)
- **Issue V003:** Header fallback allows ambient authority

**CORS Configuration:**
- **Issue V012:** Allow-any-origin in debug mode
- Production: Whitelist from ALLOWED_ORIGINS (not verified)

### Database Connections

**PostgreSQL:**
- **Issue V005:** No enforced TLS
- Default: Unencrypted connection string
- Pool: SQLx max 10 connections
- **Risk:** Man-in-the-middle on database traffic

**SQLite:**
- Local file access only (no network transmission)

---

## Multi-Tenant Data Isolation Analysis

### Isolation Enforcement Layers

**Layer 1: Path Parameter Extraction**
- URL pattern: `/api/workspaces/:ws_id/...`
- Extracted by Axum router
- Type: Uuid (compile-time validation)
- **Status:** ✅ STRONG

**Layer 2: Header Fallback**
- Fallback: X-Workspace-Id header
- **Issue V003:** Security risk if route misconfigured
- **Recommendation:** Remove header fallback in production

**Layer 3: Query Filtering**
- ALL repository queries: `WHERE workspace_id = $1`
- Verified in 21 repository files
- SQLx parameterized queries prevent SQL injection
- **Status:** ✅ EXCELLENT

**Layer 4: Schema Foreign Keys**
- ON DELETE CASCADE for workspace deletion
- Ensures no orphaned data
- **Status:** ✅ STRONG

### Isolation Verification

**Tables with workspace_id filter (55 total):**
- ✅ employees, departments, shifts
- ✅ raw_attendance_logs, time_entries
- ✅ payslips, leave_requests
- ✅ devices, authorized_agents
- ✅ All core tables verified

**Testing:**
- schema_isolation_tests.rs validates all tables
- **Status:** ✅ Isolation tests exist

### Isolation Weaknesses

**Potential Leakage Points:**
- **V013:** Debug endpoints if enabled in production
- **V003:** Header fallback without path validation
- **V015:** Verbose error messages expose schema
- **Risk Level:** LOW-MEDIUM (requires multiple misconfigurations)

---

## Data Lifecycle Management

### Collection Points

**Primary Sources:**
- Biometric devices (fingerprint scanners) - automatic
- Manual entry by managers - time_entries with source='manual'
- CSV/Excel imports - attendance_import_jobs
- Employee self-registration - users, employees tables

**Consent Capture:** ❌ NOT IMPLEMENTED

**Purpose Documentation:** ❌ NOT DOCUMENTED

### Processing Stages

1. **Collection:** Device → Agent SQLite (1-5 minute latency)
2. **Transmission:** Agent WebSocket → Server (unencrypted)
3. **Storage:** PostgreSQL raw_attendance_logs (immutable)
4. **Transformation:** Pairing engine → time_entries (algorithmic)
5. **Aggregation:** Payroll engine → payslips (financial)
6. **Access:** Dashboard queries → JSON response

### Storage Duration

**Agent SQLite:**
- Synced logs: 365 days (cleanup_old_synced_logs runs every 24h)
- Unsynced logs: Retained until sync successful
- Parsing errors: Indefinite (no cleanup)

**PostgreSQL:**
- raw_attendance_logs: Indefinite (immutable audit trail)
- time_entries: Indefinite
- payslips: Indefinite (legal payroll record retention)
- employees: Until workspace deleted (CASCADE)
- users: Until account deleted

**Compliance Gap V033:** No documented data retention policy
**Impact:** Violates GDPR Article 5(1)(e) (storage limitation)

### Data Deletion

**Automated Deletion:**
- Agent: cleanup_old_synced_logs (365 days)
- Server: NONE documented

**Manual Deletion:**
- Workspace deletion: ON DELETE CASCADE removes all tenant data
- Employee termination: termination_date set, is_active = FALSE (data retained)

**Compliance Gaps:**
- ❌ No right to deletion workflow (GDPR Article 17)
- ❌ No anonymization process
- ❌ No deletion audit log

---

## Data Minimization Assessment

### Excessive Collection

**Issue:** Full biometric device payloads stored in raw_data JSONB
- Justification: Device captures additional metadata beyond timestamps
- Risk: May contain unnecessary fingerprint templates or photos
- Recommendation: Validate raw_data schema, reject unnecessary fields

**Issue:** Indefinite retention of raw_attendance_logs
- Justification: Immutable audit trail for compliance
- Risk: Violates GDPR Article 5(1)(e) (storage limitation)
- Recommendation: Implement retention policy (e.g., 7 years for payroll, 3 years for general)

**Issue:** Full employee profile retention after termination
- Current: termination_date marked, data retained indefinitely
- Recommendation: Delete or pseudonymize 2 years after termination

### Necessary Data Properly Collected

✅ Employee ID, timestamp, location for attendance tracking
✅ Payroll data for compensation calculation
✅ Leave requests for balance management
✅ Work hours for workforce planning

---

## Privacy Impact Assessment

### Data Minimization Compliance

**Status:** ⚠️ PARTIAL
- Excessive retention (indefinite without policy)
- Full device payloads may contain unnecessary data
- **Remediation Effort:** 12 hours

### Purpose Limitation Compliance

**Status:** ❌ NON-COMPLIANT
- **Gap:** No privacy policy documenting purposes
- **Declared Purposes (Inferred):**
  - Attendance tracking ✓
  - Payroll calculation ✓
  - Leave management ✓
  - Workforce planning ✓
- **Risk:** Cannot restrict secondary use
- **Remediation Effort:** 16 hours

### Consent Management

**Status:** ❌ NON-COMPLIANT
- No consent capture during registration
- No consent withdrawal mechanism
- No granular consent (e.g., biometric vs. manual)
- **Remediation Effort:** 32 hours

### Right to Deletion (GDPR Article 17)

**Status:** ❌ NON-COMPLIANT
- No "Delete My Data" API endpoint
- No cascade deletion beyond workspace removal
- No anonymization process
- **Remediation Effort:** 32 hours

### Data Portability (GDPR Article 20)

**Status:** ❌ NON-COMPLIANT
- No employee data export API
- No structured data format (JSON, CSV)
- **Remediation Effort:** 24 hours

### Right to Rectification (GDPR Article 16)

**Status:** ⚠️ PARTIAL
- Employees can update profile via ESS ✓
- Managers can adjust time_entries ✓
- **Gap:** No audit log for corrections
- **Gap:** No notification to data subject
- **Remediation Effort:** 20 hours

### Breach Notification (GDPR Article 33)

**Status:** ❌ NON-COMPLIANT
- No incident response plan
- No breach detection mechanisms
- No 72-hour notification workflow
- **Remediation Effort:** 32 hours

---

## External Integrations Analysis

**Finding:** NO EXTERNAL INTEGRATIONS

**Verification:**
- No HTTP client libraries found (reqwest, etc.)
- No webhook endpoints
- No payment gateway integration
- No third-party API calls
- No cloud storage integration
- No analytics integration

**Implications:**
- No data shared with third parties
- No GDPR Data Processing Agreements required
- Simplified compliance model
- Self-contained data within LeeAttend

---

## Security Controls for Data Flows

### Data in Transit Controls

| Control | Status | Effectiveness |
|---------|--------|---------------|
| TLS/HTTPS | ❌ Not enforced | N/A |
| WebSocket TLS (wss) | ❌ Not enforced | N/A |
| Database TLS | ❌ Not enforced | N/A |
| JWT Authentication | ✅ Implemented | High |
| API Rate Limiting | ❌ Missing | N/A |
| CORS Policy | ⚠️ Weak | Low |

### Data at Rest Controls

| Control | Status | Effectiveness |
|---------|--------|---------------|
| Database Encryption | ❌ Missing | N/A |
| SQLite Encryption | ❌ Missing | N/A |
| Field-Level Encryption | ❌ Missing | N/A |
| Backup Encryption | ❌ Missing | N/A |
| Access Controls | ⚠️ Partial | Medium |
| Multi-Tenant Isolation | ✅ Strong | High |

### Data Access Controls

| Control | Status | Effectiveness |
|---------|--------|---------------|
| Authentication (JWT) | ✅ Implemented | High |
| Authorization (RBAC) | ✅ Implemented | High |
| Workspace Isolation | ✅ Strong | High |
| Audit Logging | ⚠️ Partial | Medium |
| Data Minimization | ⚠️ Partial | Medium |

---

## Key Findings Summary

### Critical Issues

1. **V037 - WebSocket Transmitted Unencrypted**
   - Biometric data exposed in transit
   - GDPR/PDPA breach risk
   - Effort: 8 hours to remediate

2. **V030 - No Encryption at Rest**
   - All PII stored plaintext in PostgreSQL
   - Physical security breach exposes all data
   - Effort: 64 hours

3. **V031 - No Field-Level Encryption**
   - Salary amounts plaintext
   - DBAs can view sensitive financial data
   - Effort: 24 hours

### High Priority Issues

4. **V033 - No Data Retention Policy**
   - Indefinite retention violates GDPR Article 5(1)(e)
   - Effort: 24 hours (policy + implementation)

5. **V035 - SQLite Unencrypted**
   - Edge agent cache exposed on gateway theft
   - Effort: 8 hours (SQLCipher implementation)

6. **No Data Subject Rights Workflows**
   - GDPR right to deletion missing
   - GDPR right to data portability missing
   - GDPR right to be informed missing
   - Effort: 80+ hours

---

## Recommendations

### Immediate (Critical)

1. **Enforce WebSocket TLS (wss://)**
   - Impact: Eliminate biometric data exposure in transit
   - Effort: 8 hours

2. **Enable Database Encryption at Rest**
   - Impact: Protect all PII from physical compromise
   - Effort: 64 hours

3. **Implement Data Retention Policy**
   - Impact: Achieve GDPR compliance on storage limitation
   - Effort: 24 hours

### Within 30 Days (High)

4. **Enforce Database TLS**
   - Effort: 4 hours

5. **Implement Consent Management**
   - Effort: 32 hours

6. **Build Data Subject Rights Workflows**
   - Export API (right to data portability): 16 hours
   - Deletion API (right to erasure): 32 hours
   - Rectification audit log: 12 hours

### Within 90 Days (Medium)

7. **Field-Level Encryption for Financial Data**
   - Effort: 24 hours

8. **Centralized Security Logging**
   - Effort: 16 hours

9. **Incident Response Plan**
   - Effort: 16 hours

---

## Conclusion

LeeAttend handles highly sensitive data (biometric, PII, financial) with strong multi-tenant isolation but critical gaps in encryption and privacy workflows. Most urgent issues:

1. Biometric data transmitted without TLS (CRITICAL)
2. No encryption at rest (CRITICAL)
3. No GDPR data subject rights workflows (HIGH)
4. No documented retention policy (HIGH)
5. No consent management (HIGH)

**Total Remediation Effort:** ~250 hours across data flow and privacy improvements

**Recommendation:** Do not process regulated data until encryption at rest and in transit are implemented. Conduct DPIA (Data Protection Impact Assessment) before production deployment.

---

**Data Flow Audit By:** Privacy & Data Flow Review Team
**Distribution:** Privacy Officer, Security Officer, CTO, Compliance Manager
