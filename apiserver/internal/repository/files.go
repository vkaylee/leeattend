package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type FileRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r FileRepository) db(ctx context.Context, wsID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("file repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, wsID)
}

func (r FileRepository) CreateFile(ctx context.Context, file api.WorkspaceFileRecord) error {
	db, err := r.db(ctx, file.WorkspaceID)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO workspace_files (
		id, workspace_id, file_name, storage_key, file_size, mime_type,
		category, owner_type, owner_id, uploaded_by, is_active, scan_status,
		version, is_latest, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6,
		$7, $8, $9, $10, $11, $12,
		$13, $14, NOW(), NOW()
	)`, file.ID, file.WorkspaceID, file.FileName, file.StorageKey, file.FileSize, file.MimeType,
		file.Category, file.OwnerType, file.OwnerID, file.UploadedBy, file.IsActive, file.ScanStatus,
		file.Version, file.IsLatest)
	return err
}

func (r FileRepository) GetFile(ctx context.Context, wsID, fileID uuid.UUID) (api.WorkspaceFileRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.WorkspaceFileRecord{}, err
	}
	var f api.WorkspaceFileRecord
	var created, updated time.Time
	err = db.QueryRow(ctx, `SELECT
		id, workspace_id, file_name, storage_key, file_size, mime_type,
		category, owner_type, owner_id, uploaded_by, is_active, scan_status,
		version, is_latest, created_at, updated_at
	FROM workspace_files
	WHERE workspace_id = $1 AND id = $2 AND is_active = true`, wsID, fileID).Scan(
		&f.ID, &f.WorkspaceID, &f.FileName, &f.StorageKey, &f.FileSize, &f.MimeType,
		&f.Category, &f.OwnerType, &f.OwnerID, &f.UploadedBy, &f.IsActive, &f.ScanStatus,
		&f.Version, &f.IsLatest, &created, &updated)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.WorkspaceFileRecord{}, api.ErrFileNotFound
		}
		return api.WorkspaceFileRecord{}, err
	}
	f.CreatedAt = created.UTC().Format(time.RFC3339)
	f.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return f, nil
}

func (r FileRepository) DeleteFile(ctx context.Context, wsID, fileID uuid.UUID) error {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE workspace_files SET is_active = false, updated_at = NOW() WHERE workspace_id = $1 AND id = $2 AND is_active = true`, wsID, fileID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrFileNotFound
	}
	return nil
}

func (r FileRepository) CreateSignatureRequest(ctx context.Context, req api.SignatureRequestRecord) error {
	db, err := r.db(ctx, req.WorkspaceID)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO signature_requests (
		id, workspace_id, document_id, signer_id, status, created_at, updated_at
	) VALUES ($1, $2, $3, $4, $5, NOW(), NOW())`, req.ID, req.WorkspaceID, req.DocumentID, req.SignerID, req.Status)
	return err
}

func (r FileRepository) ListPendingSignatures(ctx context.Context, wsID, signerID uuid.UUID) ([]api.SignatureRequestRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, workspace_id, document_id, signer_id, status, signed_at, created_at, updated_at
	FROM signature_requests
	WHERE workspace_id = $1 AND signer_id = $2 AND status = 'pending'
	ORDER BY created_at DESC`, wsID, signerID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.SignatureRequestRecord, 0)
	for rows.Next() {
		var s api.SignatureRequestRecord
		var signed *time.Time
		var created, updated time.Time
		if err := rows.Scan(&s.ID, &s.WorkspaceID, &s.DocumentID, &s.SignerID, &s.Status, &signed, &created, &updated); err != nil {
			return nil, err
		}
		if signed != nil {
			formatted := signed.UTC().Format(time.RFC3339)
			s.SignedAt = &formatted
		}
		s.CreatedAt = created.UTC().Format(time.RFC3339)
		s.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, s)
	}
	return result, rows.Err()
}

func (r FileRepository) SignDocument(ctx context.Context, wsID, requestID uuid.UUID) error {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE signature_requests SET status = 'signed', signed_at = NOW(), updated_at = NOW() WHERE workspace_id = $1 AND id = $2 AND status = 'pending'`, wsID, requestID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrFileNotFound
	}
	return nil
}
