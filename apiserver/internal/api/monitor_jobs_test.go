package api

import (
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
