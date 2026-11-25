package api

import (
	"bytes"
	"context"
	"errors"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

func TestEmployeeIdentityMetrics_RecordsAggregateCountsWithoutPII(t *testing.T) {
	var logBuf bytes.Buffer
	metrics := NewEmployeeIdentityMetrics(slog.New(slog.NewJSONHandler(&logBuf, nil)))

	ctx := context.Background()
	metrics.ObserveMatch(ctx, MatchTierNone, 0, 12*time.Millisecond)
	metrics.ObserveMatch(ctx, MatchTierSoft, 2, 18*time.Millisecond)
	metrics.ObserveMatch(ctx, MatchTierHard, 1, 9*time.Millisecond)
	metrics.ObserveMatch(ctx, "error", 0, 5*time.Millisecond)
	metrics.ObserveWrite(ctx, "create", ErrConflict, 15*time.Millisecond)
	metrics.ObserveWrite(ctx, "create", errors.New("tx rollback"), 22*time.Millisecond)
	metrics.ObserveWrite(ctx, "reuse_tenure", ErrConflict, 14*time.Millisecond)
	metrics.ObserveWrite(ctx, "reuse_tenure", errors.New("db disconnect"), 30*time.Millisecond)

	want := EmployeeIdentityMetricsSnapshot{
		MatchTierNone:  1,
		MatchTierSoft:  1,
		MatchTierHard:  1,
		MatchError:     1,
		CreateConflict: 1,
		CreateRollback: 1,
		TenureConflict: 1,
		TenureRollback: 1,
	}
	if snap := metrics.Snapshot(); snap != want {
		t.Fatalf("snapshot = %+v, want %+v", snap, want)
	}

	logged := logBuf.String()
	for _, want := range []string{`"employee_identity_metric"`, `"latency_ms"`, `"candidate_count":2`, `"outcome":"conflict"`, `"outcome":"rollback"`} {
		if !strings.Contains(logged, want) {
			t.Fatalf("metric log missing %s: %s", want, logged)
		}
	}
	for _, forbidden := range []string{"tx rollback", "db disconnect", `"phone"`, `"national_id"`, `"employee_id"`} {
		if strings.Contains(logged, forbidden) {
			t.Fatalf("metric log leaked %q: %s", forbidden, logged)
		}
	}
}

func TestEmployeeHandler_MatchEmitsPIIFreeMetrics(t *testing.T) {
	var logBuf bytes.Buffer
	metrics := NewEmployeeIdentityMetrics(slog.New(slog.NewJSONHandler(&logBuf, nil)))
	handler := newTestMatchHandler(
		t,
		map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true},
		&stubIdentityMatcher{
			candidates: []EmployeeMatchCandidate{{
				EmployeeID:   uuid.New(),
				DisplayName:  "Nguyen Van An",
				EmployeeCode: "EMP-990",
				Status:       MatchStatusTerminated,
				Tier:         MatchTierHard,
				Reasons:      []string{MatchReasonPhoneExact},
			}},
		},
		&stubMatchAudit{},
	)
	handler.Metrics = metrics

	req := newAuthedMatchRequest("/api/v1/employees/match", `{"phone":"+84901234567","national_id":"079090001234"}`, uuid.New())
	rec := httptest.NewRecorder()
	handler.Match(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200: %s", rec.Code, rec.Body.String())
	}
	if snap := metrics.Snapshot(); snap.MatchTierHard != 1 {
		t.Fatalf("MatchTierHard = %d, want 1", snap.MatchTierHard)
	}
	logged := logBuf.String()
	for _, forbidden := range []string{"+84901234567", "079090001234", "Nguyen Van An", "EMP-990"} {
		if strings.Contains(logged, forbidden) {
			t.Fatalf("match metric log leaked sensitive value %q: %s", forbidden, logged)
		}
	}
}
