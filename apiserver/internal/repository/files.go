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

func (r FileRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("file repository: database is not configured")
	}
	return r.DB, nil
}

func (r FileRepository) GetOrganizationID(ctx context.Context) (uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return uuid.Nil, err
	}
	var count int
	if err := db.QueryRow(ctx, `SELECT count(*) FROM organization_settings`).Scan(&count); err != nil {
		return uuid.Nil, err
	}
	if count != 1 {
		return uuid.Nil, errors.New("file repository: expected exactly one organization setting row")
	}
	var orgID uuid.UUID
	if err := db.QueryRow(ctx, `SELECT id FROM organization_settings LIMIT 1`).Scan(&orgID); err != nil {
		return uuid.Nil, err
	}
	if orgID == uuid.Nil {
		return uuid.Nil, errors.New("file repository: organization setting row has nil id")
	}
	return orgID, nil
}

func (r FileRepository) CreateFile(ctx context.Context, file api.FileRecord) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	var expiresAt *time.Time
	if file.ExpiresAt != nil {
		if t, err := time.Parse(time.RFC3339, *file.ExpiresAt); err == nil {
			expiresAt = &t
		}
	}
	_, err = db.Exec(ctx, `INSERT INTO files (
		id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, version, parent_file_id, is_latest, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6, $7,
		$8, $9, $10, $11, $12, $13, $14,
		$15, $16, $17, NOW(), NOW()
	)`, file.ID, file.FileName, file.StorageKey, file.FileSize, file.MimeType, file.ContentHash,
		file.Category, file.FolderID, file.OwnerType, file.OwnerID, file.UploadedBy, file.IsActive, expiresAt,
		file.ScanStatus, file.Version, file.ParentFileID, file.IsLatest)
	return err
}

func (r FileRepository) GetFile(ctx context.Context, fileID uuid.UUID) (api.FileRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.FileRecord{}, err
	}
	var f api.FileRecord
	var created, updated time.Time
	var expiresAt, scannedAt *time.Time
	err = db.QueryRow(ctx, `SELECT
		id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, scanned_at, version, parent_file_id, is_latest, created_at, updated_at
	FROM files
	WHERE id = $1 AND is_active = true`, fileID).Scan(
		&f.ID, &f.FileName, &f.StorageKey, &f.FileSize, &f.MimeType, &f.ContentHash,
		&f.Category, &f.FolderID, &f.OwnerType, &f.OwnerID, &f.UploadedBy, &f.IsActive, &expiresAt,
		&f.ScanStatus, &scannedAt, &f.Version, &f.ParentFileID, &f.IsLatest, &created, &updated)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.FileRecord{}, api.ErrFileNotFound
		}
		return api.FileRecord{}, err
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

func (r FileRepository) DeleteFile(ctx context.Context, fileID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	var signed bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM signature_requests WHERE document_id = $1 AND status = 'signed')`, fileID).Scan(&signed); err != nil {
		return err
	}
	if signed {
		return api.ErrFileImmutable
	}
	tag, err := db.Exec(ctx, `UPDATE files SET is_active = false, updated_at = NOW() WHERE id = $1 AND is_active = true`, fileID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrFileNotFound
	}
	return nil
}

func (r FileRepository) CreateSignatureRequest(ctx context.Context, req api.SignatureRequestRecord) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO signature_requests (
		id, document_id, signer_id, status, created_at, updated_at
	) VALUES ($1, $2, $3, $4, NOW(), NOW())`, req.ID, req.DocumentID, req.SignerID, req.Status)
	return err
}

