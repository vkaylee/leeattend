# LeeAttend — Security Assessment

## Executive Summary

LeeAttend implements a multi-tier security architecture with JWT authentication, role-based access control, and workspace-level data isolation. The following assessment covers authentication, authorization, data protection, and infrastructure security.

## Authentication

### Current Implementation
| Control | Status | Details |
|---|---|---|
| Password hashing | ✅ Secure | `bcrypt` (cost factor default) |
| JWT tokens | ✅ Implemented | `jsonwebtoken` crate with configurable secret |
| Token refresh | ✅ Implemented | Separate refresh token flow |
| Dev mode bypass | ⚠️ Caution | Debug auth bypass enabled in dev mode with clear warning log |

### Findings

- **SEC-001** (Low): Dev mode auth bypass exists but is properly gated behind `DEV_MODE` flag with runtime warning
- **SEC-002** (Medium): JWT secret is configured via `JWT_SECRET` environment variable — ensure strong secrets in production

## Authorization

### Role-Based Access Control
| Role | Capabilities |
|---|---|
| `Owner` | Full workspace control, member management |
| `Admin` | Employee, shift, holiday, payroll management |
| `Member` | Self-service only (ESS endpoints) |

### Implementation
- `MemberContext` middleware extracts role from JWT and workspace membership
- `is_admin_or_owner()` guard on sensitive endpoints
- All queries scoped by `workspace_id` for multi-tenant isolation

### Findings

- **SEC-003** (Low): Authorization is primarily enforced at handler level — consider middleware-level route guards for defense in depth

## Input Validation

| Area | Validation |
|---|---|
| Request body | Go JSON decoding with explicit validation |
| SQL queries | Parameterized `pgx` queries and repository validation |
| Path parameters | UUID parsing at handler boundary |
| Date inputs | Explicit Go time parsing and domain validation |

### Findings

- **SEC-004** (Low): No explicit rate limiting on authentication endpoints
- **SEC-005** (Info): Parameterized repository queries reduce injection risk; continue boundary and integration testing

## Infrastructure Security

| Control | Status |
|---|---|
| HTTPS | ⚠️ Depends on deployment (Nginx/reverse proxy) |
| CORS | ✅ Configured in Go HTTP middleware |
| Database connections | ✅ Connection pool via `pgxpool` |
| Redis | ⚠️ Connection without TLS in dev mode |
| Docker isolation | ✅ All services containerized |
| Secret management | ⚠️ Environment variables (`.env` file) |

## Dependency Security

### Go Dependencies
- Dependencies are pinned in `apiserver/go.mod` and `apiserver/go.sum`.
- CI runs formatting, vet, static analysis, vulnerability scanning, and module verification.

### Frontend Dependencies
- React 19 (latest stable)
- No known critical vulnerabilities in current dependency versions
- `@tanstack/react-query` for safe async data fetching

## OWASP Top 10 Assessment

| Category | Risk Level | Status |
|---|---|---|
| A01: Broken Access Control | Low | RBAC implemented, workspace isolation |
| A02: Cryptographic Failures | Low | bcrypt for passwords, JWT for tokens |
| A03: Injection | Low | Parameterized SQL and input validation |
| A04: Insecure Design | Low | Layered architecture, input validation |
| A05: Security Misconfiguration | Medium | Dev mode bypass needs production hardening |
| A06: Vulnerable Components | Low | Up-to-date dependencies |
| A07: Auth Failures | Low | JWT with refresh, bcrypt hashing |
| A08: Data Integrity | Low | Typed Go repositories, transactions, and database constraints |
| A09: Logging & Monitoring | Medium | Basic tracing, needs audit logging |
| A10: SSRF | Low | No user-controlled URL fetching |
