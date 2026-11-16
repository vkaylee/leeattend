# Standalone Backup and Restore Runbook

Scope: one LeeAttend Linux appliance with PostgreSQL, MinIO, Redis, API, and Dashboard.

## Backup

Run during a maintenance window or from a consistent snapshot-capable host. Store the output on encrypted media separate from the appliance. The `pg_dump` and `pg_restore` clients connect to PostgreSQL only through PgBouncer.

```sh
mkdir -p backups/2026-11-03

docker compose -f docker-compose.yml exec -T db sh -c \
  'PGPASSWORD="$POSTGRES_PASSWORD" pg_dump -h pgbouncer -p 6432 -U "$POSTGRES_USER" -d "$POSTGRES_DB" --format=custom --no-owner --no-acl' \
  > backups/2026-11-03/leeattend.dump

docker run --rm \
  -v leeattend_standalone_files:/source:ro \
  -v "$PWD/backups/2026-11-03":/backup \
  alpine tar czf /backup/minio-files.tgz -C /source .
sha256sum backups/2026-11-03/* > backups/2026-11-03/SHA256SUMS
```

The exact Compose project volume prefix can differ. Confirm it with `docker volume ls` before copying MinIO data. Encrypt the backup directory with the customer backup key; never reuse `JWT_SECRET`, `ENCRYPTION_KEYS_JSON`, or MinIO credentials.

Daily encrypted backups are the minimum production schedule. Retain copies according to the customer retention policy and test one restore every release.

## Restore rehearsal

1. Provision a blank Linux host with the same release bundle.
2. Start only PostgreSQL and MinIO.
3. Restore the database:

   ```sh
   cat backups/2026-11-03/leeattend.dump | \
     docker compose -f docker-compose.yml exec -T db sh -c \
     'PGPASSWORD="$POSTGRES_PASSWORD" pg_restore -h pgbouncer -p 6432 -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists --no-owner --no-acl'
   ```

4. Restore object storage into the standalone MinIO volume. Preserve ownership and permissions.
5. Verify checksums before starting API and Dashboard.
6. Start the complete stack and confirm:
   - API readiness and license verification.
   - Administrator login and RBAC.
   - One employee detail view.
   - One uploaded document download.
   - Attendance log and summary.
   - Audit log continuity.
7. Record restore duration, data timestamp, missing rows/files, and operator. A restore that cannot complete without Internet is a release failure.

## Disaster recovery

If the appliance is lost, do not reuse the old host secrets without incident review. Install the signed bundle on a clean host, restore the encrypted backup, install the same perpetual license, rotate database/Redis/MinIO/JWT secrets, rotate TLS certificates, and revoke all sessions before reopening LAN access.

## Rollback

Stop the stack, preserve current logs and volumes, restore the previous application image digest, and restore the last known-good database backup if the migration is not backward-compatible. Never run a destructive migration rollback against the only copy of production data.
