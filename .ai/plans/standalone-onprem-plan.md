# Standalone On-Prem Migration

Status: Awaiting approval

## Goal
Ship LeeAttend as one-company, LAN/on-prem appliance: fresh install, no SaaS dependency or outbound cloud calls, offline Ed25519-signed license verification.

## Chosen boundary
- **Deployment standalone, not schema rewrite.** Retain one immutable internal `workspace_id` as organization root and preserve workspace-scoped tables/routes internally. This removes SaaS operation without rewriting every business query, API URL, RBAC rule, test, and generated type.
- Bootstrap creates exactly one organization and initial administrator during first run. Disable workspace creation/list/switch, platform tenant administration, tenant DB registration, and public self-registration.
- Appliance runs on a **Linux server container host**. Retain PostgreSQL, Redis, and MinIO as local services; they support existing authentication revocation/rate limiting, distributed jobs, and object storage. Remove dynamic per-workspace pool creation; bind all tenant repositories to one configured business database.
- Ship Dashboard as local SPA behind one HTTPS endpoint. Agents remain separate LAN processes/services and use local API URL plus per-agent credentials.
- No telemetry, activation, update check, SaaS log pull, Supavisor, Cloudflare, or external S3. Network egress is denied in the production Compose profile.
- License is a **perpetual** offline-verifiable Ed25519-signed document. It carries product, edition, customer name, issue date, and feature entitlements. Bundle only verification public key; never add hardware fingerprinting or expiry enforcement.
- Availability is one appliance node with encrypted daily backups and restore rehearsal. HA is out of scope for first release.

## Evidence and constraints
- Go runtime currently initializes system pools and dynamic tenant pools in `apiserver/main.go:42-103`; `internal/db/pool/manager.go:93-363` derives `tenant_<workspace UUID>` names and lazily opens pools.
- API routes and UI are workspace-addressed: `apiserver/main.go:121-157, 179-403`; `webdashboard/src/components/common/workspace-guard.tsx:18-30`. Removing workspace from public contracts would be a system-wide breaking API change.
- Current Compose runs `db_system`, `db_workspace`, two PgBouncer pairs, Redis, MinIO, API, Dashboard, and Agent: `docker-compose.yml:15-306`.
- Platform tenant administration is mounted at `apiserver/main.go:607-623`; this is SaaS-only surface.
- Existing configuration still documents Supavisor and external attendance pull: `.env.example:31-45, 148-185`; production Compose configures `LOG_PULL_*` at `docker-compose.yml:219-227`.
- Existing docs conflict with active Go runtime: `docs/deployment-guide.md` still describes Rust/Python components; `apiserver/README.md:5` identifies Go as active implementation.

## Rules
- **DB-1 / Database Rules §1:** retain authorization and workspace scope until a proven single-organization binding replaces it; no cross-scope query regression.
- **DB-4 / Database Rules §4:** fresh-install migrations require reversible `down.sql`; destructive changes need explicit sign-off.
- **AC-1 / Access Control §1:** every data endpoint remains authenticated and permission-gated. Single organization is not single user.
- **CFG-2 / Configuration Management §2:** fail fast for required secrets, local service URLs, TLS, license path, and bootstrap state; never generate insecure production defaults.
- **ENC-1 / Encryption Rules §1:** TLS remains mandatory on LAN client and agent traffic; database/backup encryption remains required for PII.
- **DG-3 / Data Governance §3:** backup, retention, erasure, and audit policy remain local operational responsibilities.

## Tasks

1. **Freeze standalone contract and ADR** — Add ADR under `.ai/ADRs/` defining singleton-workspace compatibility, local service topology, zero-egress policy, supported OS/runtime, backup ownership, upgrade path, and license trust model. Record all removed SaaS surfaces.  
   **Verify:** architecture review confirms no public API/schema break is needed and approval records accepted trade-off.

2. **Create deterministic first-run bootstrap** — Add an idempotent bootstrap command/migration that creates one organization, initial administrator, default roles/permissions, storage configuration, and immutable installation ID. Reject a second bootstrap and emit no bootstrap secret/PII in logs. Replace self-registration with a setup-only flow that closes after first admin creation.  
   **Verify:** clean database starts once, creates exactly one workspace/admin; repeat invocation fails safely; normal login and RBAC work.

