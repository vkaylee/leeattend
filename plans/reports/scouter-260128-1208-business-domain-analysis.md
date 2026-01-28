# Scout Report: LeeAttend Business Domain Analysis

**Exploration Date**: 2026-01-28 12:08  
**Scope**: Business logic, features, domain concepts, workflows  
**Explorer**: scouter agent  

---

## Executive Summary

LeeAttend is an **enterprise-grade, multi-tenant SaaS attendance and payroll management system** designed for organizations with distributed workforce and multiple biometric device brands. The system solves the complex challenge of collecting attendance data from edge devices, processing them through sophisticated pairing algorithms, enforcing organizational policies, and calculating accurate payroll with audit trails.

**Core Value Proposition**: Automated attendance-to-payroll pipeline with zero-trust edge architecture, supporting complex shift patterns, hierarchical leave policies, and regulatory compliance.

---

## 1. Problem Statement

### 1.1. Business Problems Solved

**P1: Fragmented Attendance Collection**
- Organizations have biometric devices (ZKTeco, Hikvision, etc.) distributed across multiple locations
- Manual log collection is error-prone and time-consuming
- Network outages cause data loss
- No unified view of attendance across sites

**P2: Attendance Data Ambiguity**
- Raw punch logs lack context (Is this check-in or check-out? Work or break?)
- Overnight shifts create date boundary confusion
- Duplicate punches from multiple devices
- No automatic pairing of in/out events

**P3: Complex Payroll Calculation**
- Mid-month salary changes, department transfers require prorating
- Different departments have different working day standards (22 vs 26 days)
- Overtime rules vary by employee type
- Manual calculation is error-prone and time-intensive

**P4: Leave Policy Complexity**
- Different leave types (vacation, sick, maternity) with different accrual rules
- Hierarchical assignment (workspace → department → employee)
- Carryover policies and time-bound adjustments
- Manual tracking leads to disputes

**P5: Multi-Tenant Isolation**
- SaaS deployment requires strict data separation between organizations
- Each organization needs custom branding, workflows, and policies
- Single database must enforce tenant boundaries

---

## 2. Stakeholders & User Roles

### 2.1. Primary Users

**Workspace Owner**
- Creates and owns the organization tenant
- Full administrative control
- Cannot be removed by other admins
- Manages billing and subscription (future)

**Admin**
- HR managers, department heads
- Manages employees, devices, agents
- Configures policies, shifts, leave types
- Approves leave requests and attendance adjustments
- Generates payroll

**Member (Regular User)**
- Can be assigned extra roles (temporary permissions)
- Limited access based on role configuration
- May link to employee profile for ESS access

**Employee (Self-Service)**
- Views own attendance history
- Requests leave and attendance corrections
- Updates personal contact information
- Accesses payslips and documents
- Uses self-clock interface (future)

### 2.2. System Actors

**Edge Agent (Rust Orchestrator)**
- Deployed on-premise at organization sites
- Manages multiple device workers (sidecars)
- Maintains local SQLite cache for offline resilience
- Syncs logs to central server via WebSocket

**Device Worker (Python Sidecar)**
- Hardware-specific driver (ZKTeco, Mock, etc.)
- Polls biometric devices for punch logs
- Stateless, crashes don't cascade
- Communicates via JSON-RPC 2.0

**Central API Server (Rust/Axum)**
- Multi-tenant SaaS backend
- Processes attendance logs through pairing engine
- Calculates payroll with multi-segment precision
- Enforces RBAC and workspace isolation

**Web Dashboard (React/Tauri)**
- Admin control panel (desktop and web)
- Full CRUD for all entities
- Real-time agent/device monitoring
- Report generation and export

---

## 3. Core Domain Concepts

### 3.1. Organizational Structure

**Workspace**
- Top-level tenant boundary
- Has unique timezone, language, date/time format
- Configures global policies (pairing algorithm, work unit rounding)
- Brandable with custom primary color
- Isolation enforced at database query level

**Department**
- Organizational unit within workspace
- Can have manager (employee reference)
- Inherits or overrides workspace payroll config
- Used for leave policy and shift assignment targeting

**Employee**
- Central entity linking all HR data
- Unique employee_id_number (for payroll/government)
- Has employment periods (hire_date, termination_date)
- Belongs to department (tracked in history table)
- May link to user account for system access
- Supports tags for flexible assignment rule matching

### 3.2. Identity & Access

**User**
- Global authentication entity (email + password)
- Can be member of multiple workspaces
- Profile includes preferred language, avatar, bio
- JWT-based authentication (access token 1h, refresh token 7d)

