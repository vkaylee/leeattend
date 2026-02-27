# LeeAttend — Remediation Recommendations

## Priority Matrix

| Priority | ID | Finding | Effort | Impact |
|---|---|---|---|---|
| 🔴 High | REC-001 | Add audit logging | Medium | High |
| 🔴 High | REC-002 | Implement rate limiting on auth endpoints | Low | High |
| 🟡 Medium | REC-003 | Add data retention policy | Medium | Medium |
| 🟡 Medium | REC-004 | Enforce OT daily/monthly limits | Medium | Medium |
| 🟡 Medium | REC-005 | Add HTTPS enforcement | Low | Medium |
| 🟢 Low | REC-006 | Add Redis TLS | Low | Low |
| 🟢 Low | REC-007 | Improve secret management | Medium | Medium |
| 🟢 Low | REC-008 | Add employee data deletion workflow | Medium | Low |

---

## REC-001: Audit Logging (High Priority)

**Finding**: No audit trail for user actions (SEC-003, CMP-002)

**Recommendation**:
1. Add an `audit_logs` table:
   ```sql
   CREATE TABLE audit_logs (
     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
     workspace_id UUID NOT NULL,
     user_id UUID NOT NULL,
     action VARCHAR(100) NOT NULL,
     entity_type VARCHAR(50) NOT NULL,
     entity_id UUID,
     details JSONB,
     ip_address VARCHAR(45),
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );
   ```
2. Add middleware to capture audit events on create/update/delete operations
3. Priority: Holiday changes, payroll calculations, leave approvals, employee changes

**Effort**: 2-3 days | **Risk reduction**: High

---

## REC-002: Rate Limiting (High Priority)

**Finding**: No rate limiting on authentication endpoints (SEC-004)

**Recommendation**:
1. Add `tower::limit::RateLimit` middleware on `/auth/login` and `/auth/register`
2. Limits: 5 login attempts per minute per IP, 10 registrations per hour per IP
3. Use Redis for distributed rate limiting across instances

**Effort**: 1 day | **Risk reduction**: High (prevents brute-force attacks)

---

## REC-003: Data Retention Policy (Medium Priority)

**Finding**: No automated data cleanup (CMP-001)

**Recommendation**:
1. Add workspace-level retention settings:
   - `raw_log_retention_days` (default: 365)
   - `attendance_summary_retention_years` (default: 5)
   - `payslip_retention_years` (default: 7)
2. Create a scheduled background job for data cleanup
3. Archive before deletion (export to compressed format)

**Effort**: 3-4 days | **Risk reduction**: Medium

---

## REC-004: OT Limit Enforcement (Medium Priority)

**Finding**: OT hours tracked but not enforced against legal limits (CMP-003)

**Recommendation**:
1. Add workspace-level OT limit settings:
   - `max_daily_ot_hours` (Vietnamese law: 4 hours/day)
   - `max_monthly_ot_hours` (Vietnamese law: 40 hours/month)
2. Add validation warnings when limits are exceeded
3. Generate compliance reports for HR review

**Effort**: 2-3 days | **Risk reduction**: Medium (legal compliance)

---

## REC-005: HTTPS Enforcement (Medium Priority)

**Finding**: HTTPS depends on deployment configuration

**Recommendation**:
1. Add HSTS headers via Nginx configuration
2. Configure TLS termination at reverse proxy level
3. Add `Strict-Transport-Security` header in production
4. Ensure all API responses include security headers (X-Content-Type-Options, X-Frame-Options)

**Effort**: 0.5 days | **Risk reduction**: Medium

---

## REC-006: Redis TLS (Low Priority)

**Finding**: Redis connection without TLS in dev mode

**Recommendation**:
1. Enable TLS for Redis in production deployments
2. Use `rediss://` scheme with certificate verification
3. Dev mode can remain unencrypted for simplicity

**Effort**: 0.5 days | **Risk reduction**: Low

---

## REC-007: Secret Management (Low Priority)

**Finding**: Secrets stored in `.env` file (SEC-002)

**Recommendation**:
1. Migrate to a secrets manager (e.g., HashiCorp Vault, AWS Secrets Manager, or Docker Secrets)
2. Remove secrets from `.env` file in production
3. Rotate JWT secret periodically

**Effort**: 2-3 days | **Risk reduction**: Medium

---

## REC-008: Employee Data Deletion (Low Priority)

**Finding**: No self-service data deletion (CMP-004)

**Recommendation**:
1. Add admin endpoint: `DELETE /workspaces/:ws_id/employees/:id/data`
2. Implement cascading soft-delete with configurable grace period
3. Generate data deletion report for compliance records

**Effort**: 2-3 days | **Risk reduction**: Low

---

## Implementation Roadmap

### Phase 1 (Immediate — 1 week)
- REC-002: Rate limiting on auth endpoints
- REC-005: HTTPS enforcement headers

### Phase 2 (Short-term — 2 weeks)
- REC-001: Audit logging foundation
- REC-004: OT limit warnings

### Phase 3 (Medium-term — 1 month)
- REC-003: Data retention policy
- REC-007: Secret management migration
- REC-006: Redis TLS

### Phase 4 (Long-term)
- REC-008: Employee data deletion workflow
