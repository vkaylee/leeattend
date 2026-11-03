# LeeAttend Standalone Deployment

Single-company Linux appliance. No SaaS control plane, telemetry, activation call, or outbound cloud dependency.

## Topology

`docker-compose.yml` runs one PostgreSQL database, Redis, MinIO, Go API, and local React Dashboard. Database, Redis, and MinIO have no host ports. Frontend and API networks are internal; publish only a customer-managed TLS reverse proxy to the LAN.

System and business tables share one PostgreSQL database. The Phase 1 baseline is `apiserver/migrations/20261103000000_init/`; it applies both schema groups in one ordered migration. Existing workspace-compatible columns remain temporarily while later phases remove workspace data contracts.

The target architecture is one company, one database, and one schema. Backend core no longer accepts workspace identity configuration or routes pools by workspace; repository and API compatibility fields remain temporarily for later phases.

## Prerequisites

- Linux host with Docker Compose or Podman Compose.
- Encrypted host storage and a customer-managed internal CA certificate.
- Offline image bundle built from reviewed, digest-pinned release artifacts.
- A perpetual LeeAttend license file and matching Ed25519 verification public key.

## Configuration

Copy the release `.env.standalone.example` to a root-owned runtime environment file. Set:

- `DB_NAME`, `DB_USER`, `DB_PASSWORD`
- `POSTGRES_TLS_DIR` containing `server.crt` and `server.key` with restrictive permissions
- `REDIS_PASSWORD`
- `MINIO_ROOT_USER`, `MINIO_ROOT_PASSWORD`
- `JWT_SECRET`, `ENCRYPTION_KEYS_JSON`
- `ALLOWED_ORIGINS` to the HTTPS Dashboard origin
- `ORGANIZATION_NAME`
- `STANDALONE_ADMIN_EMAIL`, `STANDALONE_ADMIN_PASSWORD`
- `LICENSE_PUBLIC_KEY`, `LICENSE_FILE`

Secrets MUST come from a protected secret store or root-readable file. Never commit `.env` or the license file.

## Install

1. Import the offline image bundle and verify image digests against the release manifest.
2. Provision the release-pre-migrated PostgreSQL data volume and apply `apiserver/migrations/20261103000000_init/` to the single database before API bootstrap. The migration contains system objects first and business objects second.
3. Place the license file on encrypted storage with mode `0600`.
4. Configure TLS termination and proxy `/api/` to `apiserver:3000`; do not expose API, database, Redis, or MinIO directly.
5. Apply the reviewed Compose file:

   ```sh
   docker compose -f docker-compose.yml up -d
   ```

6. Watch `apiserver` logs for successful license verification and bootstrap. Bootstrap creates exactly one organization and first administrator; a second organization fails closed.
7. Open the Dashboard through HTTPS and rotate the bootstrap administrator password immediately.

## Network policy

Allow LAN clients to the TLS reverse proxy only. Deny container egress at host firewall/network policy. The release smoke test MUST prove that API startup, authentication, attendance processing, and file storage work with Internet access disabled.

## Upgrade and rollback

Use a signed offline release bundle. Before upgrade: run the backup/restore procedure, record image digests, and confirm rollback images are available. Apply one release at a time. Roll back application images only after confirming database migration compatibility; never delete volumes during rollback.

## Operational limits

This release is single-node. High availability, SaaS-to-standalone data migration, automatic update checks, cloud storage, online activation, and external log pull are not supported.
