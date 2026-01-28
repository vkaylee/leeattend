# LeeAttend Product Requirements Document (PRD)

**Version**: 1.0
**Last Updated**: 2026-01-28
**Status**: Active

---

## Executive Summary

LeeAttend is an enterprise-grade, multi-tenant SaaS attendance and payroll management system designed for organizations with distributed workforce and multiple biometric device brands. The system automates the complete pipeline from attendance collection through payroll calculation, eliminating manual processes and reducing payroll processing time from days to minutes.

**Core Value Proposition**: Automated attendance-to-payroll pipeline with offline-first edge architecture, supporting complex shift patterns, hierarchical leave policies, and regulatory compliance across multiple organizational sites.

---

## Product Vision

Enable organizations to:
- Collect attendance automatically from biometric devices across multiple locations
- Process complex payroll with exact prorating for mid-month changes
- Manage leave policies flexibly across workspace, department, and employee levels
- Empower employees with self-service access to attendance and payroll data
- Maintain complete audit trails for regulatory compliance

---

## Business Goals

| Goal ID | Objective | Target | Priority |
|---------|-----------|--------|----------|
| G1 | Automate attendance-to-payroll pipeline | 99.9% calculation accuracy | Critical |
| G2 | Enable multi-site operations | Support 100+ concurrent agents | Critical |
| G3 | Support complex HR policies | 3-tier policy hierarchy | Critical |
| G4 | Provide multi-tenant SaaS platform | Scalable to 1000+ organizations | High |
| G5 | Empower employees with self-service | 60%+ ESS adoption within 3 months | High |

---

## Target Market & Stakeholders

### Primary User Segments

| Segment | Role | Key Needs | Size |
|---------|------|-----------|------|
| Workspace Owner | Organization administrator | Billing, compliance, configuration | 1 per tenant |
| HR Managers | Employee/policy management | Leave approval, attendance adjustments | 2-5 per org |
| Payroll Specialists | Payroll calculation | Accurate calculations, audit trails | 1-2 per org |
| Department Heads | Team scheduling | Shift assignments, attendance monitoring | 2-10 per org |
| Employees | Self-service | View attendance, request leave | 10-1000 per org |
| System Administrators | Infrastructure | Agent registration, device management | 1-2 per org |

### Stakeholder Engagement

- **Primary**: HR and Payroll teams (direct product users)
- **Secondary**: Finance (payroll accuracy), Legal (compliance)
- **Tertiary**: IT/Operations (system deployment and monitoring)

---

## Product Scope

### In Scope (Current & Planned)

- Multi-tenant workspace management with RBAC
- Biometric device integration (ZKTeco, with architecture for Hikvision/Suprema)
- Offline-first edge agent architecture
- Attendance collection, pairing, and correction workflows
- Flexible shift assignment and scheduling
- Multi-tier leave management with approval workflows
- Multi-segment payroll calculation with prorating
- Payroll incentives and bonuses
- Employee self-service portal
- Admin dashboard with real-time monitoring
- Comprehensive reporting

### Out of Scope (Future)

- Mobile app (Phase 3)
- Facial recognition (Phase 3)
- Third-party HRIS integrations (Phase 3)
- Geofencing for clock-in restrictions (Phase 3)
- Advanced BI/analytics tools (Phase 4+)
- Government filing automation (Phase 4+)

---

## Key Features Overview

### Attendance Management (95+ Features)

| Category | Features | Status |
|----------|----------|--------|
| Collection | Device polling, manual entry, bulk import, duplicate detection | Complete |
| Processing | Pairing algorithms (Anchor-First, FSM), overnight shift handling | Complete |
| Corrections | Adjustment requests, approval workflow, manual overrides | Complete |
| Monitoring | Real-time device status, sync latency tracking | Complete |

### Scheduling & Assignment

- Shift templates with break durations
- 7-day working patterns (recurring)
- Direct shift roster (date-specific overrides)
- Assignment rules with tag-based automation
- Bulk assignment execution
- Trace hints for transparency

### Leave Management

