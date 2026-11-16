# LeeAttend — Data Flow Analysis

## System Data Flow Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                         DATA SOURCES                                │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ ZKTeco Device │  │ CSV/Excel    │  │ Manual Entry │             │
│  │ (Biometric)   │  │ Import       │  │ (Admin UI)   │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │                  │                  │                      │
└─────────┼──────────────────┼──────────────────┼─────────────────────┘
          │                  │                  │
          ▼                  ▼                  ▼
┌────────────────────────────────────────────────────────────────────┐
│                      INGESTION LAYER                                │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ Agent (TCP)   │  │ Import       │  │ REST API     │             │
│  │ → API sync    │  │ Service      │  │ Handlers     │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │                  │                  │                      │
│         ▼                  ▼                  ▼                      │
│  ┌──────────────────────────────────────────────┐                   │
│  │          raw_attendance_logs (PostgreSQL)      │                   │
│  │          Dedup by device + timestamp hash     │                   │
│  └──────────────────────┬───────────────────────┘                   │
└─────────────────────────┼───────────────────────────────────────────┘
                          │
                          ▼
┌────────────────────────────────────────────────────────────────────┐
│                     PROCESSING LAYER                                │
│                                                                     │
│  ┌──────────────────────┐                                          │
│  │ Pairing Service       │ ← Match punch-in/out pairs              │
│  └──────────┬───────────┘                                          │
│             ▼                                                       │
│  ┌──────────────────────┐                                          │
│  │ time_entries          │ ← Paired attendance records              │
│  └──────────┬───────────┘                                          │
│             ▼                                                       │
│  ┌──────────────────────┐  ┌──────────────────────┐               │
│  │ Attendance Calculator │  │ Holiday Repository    │               │
│  │ (work units, rounding)│  │ (is_holiday lookup)   │               │
│  └──────────┬───────────┘  └──────────┬───────────┘               │
│             ▼                          ▼                            │
│  ┌──────────────────────────────────────────────┐                   │
│  │ attendance_daily_summaries (PostgreSQL)        │                   │
│  │ Fields: work_minutes, ot_minutes,             │                   │
│  │         is_holiday, holiday_name               │                   │
│  └──────────────────────┬───────────────────────┘                   │
└─────────────────────────┼───────────────────────────────────────────┘
                          │
              ┌───────────┼───────────┐
              ▼                       ▼
┌──────────────────────┐  ┌──────────────────────┐
│   REPORTING LAYER    │  │   PAYROLL LAYER      │
│                      │  │                      │
│ Report Service       │  │ Payroll Service      │
│  - Daily report      │  │  - Monthly payslip   │
│  - Summary report    │  │  - OT pay (holiday×) │
│  - Status logic      │  │  - Incentive tier    │
│    (holiday/holiday_  │  │  - Late exclusion    │
│     work/present/...) │  │                      │
│                      │  │ Payroll Evaluator    │
│ Excel Writer         │  │  - Tier matching     │
│  - Timesheet matrix  │  │                      │
│  - Raw details       │  │                      │
│  - Holiday column    │  │                      │
└──────────┬───────────┘  └──────────┬───────────┘
           ▼                         ▼
┌──────────────────────┐  ┌──────────────────────┐
│ .xlsx File Download  │  │ payslips (PostgreSQL)│
│ WebSocket Progress   │  │ payslip_segments     │
└──────────────────────┘  └──────────────────────┘
```

## Sensitive Data Flows

### 1. Authentication Flow
```
User credentials (email/password)
  → HTTPS POST /auth/login
  → bcrypt verify
  → JWT token generated (user_id, role, exp)
  → Returned to client
  → Stored in browser (localStorage/sessionStorage)
  → Sent as Authorization header on subsequent requests
```

### 2. Biometric Data Flow
```
Employee fingerprint/face
  → ZKTeco device (on-premise, NOT sent to cloud)
  → Device stores user_id + timestamp
  → Agent reads via TCP (rustzk)
  → Sends user_id + timestamp to API (NO biometric data transmitted)
  → API stores as raw_attendance_log
```

**Important**: Raw biometric data (fingerprints, face templates) never leaves the ZKTeco device. Only the event metadata (user ID, timestamp) is transmitted.

### 3. Payroll Data Flow
```
Attendance summaries + salary components
  → Payroll calculation (server-side only)
  → Results stored in payslips table
  → Accessible via admin API
  → Excel export available
```

## Data Storage Locations

| Data | Storage | Encryption | Retention |
|---|---|---|---|
| User credentials | PostgreSQL (`users.password_hash`) | bcrypt hashed | Permanent |
| Employee PII | PostgreSQL (`employees`, `employee_profiles`) | At-rest (host-dependent) | Permanent |
| Attendance logs | PostgreSQL (`raw_attendance_logs`) | At-rest (host-dependent) | Permanent |
| Attendance summaries | PostgreSQL (`attendance_daily_summaries`) | At-rest (host-dependent) | Permanent |
| Payroll data | PostgreSQL (`payslips`, `payslip_segments`) | At-rest (host-dependent) | Permanent |
| JWT tokens | Client-side (browser) | Signed (HS256) | Configurable expiry |
| Agent local cache | SQLite (agent-main) | None | Until sync |
| Redis cache | In-memory | None | Ephemeral |

## External Integrations

| Integration | Protocol | Data Exchanged |
|---|---|---|
| ZKTeco devices | TCP/UDP (rustzk) | Attendance event metadata |
| PostgreSQL | TCP (native protocol) | All application data |
| Redis | TCP | Job queue, presence, cache |
| Browser clients | HTTPS/WSS | API responses, real-time updates |

## Data Boundaries

```
┌─────────────────────────────────────────┐
│          TRUST BOUNDARY                  │
│                                          │
│  ┌────────────────────┐                 │
│  │ API Server         │ ← All business  │
│  │ PostgreSQL         │   logic and data│
│  │ Redis              │   processing    │
│  └────────────────────┘                 │
│                                          │
└──────────┬──────────────────────────────┘
           │
   ┌───────┼───────┐
   ▼                ▼
┌──────┐     ┌──────────┐
│Agent │     │ Browser  │
│(LAN) │     │ (WAN)    │
└──────┘     └──────────┘
```
