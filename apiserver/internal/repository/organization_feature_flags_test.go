package repository

import (
	"context"
	"encoding/json"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestOrganizationRepository_UpdateFeatureFlagBindsAllPatchFields(t *testing.T) {
	flagID := uuid.New()
	metadata := []byte(`{"rollout_percent":100}`)
	createdAt := time.Date(2026, 1, 2, 3, 4, 5, 0, time.UTC)
	updatedAt := createdAt.Add(time.Minute)
	var query string
	var args []any
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			query, args = gotQuery, gotArgs
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = flagID
				*(dest[1].(*string)) = "payroll"
				*(dest[2].(*bool)) = true
				*(dest[3].(**string)) = nil
				*(dest[4].(*[]byte)) = metadata
				*(dest[5].(*time.Time)) = createdAt
				*(dest[6].(*time.Time)) = updatedAt
				return nil
			}}
		},
	}
	repo := OrganizationRepository{DB: db}
	enabled := true
	description := "updated"
	patchMetadata := json.RawMessage(`{"rollout_percent":100}`)
	record, err := repo.UpdateFeatureFlag(context.Background(), flagID, api.FeatureFlagPatch{
		Enabled:     &enabled,
		Description: &description,
		Metadata:    &patchMetadata,
	})
	if err != nil {
		t.Fatalf("update feature flag: %v", err)
	}
	if record.FlagKey != "payroll" || !record.Enabled || string(record.Metadata) != string(metadata) {
		t.Fatalf("unexpected record: %+v", record)
	}
	if !strings.Contains(query, "enabled = $1") || !strings.Contains(query, "description = $2") || !strings.Contains(query, "metadata = $3") || !strings.Contains(query, "WHERE id = $4") {
		t.Fatalf("unexpected placeholder layout: %s", query)
	}
	if strings.Contains(query, "workspace_id") {
		t.Fatalf("unexpected workspace_id in query: %s", query)
	}
	if len(args) != 4 || args[3] != flagID {
		t.Fatalf("unexpected query args: %#v", args)
	}
}

func TestOrganizationRepository_GetFeatureFlag(t *testing.T) {
	flagID := uuid.New()
	var query string
	var args []any
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			query, args = gotQuery, gotArgs
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = flagID
				*(dest[1].(*string)) = "dark_mode"
				*(dest[2].(*bool)) = false
				*(dest[3].(**string)) = nil
				*(dest[4].(*[]byte)) = []byte(`{}`)
				*(dest[5].(*time.Time)) = time.Unix(0, 0).UTC()
				*(dest[6].(*time.Time)) = time.Unix(60, 0).UTC()
				return nil
			}}
		},
	}
	repo := OrganizationRepository{DB: db}
	record, err := repo.GetFeatureFlag(context.Background(), "dark_mode")
	if err != nil {
		t.Fatalf("get feature flag: %v", err)
	}
	if record.FlagKey != "dark_mode" {
		t.Fatalf("unexpected flag key: %s", record.FlagKey)
	}
	if !strings.Contains(query, "FROM feature_flags WHERE flag_key = $1") {
		t.Fatalf("query missing key predicate: %s", query)
	}
	if strings.Contains(query, "workspace_id") {
		t.Fatalf("query unexpectedly contains workspace_id: %s", query)
	}
	if len(args) != 1 || args[0] != "dark_mode" {
		t.Fatalf("unexpected query args: %#v", args)
	}
}
