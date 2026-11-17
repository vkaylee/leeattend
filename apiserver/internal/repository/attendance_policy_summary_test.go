package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

type attendancePolicyRows struct {
	rows [][]any
	idx  int
}

func (r *attendancePolicyRows) Close()                                       {}
func (r *attendancePolicyRows) Err() error                                   { return nil }
func (r *attendancePolicyRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *attendancePolicyRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *attendancePolicyRows) Next() bool {
	if r.idx >= len(r.rows) {
		return false
	}
	r.idx++
	return true
}
func (r *attendancePolicyRows) Values() ([]any, error) { return nil, nil }
func (r *attendancePolicyRows) RawValues() [][]byte    { return nil }
func (r *attendancePolicyRows) Conn() *pgx.Conn        { return nil }
func (r *attendancePolicyRows) Scan(dest ...any) error {
	row := r.rows[r.idx-1]
	for i, value := range row {
		switch target := dest[i].(type) {
		case *uuid.UUID:
			*target = value.(uuid.UUID)
		case **uuid.UUID:
			if value != nil {
				id := value.(uuid.UUID)
				*target = &id
			}
		case *string:
			*target = value.(string)
		case *int:
			*target = value.(int)
		case *float64:
			*target = value.(float64)
		case *bool:
			*target = value.(bool)
		case **string:
			if value != nil {
				text := value.(string)
				*target = &text
			}
		case *interface{}:
			*target = value
		}
	}
	return nil
}

func TestAttendanceSummariesUseEffectiveWorkPolicy(t *testing.T) {
	employmentID, summaryID, versionID := uuid.New(), uuid.New(), uuid.New()
	var query string
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 1
				return nil
			}}
		},
		queryFn: func(_ context.Context, sql string, _ ...any) (pgx.Rows, error) {
			query = sql
			return &attendancePolicyRows{rows: [][]any{{
				summaryID, employmentID, "2026-06-15", 450, 450, 0, 0, 0, 0, 0, 0,
				450.0, versionID, "none", nil, nil, false, false, nil,
				"2026-06-15T00:00:00Z", "2026-06-15T00:00:00Z",
			}}}, nil
		},
	}

	result, err := (AttendanceRepository{DB: db}).GetAttendanceSummaries(context.Background(), "2026-06-01", "2026-06-30", nil, nil, 1, 50)
	if err != nil {
		t.Fatalf("get summaries: %v", err)
	}
	if !strings.Contains(query, "m.effective_to IS NULL OR m.effective_to > s.date") || !strings.Contains(query, "v.effective_to IS NULL OR v.effective_to > s.date") {
		t.Fatalf("query must use exclusive policy boundaries: %s", query)
	}
	if len(result.Data) != 1 || result.Data[0].StandardCredits != 1 || result.Data[0].WorkPolicyVersionID == nil || *result.Data[0].WorkPolicyVersionID != versionID {
		t.Fatalf("unexpected policy summary: %+v", result.Data)
	}
}

func TestAttendanceSummariesKeepMissingPolicyUnconfigured(t *testing.T) {
	employmentID, summaryID := uuid.New(), uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 1
				return nil
			}}
		},
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &attendancePolicyRows{rows: [][]any{{
				summaryID, employmentID, "2026-06-15", 90, 90, 0, 0, 0, 0, 0, 0,
				0.0, nil, "none", nil, nil, false, false, nil,
				"2026-06-15T00:00:00Z", "2026-06-15T00:00:00Z",
			}}}, nil
		},
	}

	result, err := (AttendanceRepository{DB: db}).GetAttendanceSummaries(context.Background(), "2026-06-01", "2026-06-30", nil, nil, 1, 50)
	if err != nil {
		t.Fatalf("get summaries: %v", err)
	}
	if len(result.Data) != 1 || result.Data[0].StandardCredits != 0 || result.Data[0].WorkPolicyVersionID != nil {
		t.Fatalf("missing policy must remain unconfigured: %+v", result.Data)
	}
}