func (r FileRepository) ListPendingSignatures(ctx context.Context, signerID uuid.UUID) ([]api.SignatureRequestRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, document_id, signer_id, status, signed_at, created_at, updated_at
	FROM signature_requests
	WHERE signer_id = $1 AND status = 'pending'
	ORDER BY created_at DESC`, signerID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.SignatureRequestRecord, 0)
	for rows.Next() {
		var s api.SignatureRequestRecord
		var signed *time.Time
		var created, updated time.Time
		if err := rows.Scan(&s.ID, &s.DocumentID, &s.SignerID, &s.Status, &signed, &created, &updated); err != nil {
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

func (r FileRepository) SignDocument(ctx context.Context, requestID, signerID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `
		UPDATE signature_requests sr
		SET status = 'signed', signed_at = NOW(), updated_at = NOW()
		FROM employee_tenures t
		JOIN employees e ON e.id = t.employee_id AND e.deleted_at IS NULL
		WHERE sr.id = $1
		  AND sr.signer_id = t.employee_id
		  AND t.user_id = $2
		  AND t.deleted_at IS NULL
		  AND t.start_date <= `+orgTodaySQL+`
		  AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`)
		  AND sr.status = 'pending'
	`, requestID, signerID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrFileNotFound
	}
	return nil
}

func (r FileRepository) CreateFileVersion(ctx context.Context, oldFileID uuid.UUID, newFile api.FileRecord) (api.FileRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.FileRecord{}, err
	}
	var signed bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM signature_requests WHERE document_id = $1 AND status = 'signed')`, oldFileID).Scan(&signed); err != nil {
		return api.FileRecord{}, err
	}
	if signed {
		return api.FileRecord{}, api.ErrFileImmutable
	}
	_, err = db.Exec(ctx, `UPDATE files SET is_latest = false, updated_at = NOW() WHERE id = $1`, oldFileID)
	if err != nil {
		return api.FileRecord{}, err
	}

	var expiresAt *time.Time
	if newFile.ExpiresAt != nil {
		if t, err := time.Parse(time.RFC3339, *newFile.ExpiresAt); err == nil {
			expiresAt = &t
		}
	}
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO files (
		id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, version, parent_file_id, is_latest, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6, $7,
		$8, $9, $10, $11, $12, $13, $14,
		$15, $16, $17, NOW(), NOW()
	) RETURNING created_at, updated_at`,
		newFile.ID, newFile.FileName, newFile.StorageKey, newFile.FileSize, newFile.MimeType, newFile.ContentHash,
		newFile.Category, newFile.FolderID, newFile.OwnerType, newFile.OwnerID, newFile.UploadedBy, newFile.IsActive, expiresAt,
		newFile.ScanStatus, newFile.Version, newFile.ParentFileID, newFile.IsLatest).Scan(&created, &updated)
	if err != nil {
		return api.FileRecord{}, err
	}
	newFile.CreatedAt = created.UTC().Format(time.RFC3339)
	newFile.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return newFile, nil
}

func (r FileRepository) ListFileVersions(ctx context.Context, fileID uuid.UUID) ([]api.FileRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `WITH RECURSIVE lineage AS (
		SELECT * FROM files WHERE id = $1
		UNION ALL
		SELECT wf.* FROM files wf
		JOIN lineage l ON wf.id = l.parent_file_id
	)
	SELECT id, file_name, storage_key, file_size, mime_type, content_hash,
	       category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
	       scan_status, scanned_at, version, parent_file_id, is_latest, created_at, updated_at
	FROM lineage ORDER BY version DESC`, fileID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.FileRecord, 0)
	for rows.Next() {
		var f api.FileRecord
		var created, updated time.Time
		var expiresAt, scannedAt *time.Time
		if err := rows.Scan(
			&f.ID, &f.FileName, &f.StorageKey, &f.FileSize, &f.MimeType, &f.ContentHash,
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

func (r FileRepository) ListFilesByOwner(ctx context.Context, ownerType string, ownerID uuid.UUID) ([]api.FileRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, file_name, storage_key, file_size, mime_type, content_hash,
		category, folder_id, owner_type, owner_id, uploaded_by, is_active, expires_at,
		scan_status, scanned_at, version, parent_file_id, is_latest, created_at, updated_at
	FROM files
	WHERE owner_type = $1 AND owner_id = $2 AND is_active = true
	ORDER BY created_at DESC`, ownerType, ownerID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.FileRecord, 0)
	for rows.Next() {
		var f api.FileRecord
		var created, updated time.Time
		var expiresAt, scannedAt *time.Time
		if err := rows.Scan(
			&f.ID, &f.FileName, &f.StorageKey, &f.FileSize, &f.MimeType, &f.ContentHash,
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
