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

func ptrTime(value time.Time) *time.Time { return &value }

func TestValidatePolicyMembershipRangeRejectsOutsideRelationship(t *testing.T) {
	start := time.Date(2026, 1, 10, 0, 0, 0, 0, time.UTC)
	end := time.Date(2026, 2, 1, 0, 0, 0, 0, time.UTC)
	exclusiveEnd := end.AddDate(0, 0, 1)
	if err := validatePolicyMembershipRange(ptrTime(time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)), &exclusiveEnd, start, &end); !errors.Is(err, api.ErrWorkPolicyGroupValidation) {
		t.Fatalf("outside membership error=%v, want validation", err)
	}
	if err := validatePolicyMembershipRange(ptrTime(start), nil, start, &end); !errors.Is(err, api.ErrWorkPolicyGroupValidation) {
		t.Fatalf("open membership on closed relationship error=%v, want validation", err)
	}
	if err := validatePolicyMembershipRange(ptrTime(start), &exclusiveEnd, start, &end); err != nil {
		t.Fatalf("boundary membership error=%v", err)
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

func TestWorkPolicyResolver_ResolveAtFiltersPublishedAndExclusiveEnd(t *testing.T) {
	relID := uuid.New()
	var capturedQuery string
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
		capturedQuery = query
		if len(args) != 2 || args[0] != relID || args[1] != "2026-06-01" {
			t.Fatalf("unexpected resolver args: %#v", args)
		}
		return fakeRow{}
	}}

	_, err := (WorkPolicyResolver{}).ResolveAt(context.Background(), db, relID, time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC))
	if !errors.Is(err, api.ErrWorkPolicyResolutionNotFound) {
		t.Fatalf("expected ErrWorkPolicyResolutionNotFound, got %v", err)
	}
	for _, required := range []string{
		"FROM work_policy_memberships m",
		"JOIN employment_relationships er ON er.id = m.employment_relationship_id",
		"v.status = 'published'",
		"m.effective_to > $2::date",
		"v.effective_to > $2::date",
	} {
		if !strings.Contains(capturedQuery, required) {
			t.Fatalf("resolver query missing %q: %s", required, capturedQuery)
		}
	}
}

func TestWorkPolicyRepository_GetWorkPolicyVersionScopesGroupAndVersion(t *testing.T) {
	groupID, versionID := uuid.New(), uuid.New()
	var captured string
	db := &fakeDBTX{queryRowFn: func(_ context.Context, query string, args ...any) pgx.Row {
		captured = query
		if len(args) != 2 || args[0] != groupID || args[1] != versionID {
			t.Fatalf("unexpected detail args: %#v", args)
		}
		return fakeRow{}
	}}
	_, err := (WorkPolicyRepository{DB: db}).GetWorkPolicyVersion(context.Background(), workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if !errors.Is(err, api.ErrWorkPolicyGroupNotFound) {
		t.Fatalf("expected not found, got %v", err)
	}
	if !strings.Contains(captured, "WHERE v.group_id = $1 AND v.id = $2") {
		t.Fatalf("detail query must scope group and version: %s", captured)
	}
}

func TestFindResolvedPolicyForDateExclusiveBoundary(t *testing.T) {
	split := time.Date(2026, 6, 15, 0, 0, 0, 0, time.UTC)
	v1 := workpolicy.ResolvedPolicy{
		VersionID:               uuid.New(),
		EffectiveFrom:           time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
		EffectiveTo:             &split,
		MembershipEffectiveFrom: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
	}
	v2 := workpolicy.ResolvedPolicy{
		VersionID:               uuid.New(),
		EffectiveFrom:           split,
		MembershipEffectiveFrom: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
	}
	got1, ok := FindResolvedPolicyForDate([]workpolicy.ResolvedPolicy{v1, v2}, time.Date(2026, 6, 14, 0, 0, 0, 0, time.UTC))
	if !ok || got1.VersionID != v1.VersionID {
		t.Fatalf("expected v1 before boundary, got %+v", got1)
	}
	got2, ok := FindResolvedPolicyForDate([]workpolicy.ResolvedPolicy{v1, v2}, split)
	if !ok || got2.VersionID != v2.VersionID {
		t.Fatalf("expected v2 on boundary, got %+v", got2)
	}
}
