package repository

import (
	"context"
	"encoding/json"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"strings"
	"testing"
	"time"
)

func TestWorkspaceRepository_UpdateFeatureFlagBindsAllPatchFields(t *testing.T) {
	workspaceID, flagID := uuid.New(), uuid.New()
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
				*(dest[1].(*uuid.UUID)) = workspaceID
				*(dest[2].(*string)) = "payroll"
				*(dest[3].(*bool)) = true
				*(dest[4].(**string)) = nil
				*(dest[5].(*[]byte)) = metadata
				*(dest[6].(*time.Time)) = createdAt
				*(dest[7].(*time.Time)) = updatedAt
				return nil
			}}
		},
	}
	repo := WorkspaceRepository{DB: db}
	enabled := true
	description := "updated"
	patchMetadata := json.RawMessage(`{"rollout_percent":100}`)
	record, err := repo.UpdateFeatureFlag(context.Background(), workspaceID, flagID, api.FeatureFlagPatch{
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
	if !strings.Contains(query, "enabled = $1") || !strings.Contains(query, "description = $2") || !strings.Contains(query, "metadata = $3") || !strings.Contains(query, "id = $4") || !strings.Contains(query, "workspace_id = $5") {
		t.Fatalf("unexpected placeholder layout: %s", query)
	}
	if len(args) != 5 || args[3] != flagID || args[4] != workspaceID {
		t.Fatalf("unexpected query args: %#v", args)
	}
}

func TestWorkspaceRepository_GetFeatureFlagScopesWorkspace(t *testing.T) {
	workspaceID, flagID := uuid.New(), uuid.New()
	var query string
	var args []any
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			query, args = gotQuery, gotArgs
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = flagID
				*(dest[1].(*uuid.UUID)) = workspaceID
				*(dest[2].(*string)) = "dark_mode"
				*(dest[3].(*bool)) = false
				*(dest[4].(**string)) = nil
				*(dest[5].(*[]byte)) = []byte(`{}`)
				*(dest[6].(*time.Time)) = time.Unix(0, 0).UTC()
				*(dest[7].(*time.Time)) = time.Unix(60, 0).UTC()
				return nil
			}}
		},
	}
	repo := WorkspaceRepository{DB: db}
	if _, err := repo.GetFeatureFlag(context.Background(), workspaceID, "dark_mode"); err != nil {
		t.Fatalf("get feature flag: %v", err)
	}
	if !strings.Contains(query, "workspace_id = $1") || !strings.Contains(query, "flag_key = $2") {
		t.Fatalf("query missing tenant/key predicates: %s", query)
	}
	if len(args) != 2 || args[0] != workspaceID || args[1] != "dark_mode" {
		t.Fatalf("unexpected query args: %#v", args)
	}
}
