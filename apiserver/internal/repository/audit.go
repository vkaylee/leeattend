package repository

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
)

// AuditLogWriter stores sanitized audit events in the organization audit log.
type AuditLogWriter struct {
	DB DBTX
}

func (w AuditLogWriter) Emit(ctx context.Context, event audit.Event) error {
	db, err := requireDB(w.DB, "audit log writer")
	if err != nil {
		return err
	}
	actorID, err := uuid.Parse(event.ActorID)
	if err != nil {
		return fmt.Errorf("audit log writer: invalid actor id: %w", err)
	}
	details, err := json.Marshal(event.Details)
	if err != nil {
		return fmt.Errorf("audit log writer: marshal details: %w", err)
	}
	_, err = db.Exec(ctx, `
		INSERT INTO audit_logs (actor_id, action, details, created_at)
		VALUES ($1, $2, $3, COALESCE(NULLIF($4::timestamptz, 'epoch'::timestamptz), CURRENT_TIMESTAMP))
	`, actorID, event.Name, details, event.At)
	return err
}
