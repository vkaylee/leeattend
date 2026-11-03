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
	DB DBTX
}

func (r FileRepository) db(ctx context.Context, wsID uuid.UUID) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("file repository: tenant database is not configured")
	}
	return r.DB, nil
}

func (r FileRepository) CreateFile(ctx context.Context, file api.WorkspaceFileRecord) error {
	db, err := r.db(ctx, file.WorkspaceID)
	if err != nil {
		return err
	}
	var expiresAt *time.Time
	if file.ExpiresAt != nil {
		if t, err := time.Parse(time.RFC3339, *file.ExpiresAt); err == nil {
			expiresAt = &t
		}
	}
	_, err = db.Exec(ctx, `INSERT INTO workspace_files (
		id, workspace_id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, version, parent_file_id, is_latest, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6, $7,
		$8, $9, $10, $11, $12, $13, $14,
		$15, $16, $17, $18, NOW(), NOW()
	)`, file.ID, file.WorkspaceID, file.FileName, file.StorageKey, file.FileSize, file.MimeType, file.ContentHash,
		file.Category, file.FolderID, file.OwnerType, file.OwnerID, file.UploadedBy, file.IsActive, expiresAt,
		file.ScanStatus, file.Version, file.ParentFileID, file.IsLatest)
	return err
}

func (r FileRepository) GetFile(ctx context.Context, wsID, fileID uuid.UUID) (api.WorkspaceFileRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.WorkspaceFileRecord{}, err
	}
	var f api.WorkspaceFileRecord
	var created, updated time.Time
	var expiresAt, scannedAt *time.Time
	err = db.QueryRow(ctx, `SELECT
		id, workspace_id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, scanned_at, version, parent_file_id, is_latest, created_at, updated_at
	FROM workspace_files
	WHERE workspace_id = $1 AND id = $2 AND is_active = true`, wsID, fileID).Scan(
		&f.ID, &f.WorkspaceID, &f.FileName, &f.StorageKey, &f.FileSize, &f.MimeType, &f.ContentHash,
		&f.Category, &f.FolderID, &f.OwnerType, &f.OwnerID, &f.UploadedBy, &f.IsActive, &expiresAt,
		&f.ScanStatus, &scannedAt, &f.Version, &f.ParentFileID, &f.IsLatest, &created, &updated)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.WorkspaceFileRecord{}, api.ErrFileNotFound
		}
		return api.WorkspaceFileRecord{}, err
	}
	f.CreatedAt = created.UTC().Format(time.RFC3339)
	f.UpdatedAt = updated.UTC().Format(time.RFC3339)
	if expiresAt != nil {
		exp := expiresAt.UTC().Format(time.RFC3339)
		f.ExpiresAt = &exp
	}
	if scannedAt != nil {
		sc := scannedAt.UTC().Format(time.RFC3339)
		f.ScannedAt = &sc
	}
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

