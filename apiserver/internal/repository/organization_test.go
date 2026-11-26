package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type organizationRows struct {
	data  [][]any
	index int
}

func (r *organizationRows) Close()                                       {}
func (r *organizationRows) Err() error                                   { return nil }
func (r *organizationRows) CommandTag() pgconn.CommandTag                { return pgconn.NewCommandTag("SELECT") }
func (r *organizationRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *organizationRows) Next() bool {
	if r.index >= len(r.data) {
		return false
	}
	r.index++
	return true
}
func (r *organizationRows) Scan(dest ...any) error {
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
func (r *organizationRows) Values() ([]any, error) { return r.data[r.index-1], nil }
func (r *organizationRows) RawValues() [][]byte    { return nil }
func (r *organizationRows) Conn() *pgx.Conn        { return nil }

func TestOrganizationRepository_GetOrganization(t *testing.T) {
	internalID := uuid.New()
	now := time.Now().UTC()
	var query string
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			query = gotQuery
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = internalID
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
	repo := OrganizationRepository{DB: db}
	record, err := repo.GetOrganization(context.Background())
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if record.Name != "Acme Corp" || record.HolidayOvernight != "split_at_midnight" {
		t.Fatalf("unexpected record: %+v", record)
	}
	if !strings.Contains(query, "FROM organization_settings") || !strings.Contains(query, "LIMIT 1") {
		t.Fatalf("unexpected query: %s", query)
	}
}

func TestOrganizationRepository_UpdateOrganization(t *testing.T) {
	internalID := uuid.New()
	now := time.Now().UTC()
	var updateQuery string
	var updateArgs []any
	db := &fakeDBTX{
		execFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgconn.CommandTag, error) {
			updateQuery, updateArgs = gotQuery, gotArgs
			return pgconn.NewCommandTag("UPDATE 1"), nil
		},
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*uuid.UUID)) = internalID
				*(dest[1].(*string)) = "Updated Corp"
				*(dest[2].(*string)) = "DD/MM/YYYY"
				*(dest[3].(*string)) = "12h"
				*(dest[4].(*string)) = "Asia/Ho_Chi_Minh"
				*(dest[5].(*string)) = "#10b981"
				*(dest[6].(*string)) = "vi"
				*(dest[7].(**string)) = nil
				*(dest[8].(**string)) = nil
				*(dest[9].(*time.Time)) = now
				return nil
			}}
		},
	}
	repo := OrganizationRepository{DB: db}
	newName := "Updated Corp"
	patch := api.OrganizationPatch{Name: &newName}
	record, err := repo.UpdateOrganization(context.Background(), patch)
	if err != nil {
		t.Fatalf("unexpected update error: %v", err)
	}
	if record.Name != "Updated Corp" {
		t.Fatalf("unexpected record name: %s", record.Name)
	}
	if !strings.Contains(updateQuery, "UPDATE organization_settings SET name = $1") {
		t.Fatalf("unexpected update query: %s", updateQuery)
	}
	if len(updateArgs) != 1 || updateArgs[0] != "Updated Corp" {
		t.Fatalf("unexpected update args: %#v", updateArgs)
	}
}

func TestOrganizationRepository_PutConfigBindsTypedValues(t *testing.T) {
	actorID := uuid.New()
	var query string
	var args []any
	db := &fakeDBTX{
		execFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgconn.CommandTag, error) {
			query, args = gotQuery, gotArgs
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}
	repo := OrganizationRepository{DB: db}
	valText := "dark"
	err := repo.PutConfig(context.Background(), api.ConfigEntryRequest{
		ScopeType:     "organization",
		ConfigKey:     "theme_mode",
		Value:         api.ConfigValue{Text: &valText},
		EffectiveFrom: "2026-01-01",
	}, actorID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(query, "INSERT INTO config_entries") || !strings.Contains(query, "ON CONFLICT (scope_type, scope_id, config_key, effective_from)") {
		t.Fatalf("unexpected query: %s", query)
	}
	if len(args) != 11 || args[0] != "organization" || args[1] != (*uuid.UUID)(nil) || args[2] != "theme_mode" || *(args[5].(*string)) != "dark" {
		t.Fatalf("unexpected args: %#v", args)
	}
}

func TestOrganizationRepository_PutConfigRejectsDeletedEmployer(t *testing.T) {
	employerID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
			if !strings.Contains(query, "FROM employers") || len(args) != 1 || args[0] != employerID {
				t.Fatalf("unexpected employer validation query: %s args=%v", query, args)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*bool)) = false
				return nil
			}}
		},
		execFn: func(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
			t.Fatal("deleted employer must not receive config write")
			return pgconn.CommandTag{}, nil
		},
	}
	repo := OrganizationRepository{DB: db}
	enabled := true
	err := repo.PutConfig(context.Background(), api.ConfigEntryRequest{
		ScopeType:     "employer",
		ScopeID:       &employerID,
		ConfigKey:     "schedule.lifecycle.enabled",
		Value:         api.ConfigValue{Bool: &enabled},
		EffectiveFrom: "2026-01-01",
	}, uuid.New())
	if !errors.Is(err, api.ErrEmployerNotFound) {
		t.Fatalf("expected missing employer error, got %v", err)
	}
}

func TestOrganizationRepository_ListAndGetConfigs(t *testing.T) {
	entryID := uuid.New()
	now := time.Now().UTC()
	var listQuery, deleteQuery string
	db := &fakeDBTX{
		queryFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgx.Rows, error) {
			listQuery = gotQuery
			return &organizationRows{
				data: [][]any{
					{entryID, "organization", nil, "max_overtime_hours", nil, nil, nil, nil, []byte(`{}`), now, nil, nil, now, now},
				},
			}, nil
		},
		queryRowFn: func(_ context.Context, gotQuery string, gotArgs ...any) pgx.Row {
			if !strings.Contains(gotQuery, "scope_id IS NOT DISTINCT FROM") || len(gotArgs) != 4 {
				t.Fatalf("expected point-in-time scoped config query, query=%s args=%v", gotQuery, gotArgs)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				return pgx.ErrNoRows
			}}
		},
		execFn: func(_ context.Context, gotQuery string, gotArgs ...any) (pgconn.CommandTag, error) {
			deleteQuery = gotQuery
			return pgconn.NewCommandTag("DELETE 1"), nil
		},
	}
	repo := OrganizationRepository{DB: db}
	entries, err := repo.ListConfigs(context.Background(), api.ConfigListRequest{ScopeType: "organization"})
	if err != nil {
		t.Fatalf("unexpected list error: %v", err)
	}
	if len(entries) != 1 || entries[0].ID != entryID {
		t.Fatalf("unexpected entries: %+v", entries)
	}
	if !strings.Contains(listQuery, "FROM config_entries") || !strings.Contains(listQuery, "ORDER BY created_at DESC") {
		t.Fatalf("unexpected list query: %s", listQuery)
	}

	_, err = repo.GetConfig(context.Background(), "non_existent", api.ConfigReadRequest{ScopeType: "organization", AsOf: now})
	if !errors.Is(err, pgx.ErrNoRows) || !errors.Is(err, api.ErrNotFound) {
		t.Fatalf("expected ErrNoRows/ErrNotFound, got: %v", err)
	}

	if err := repo.DeleteConfig(context.Background(), entryID); err != nil {
		t.Fatalf("unexpected delete error: %v", err)
	}
	if !strings.Contains(deleteQuery, "DELETE FROM config_entries WHERE id = $1") {
		t.Fatalf("unexpected delete query: %s", deleteQuery)
	}
}
