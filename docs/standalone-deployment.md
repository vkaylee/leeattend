# LeeAttend Standalone Deployment

Single-company Linux appliance. No SaaS control plane, telemetry, activation call, or outbound cloud dependency.

## Topology

`docker-compose.yml` runs one PostgreSQL database, PgBouncer, Redis, MinIO, Go API, and local React Dashboard. PostgreSQL has no host port; every application and operator connection goes through PgBouncer. Its only host binding is loopback-only for local migration and maintenance tools. Redis and MinIO have no host ports. Frontend and API networks are internal; publish only a customer-managed TLS reverse proxy to the LAN.

System and business tables share one PostgreSQL database. The baseline is `apiserver/migrations/20261103000000_init/`; `apiserver/migrations/20261108000000_single_organization_cutover/` converts it to the final schema with `organization_settings`, `roles`, `user_roles`, and no workspace columns or tables.

The architecture is one company, one database, and one schema. Organization scope is fixed during API bootstrap; it is never selected from URL, header, local storage, or client state. Repositories therefore carry no organization predicate.

## Prerequisites

- Linux host with Docker Compose or Podman Compose.
- Encrypted host storage and a customer-managed internal CA certificate.
- Separate PostgreSQL and PgBouncer TLS directories with `server.crt` and `server.key`; each private key must be readable only by its service.
- Offline image bundle built from reviewed, digest-pinned release artifacts.
- A perpetual LeeAttend license file and matching Ed25519 verification public key.

## Configuration

Copy the release `.env.standalone.example` to a root-owned runtime environment file. Set:

- `DB_NAME`, `DB_USER`, `DB_PASSWORD`
- `POSTGRES_TLS_DIR` containing `server.crt` and `server.key` with restrictive permissions
- `PGBOUNCER_TLS_DIR` containing PgBouncer's `server.crt` and `server.key`
- `PGBOUNCER_IMAGE` pinned by digest and `PGBOUNCER_HOST_PORT` bound to loopback
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
2. Start only PostgreSQL and PgBouncer, then apply migrations over the PgBouncer endpoint before API bootstrap:

   ```sh
   docker compose -f docker-compose.yml up -d db pgbouncer
   docker compose -f docker-compose.yml exec -T db sh -c \
     'PGPASSWORD="$POSTGRES_PASSWORD" psql "host=pgbouncer port=6432 user=$POSTGRES_USER dbname=$POSTGRES_DB sslmode=require" -v ON_ERROR_STOP=1 --single-transaction' \
     < apiserver/migrations/20261103000000_init/up.sql
   ```

   Apply subsequent migrations through the same endpoint in version order. The baseline contains system objects first and business objects second.
3. Place the license file on encrypted storage with mode `0600`.
4. Configure TLS termination and proxy `/api/` to `apiserver:3000`; do not expose API, database, Redis, or MinIO directly.
5. Apply the reviewed Compose file:

   ```sh
   docker compose -f docker-compose.yml up -d
   ```

6. Watch `apiserver` logs for successful license verification and bootstrap. Bootstrap creates exactly one organization and first administrator; a second organization fails closed.
7. Open the Dashboard through HTTPS and rotate the bootstrap administrator password immediately.

## Network policy

Allow LAN clients to the TLS reverse proxy only. Keep the PgBouncer host port loopback-bound; never publish PostgreSQL. Deny container egress at host firewall/network policy. The release smoke test MUST prove that API startup, authentication, attendance processing, and file storage work with Internet access disabled.

## Upgrade and rollback

Use a signed offline release bundle. Before upgrade: run the backup/restore procedure, record image digests, and confirm rollback images are available. Apply one release at a time. Roll back application images only after confirming database migration compatibility; never delete volumes during rollback.

## Operational limits

This release is single-node. High availability, SaaS-to-standalone data migration, automatic update checks, cloud storage, online activation, and external log pull are not supported.