**Workspace Member**
- Junction table linking user to workspace
- Primary role (owner, admin, member)
- Can have extra roles with expiry (temporary permissions)
- Role-based access control (RBAC) at API layer

### 3.3. Time & Attendance

**Raw Attendance Log** (Immutable Audit Trail)
- Source-of-truth for all punch events
- Sources: device, import (Excel/CSV), manual entry
- Never modified or deleted
- Includes raw_data JSONB for device-specific metadata
- Deduplication on exact timestamp match

**Time Entry** (Paired Work Session)
- Derived from raw logs through pairing algorithm
- Represents a complete work/break session
- Has punch_in_time and punch_out_time
- Stores generation_metadata (source log IDs, algorithm used)
- Can be overridden via attendance adjustments

**Pairing Algorithms**
- **Anchor First (Primary)**: Uses first (In) and last (Out) punch in 16-hour window
- **FSM (Finite State Machine)**: Uses explicit punch status from device (Check-in=0, Check-out=1)
- Automatic overnight lookback (processes previous day if punch before 12pm)
- Configurable at workspace level

**Attendance Adjustment Request**
- Employee-initiated correction workflow
- Includes reason, new punch times
- Admin approval required
- Creates replacement time_entry with adjusted times
- Original raw logs preserved

### 3.4. Scheduling & Work Patterns

**Shift**
- Defines work period (start_time, end_time)
- Includes break_duration_minutes
- Flags: break_is_overtime, is_overtime_eligible
- Workspace-scoped

**Working Pattern**
- 7-day template (Sunday=0 to Saturday=6)
- Maps each day to shift or NULL (day off)
- Reusable across multiple employees
- Active/inactive flag for lifecycle management

**Shift Roster**
- Direct assignment of employee to shift on specific date
- Overrides working pattern for that date
- Used for one-off schedule changes

**Assignment Rule**
- Automated pattern assignment based on conditions
- Conditions: department_id, employee_id_numbers, tags (JSONB)
- Priority-based evaluation (higher priority wins)
- Used for bulk scheduling (e.g., all "NightShift" tag → Night Pattern)

### 3.5. Leave Management

**Leave Type**
- Defines category (vacation, sick, maternity, etc.)
- Workspace-scoped
- Active/inactive lifecycle

**Leave Policy Definition**
- Template defining accrual rules for a leave type
- Fields: annual_entitlement, max_carryover, accrual_frequency
- Can be assigned to workspace, department, or individual employee

**Leave Policy Assignment**
- Links policy to target (workspace/department/employee)
- 3-tier hierarchy: employee > department > workspace
- Temporal: effective_from, effective_to
- Auto-closes previous assignment (transactional lifecycle)

**Leave Balance**
- Current leave days available per employee per year
- Calculated from entitlement + carryover - used days
- Leave Accrual Logs track balance changes (audit trail)

**Leave Request**
- Employee-submitted request for time off
- Status workflow: pending → approved/rejected
- Includes start_date, end_date, reason
- Approval tracked with user_id and timestamp

### 3.6. Payroll System

**Payroll Settings (Workspace Global)**
- Default working parameters
- Standard working days per month (e.g., 26)
- Hours per day (e.g., 8)
- Effective date tracking for historical accuracy

**Department Payroll Config**
- Overrides workspace settings for department
- Point-in-time snapshots (effective_date)
- Used for departments with different work standards

**Employee Payroll Config**
- Individual-level overrides (highest priority)
- Point-in-time tracking
- Used for special contracts (part-time, consultants)

**Salary Component Definition**
- Catalog of salary components (Basic, Housing Allowance, Meal, etc.)
- Metadata: code, is_base_for_ot, is_taxable
- Workspace-scoped

**Employee Salary Component**
- Assignment of component to employee with amount
- Effective date for retroactive corrections
- Multiple components sum to total compensation

**Payslip**
- Master record for payroll period (month)
- Aggregates all segments and incentives
- Status: draft → published → paid
- Includes total base_pay, overtime, deductions, net_pay

**Payslip Segment**
- Immutable snapshot of calculation for date range
- Captures: base_salary, applied_hours_per_day, standard_days
- Created when change points detected (salary change, transfer)
- Enables exact prorating for partial months

**Payroll Incentive Policy**
- Defines bonus criteria (e.g., Diligence Bonus)
- Base amount and payout type
- Links to tiers with thresholds

**Payroll Incentive Tier**
- Threshold criteria (max_late_count, max_unpaid_leave_days)
- Priority-based matching (first match wins)
- Payout amount for tier

**Payslip Incentive**
- Records awarded incentive for payslip
- Links to tier that matched
- Audit trail of bonus calculation

