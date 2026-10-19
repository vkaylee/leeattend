package repository

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type workspaceRows struct {
	data  [][]any
	index int
}

func (r *workspaceRows) Close()                                       {}
func (r *workspaceRows) Err() error                                   { return nil }
func (r *workspaceRows) CommandTag() pgconn.CommandTag                { return pgconn.NewCommandTag("SELECT") }
func (r *workspaceRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *workspaceRows) Next() bool {
	if r.index >= len(r.data) {
		return false
	}
	r.index++
	return true
}
func (r *workspaceRows) Scan(dest ...any) error {
	values := r.data[r.index-1]
	for i, value := range values {
		switch target := dest[i].(type) {
		case *uuid.UUID:
			if value != nil {
				*target = value.(uuid.UUID)
			}
		case *string:
			if value != nil {
				*target = value.(string)
			}
		case **string:
			if value != nil {
				v := value.(string)
				*target = &v
			}
		case *time.Time:
			if value != nil {
				*target = value.(time.Time)
			}
		case **time.Time:
			if value != nil {
				v := value.(time.Time)
				*target = &v
			}
		case *[]byte:
			if value != nil {
				*target = value.([]byte)
			}
		}
	}
	return nil
}
func (r *workspaceRows) Values() ([]any, error) { return r.data[r.index-1], nil }
func (r *workspaceRows) RawValues() [][]byte    { return nil }
func (r *workspaceRows) Conn() *pgx.Conn        { return nil }

func TestWorkspaceRepository_ListWorkspacesQueriesMembership(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()
	now := time.Now().UTC()
	var query string
	var args []any
	db := &fakeDBTX{
		queryFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgx.Rows, error) {
			query, args = gotQuery, gotArgs
			return &workspaceRows{
				data: [][]any{
					{wsID, "Acme Corp", "YYYY-MM-DD", "24h", "UTC", "#7c3aed", "en", nil, nil, now},
				},
			}, nil
		},
	}
	repo := WorkspaceRepository{SystemDB: db}
	records, err := repo.ListWorkspaces(context.Background(), userID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(records) != 1 || records[0].ID != wsID || records[0].Name != "Acme Corp" {
		t.Fatalf("unexpected records: %+v", records)
	}
	if !strings.Contains(query, "workspace_members") || !strings.Contains(query, "WHERE wm.user_id = $1") {
		t.Fatalf("unexpected query: %s", query)
	}
	if len(args) != 1 || args[0] != userID {
		t.Fatalf("unexpected args: %#v", args)
	}
}

func TestWorkspaceRepository_GetWorkspaceByID(t *testing.T) {
	wsID := uuid.New()
	now := time.Now().UTC()
	var query string
	var args []any
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			query, args = gotQuery, gotArgs
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = wsID
				*(dest[1].(*string)) = "Acme Corp"
				*(dest[2].(*string)) = "YYYY-MM-DD"
				*(dest[3].(*string)) = "24h"
				*(dest[4].(*string)) = "UTC"
				*(dest[5].(*string)) = "#7c3aed"
				*(dest[6].(*string)) = "en"
				*(dest[7].(**string)) = nil
				*(dest[8].(**string)) = nil
				*(dest[9].(*time.Time)) = now
				return nil
			}}
		},
	}
	repo := WorkspaceRepository{SystemDB: db}
	record, err := repo.GetWorkspace(context.Background(), wsID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if record.ID != wsID || record.Name != "Acme Corp" || record.HolidayOvernight != "split_at_midnight" {
		t.Fatalf("unexpected record: %+v", record)
	}
	if !strings.Contains(query, "WHERE id = $1") {
		t.Fatalf("unexpected query: %s", query)
	}
	if len(args) != 1 || args[0] != wsID {
		t.Fatalf("unexpected args: %#v", args)
	}
}

func TestWorkspaceRepository_PutConfigBindsTypedValues(t *testing.T) {
	wsID, actorID := uuid.New(), uuid.New()
	var query string
	var args []any
	db := &fakeDBTX{
		execFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgconn.CommandTag, error) {
			query, args = gotQuery, gotArgs
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}
	repo := WorkspaceRepository{TenantDB: func(context.Context, uuid.UUID) (DBTX, error) { return db, nil }}
	valText := "dark"
	err := repo.PutConfig(context.Background(), wsID, api.ConfigEntryRequest{
		ScopeType:     "workspace",
		ConfigKey:     "theme_mode",
		Value:         api.ConfigValue{Text: &valText},
		EffectiveFrom: "2026-01-01",
	}, actorID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(query, "INSERT INTO workspace_config_entries") || !strings.Contains(query, "ON CONFLICT") {
		t.Fatalf("unexpected query: %s", query)
	}
	if len(args) != 12 || args[0] != wsID || args[3] != "theme_mode" || *(args[6].(*string)) != "dark" {
		t.Fatalf("unexpected args: %#v", args)
	}
}

func TestWorkspaceRepository_ListAndGetConfigs(t *testing.T) {
	wsID, entryID := uuid.New(), uuid.New()
	now := time.Now().UTC()
	var listQuery, deleteQuery string
	db := &fakeDBTX{
		queryFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgx.Rows, error) {
			listQuery = gotQuery
			return &workspaceRows{
				data: [][]any{
					{entryID, wsID, "workspace", nil, "max_overtime_hours", nil, nil, nil, nil, []byte(`{}`), now, nil, nil, now, now},
				},
			}, nil
		},
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				return pgx.ErrNoRows
			}}
		},
		execFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgconn.CommandTag, error) {
			deleteQuery = gotQuery
			return pgconn.NewCommandTag("DELETE 1"), nil
		},
	}
	repo := WorkspaceRepository{TenantDB: func(context.Context, uuid.UUID) (DBTX, error) { return db, nil }}
	entries, err := repo.ListConfigs(context.Background(), wsID)
	if err != nil {
		t.Fatalf("unexpected list error: %v", err)
	}
	if len(entries) != 1 || entries[0].ID != entryID {
		t.Fatalf("unexpected entries: %+v", entries)
	}
	if !strings.Contains(listQuery, "ORDER BY created_at DESC") {
		t.Fatalf("unexpected list query: %s", listQuery)
	}

	resolved, err := repo.GetConfig(context.Background(), wsID, "non_existent")
	if err != nil {
		t.Fatalf("unexpected get error: %v", err)
	}
	if resolved.Key != "non_existent" || resolved.Value.Text == nil || *resolved.Value.Text != "mock" {
		t.Fatalf("unexpected resolved config fallback: %+v", resolved)
	}

	if err := repo.DeleteConfig(context.Background(), wsID, entryID); err != nil {
		t.Fatalf("unexpected delete error: %v", err)
	}
	if !strings.Contains(deleteQuery, "DELETE FROM workspace_config_entries WHERE id = $1 AND workspace_id = $2") {
		t.Fatalf("unexpected delete query: %s", deleteQuery)
	}
}
