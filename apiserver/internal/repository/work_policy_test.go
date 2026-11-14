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

func TestParseWorkPolicyRangeRejectsNonIncreasingEnd(t *testing.T) {
	for _, end := range []string{"2026-01-01", "2025-12-31"} {
		_, _, err := parseWorkPolicyRange("2026-01-01", &end)
		if !errors.Is(err, api.ErrWorkPolicyGroupValidation) {
			t.Fatalf("effective_to=%q: expected validation error, got %v", end, err)
		}
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
