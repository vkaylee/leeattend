package api

import (
	"net/http/httptest"
	"testing"
	"time"
)

func TestDecodeMonitorJob(t *testing.T) {
	fallback := time.Date(2026, 1, 2, 3, 4, 5, 0, time.UTC)
	if _, ok := decodeMonitorJob("{", fallback); ok {
		t.Fatal("invalid JSON must be rejected")
	}
	job, ok := decodeMonitorJob(`{"status":"completed","created_at":"invalid","completed_at":"2026-01-03T03:04:05Z"}`, fallback)
	if !ok || job.Status != "completed" || !job.CreatedAt.Equal(fallback) || job.CompletedAt == nil || !job.CompletedAt.Equal(fallback.AddDate(0, 0, 1)) {
		t.Fatalf("unexpected job: %+v (ok=%v)", job, ok)
	}
}

func TestMonitorQueryHelpers(t *testing.T) {
	if got := parseMonitorLimit("", 20, 100); got != 20 {
		t.Fatalf("empty limit = %d, want 20", got)
	}
	if got := parseMonitorLimit("250", 20, 100); got != 100 {
		t.Fatalf("capped limit = %d, want 100", got)
	}
	if got := parseMonitorLimit("-1", 20, 100); got != 20 {
		t.Fatalf("invalid limit = %d, want 20", got)
	}
	if !isValidTraceID("trace_01-ab.CD") || isValidTraceID("bad space") {
		t.Fatal("unexpected trace ID validation result")
	}
	req := httptest.NewRequest("GET", "/?status=invalid", nil)
	if _, ok := parseMonitorJobsQuery(req); ok {
		t.Fatal("expected invalid status rejection")
	}
}