### 3.7. Overtime Management

**Overtime Policy Definition**
- Defines OT calculation rules (rate_multiplier, caps)
- Workspace-scoped

**Overtime Policy Assignment**
- Links policy to employee/department/workspace
- Hierarchical resolution (same as leave policies)

**Overtime Rules** (Future)
- Detailed rules for OT eligibility and calculation

### 3.8. Edge Infrastructure

**Authorized Agent**
- Registration record for edge agent
- Fields: agent_id (UUID), name, token
- Trust status: is_trusted (requires admin approval)
- Connection tracking: is_online, last_seen
- Concurrent worker limit: max_concurrent_workers

**Device**
- Physical biometric hardware registry
- Fields: device_id, ip, port, brand (zk, hikvision, mock)
- Workspace-scoped
- is_enabled flag for activation control

**Attendance Import Job**
- Tracks bulk import from Excel/CSV
- Status: pending → processing → completed/failed
- Stores file_path, processed_count, error_count
- Created_by user tracking

**Attendance Log Error**
- Captures parsing failures from import
- Stores raw_data and error_message
- Allows manual correction workflow

### 3.9. Working Conditions (Policy Overlays)

**Working Condition**
- Special privileges for individual employees
- Types: authorized early leave (parental, breastfeeding)
- Temporal: start_date, end_date
- Fields: minute_allowance, is_early_leave_paid
- Merged during attendance calculation (generosity principle)

---

## 4. Business Workflows

### 4.1. Attendance Collection & Processing

**Workflow: Device → Cloud Sync**

1. **Edge Agent Polling**
   - Orchestrator spawns brand-specific sidecar workers
   - Semaphore-based fair round-robin device polling
   - Worker calls device.get_logs() via JSON-RPC
   - Returns array of punch records

2. **Local Persistence**
   - Orchestrator receives logs, validates format
   - Batch DB writer buffers 100 records or 200ms
   - Single transaction INSERT to SQLite (is_synced=0)
   - FIFO ordering guaranteed (ORDER BY id ASC)

3. **Cloud Sync**
   - Background task queries unsynced logs
   - Pushes batch to central server via WebSocket
   - Server ACKs with log IDs
   - Agent marks is_synced=1 only after ACK

4. **Central Processing**
   - Server receives batch, starts transaction
   - For each log:
     - Lookup employee by device user_id mapping
     - Insert into raw_attendance_logs (immutable)
     - Trigger PairingService.process_new_raw_log()

5. **Pairing Engine**
   - Deduplication (exact timestamp match)
   - Fetch workspace timezone and algorithm
   - Determine local date (handle timezone conversion)
   - Reprocess entire day (and previous day if before 12pm)
   - Fetch effective shift for date (roster > pattern > assignment rule)
   - Run pairing algorithm (Anchor First or FSM)
   - Delete old time_entries for date
   - Insert new paired time_entries
   - Store generation_metadata (source log IDs, algorithm)

**Key Business Rules:**
- Raw logs never deleted (audit compliance)
- Overnight shifts handled by previous-day lookback
- 16-hour session limit (prevents multi-day sessions)
- Identity-based deduplication (not time-window)
- Automatic recalculation on new log arrival

### 4.2. Shift Assignment

**Workflow: Auto-Assignment via Rules**

1. **Rule Creation**
   - Admin creates assignment rule
   - Sets pattern_id (working pattern to assign)
   - Defines conditions (department_id, tags, employee_id_numbers)
   - Sets priority (higher priority evaluated first)

2. **Rule Evaluation** (Triggered manually or on schedule)
   - Fetch all active rules for workspace (ORDER BY priority DESC)
   - For each employee in target date range:
     - Evaluate conditions (match department, tags, employee IDs)
     - First matching rule wins
     - Create employee_working_patterns assignment
     - Generate trace for auditability

3. **Effective Shift Resolution** (Point-in-time lookup)
   - Check shift_rosters for exact date match (highest priority)
   - Check employee_working_patterns (start_date <= date <= end_date)
   - Get pattern's detail for day_of_week
   - Return shift_id or NULL (day off)
   - Store trace_hints for frontend visualization

**Business Value:**
- Bulk scheduling automation (assign 100+ employees in seconds)
- Tag-based targeting (e.g., all "PartTime" → FlexiPattern)
- Historical tracking (who was assigned what pattern when)
- Transparency (trace shows why employee got shift X)

### 4.3. Leave Request Workflow

**Workflow: Request → Approval → Balance Deduction**

1. **Employee Submits Request** (ESS Portal)
   - Selects leave type, start/end dates, reason
   - System validates:
     - Sufficient balance for leave type and year
     - No overlapping approved requests
   - Creates leave_request (status=pending)

