# LeeAttend Domain Model

## Domain Overview

LeeAttend's domain model organizes around attendance, employee management, scheduling, leave workflows, and payroll. The system uses a multi-tenant structure where workspaces are the root isolation boundary. All domain entities include workspace scope and temporal dimensions.

## Core Domain Entities

### Workspace (Tenant)

Represents a single organization or tenant instance.

**Database Table**: `workspaces`

**Attributes**:
- `id` (UUID) - Primary identifier
- `name` (text) - Organization name
- `created_at` (timestamp) - Creation timestamp
- `updated_at` (timestamp) - Last modification

**Relationships**:
- 1:N with `users` (via workspace_members)
- 1:N with `devices` (biometric hardware)
- 1:N with `employees` (staff records)
- 1:N with `departments` (organizational structure)
- 1:N with `shifts` (work schedules)

**Key Characteristics**:
- Data isolation boundary
- Enforced at query level: `WHERE workspace_id = ?`
- All tenant operations filtered by workspace context
- No shared data between workspaces

### User & Authentication

**Database Tables**:
- `users` - Global user accounts
- `workspace_members` - RBAC per workspace

**User Attributes**:
- `id` (UUID) - Primary identifier
- `email` (text) - Unique login credential
- `password_hash` (text) - bcrypt hash
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Workspace Member Attributes**:
- `id` (UUID) - Primary identifier
- `user_id` (UUID) - Foreign key to users
- `workspace_id` (UUID) - Foreign key to workspaces
- `role` (enum) - owner, admin, manager, employee
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Authentication Flow**:
1. POST `/api/auth/login` with email/password
2. Server validates password (bcrypt)
3. JWT issued with user_id and workspace_id
4. Client includes JWT in Authorization header
5. Server validates JWT and extracts workspace context

**Authorization Model**:
- Role-based (RBAC)
- Roles: Owner, Admin, Manager, Employee
- Enforced per handler (e.g., admin-only endpoints)
- Claims encoded in JWT payload

### Employee

Represents a staff member in the organization.

**Database Table**: `employees`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `employee_id` (text) - Unique employee number
- `first_name` (text)
- `last_name` (text)
- `email` (text) - Work email
- `phone` (text) - Contact number
- `department_id` (UUID) - Department assignment
- `designation` (text) - Job title
- `hire_date` (date) - Employment start date
- `status` (enum) - active, inactive, on_leave
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `departments` (belongs to)
- 1:N with `raw_attendance_logs` (punch records)
- 1:N with `time_entries` (work sessions)
- 1:N with `leave_requests` (leave applications)
- 1:N with `leave_balances` (leave accrual)
- 1:N with `payslips` (payroll records)
- 1:1 with `employee_payroll_configs` (policy override)

**Key Characteristics**:
- Central to all business logic
- Linked to biometric identification
- Temporal status tracking
- Multi-level policy inheritance

### Department

Represents organizational structure.

**Database Table**: `departments`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `name` (text) - Department name
- `code` (text) - Unique identifier within workspace
- `manager_id` (UUID) - Optional manager reference
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- 1:N with `employees` (staff members)
- 1:N with `department_payroll_configs` (policy override)

### Device (Biometric Hardware)

Represents physical attendance devices (ZKTeco units).

**Database Table**: `devices`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `name` (text) - Device label (e.g., "Main Gate")
- `brand` (enum) - zk, suprema, hikvision, etc.
- `ip_address` (text) - Network address
- `port` (integer) - Communication port (default 5005)
- `location` (text) - Physical location
- `timezone` (text) - Device timezone
- `is_active` (boolean) - Enable/disable device
- `last_sync` (timestamp) - Last successful sync
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- 1:N with `raw_attendance_logs` (punch records)
- 1:1 with `authorized_agents` (responsible agent)

**Key Characteristics**:
- Static configuration
- Multiple devices per workspace
- Brand-specific driver routing
- Health monitoring via last_sync

### Authorized Agent

Represents an edge agent authorized to collect from devices.

**Database Table**: `authorized_agents`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `agent_id` (text) - Unique agent identifier
- `name` (text) - Display name
- `status` (enum) - active, inactive, disconnected
- `last_heartbeat` (timestamp) - Last WebSocket ping
- `config` (jsonb) - Agent-specific configuration
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- 1:N with `devices` (assigned devices)

