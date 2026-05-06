# 🐘 Database Rules (Postgres / SQLx)

## 1. Migrations
- **Mandatory Rollbacks:** EVERY migration MUST include a corresponding `down.sql` script for Disaster Recovery. No exceptions.
- **No Destructive Actions:** NEVER `DROP TABLE/COLUMN` in production migrations without user sign-off.

## 2. Structure & Integrity
- **Indexing:** Add indexes for foreign keys and frequently queried columns.
- **Validation:** Enforce DB-level constraints (NOT NULL, UNIQUE, CHECK).

## 3. Execution
- NEVER run `sqlx` directly. 
- ALWAYS use the hermetic toolbox: `./scripts/_sqlx.sh migrate run`
- To update SQLx offline data cache: run `./scripts/_sqlx.sh prepare --workspace` (Wait for test environment to be fully up first).
