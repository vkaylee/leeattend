# LeeAttend Source of Truth

Use one canonical document per concern. If docs conflict with code, code and migrations win; update docs in same change.

| Concern | Canonical location |
|---|---|
| Runtime architecture | [`architecture/system-architecture.md`](architecture/system-architecture.md) |
| Repository/component map | [`architecture/codebase-summary.md`](architecture/codebase-summary.md) |
| Database schema | [`architecture/database-architecture.md`](architecture/database-architecture.md) and `apiserver/migrations/` |
| HTTP/WebSocket API | [`api/api-reference.md`](api/api-reference.md) and generated API types |
| Deployment | [`operations/deployment-guide.md`](operations/deployment-guide.md), [`operations/standalone-deployment.md`](operations/standalone-deployment.md) |
| Backup/restore | [`operations/standalone-backup-restore.md`](operations/standalone-backup-restore.md) |
| Worker/reliability | [`operations/background-worker-runner.md`](operations/background-worker-runner.md), [`operations/infra-job-reliability-runbook.md`](operations/infra-job-reliability-runbook.md) |
| Coding/testing/UI rules | [`development/`](development/) plus `.agent/rules/` |
| Product/domain | [`business/`](business/) and [`domain/`](domain/) |
| Compliance/security audit | [`audit/`](audit/) |
| Durable architecture decisions | [`decisions/`](decisions/) |
| Active implementation plans | [`plans/`](plans/) |
| Completed or superseded material | [`archive/`](archive/) |

Root entrypoints stay limited to `README.md`, `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, and `CHANGELOG.md`.