## Attendance Domain

### Raw Attendance Log

Immutable record of device punch events.

**Database Table**: `raw_attendance_logs`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `device_id` (UUID) - Source device
- `employee_id` (UUID) - Punched employee
- `log_type` (enum) - check_in, check_out, undefined
- `timestamp` (timestamp) - Punch time (with timezone)
- `raw_payload` (jsonb) - Device-specific data
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `devices` (source)
- M:1 with `employees` (subject)
- 1:N with `time_entries` (generated from)

**Key Characteristics**:
- Immutable (never updated, only inserted)
- Point-in-time record
- Multiple records per day per employee
- Foundation for pairing engine

### Time Entry (Paired Work Session)

Generated from consecutive raw logs, represents a work session.

**Database Table**: `time_entries`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `employee_id` (UUID) - Working employee
- `date` (date) - Work date
- `check_in_time` (timestamp) - Session start
- `check_out_time` (timestamp) - Session end (nullable)
- `duration_minutes` (integer) - Computed duration
- `status` (enum) - in_progress, completed, manual
- `device_id` (UUID) - Source device
- `generation_metadata` (jsonb) - Source log IDs and algorithm
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `employees` (working employee)
- M:1 with `devices` (primary device)
- M:1 with `raw_attendance_logs` (source - via generation_metadata)
- 1:1 with `payslip_segments` (payroll line item)

**Pairing Algorithms**:

**Anchor-First (Primary)**:
- Pair first check_in with first check_out
- Handle mid-day breaks as separate sessions
- Clear audit trail via generation_metadata

**FSM (Finite State Machine)**:
- Explicit state transitions
- Accounts for complex schedules
- Handles break logic

**Manual Entries**:
- Created by managers via UI
- Not derived from device logs
- Status: manual (distinguished from paired)

## Shift Domain

### Shift

Work schedule template.

**Database Table**: `shifts`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `name` (text) - Shift name (e.g., "Morning")
- `start_time` (time) - Shift start (HH:MM)
- `end_time` (time) - Shift end (HH:MM)
- `break_start` (time) - Optional break start
- `break_end` (time) - Optional break end
- `duration_minutes` (integer) - Expected hours
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- 1:N with `shift_rosters` (assignments)

**Key Characteristics**:
- Time-of-day based (not date-dependent)
- Supports breaks (deducted from duration)
- Template for roster assignments

### Shift Roster

Assignment of employees to shifts on specific dates.

**Database Table**: `shift_rosters`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `employee_id` (UUID) - Assigned employee
- `shift_id` (UUID) - Assigned shift
- `assigned_date` (date) - Date of assignment
- `effective_from` (date) - Start date (temporal)
- `effective_to` (date) - End date (temporal, nullable)
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `employees` (assigned employee)
- M:1 with `shifts` (assigned shift)

**Key Characteristics**:
- Temporal data (effective_from/to)
- Supports roster planning (future dates)
- Supports policy changes (historical data)

### Working Pattern

Schedule template (recurring pattern).

**Database Table**: `working_patterns`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `name` (text) - Pattern name (e.g., "5x8")
- `pattern_json` (jsonb) - Weekly schedule (Mon-Sun shifts)
- `total_hours` (decimal) - Weekly total
- `effective_from` (date) - Start date
- `effective_to` (date) - End date (nullable)
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Pattern Structure**:
```json
{
  "monday": "shift_id_1",
  "tuesday": "shift_id_1",
  "wednesday": "shift_id_1",
  "thursday": "shift_id_1",
  "friday": "shift_id_1",
  "saturday": null,
  "sunday": null
}
```

**Key Characteristics**:
- Repeating weekly pattern
- Temporal versioning (policy changes)
- Bulk assignment to employees

### Assignment Rule

Automatic scheduling logic.

**Database Table**: `assignment_rules`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `name` (text) - Rule name
- `department_id` (UUID) - Target department (nullable)
- `rule_type` (enum) - pattern_based, manual
- `pattern_id` (UUID) - Working pattern (if pattern_based)
- `effective_from` (date) - Start date
- `effective_to` (date) - End date (nullable)
- `is_active` (boolean) - Enable/disable
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `working_patterns` (schedule template)
- M:1 with `departments` (target scope)

