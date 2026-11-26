package repository

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestSignedDocumentMutationDoesNotRewriteFileSnapshot(t *testing.T) {
	var query string
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		},
		execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
			query = sql
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}
	if err := (FileRepository{DB: db}).SignDocument(context.Background(), uuid.New(), uuid.New()); err != nil {
		t.Fatalf("sign document: %v", err)
	}
	if !strings.Contains(query, "UPDATE signature_requests") || strings.Contains(query, "UPDATE files") || strings.Contains(query, "storage_key") || strings.Contains(query, "content_hash") {
		t.Fatalf("signing must update signature state without rewriting file snapshot: %s", query)
	}
}

func TestSignedDocumentCannotBeDeleted(t *testing.T) {
	execCalled := false
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
		execFn: func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
			execCalled = true
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
	}
	if err := (FileRepository{DB: db}).DeleteFile(context.Background(), uuid.New()); !errors.Is(err, api.ErrFileImmutable) {
		t.Fatalf("delete signed file error = %v, want immutable", err)
	}
	if execCalled {
		t.Fatal("signed file delete must not execute update")
	}
}

func TestSignedDocumentCannotReceiveNewVersion(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = true
				return nil
			}}
		},
	}
	_, err := (FileRepository{DB: db}).CreateFileVersion(context.Background(), uuid.New(), api.FileRecord{ID: uuid.New()})
	if !errors.Is(err, api.ErrFileImmutable) {
		t.Fatalf("version signed file error = %v, want immutable", err)
	}
}

func TestDocumentTemplateVariablesResolveNumberThroughEmploymentTenure(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if strings.Contains(query, "FROM employment_segments") {
				if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "em.start_date") {
					t.Fatalf("document variables must resolve tenure number by employment date: %s", query)
				}
				return fakeRow{scanFn: func(dest ...any) error {
					return pgx.ErrNoRows
				}}
			}
			return fakeRow{scanFn: func(dest ...any) error {
				return pgx.ErrNoRows
			}}
		},
	}
	if _, err := (DocumentRepository{DB: db}).GetTemplateVariables(context.Background(), uuid.New()); err != nil {
		t.Fatalf("get template variables: %v", err)
	}
}