func (r FileRepository) CreateFileVersion(ctx context.Context, wsID, oldFileID uuid.UUID, newFile api.WorkspaceFileRecord) (api.WorkspaceFileRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return api.WorkspaceFileRecord{}, err
	}
	_, err = db.Exec(ctx, `UPDATE workspace_files SET is_latest = false, updated_at = NOW() WHERE workspace_id = $1 AND id = $2`, wsID, oldFileID)
	if err != nil {
		return api.WorkspaceFileRecord{}, err
	}

	var expiresAt *time.Time
	if newFile.ExpiresAt != nil {
		if t, err := time.Parse(time.RFC3339, *newFile.ExpiresAt); err == nil {
			expiresAt = &t
		}
	}
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO workspace_files (
		id, workspace_id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, version, parent_file_id, is_latest, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6, $7,
		$8, $9, $10, $11, $12, $13, $14,
		$15, $16, $17, $18, NOW(), NOW()
	) RETURNING created_at, updated_at`,
		newFile.ID, wsID, newFile.FileName, newFile.StorageKey, newFile.FileSize, newFile.MimeType, newFile.ContentHash,
		newFile.Category, newFile.FolderID, newFile.OwnerType, newFile.OwnerID, newFile.UploadedBy, newFile.IsActive, expiresAt,
		newFile.ScanStatus, newFile.Version, newFile.ParentFileID, newFile.IsLatest).Scan(&created, &updated)
	if err != nil {
		return api.WorkspaceFileRecord{}, err
	}
	newFile.CreatedAt = created.UTC().Format(time.RFC3339)
	newFile.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return newFile, nil
}

func (r FileRepository) ListFileVersions(ctx context.Context, wsID, fileID uuid.UUID) ([]api.WorkspaceFileRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `WITH RECURSIVE lineage AS (
		SELECT * FROM workspace_files WHERE workspace_id = $1 AND id = $2
		UNION ALL
		SELECT wf.* FROM workspace_files wf
		JOIN lineage l ON wf.id = l.parent_file_id
		WHERE wf.workspace_id = $1
	)
	SELECT id, workspace_id, file_name, storage_key, file_size, mime_type, content_hash,
	       category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
	       scan_status, scanned_at, version, parent_file_id, is_latest, created_at, updated_at
	FROM lineage ORDER BY version DESC`, wsID, fileID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.WorkspaceFileRecord, 0)
	for rows.Next() {
		var f api.WorkspaceFileRecord
		var created, updated time.Time
		var expiresAt, scannedAt *time.Time
		if err := rows.Scan(
			&f.ID, &f.WorkspaceID, &f.FileName, &f.StorageKey, &f.FileSize, &f.MimeType, &f.ContentHash,
			&f.Category, &f.FolderID, &f.OwnerType, &f.OwnerID, &f.UploadedBy, &f.IsActive, &expiresAt,
			&f.ScanStatus, &scannedAt, &f.Version, &f.ParentFileID, &f.IsLatest, &created, &updated,
		); err != nil {
			return nil, err
		}
		f.CreatedAt = created.UTC().Format(time.RFC3339)
		f.UpdatedAt = updated.UTC().Format(time.RFC3339)
		if expiresAt != nil {
			exp := expiresAt.UTC().Format(time.RFC3339)
			f.ExpiresAt = &exp
		}
		if scannedAt != nil {
			sc := scannedAt.UTC().Format(time.RFC3339)
			f.ScannedAt = &sc
		}
		result = append(result, f)
	}
	return result, rows.Err()
}

func (r FileRepository) ListFilesByOwner(ctx context.Context, wsID uuid.UUID, ownerType string, ownerID uuid.UUID) ([]api.WorkspaceFileRecord, error) {
	db, err := r.db(ctx, wsID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, workspace_id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, scanned_at, version, parent_file_id, is_latest, created_at, updated_at
	FROM workspace_files
	WHERE workspace_id = $1 AND owner_type = $2 AND owner_id = $3 AND is_active = true
	ORDER BY created_at DESC`, wsID, ownerType, ownerID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.WorkspaceFileRecord, 0)
	for rows.Next() {
		var f api.WorkspaceFileRecord
		var created, updated time.Time
		var expiresAt, scannedAt *time.Time
		if err := rows.Scan(
			&f.ID, &f.WorkspaceID, &f.FileName, &f.StorageKey, &f.FileSize, &f.MimeType, &f.ContentHash,
			&f.Category, &f.FolderID, &f.OwnerType, &f.OwnerID, &f.UploadedBy, &f.IsActive, &expiresAt,
			&f.ScanStatus, &scannedAt, &f.Version, &f.ParentFileID, &f.IsLatest, &created, &updated,
		); err != nil {
			return nil, err
		}
		f.CreatedAt = created.UTC().Format(time.RFC3339)
		f.UpdatedAt = updated.UTC().Format(time.RFC3339)
		if expiresAt != nil {
			exp := expiresAt.UTC().Format(time.RFC3339)
			f.ExpiresAt = &exp
		}
		if scannedAt != nil {
			sc := scannedAt.UTC().Format(time.RFC3339)
			f.ScannedAt = &sc
		}
		result = append(result, f)
	}
	return result, rows.Err()
}