## Leave Domain

### Leave Type

Policy definition for leave categories.

**Database Table**: `leave_types`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `name` (text) - Type name (e.g., "Annual Leave")
- `code` (text) - Short code (PTO, SICK)
- `description` (text) - Policy description
- `annual_entitlement` (decimal) - Yearly allocation
- `carry_forward_allowed` (boolean) - Rollover policy
- `requires_approval` (boolean) - Workflow enabled
- `is_paid` (boolean) - Salary impact
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- 1:N with `leave_requests` (applications)
- 1:N with `leave_balances` (accrual tracking)

### Leave Request

Employee leave application.

**Database Table**: `leave_requests`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `employee_id` (UUID) - Requesting employee
- `leave_type_id` (UUID) - Leave category
- `start_date` (date) - First day of leave
- `end_date` (date) - Last day of leave
- `duration_days` (decimal) - Number of days
- `reason` (text) - Leave reason (optional)
- `status` (enum) - pending, approved, rejected, cancelled
- `approver_id` (UUID) - Manager approving (nullable)
- `approval_date` (timestamp) - When approved (nullable)
- `rejection_reason` (text) - Reason for rejection (nullable)
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `employees` (requesting employee)
- M:1 with `leave_types` (leave category)
- M:1 with `users` (approver)

**Workflow States**:
```
Pending → Approved → (leave taken) → Complete
     ↓
   Rejected
     ↓
   Cancelled
```

### Leave Balance

Accrual and consumption tracking.

**Database Table**: `leave_balances`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `employee_id` (UUID) - Employee record
- `leave_type_id` (UUID) - Leave type
- `fiscal_year` (integer) - Policy year (e.g., 2026)
- `opening_balance` (decimal) - Start of period
- `accrued` (decimal) - YTD accrual
- `consumed` (decimal) - YTD usage
- `closing_balance` (decimal) - Current available
- `carry_forward` (decimal) - Rolled from prior year
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `employees` (employee record)
- M:1 with `leave_types` (leave category)

**Calculation**:
```
Closing Balance = Opening + Accrued + Carry-Forward - Consumed
```

**Key Characteristics**:
- Fiscal year scoping
- Separate tracking by leave type
- Point-in-time balances
- Historical audit trail

## Payroll Domain

### Payroll Settings

Workspace-level payroll policy configuration.

**Database Table**: `payroll_settings`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `overtime_multiplier` (decimal) - OT rate (e.g., 1.5)
- `regular_hours` (decimal) - Standard week (e.g., 40)
- `overtime_threshold` (decimal) - Hours triggering OT (e.g., 45)
- `weekend_multiplier` (decimal) - Weekend rate
- `public_holiday_multiplier` (decimal) - Holiday rate
- `effective_from` (date) - Policy start
- `effective_to` (date) - Policy end (nullable)
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- N:1 with `workspaces` (policy scope)

**Key Characteristics**:
- Workspace-level baseline
- Temporal versioning (policy changes)
- Overrideable at department/employee level

### Department Payroll Config

Department-level policy override.

**Database Table**: `department_payroll_configs`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `department_id` (UUID) - Department scope
- `overtime_multiplier` (decimal) - Override rate
- `overtime_threshold` (decimal) - Override threshold
- `bonus_structure` (jsonb) - Department-specific bonuses
- `effective_from` (date) - Policy start
- `effective_to` (date) - Policy end (nullable)
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `departments` (override scope)

### Employee Payroll Config

Employee-level policy override.

**Database Table**: `employee_payroll_configs`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `employee_id` (UUID) - Employee scope
- `base_salary` (decimal) - Fixed salary
- `hourly_rate` (decimal) - Hourly compensation
- `overtime_multiplier` (decimal) - Override rate
- `incentive_structure` (jsonb) - Individual incentives
- `effective_from` (date) - Policy start
- `effective_to` (date) - Policy end (nullable)
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `employees` (override scope)

### Payslip

Frozen payroll snapshot.

