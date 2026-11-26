package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestStorageConfigRepository_RequiresDatabase(t *testing.T) {
	repo := StorageConfigRepository{}
	ctx := context.Background()

	if _, err := repo.GetStorageConfig(ctx); err == nil {
		t.Fatal("expected nil db error")
	}
	if _, err := repo.UpsertStorageConfig(ctx, api.StorageConfigRecord{}); err == nil {
		t.Fatal("expected nil db error")
	}
	if err := repo.DeleteStorageConfig(ctx); err == nil {
		t.Fatal("expected nil db error")
	}
}

func TestStorageConfigRepository_CRUD(t *testing.T) {
	ctx := context.Background()
	cfgID := uuid.New()
	endpoint := "https://s3.example.com"
	region := "us-east-1"
	bucket := "leeattend"

	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, sql string, args ...any) pgx.Row {
			if strings.Contains(sql, "FROM storage_configs LIMIT 1") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = cfgID
					*(dest[1].(*string)) = endpoint
					*(dest[2].(*string)) = region
					*(dest[3].(*string)) = bucket
					*(dest[4].(**string)) = nil
					*(dest[5].(*string)) = "enc_access"
					*(dest[6].(*string)) = "enc_secret"
					*(dest[7].(*string)) = "v1"
					*(dest[8].(*bool)) = true
					*(dest[9].(*int32)) = 900
					*(dest[10].(*int64)) = 10485760
					*(dest[11].(*bool)) = true
					return nil
				}}
			}
			if strings.Contains(sql, "UPDATE storage_configs SET") {
				return fakeRow{scanFn: func(dest ...any) error {
					*(dest[0].(*uuid.UUID)) = cfgID
					*(dest[1].(*string)) = endpoint
					*(dest[2].(*string)) = region
					*(dest[3].(*string)) = bucket
					*(dest[4].(**string)) = nil
					*(dest[5].(*string)) = "enc_access"
					*(dest[6].(*string)) = "enc_secret"
					*(dest[7].(*string)) = "v1"
					*(dest[8].(*bool)) = true
					*(dest[9].(*int32)) = 900
					*(dest[10].(*int64)) = 10485760
					*(dest[11].(*bool)) = true
					return nil
				}}
			}
			return fakeRow{}
		},
		execFn: func(_ context.Context, sql string, arguments ...any) (pgconn.CommandTag, error) {
			if strings.Contains(sql, "DELETE FROM storage_configs") {
				return pgconn.NewCommandTag("DELETE 1"), nil
			}
			return pgconn.CommandTag{}, nil
		},
	}

	repo := StorageConfigRepository{SystemDB: db}

	cfg, err := repo.GetStorageConfig(ctx)
	if err != nil || cfg == nil {
		t.Fatalf("GetStorageConfig failed: %v", err)
	}
	if cfg.ID != cfgID || cfg.BucketName != bucket {
		t.Fatalf("unexpected storage config: %+v", cfg)
	}

	saved, err := repo.UpsertStorageConfig(ctx, *cfg)
	if err != nil || saved == nil {
		t.Fatalf("UpsertStorageConfig failed: %v", err)
	}

	if err := repo.DeleteStorageConfig(ctx); err != nil {
		t.Fatalf("DeleteStorageConfig failed: %v", err)
	}
}
