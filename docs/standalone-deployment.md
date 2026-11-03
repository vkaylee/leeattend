# LeeAttend Standalone Deployment

Single-company Linux appliance. No SaaS control plane, telemetry, activation call, or outbound cloud dependency.

## Topology

`docker-compose.standalone.yml` runs one PostgreSQL database, Redis, MinIO, Go API, and local React Dashboard. Database, Redis, and MinIO have no host ports. Frontend and API networks are internal; publish only a customer-managed TLS reverse proxy to the LAN.

The database keeps one internal `workspace_id` for compatibility with existing business tables and authorization queries. Standalone mode rejects every other workspace ID and blocks workspace provisioning, tenant administration, public registration, Supavisor, and external attendance log pull.

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
- `STANDALONE_WORKSPACE_ID`, `STANDALONE_WORKSPACE_NAME`
- `STANDALONE_ADMIN_EMAIL`, `STANDALONE_ADMIN_PASSWORD`
- `LICENSE_PUBLIC_KEY`, `LICENSE_FILE`

Secrets MUST come from a protected secret store or root-readable file. Never commit `.env` or the license file.

## Install

1. Import the offline image bundle and verify image digests against the release manifest.
2. Place the license file on encrypted storage with mode `0600`.
3. Configure TLS termination and proxy `/api/` to `apiserver:3000`; do not expose API, database, Redis, or MinIO directly.
4. Apply the reviewed Compose file:

   ```sh
   docker compose -f docker-compose.standalone.yml up -d
   ```

5. Watch `apiserver` logs for successful license verification and bootstrap. Bootstrap creates exactly one organization and first administrator. A second organization or mismatched configured workspace fails closed.
6. Open the Dashboard through HTTPS and rotate the bootstrap administrator password immediately.

## Network policy

Allow LAN clients to the TLS reverse proxy only. Deny container egress at host firewall/network policy. The release smoke test MUST prove that API startup, authentication, attendance processing, and file storage work with Internet access disabled.

## Upgrade and rollback

Use a signed offline release bundle. Before upgrade: run the backup/restore procedure, record image digests, and confirm rollback images are available. Apply one release at a time. Roll back application images only after confirming database migration compatibility; never delete volumes during rollback.

## Operational limits

This release is single-node. High availability, SaaS-to-standalone data migration, automatic update checks, cloud storage, online activation, and external log pull are not supported.
