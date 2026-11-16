# LeeAttend — Product Requirements Document (PRD)

## Product Vision

LeeAttend is an enterprise-grade **Attendance Management Platform** that bridges the gap between physical biometric devices (ZKTeco) and modern workforce management needs. It automates the entire attendance lifecycle from device data collection to payroll-ready reports.

## Target Users

| User Type | Description | Key Needs |
|---|---|---|
| **HR Admin** | Human resources managers | Configure shifts, holidays, leave policies; generate payroll |
| **Department Manager** | Team supervisors | Approve OT claims, review attendance, manage schedules |
| **Employee** | Office/factory staff | View attendance, submit leave requests, check payslips |
| **IT Administrator** | System administrator | Manage agents, devices, workspace settings |
| **Business Owner** | Company leadership | Dashboard overviews, compliance reports |

## Problem Statement

Vietnamese and Southeast Asian enterprises using ZKTeco biometric devices face:
1. **Manual data extraction** — No automated device-to-cloud sync
2. **Complex shift schedules** — Rotating patterns, overnight shifts, multiple break policies
3. **Holiday compliance** — Labor law OT multipliers (300% for national holidays)
4. **Error-prone payroll** — Manual attendance → payroll translation causes salary disputes
5. **No self-service** — Employees cannot check their own attendance or submit requests

## Product Goals

| Priority | Goal | Success Metric |
|---|---|---|
| P0 | Automated attendance collection from ZKTeco devices | Zero manual data entry |
| P0 | Accurate payroll calculation with OT and holiday multipliers | < 0.1% calculation error rate |
| P1 | Holiday-aware scheduling and reporting | All statuses correctly reflect holidays |
| P1 | Employee Self-Service portal | 80% employee adoption |
| P2 | Multi-workspace SaaS operation | Support 50+ independent organizations |

## Non-Functional Requirements

| Requirement | Target |
|---|---|
| Availability | 99.5% uptime |
| Latency | < 500ms API response |
| Data Integrity | Go compiler, typed SQL, and database constraints |
| Security | JWT authentication, role-based access |
| Internationalization | English + Vietnamese |
| Testing | Automated CI via Docker containers |
| Scalability | Multi-workspace data isolation |

## Constraints

- Must work on **WSL2** (primary development environment)
- All tests must run inside **Docker containers**
- ZKTeco device protocol via the approved attendance connector (TCP/UDP, proprietary)
- PostgreSQL as the only supported RDBMS
- Database integration tests require a live PostgreSQL service

## Stakeholders

| Stakeholder | Role | Involvement |
|---|---|---|
| Product Owner | Feature prioritization, business rules | High |
| HR Department | Domain expertise, validation | Medium |
| Employees | End users, feedback | Low |
| IT Operations | Deployment, infrastructure | Medium |
