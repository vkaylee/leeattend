package repository

import (
	"context"
)

// EnqueueRosterCacheInvalidation writes a durable outbox job for roster cache invalidation.
func EnqueueRosterCacheInvalidation(ctx context.Context, db DBTX) error {
	_, err := db.Exec(ctx, `INSERT INTO outbox_jobs (id, job_type, payload, status, error_message, created_at, updated_at)
		VALUES (gen_random_uuid(), 'RosterCacheInvalidation', '{}'::jsonb, 'pending', NULL, NOW(), NOW())`)
	return err
}
