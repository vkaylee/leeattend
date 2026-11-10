-- Fresh-only baseline rollback. Restore backup or recreate disposable database.
BEGIN;
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
COMMIT;
