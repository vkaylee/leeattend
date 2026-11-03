package repository

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// AttendanceSourceRepository reads workspace-scoped external attendance sources.
type AttendanceSourceRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r AttendanceSourceRepository) ListAttendanceSources(ctx context.Context, workspaceID uuid.UUID) ([]service.AttendanceSource, error) {
	if r.TenantDB == nil {
		return nil, errors.New("attendance source repository: tenant database is not configured")
	}
	db, err := r.TenantDB(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, provider_source_id FROM attendance_sources WHERE workspace_id = $1 AND is_enabled = TRUE ORDER BY created_at, id`, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	sources := make([]service.AttendanceSource, 0)
	for rows.Next() {
		var source service.AttendanceSource
		if err := rows.Scan(&source.ID, &source.ProviderID); err != nil {
			return nil, err
		}
		sources = append(sources, source)
	}
	return sources, rows.Err()
}