3. **Replace dynamic tenancy with singleton binding** — Introduce one standalone database binding resolved from immutable local configuration/bootstrapped workspace. Refactor `db/pool.Manager`, repositories, workspace middleware, agent/device lookup, dashboard API client, and workspace guard so all requests use this binding and cannot select a different workspace via URL/header/local storage. Keep internal `workspace_id` predicates and schema columns.  
   **Verify:** forged workspace UUID/header/local-storage value cannot access data; all core HR, attendance, leave, payroll, files, agent, and audit flows work under singleton binding.

4. **Remove SaaS control plane and tenant lifecycle** — Unmount/delete platform tenant/db-cluster CRUD, workspace create/list/switch flows, tenant pool cache/eviction, Supavisor registration/configuration, cross-subdomain cookie configuration, and external attendance pull. Remove dead generated OpenAPI paths/types and frontend menus/hooks. Do not leave compatibility aliases.  
   **Verify:** API specification exposes neither system tenant administration nor workspace provisioning; static outbound-dependency allowlist contains only configured LAN services.

5. **Build offline license verification** — Add a small license module with canonical payload parsing, Ed25519 signature verification, schema/version checks, and entitlement evaluation. Load a read-only license file at startup; expose redacted license status locally to admins. Keep private signing key outside repository/build/distribution.  
   **Verify:** valid signature enables entitled features; invalid/tampered/wrong-product license blocks protected use; server performs zero outbound requests during all cases.

6. **Package hardened appliance** — Create standalone Compose/profile plus installer configuration for PostgreSQL, Redis, MinIO, API, Dashboard, TLS reverse proxy, named encrypted volumes, non-default local secrets, health checks, resource ceilings, firewall/egress policy, and per-agent LAN enrollment. Remove public MinIO console ports by default. Pin images by digest; retain SBOM and offline image bundle procedure.  
   **Verify:** install on clean isolated host succeeds without Internet after image bundle import; browser and an agent connect through HTTPS; containers have no public DB/Redis/MinIO exposure and cannot reach Internet.

7. **Deliver local operations** — Add audited backup/restore, license replacement, certificate rotation, admin recovery, agent enrollment/revocation, upgrade, rollback, monitoring, and disaster-recovery runbooks. Backups include Postgres and MinIO, encrypted with a key distinct from application encryption keys; restore rehearsal is mandatory.  
   **Verify:** restore into a blank appliance recreates a usable instance; agent resumes; representative employee, document, attendance, payroll, and audit data reconcile.

8. **Update contracts, documentation, and tests** — Regenerate OpenAPI/client types, update deployment/system architecture/README/config samples/data inventory/CHANGELOG, delete Rust-era and cloud/SaaS instructions, and add scenario tests for bootstrap, singleton enforcement, license failure modes, zero-egress, backup restore, dashboard onboarding, and LAN agent sync.  
   **Verify:** documentation matches command/config surface; `./leedevkit test --all` passes; clean-host smoke test completes.

## Delivery gates
- **Gate A — Design approval:** ADR accepts singleton-workspace strategy, supported OS, TLS/certificate ownership, storage encryption, backup RPO/RTO, and license expiry policy.
- **Gate B — Security review:** two approved reviewers for auth/authorization/license/crypto changes. Confirm no private signing key, SaaS URL, cloud credential, telemetry, or external network route remains in runtime artifacts.
- **Gate C — Release rehearsal:** isolated no-Internet fresh install, bootstrap, Dashboard login, employee creation, device sync, file upload, attendance processing, backup, restore, upgrade, and rollback all pass.

## Decisions recorded
1. **Target host:** Linux server container appliance.
2. **License lifetime:** Perpetual signed entitlement. No expiry enforcement or hardware fingerprinting.
3. **Availability:** Single node with encrypted daily backups and mandatory restore rehearsal.
4. **TLS authority:** Proposed default: customer-managed internal CA/certificate supplied at installation. This remains approval-bound because it defines onboarding and certificate rotation responsibility.
5. **Upgrade delivery:** Proposed default: signed, manually imported offline release bundle. This remains approval-bound; no update check or automatic download.

## Done when
- One clean, air-gapped/on-LAN install runs all required services locally and serves exactly one organization.
- No runtime path can provision/switch/manage multiple tenants or call a SaaS/cloud endpoint.
- RBAC, audit, PII protection, retention, backups, restore, TLS, local agent sync, and offline license verification are operational.
- Clean-install and restore rehearsals pass using production artifacts, not only unit tests.
