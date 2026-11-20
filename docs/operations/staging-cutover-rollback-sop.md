# Staging Deployment, Health Checks & Rollback SOP

**Document ID:** `SOP-DEPLOY-01`
**Runtime:** Go API, React/Vite dashboard, PostgreSQL, Redis, MinIO
**Gateway:** Caddy reverse proxy
**Last Updated:** 2026-11-05

## 1. Runtime topology

```text
Internet → Caddy → Go API (:3000) → PostgreSQL / Redis / MinIO
                    └──────────────→ React/Vite dashboard
```

LeeAttend has one active API runtime: the Go `apiserver`. Retired implementation archives are not deployment targets.

## 2. Staging deployment

1. Validate configuration and Compose files:
   ```bash
   ./scripts/_compose-validate.sh
   ```
2. Start the environment through the orchestrator:
   ```bash
   leedevkit manage up staging
   ```
3. Verify API liveness and readiness:
   ```bash
   curl --fail --silent http://127.0.0.1:3000/api/v1/health
   curl --fail --silent http://127.0.0.1:3000/api/v1/health/ready
   ```
4. Run the complete target suite before traffic release:
   ```bash
   leedevkit test all
   ```

## 3. Release gates

- Compose validation passes.
- API and web lint, unit, integration, and E2E phases pass.
- Health and readiness remain successful for three consecutive probes.
- No unapproved migration, authentication, authorization, PII, or data-integrity warnings exist.
- Rollback owner, previous image, and recovery command are recorded before release.

## 4. Rollback

Trigger rollback for sustained 5xx errors, readiness failure, unacceptable latency, dependency failure, or data-integrity alarms.

1. Stop ingress or the affected service using the deployment platform's standard command.
2. Restore the previous approved Go image/configuration.
3. Start the service and verify `/api/v1/health` and `/api/v1/health/ready`.
4. Confirm PostgreSQL migration state and application error rate.
5. Record operator, timestamp, trigger, restored version, and verification evidence in the deployment incident record.

Do not downgrade PostgreSQL schema automatically. Restore a database backup only under the approved disaster-recovery procedure.

## 5. Failure drills

Run the repository drill suite against a disposable staging environment:

```bash
./scripts/test_failure_drills.sh
```

| Drill | Expected behavior |
|---|---|
| API unavailable | Gateway returns a controlled failure; no partial write occurs. |
| PostgreSQL unavailable | Readiness fails; writes fail safely; no corrupted transaction commits. |
| Redis unavailable | Cache-dependent operations follow the documented fail-closed/degraded policy; no panic or deadlock. |
| Request timeout | Context cancellation reaches downstream work and releases resources. |
| Invalid token | Authentication rejects the request before service/repository access. |
| Worker queue stall | Lease and retry policy prevents duplicate or lost work. |
| Partial dependency failure | Healthy dependencies continue only where the contract permits; dependent writes fail explicitly. |

## 6. Evidence

Capture command output under `logs/`, retain deployment and migration records, and never include raw credentials or PII in evidence.
