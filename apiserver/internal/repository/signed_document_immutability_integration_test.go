//go:build integration

package repository

import (
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationSignedDocumentCannotBeDeletedOrVersioned(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employeeID := uuid.New()
	fileID := uuid.New()
	signatureID := uuid.New()
	if _, err := tx.Exec(ctx, `
		INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date)
		VALUES ($1, 'signed-doc-employee', 'signed-document-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID); err != nil {
		t.Fatalf("insert signer employee: %v", err)
	}
	if _, err := tx.Exec(ctx, `
		INSERT INTO files (id, file_name, storage_key, file_size, mime_type, category, owner_type, owner_id, scan_status, version, is_latest)
		VALUES ($1, 'signed.pdf', 'signed-key', 4, 'application/pdf', 'contract', 'employee', $2, 'clean', 1, true)`, fileID, employeeID); err != nil {
		t.Fatalf("insert signed file: %v", err)
	}
	if _, err := tx.Exec(ctx, `
		INSERT INTO signature_requests (id, document_id, signer_id, status, signed_at)
		VALUES ($1, $2, $3, 'signed', NOW())`, signatureID, fileID, employeeID); err != nil {
		t.Fatalf("insert signed request: %v", err)
	}

	repo := FileRepository{DB: tx}
	if err := repo.DeleteFile(ctx, fileID); !errors.Is(err, api.ErrFileImmutable) {
		t.Fatalf("delete signed file error = %v, want immutable", err)
	}
	_, err = repo.CreateFileVersion(ctx, fileID, api.FileRecord{
		ID:         uuid.New(),
		FileName:   "replacement.pdf",
		StorageKey: "replacement-key",
		FileSize:   4,
		MimeType:   "application/pdf",
		Category:   "contract",
		OwnerType:  "employee",
		OwnerID:    employeeID,
		IsActive:   true,
		ScanStatus: "pending",
		Version:    2,
		IsLatest:   true,
	})
	if !errors.Is(err, api.ErrFileImmutable) {
		t.Fatalf("version signed file error = %v, want immutable", err)
	}

	var active, latest bool
	if err := tx.QueryRow(ctx, `SELECT is_active, is_latest FROM files WHERE id = $1`, fileID).Scan(&active, &latest); err != nil {
		t.Fatalf("read signed file state: %v", err)
	}
	if !active || !latest {
		t.Fatalf("signed file state changed: active=%t latest=%t", active, latest)
	}
	var versionCount int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM files WHERE parent_file_id = $1`, fileID).Scan(&versionCount); err != nil {
		t.Fatalf("count signed file versions: %v", err)
	}
	if versionCount != 0 {
		t.Fatalf("signed file received %d replacement versions", versionCount)
	}
}