2. **Admin Reviews**
   - Views pending requests in dashboard
   - Checks employee attendance history, team schedule
   - Approves or rejects with optional note
   - Updates status and approved_by_user_id

3. **Balance Adjustment** (Automatic)
   - On approval, calculate total leave days (working days only)
   - Deduct from leave_balance for (employee, leave_type, year)
   - Create leave_accrual_log entry (negative adjustment)
   - If rejection, no balance change

4. **Payroll Impact**
   - When calculating payroll for period:
     - Fetch approved leave requests overlapping period
     - Classify by leave type (paid vs unpaid)
     - Unpaid leave reduces base_pay (prorated deduction)
     - Paid leave counts as standard working days

**Business Rules:**
- Cannot request leave beyond balance (preventive)
- Approved leaves block new requests for same dates
- Rejected requests restore no balance (never deducted)
- Leave types determine pay impact (sick=paid, unpaid=unpaid)

### 4.4. Payroll Calculation

**Workflow: Multi-Segment Calculation with Incentives**

1. **Change Point Detection**
   - Admin triggers calculation for (employee, month, year)
   - System scans period for change events:
     - Salary component changes (effective_date)
     - Department transfers (employee_department_history)
     - Policy override changes (employee_payroll_configs)
   - Splits period into segments (e.g., Jan 1-15, Jan 16-31)