**Database Table**: `payslips`

**Attributes**:
- `id` (UUID) - Primary identifier
- `workspace_id` (UUID) - Workspace scope
- `employee_id` (UUID) - Paid employee
- `payroll_period_start` (date) - Period start
- `payroll_period_end` (date) - Period end
- `gross_salary` (decimal) - Total earnings
- `net_salary` (decimal) - After deductions
- `status` (enum) - draft, finalized, paid
- `payment_date` (date) - When paid (nullable)
- `created_at` (timestamp)
- `updated_at` (timestamp)

**Relationships**:
- M:1 with `employees` (paid employee)
- 1:N with `payslip_segments` (line items)

**Key Characteristics**:
- Immutable once finalized
- Audit trail of payroll history
- Detailed breakdown via segments

### Payslip Segment

Line item in payslip breakdown.

**Database Table**: `payslip_segments`

**Attributes**:
- `id` (UUID) - Primary identifier
- `payslip_id` (UUID) - Parent payslip
- `segment_type` (enum) - basic, ot, bonus, deduction
- `description` (text) - Line label
- `amount` (decimal) - Segment value
- `rate` (decimal) - Applied rate (if OT)
- `hours` (decimal) - Hours worked (if OT)
- `metadata` (jsonb) - Calculation details
- `created_at` (timestamp)

**Relationships**:
- M:1 with `payslips` (parent)
- M:1 with `time_entries` (source - for OT)

**Common Segment Types**:
- `basic` - Base salary
- `ot` - Overtime premium
- `bonus` - Performance/incentive
- `deduction` - Tax, benefits, garnishments

## API Request/Response Contracts

### Authentication Endpoints

**POST /api/auth/login**

Request:
```json
{
  "email": "user@example.com",
  "password": "secure_password"
}
```

