package repository

import (
	"context"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestWorkPolicyRepository_ListWorkPolicyGroupHistoryPaginatesByGroup(t *testing.T) {
	groupID := uuid.New()
	versionID := uuid.New()
	var dataQuery string
	db := &fakeDBTX{
		queryFn: func(_ context.Context, query string, args ...any) (pgx.Rows, error) {
			dataQuery = query
			if len(args) != 3 || args[0] != groupID || args[1] != 20 || args[2] != 20 {
				t.Fatalf("unexpected history args: %#v", args)
			}
			return &rowsStub{data: [][]any{{
				versionID, groupID, "Operations", uuid.New(), "Work credit", 450,
				uuid.New(), uuid.New(), uuid.New(), "Pay basis", "monthly", "fixed", "22",
				uuid.New(), "2026-01-01", nil,
			}}}, nil
		},
		queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
			if !strings.Contains(query, "count(*) FROM labor_policy_group_versions WHERE group_id = $1") || len(args) != 1 || args[0] != groupID {
				t.Fatalf("unexpected history count query: %s %#v", query, args)
			}
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 21
				return nil
			}}
		},
	}

	result, err := (WorkPolicyRepository{DB: db}).ListWorkPolicyGroupHistory(context.Background(), groupID, 2, 20)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if result.Total != 21 || result.Page != 2 || result.Limit != 20 || len(result.Data) != 1 || result.Data[0].ID != versionID {
		t.Fatalf("unexpected history response: %+v", result)
	}
	if !strings.Contains(dataQuery, "WHERE v.group_id = $1") || !strings.Contains(dataQuery, "ORDER BY v.effective_from DESC") {
		t.Fatalf("history query must be scoped and stable: %s", dataQuery)
	}
}

func TestParseWorkPolicyRangeRejectsNonIncreasingEnd(t *testing.T) {
	for _, end := range []string{"2026-01-01", "2025-12-31"} {
		_, _, err := parseWorkPolicyRange("2026-01-01", &end)
		if !errors.Is(err, api.ErrWorkPolicyGroupValidation) {
			t.Fatalf("effective_to=%q: expected validation error, got %v", end, err)
		}
	}
}

func TestWorkPolicyAssignmentAtSplitsExclusiveBoundaries(t *testing.T) {
	versionOne := api.WorkPolicyGroupVersion{ID: uuid.New(), EffectiveFrom: "2026-01-01", EffectiveTo: func() *string { value := "2026-06-15"; return &value }()}
	versionTwo := api.WorkPolicyGroupVersion{ID: uuid.New(), EffectiveFrom: "2026-06-15"}
	assignments := []workPolicyAssignment{
		{policy: versionOne, membershipFrom: "2026-01-01"},
		{policy: versionTwo, membershipFrom: "2026-06-15"},
	}
	before := workPolicyAssignmentAt(assignments, time.Date(2026, 6, 14, 0, 0, 0, 0, time.UTC))
	if before == nil || before.ID != versionOne.ID {
		t.Fatalf("expected first policy before boundary, got %+v", before)
	}
	onBoundary := workPolicyAssignmentAt(assignments, time.Date(2026, 6, 15, 0, 0, 0, 0, time.UTC))
	if onBoundary == nil || onBoundary.ID != versionTwo.ID {
		t.Fatalf("expected replacement policy at exclusive boundary, got %+v", onBoundary)
	}
}

func TestResolveWorkPolicyUsesExclusiveEndBoundary(t *testing.T) {
	employmentID := uuid.New()
	policyID := uuid.New()
	groupID := uuid.New()
	var capturedQuery string
	query := func(_ context.Context, sql string, args ...any) pgx.Row {
		capturedQuery = sql
		if !strings.Contains(sql, "labor_policy_memberships") {
			return fakeRow{}
		}
		if len(args) > 1 && args[1] == "2026-06-01" {
			return fakeRow{}
		}
		return fakeRow{scanFn: func(dest ...any) error {
			*(dest[0].(*uuid.UUID)) = policyID
			*(dest[1].(*uuid.UUID)) = groupID
			*(dest[2].(*string)) = "Operations"
			*(dest[3].(*uuid.UUID)) = uuid.New()
			*(dest[4].(*string)) = "Work credit"
			*(dest[5].(*int)) = 450
			*(dest[6].(*uuid.UUID)) = uuid.New()
			*(dest[7].(*uuid.UUID)) = uuid.New()
			*(dest[8].(*uuid.UUID)) = uuid.New()
			*(dest[9].(*string)) = "Pay basis"
			*(dest[10].(*string)) = string(workpolicy.PayMethodHourly)
			*(dest[11].(**string)) = nil
			*(dest[12].(**string)) = nil
			*(dest[13].(*uuid.UUID)) = uuid.New()
			*(dest[14].(*string)) = "2026-01-01"
			value := "2026-06-01"
			*(dest[15].(**string)) = &value
			return nil
		}}
	}
	db := &fakeDBTX{queryRowFn: query}
	repo := WorkPolicyRepository{DB: db}
	got, err := repo.ResolveWorkPolicy(context.Background(), employmentID, time.Date(2026, 5, 31, 0, 0, 0, 0, time.UTC))
	if err != nil || got.ID != policyID {
		t.Fatalf("expected policy before exclusive end, got %+v, err=%v", got, err)
	}
	if !strings.Contains(capturedQuery, "effective_to > $2::date") {
		t.Fatalf("resolver must use exclusive effective_to boundary: %s", capturedQuery)
	}
	_, err = repo.ResolveWorkPolicy(context.Background(), employmentID, time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC))
	if !errors.Is(err, api.ErrWorkPolicyResolutionNotFound) {
		t.Fatalf("expected boundary resolution miss, got %v", err)
	}
}