- Multiple leave types (vacation, sick, maternity, etc.)
- 3-tier policy hierarchy (workspace → department → employee)
- Leave balance tracking and accrual
- Request workflow with approval
- Overlap validation
- Paid vs unpaid classification

### Payroll System

- Multi-segment calculation for mid-month changes
- Change point detection (salary, transfer, policy)
- Exact prorating for partial months
- Point-in-time policy lookup
- Overtime calculation and rate management
- Payroll incentive policies with tier-based eligibility
- Payslip generation and history

### Employee Self-Service (ESS)

- Identity-linked employee portal
- Profile and contact management
- Attendance history viewing
- Leave balance display and request submission
- Payslip access
- Attendance correction requests
- Self-clock interface (Phase 1 - future)

### Admin Dashboard

- System overview with HR and system tabs
- Agent and device status monitoring
- Employee, department, shift management
- Leave and attendance adjustment approvals
- Payroll calculation and review
- Bulk import capabilities
- Configurable settings

---

## Success Metrics & KPIs

### Operational Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Agent uptime | 99%+ | 30-day rolling window |
| Log sync latency | <10 seconds | P95 measurement |
| Payroll calculation time | <2 minutes per 100 employees | Full calculation |
| Pairing accuracy | 95%+ | Manual corrections / total entries |

### Business Metrics

| Metric | Target | Frequency |
|--------|--------|-----------|
| Active workspaces | 100+ within Year 1 | Monthly tracking |
| Average employees per workspace | 50-200 | Per workspace |
| Daily log volume | 30,000+ logs per agent | Peak capacity |
| Leave request approval time | <24 hours (90th %ile) | Weekly reporting |

### User Metrics

| Metric | Target | Timeframe |
|--------|--------|-----------|
| ESS adoption rate | 60%+ of employees | 3 months post-onboarding |
| Leave requests via ESS | 70% of total | Ongoing measurement |
| Dashboard active users | 70%+ of admins | Monthly |
| Support tickets | <2 per 100 employees/month | Ongoing |

---

## Product Roadmap

### Phase 1 (COMPLETE - as of 2026-01-28)

**Duration**: Initial development complete

**Deliverables**:
- Core attendance collection and pairing
- Basic payroll calculation
- Leave management with approval workflow
- Admin dashboard (core features)
- ESS portal (view-only access)
- Edge agent with offline persistence

**Status**: Production ready with 817+ test files

### Phase 2 (CURRENT - Q2-Q3 2026)

**Duration**: 3-4 months

**Planned Features**:
- Granular salary components management
- Payroll incentive policies and tier matching
- Overtime policy engine with flexible rules
- Working condition overlays (parental leave, etc.)
- Assignment rule automation (tag-based targeting)
- Attendance adjustment workflow
- Advanced device brand support (Hikvision, Suprema)
- Enhanced reporting with export options

**Success Criteria**:
- All features tested with integration tests
- Zero regression in Phase 1 features
- Performance benchmarks met (<2min payroll calc)

### Phase 3 (FUTURE - Q4 2026+)

**Planned Features**:
- Self-clock interface (mobile-friendly punch in/out)
- Mobile app (React Native or Flutter)
- Geofencing for location-based clock-in
- Advanced reporting and BI integration
- Webhook support for third-party integrations
- HRIS data connectors
- Performance analytics dashboard

**Success Criteria**:
- Mobile app available on iOS and Android
- ESS adoption reaches 80%+
- BI/Analytics queries <500ms

---

## Competitive Differentiators

| Differentiator | Description | Competitive Edge |
|---|---|---|
| **Offline-First Architecture** | Edge agents work during network outages with local SQLite cache | Prevents data loss, unlike cloud-only competitors |
| **Multi-Brand Device Support** | Sidecar pattern allows quick addition of new device brands | Not locked to single vendor (e.g., ZKTeco-only) |
| **Exact Payroll Prorating** | Segment-based calculation for mid-month changes | Competitors use rough monthly averages |
| **Transparent Shift Assignment** | Trace hints show why employee got specific shift | Reduces HR support burden |
| **Flexible Policy Hierarchy** | 3-tier configuration with point-in-time lookup | Competitors have flat, global-only policies |
| **Employee Self-Service** | Reduces HR workload by 40-60% vs manual processes | Competitors charge extra for ESS module |
| **Open API & Extensibility** | REST API designed for third-party integrations | Competitors have closed ecosystems |

