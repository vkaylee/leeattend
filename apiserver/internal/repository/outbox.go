package repository

import (
	"context"

	"github.com/google/uuid"
)

// EnqueueRosterCacheInvalidation writes a durable outbox job for roster cache invalidation.
func EnqueueRosterCacheInvalidation(ctx context.Context, db DBTX, workspaceID uuid.UUID) error {
	_, err := db.Exec(ctx, `INSERT INTO outbox_jobs (id, workspace_id, job_type, payload, status, error_message, created_at, updated_at)
		VALUES (gen_random_uuid(), $1, 'RosterCacheInvalidation', '{}'::jsonb, 'pending', NULL, NOW(), NOW())`, workspaceID)
	return err
}