Response (200 OK):
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user_id": "uuid",
  "workspace_id": "uuid",
  "role": "admin"
}
```

### Employee Endpoints

**GET /api/workspaces/:ws_id/employees**

Query params: `page=1&limit=20&status=active`

Response (200 OK):
```json
{
  "data": [
    {
      "id": "uuid",
      "employee_id": "EMP001",
      "first_name": "John",
      "last_name": "Doe",
      "email": "john@example.com",
      "department_id": "uuid",
      "status": "active",
      "hire_date": "2023-01-15"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 150
  }
}
```

**POST /api/workspaces/:ws_id/employees**

Request:
```json
{
  "employee_id": "EMP002",
  "first_name": "Jane",
  "last_name": "Smith",
  "email": "jane@example.com",
  "department_id": "uuid",
  "hire_date": "2026-02-01"
}
```

Response (201 Created):
```json
{
  "id": "uuid",
  "employee_id": "EMP002",
  "first_name": "Jane",
  "last_name": "Smith",
  "email": "jane@example.com",
  "department_id": "uuid",
  "status": "active",
  "hire_date": "2026-02-01",
  "created_at": "2026-01-28T11:30:00Z"
}
```

### Attendance Endpoints

**GET /api/workspaces/:ws_id/attendance/logs**

Query params: `device_id=uuid&start_date=2026-01-01&end_date=2026-01-31`

Response (200 OK):
```json
{
  "data": [
    {
      "id": "uuid",
      "device_id": "uuid",
      "employee_id": "uuid",
      "log_type": "check_in",
      "timestamp": "2026-01-28T09:00:00Z",
      "raw_payload": {}
    }
  ],
  "pagination": {
    "total": 500
  }
}
```

**POST /api/workspaces/:ws_id/attendance/sync**

Request (from agent):
```json
{
  "agent_id": "agent-001",
  "batch": [
    {
      "device_id": "uuid",
      "employee_id": "uuid",
      "log_type": "check_in",
      "timestamp": "2026-01-28T09:00:00Z"
    }
  ]
}
```

Response (200 OK):
```json
{
  "received": 100,
  "processed": 100,
  "errors": []
}
```

### WebSocket Endpoint

**GET /ws/agents/:agent_id**

Agent connects via WebSocket upgrade.

**Agent → Server** (periodic):
```json
{
  "type": "logs",
  "batch": [
    {"device_id": "uuid", "user_id": "uuid", "timestamp": "..."}
  ]
}
```

**Server → Agent** (acknowledgment):
```json
{
  "type": "ack",
  "ids": [1, 2, 3]
}
```

**Server → Agent** (configuration change):
```json
{
  "type": "reload_config",
  "config": {...}
}
```

## Multi-Tenant Data Isolation

### Enforcement Layers

**Layer 1: Path Parameter**
- Extract from URL: `/api/workspaces/:ws_id/...`
- Binding at router level
- Fallback to header: `X-Workspace-Id`

**Layer 2: Query Filtering**
- All repository queries include: `WHERE workspace_id = $1`
- Enforced at data access layer
- No exceptions (non-negotiable)

**Layer 3: Schema**
- Foreign keys on workspace_id (optional)
- Cascade deletes for workspace removal
- Unique constraints scoped by workspace

**Layer 4: Testing**
- `schema_isolation_tests.rs` validates coverage
- All table mutations verified with workspace filter

### Data Leakage Prevention

- Request context extracted once at handler
- Passed through service → repository chain
- Database drivers bound by parametrized queries
- No string concatenation (SQLi prevention)

## Temporal Data Patterns

### Effective Dating

Policy tables use temporal columns for versioning:

```sql
WHERE effective_from <= target_date
  AND (effective_to IS NULL OR effective_to >= target_date)
ORDER BY effective_from DESC
LIMIT 1
```

**Tables Using This Pattern**:
- `shift_rosters` - Schedule history
- `working_patterns` - Policy changes
- `payroll_settings` - Rate changes
- `department_payroll_configs` - Department policy versions
- `employee_payroll_configs` - Employee policy versions

**Benefits**:
- Historical audit trail
- Policy rollback capability
- Retroactive adjustments
- Point-in-time reporting

## Database Indexes

**Key Indexes** (assumed from design):
- `raw_attendance_logs(workspace_id, device_id, timestamp)`
- `raw_attendance_logs(workspace_id, employee_id, timestamp)`
- `time_entries(workspace_id, employee_id, date)`
- `time_entries(workspace_id, created_at)` (FIFO ordering)
- `employees(workspace_id, status)`
- `shift_rosters(workspace_id, employee_id, assigned_date)`
- `leave_requests(workspace_id, employee_id, status)`

## Business Rules

### Attendance Pairing
- Raw logs paired into time_entries via anchor-first algorithm
- Check-in followed by check-out creates one time_entry
- Intra-day breaks create separate time_entries
- Manual entries bypass pairing logic

### Leave Accrual
- Leave balance updates when leave_request approved
- Consumption happens on leave_request start_date
- Carry-forward applies based on leave_type policy
- Fiscal year resets per leave_type configuration

### Payroll Calculation
1. Select applicable payroll_configs (policy resolution)
2. Query time_entries for payroll period
3. Calculate regular hours + overtime
4. Apply appropriate rates (base, OT, holiday, weekend)
5. Generate payslip with segment breakdown
6. Mark payslip as finalized (immutable)

### Shift Assignment
- Employees assigned via shift_rosters (specific dates)
- Or via working_patterns (recurring schedule)
- Assignment_rules can auto-generate rosters
- Most specific assignment wins (date-specific > pattern)

## Error Handling

### Device Error Tracking

**Table**: `parsing_errors` (SQLite, agent-local)

Stores:
- Raw log entry that failed to parse
- Error message and stack trace
- Timestamp of error occurrence
- Manual intervention flag

### Sync Error Tracking

**WebSocket Errors**:
- Connection failures logged
- Retry attempts tracked
- Stability window resets on 60s consecutive success

**Database Errors**:
- SQLx errors surfaced in API response
- Transaction rollback on validation failure
- Audit trail preserved for investigation

## Next Steps for Domain Modeling

1. **Review Raw Log Flow**: Trace a punch from device → raw_logs → time_entries
2. **Understand Policy Resolution**: Review payroll tier selection logic
3. **Leave Workflow**: Map leave request state machine
4. **Temporal Queries**: Practice effective_from/to filtering
5. **Multi-Tenancy Verification**: Audit workspace_id presence in all queries