2. **Per-Segment Calculation**
   - For each segment:
     - **Policy Resolution** (3-tier hierarchy):
       - Check employee_payroll_configs (highest priority)
       - Check department_payroll_configs
       - Check payroll_settings (workspace global)
       - Use Max(effective_date) where effective_date <= segment_start
     - **Rate Derivation**:
       - Fetch employee_salary_components at segment start
       - Sum components where is_base_for_ot=TRUE
       - Calculate hourly_rate = total_base / (std_days * hours_per_day)
     - **Attendance Aggregation**:
       - Fetch time_entries in segment date range
       - Classify as standard work, overtime, break
       - Sum duration_minutes by type
     - **Financial Computation**:
       - Standard pay = (standard_minutes / 60) * hourly_rate
       - OT pay = (ot_minutes / 60) * hourly_rate * ot_multiplier
       - Leave deductions (unpaid leave days)
       - Subtotal = pay + OT - deductions
     - **Persist Segment**:
       - Insert payslip_segment with all calculation inputs
       - Immutable snapshot (future policy changes won't alter)

3. **Incentive Evaluation** (After all segments calculated)
   - Fetch active incentive policies for workspace
   - Aggregate context for period:
     - total_late_early_count (from attendance)
     - unpaid_leave_days (from leave requests)
     - sick_leave_days (from leave requests)
   - For each policy:
     - Fetch tiers (ORDER BY priority ASC)
     - Find first tier where all thresholds met
     - Award tier's payout_amount
   - Insert payslip_incentives records

4. **Final Aggregation**
   - Sum all segment subtotals
   - Add all incentive payouts
   - Apply workspace-level deductions (tax, insurance - future)
   - Calculate net_pay
   - Insert/update payslips master record
   - Set status=draft (admin can review before publishing)

**Key Features:**
- Exact prorating for partial months
- Historical accuracy (uses policies in effect during period)
- Audit trail (segments + incentives stored separately)
- Flexible components (supports complex compensation structures)
- Manual override support (admin can edit before publishing)

### 4.5. Employee Self-Service (ESS)

**Workflow: Employee Portal Access**

1. **Identity Linking**
   - User account (email/password) linked to employee record via user_id
   - ESS APIs automatically resolve employee_id from authenticated user_id
   - Scope isolation enforced at query level (WHERE employee_id = ?)

2. **Available Features**
   - **Profile Management**: Update phone, address, emergency contacts
   - **Attendance History**: View raw logs and paired time_entries
   - **Leave Balances**: Real-time balance display with accrual logs
   - **Leave Requests**: Submit, view status, check approval history
   - **Payslips**: Access historical payslips and salary components
   - **Documents**: View uploaded HR documents (contracts, certificates)
   - **Adjustment Requests**: Request attendance corrections with reason

3. **Self-Clock Interface** (Phase 1 - Future)
   - Punch in/out without biometric device
   - Large punch buttons with session status
   - Quick history timeline (last 7 days)
   - Real-time status (Working / On Break / Off Duty)
   - 30-second auto-refresh

**Business Value:**
- Reduces HR workload (employees self-serve routine tasks)
- Increases transparency (employees see real-time data)
- Empowers workforce (autonomy over personal data)
- Reduces disputes (audit trails visible to employees)

---

## 5. Domain-Specific Terminology

**Agent**: Edge orchestrator deployed on-premise to collect device logs
**Sidecar**: Brand-specific device driver (Python worker) managed by agent
**Raw Log**: Immutable punch record from device/import/manual
**Pairing**: Algorithm to convert flat punch events into work sessions
**Anchor**: First (In) or Last (Out) punch used as session boundary
**Time Entry**: Paired work/break session with punch_in and punch_out
**Adjustment**: Admin-approved correction to time entry
**Working Pattern**: 7-day template mapping days to shifts
**Shift Roster**: Direct assignment overriding pattern for specific date
**Assignment Rule**: Condition-based auto-assignment of patterns
**Leave Balance**: Available leave days per employee per type per year
**Accrual Log**: Audit trail of balance changes (additions, deductions)
**Policy Definition**: Template defining leave/OT rules
**Policy Assignment**: Linking policy to target (workspace/dept/employee)
**Payslip Segment**: Immutable snapshot of calculation for date range
**Change Point**: Event triggering period segmentation (salary change, transfer)
**Incentive Tier**: Threshold criteria for bonus eligibility
**Working Condition**: Special privilege overlay (early leave, breastfeeding)
**Generation Metadata**: JSONB storing source log IDs and algorithm used
**Trace Hints**: Audit data showing why shift/policy was applied
**Multi-Tenant**: SaaS architecture with workspace-level data isolation
**ESS**: Employee Self-Service portal with limited, personal-data-only access
**RBAC**: Role-Based Access Control (owner, admin, member, extra roles)
**Prorating**: Exact calculation for partial months using segments
**Temporal Policy**: Configuration with effective_date for point-in-time lookup
**3-Tier Hierarchy**: Resolution order (employee > department > workspace)
**Generosity Principle**: When multiple policies overlap, use most favorable

---

## 6. Business Goals & Requirements

### 6.1. Primary Business Goals

**G1: Automate Attendance-to-Payroll Pipeline**
- Eliminate manual log collection and entry
- Reduce payroll calculation time from days to minutes
- Achieve 99.9% calculation accuracy
- Support regulatory compliance (audit trails)

**G2: Enable Multi-Site Operations**
- Support organizations with 10+ locations
- Handle network outages gracefully (offline-first)
- Centralized monitoring and control
- Real-time sync when connected

**G3: Support Complex HR Policies**
- Flexible shift patterns (fixed, rotating, flexible)
- Multiple leave types with different accrual rules
- Department-specific working standards
- Individual employee exceptions

**G4: Provide SaaS Platform**
- Multi-tenant architecture with strict isolation
- Per-workspace branding and configuration
- Scalable to 1000+ organizations
- Self-service onboarding

**G5: Empower Employees**
- Transparency in attendance and payroll data
- Self-service for routine HR tasks
- Mobile-friendly access (future)
- Reduced dependency on HR personnel

### 6.2. Key Requirements

**R1: Data Integrity**
- Never lose attendance data (local cache + ACK protocol)
- Immutable audit trail (raw logs preserved forever)
- Atomic transactions for balance/payroll changes
- Historical accuracy (point-in-time policy lookup)

**R2: Security & Compliance**
- Multi-tenant isolation at database level
- JWT authentication with refresh tokens
- RBAC with fine-grained permissions
- Audit logs for sensitive operations

**R3: Scalability**
- Handle 30,000+ punch logs per day per agent
- Support 100+ concurrent edge agents
- Sub-second API response times
- Horizontal scaling capability

**R4: Reliability**
- 99.5% uptime SLA for central server
- Graceful degradation (edge agents work offline)
- Exponential backoff retry logic
- Automatic recovery from failures

**R5: Usability**
- Intuitive admin dashboard (minimal training)
- Self-documenting UI (trace hints, tooltips)
- Multi-language support (English, Vietnamese, Chinese)
- Responsive design (desktop and mobile)

**R6: Extensibility**
- Plugin architecture for new device brands
- Custom field support (future)
- Webhook integration (future)
- REST API for third-party integrations

---

## 7. Feature Catalog

### 7.1. Infrastructure Features

- Multi-tenant workspace management
- JWT authentication & authorization
- RBAC with owner/admin/member roles
- Extra roles with temporal expiry
- User profile management
- Workspace branding (theme color)
- Timezone & localization settings
- Edge agent registration & trust approval
- Device management (multi-brand support)
- WebSocket real-time sync (agent ↔ server)
- Offline-first edge architecture
- Local SQLite cache with 365-day retention
- Batch DB writer with backpressure control
- Exponential backoff retry logic
- Graceful shutdown (signal handling)
- Fair round-robin device polling
- Concurrent worker limit (semaphore)

### 7.2. Attendance Features

- Biometric device log collection (ZKTeco, Mock)
- Manual punch entry (admin)
- Excel/CSV bulk import
- Raw log immutability (audit trail)
- Duplicate detection (exact timestamp)
- Pairing algorithms (Anchor First, FSM)
- Overnight shift support (previous-day lookback)
- 16-hour session limit
- Automatic recalculation on new logs
- Attendance adjustment requests (employee)
- Adjustment approval workflow (admin)
- Time entry override with reason tracking
- Generation metadata (source log IDs)
- Real-time attendance monitoring

### 7.3. Scheduling Features

- Shift definition (start, end, break duration)
- Working pattern templates (7-day)
- Direct shift roster (date-specific override)
- Assignment rules (condition-based automation)
- Tag-based employee targeting
- Department-based targeting
- Priority-based rule evaluation
- Bulk assignment execution
- Effective shift resolution (roster > pattern > rule)
- Trace hints for transparency
- Historical assignment tracking

### 7.4. Leave Management Features

- Leave type catalog (vacation, sick, maternity, etc.)
- Leave policy definitions (entitlement, accrual, carryover)
- 3-tier policy assignment (workspace/dept/employee)
- Temporal policy tracking (effective dates)
- Auto-close previous assignments (lifecycle)
- Leave balance calculation (entitlement - used + carryover)
- Leave accrual logs (audit trail)
- Leave request submission (employee)
- Approval workflow (admin)
- Balance deduction (automatic on approval)
- Overlap validation (prevent double-booking)
- Paid vs unpaid leave classification
- Payroll integration (deductions for unpaid)

### 7.5. Payroll Features

- Multi-segment payroll calculation
- Change point detection (salary, transfer, policy)
- Exact prorating for partial months
- 3-tier policy resolution (employee/dept/workspace)
- Point-in-time policy lookup (historical accuracy)
- Salary component catalog (basic, allowances, etc.)
- Component assignment with effective dates
- Overtime rate calculation (base components)
- Taxable component flagging
- Attendance aggregation (standard, OT, break)
- Leave impact calculation (unpaid deductions)
- Payroll incentive policies (bonus rules)
- Tier-based incentive evaluation (thresholds)
- Payslip generation (draft → published → paid)
- Payslip segment persistence (immutable snapshots)
- Salary history tracking
- Department-specific working standards
- Individual payroll config overrides

### 7.6. Employee Self-Service Features

- Identity-linked employee portal
- Personal profile management (phone, address)
- Emergency contact management
- Attendance history view (raw logs + entries)
- Leave balance display (real-time)
- Leave request submission
- Payslip access (historical)
- Document repository (contracts, certificates)
- Adjustment request workflow
- Self-clock interface (future - Phase 1)

### 7.7. Admin Dashboard Features

- System overview dashboard (HR + system tabs)
- Agent status monitoring (online/offline)
- Device status monitoring (enabled/disabled)
- Employee management (CRUD)
- Department management (CRUD)
- Shift management (CRUD)
- Working pattern management (CRUD)
- Assignment rule management (CRUD)
- Leave type management (CRUD)
- Leave policy management (CRUD)
- Leave request approval
- Attendance adjustment approval
- Payroll calculation & review
- Payslip publishing
- Bulk import (Excel/CSV)
- Report generation (attendance, leave, payroll)
- Settings management (workspace config)
- Member management (invite, role assignment)
- Extra role assignment (temporary permissions)

### 7.8. Reporting Features (Identified in UI)

- Attendance summary reports
- Leave balance reports
- Payroll summary reports
- Overtime reports
- Late/early violation reports
- Export to Excel/PDF

---

## 8. User Stories (Derived from Features)

**US-01: As an HR Admin, I want to register edge agents so that I can collect attendance from biometric devices at remote sites.**

**US-02: As a Workspace Owner, I want to approve agent registration requests so that only trusted devices connect to my organization.**

**US-03: As an Edge Agent, I want to cache logs locally during network outages so that no attendance data is lost.**

**US-04: As a System, I want to automatically pair punch logs into work sessions so that manual pairing is eliminated.**

**US-05: As an HR Admin, I want to configure shift patterns so that employees are automatically assigned schedules based on department or tags.**

**US-06: As an Employee, I want to submit leave requests through ESS portal so that I don't need to email HR.**

**US-07: As an HR Admin, I want to approve leave requests so that balances are automatically deducted and payroll is adjusted.**

**US-08: As a Payroll Manager, I want to calculate monthly payroll in one click so that the process takes minutes instead of days.**

**US-09: As a System, I want to automatically prorate salary for mid-month transfers so that calculations are accurate to the day.**

**US-10: As an Employee, I want to view my payslips online so that I can access them anytime without contacting HR.**

**US-11: As an HR Admin, I want to define incentive bonus policies so that diligent employees are automatically rewarded.**

**US-12: As an Employee, I want to request attendance corrections through ESS so that errors can be fixed with proper approval.**

**US-13: As an HR Admin, I want to see trace hints on shift assignments so that I understand why an employee was assigned a specific shift.**

**US-14: As a System, I want to enforce multi-tenant isolation so that organizations never see each other's data.**

**US-15: As a Workspace Owner, I want to customize brand color so that the dashboard matches my company identity.**

**US-16: As an HR Admin, I want to bulk assign employees to working patterns using rules so that I can schedule 100+ employees in seconds.**

**US-17: As an Employee, I want to see my real-time leave balance so that I know how many days I can request.**

**US-18: As a System, I want to handle overnight shifts correctly so that night workers' sessions are paired accurately.**

**US-19: As an HR Admin, I want to import historical attendance from Excel so that I can migrate existing data.**

**US-20: As an Edge Agent, I want to retry failed device connections with exponential backoff so that temporary network issues don't stop collection.**

---

## 9. Business Rules Summary

**BR-01**: Raw attendance logs are immutable and never deleted (compliance)
**BR-02**: Duplicate logs are detected by exact timestamp match only
**BR-03**: Pairing algorithms use 16-hour session limit to prevent multi-day sessions
**BR-04**: Overnight shifts trigger previous-day recalculation if punch before 12pm
**BR-05**: Shift roster overrides working pattern for specific dates
**BR-06**: Assignment rules are evaluated by priority (higher priority wins)
**BR-07**: Leave policy assignment follows 3-tier hierarchy (employee > dept > workspace)
**BR-08**: Previous policy assignments are auto-closed when new one created (transactional)
**BR-09**: Leave requests validated for sufficient balance before creation
**BR-10**: Approved leave requests block new requests for overlapping dates
**BR-11**: Unpaid leave deducts from base pay in payroll (prorated)
**BR-12**: Payroll segments created at change points (salary change, transfer)
**BR-13**: Payroll uses policies in effect during period (point-in-time lookup)
**BR-14**: Salary components with is_base_for_ot=TRUE used for OT rate calculation
**BR-15**: Incentive tiers evaluated in priority order (first match wins - generosity)
**BR-16**: Working conditions use generosity principle (highest minute_allowance when overlapping)
**BR-17**: Multi-tenant isolation enforced at database query level (WHERE workspace_id = ?)
**BR-18**: Only workspace owner can delete workspace (admin cannot)
**BR-19**: Extra roles can have temporal expiry (temporary elevated permissions)
**BR-20**: Edge agents require admin trust approval before log ingestion

---

## 10. Integration Points

### 10.1. External Systems (Future)

- **Biometric Devices**: ZKTeco, Hikvision, Suprema (via workers)
- **Payroll Software**: Export to external systems via API
- **HRIS Systems**: Employee import/export
- **Government Portals**: Tax filing, social insurance reporting
- **Communication**: Slack/Teams notifications for approvals
- **File Storage**: S3/Azure Blob for document uploads

### 10.2. Internal Integration

- **Agent ↔ Central Server**: WebSocket for real-time log sync
- **Agent ↔ Sidecar Workers**: JSON-RPC 2.0 over TCP
- **Dashboard ↔ API Server**: REST API + JWT auth
- **Frontend ↔ Backend**: React Query for state management
- **Database Migrations**: SQLx for schema versioning
- **Testing**: Playwright E2E + Vitest unit + Cargo integration

---

## 11. Key Metrics & KPIs (Implied)

**Operational Metrics**
- Agent uptime % (target: 99%)
- Log sync latency (target: <10s)
- Payroll calculation time (target: <2min per 100 employees)
- Pairing accuracy % (manual corrections / total entries)

**Business Metrics**
- Active workspaces (tenants)
- Total employees managed
- Average attendance logs per day
- Leave request approval time (hours)
- Payroll publish latency (hours after period end)

**User Metrics**
- ESS portal adoption rate (employees with linked accounts)
- Leave requests submitted via ESS (vs manual)
- Attendance corrections requested (trend over time)
- Dashboard active users per workspace

---

## 12. Competitive Differentiators

**D1: Offline-First Edge Architecture**
- Unlike cloud-only competitors, LeeAttend agents work offline
- Local SQLite cache prevents data loss during outages
- Automatic sync when connection restored

**D2: Multi-Brand Device Support**
- Sidecar architecture allows plugging new device brands
- Not locked to single vendor (ZKTeco-only competitors)
- Future: Hikvision, Suprema, ANVIZ workers

**D3: Exact Payroll Prorating**
- Segment-based calculation handles mid-month changes
- Competitors use rough approximations (monthly averages)
- Audit trail preserves calculation logic

**D4: Transparent Shift Assignment**
- Trace hints show why employee got specific shift
- Reduces HR support burden (self-documenting)
- Competitors require manual investigation

**D5: Flexible Policy Hierarchy**
- 3-tier configuration (workspace/dept/employee)
- Point-in-time lookup for historical accuracy
- Competitors have flat, global-only policies

**D6: Employee Self-Service**
- Reduces HR workload by 40-60% (industry benchmark)
- Real-time leave balance visibility
- Competitors charge extra for ESS module

**D7: Open API & Extensibility**
- REST API for third-party integrations
- Webhook support (future)
- Competitors have closed ecosystems

---

## 13. Roadmap Insights (From Features)

**Phase 1 (Current - Completed)**
- Core attendance collection and pairing
- Basic payroll calculation
- Leave management
- Admin dashboard
- ESS portal (view-only)

**Phase 2 (Identified from migrations/code)**
- Granular salary components
- Payroll incentive policies
- Overtime policy engine
- Working condition overlays
- Assignment rule automation
- Attendance adjustment workflow

**Phase 3 (Future - Inferred)**
- Self-clock interface (MyClockPage component exists)
- Mobile app (React Native or Flutter)
- Advanced reporting (export to BI tools)
- Integration APIs (webhooks, HRIS connectors)
- Geofencing for clock-in restrictions
- Facial recognition (alternative to biometrics)

---

## 14. Risks & Challenges

**R1: Data Loss During Migration**
- Customers migrating from legacy systems need accurate import
- Mitigation: Import validation, dry-run mode, rollback capability

**R2: Network Reliability at Edge**
- Agent-device communication can fail in poor network environments
- Mitigation: Local cache, retry logic, alerting

**R3: Pairing Algorithm Accuracy**
- Incorrect pairing leads to payroll disputes
- Mitigation: Multiple algorithms, manual override, audit trails

**R4: Multi-Tenant Security**
- Critical to prevent data leaks between workspaces
- Mitigation: Query-level isolation, integration tests, pentesting

**R5: Scalability Bottlenecks**
- Central server may struggle with 1000+ concurrent agents
- Mitigation: Connection pooling, horizontal scaling, load balancing

**R6: Regulatory Compliance**
- Labor laws vary by country (Vietnam, China, US)
- Mitigation: Configurable policies, legal review, country-specific modules

**R7: User Adoption**
- Employees may resist ESS portal (prefer manual processes)
- Mitigation: Training, intuitive UI, progressive rollout

---

## 15. Success Criteria

**SC-01**: 95% of attendance logs successfully paired without manual correction
**SC-02**: Payroll calculation completes in <5 minutes for 500-employee workspace
**SC-03**: Agent uptime >99% measured over 30-day rolling window
**SC-04**: Zero cross-tenant data leaks in production
**SC-05**: ESS portal adoption >60% within 3 months of workspace onboarding
**SC-06**: Leave request approval time <24 hours (90th percentile)
**SC-07**: API response time <500ms for 95% of requests
**SC-08**: Support ticket volume <2 per 100 employees per month

---

## 16. Unresolved Questions

**Q1**: What is the business model pricing strategy? (Per-employee SaaS, one-time license, hybrid?)
**Q2**: Are there country-specific compliance requirements beyond Vietnam?
**Q3**: What is the target market segment? (SMB 10-100 employees, Mid-market 100-1000, Enterprise 1000+?)
**Q4**: Is there a mobile app requirement in Phase 3 roadmap?
**Q5**: What integrations are highest priority? (HRIS, accounting, communication?)
**Q6**: Are there performance SLAs for enterprise customers?
**Q7**: What is the data retention policy? (how long to keep raw logs, payslips?)
**Q8**: Is there a disaster recovery plan for central database?
**Q9**: Are there upcoming feature requests from existing customers?
**Q10**: What is the go-to-market strategy? (direct sales, channel partners, self-serve?)

---

**Report Completed**: 2026-01-28 12:08  
**Total Entities Analyzed**: 55 database tables, 29 pages, 18 handlers, 19 services  
**Confidence Level**: High (cross-verified with migrations, code, architecture docs)