---

## Risk Assessment & Mitigation

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| Database bottleneck at 1000+ agents | High | Medium | Connection pooling, horizontal scaling, load balancing |
| Multi-tenant data leakage | Critical | Low | Query-level isolation, integration tests, pentesting |
| Pairing algorithm accuracy | High | Medium | Multiple algorithms, manual override, audit trails |
| Agent network reliability | Medium | Medium | Local cache, retry logic, alerting |

### Business Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| Regulatory compliance (country-specific) | High | Medium | Configurable policies, legal review, country modules |
| User adoption resistance | Medium | Medium | Training, intuitive UI, progressive rollout |
| Customer churn (poor payroll accuracy) | Critical | Low | 99.9% accuracy target, audit trails, support |
| Pricing model mismatch | High | Low | Market research, flexible pricing tiers |

---

## Requirements Summary

### Functional Requirements (FR)

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-01 | Multi-tenant isolation at database level | P0 | Complete |
| FR-02 | Biometric device log collection and sync | P0 | Complete |
| FR-03 | Attendance pairing engine with 2 algorithms | P0 | Complete |
| FR-04 | Shift assignment and roster management | P0 | Complete |
| FR-05 | Leave request workflow with approval | P0 | Complete |
| FR-06 | Multi-segment payroll calculation | P0 | Complete |
| FR-07 | Employee self-service portal | P1 | Complete |
| FR-08 | Admin dashboard with real-time monitoring | P0 | Complete |
| FR-09 | Payroll incentive policies | P1 | In Progress |
| FR-10 | Assignment rule automation | P1 | In Progress |

### Non-Functional Requirements (NFR)

| ID | Requirement | Target | Priority |
|----|-------------|--------|----------|
| NFR-01 | API response time | <500ms (P95) | P0 |
| NFR-02 | Agent uptime | 99%+ | P0 |
| NFR-03 | Data integrity (no loss) | 100% | P0 |
| NFR-04 | Multi-language support | English, Vietnamese, Chinese | P1 |
| NFR-05 | Audit trail completeness | 100% of sensitive ops | P0 |
| NFR-06 | Security (multi-tenant isolation) | Zero leaks | P0 |
| NFR-07 | Scalability (concurrent agents) | 100+ agents | P0 |

---

## Market Positioning

### Ideal Customer Profile (ICP)

- **Organization Size**: 10-1000 employees
- **Geography**: Vietnam, Southeast Asia, China (primary)
- **Industry**: Manufacturing, logistics, retail, services
- **Problem**: Fragmented attendance collection, manual payroll, no audit trails
- **Budget**: SaaS subscription (per-employee-per-month model)

### Competitive Landscape

**Direct Competitors**:
- ADP, Paychex (cloud-only, expensive)
- SAP SuccessFactors (enterprise, complex)
- Local point-of-sale vendors (poor automation)

**Competitive Advantage**: Offline-first + exact prorating + flexible policies

---

## Unresolved Questions

1. **Pricing Strategy**: Per-employee SaaS, one-time license, or hybrid model?
2. **Geographic Compliance**: Which countries require data residency (not just localization)?
3. **Mobile Strategy**: React Native, Flutter, or web-only PWA?
4. **Integration Priority**: Which HRIS/accounting systems are highest value?
5. **SLA Tiers**: Different uptime/support tiers for different customer segments?
6. **Data Retention**: How long to keep raw logs, payslips (compliance requirements)?
7. **Backup/DR**: Recovery time objective (RTO) and recovery point objective (RPO)?
8. **Go-to-Market**: Direct sales, channel partners, or self-serve SaaS?

---

## Approval & Sign-Off

| Role | Name | Date | Status |
|------|------|------|--------|
| Product Manager | TBD | - | Pending |
| Technical Lead | TBD | - | Pending |
| Business Owner | TBD | - | Pending |

---

**Next Review Date**: Q2 2026
**Last Updated**: 2026-01-28 12:08 UTC
