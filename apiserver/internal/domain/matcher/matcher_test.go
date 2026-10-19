package matcher

import (
	"testing"
	"time"
)

func TestComputeLateMinutes(t *testing.T) {
	start := time.Date(2026, 10, 19, 9, 0, 0, 0, time.UTC)
	// On time
	if late := ComputeLateMinutes(start, start, 5, false); late != 0 {
		t.Fatalf("expected 0, got %d", late)
	}
	// Within grace (4m)
	if late := ComputeLateMinutes(start.Add(4*time.Minute), start, 5, false); late != 0 {
		t.Fatalf("expected 0 within grace, got %d", late)
	}
	// Past grace (6m late => 6 minutes)
	if late := ComputeLateMinutes(start.Add(6*time.Minute), start, 5, false); late != 6 {
		t.Fatalf("expected 6 minutes late, got %d", late)
	}
	// Holiday => 0
	if late := ComputeLateMinutes(start.Add(20*time.Minute), start, 5, true); late != 0 {
		t.Fatalf("expected 0 on holiday, got %d", late)
	}
}

func TestComputeEarlyMinutes(t *testing.T) {
	end := time.Date(2026, 10, 19, 17, 0, 0, 0, time.UTC)
	// Full day
	if early := ComputeEarlyMinutes(end, end, false); early != 0 {
		t.Fatalf("expected 0, got %d", early)
	}
	// Left 15m early
	if early := ComputeEarlyMinutes(end.Add(-15*time.Minute), end, false); early != 15 {
		t.Fatalf("expected 15, got %d", early)
	}
	// Holiday => 0
	if early := ComputeEarlyMinutes(end.Add(-30*time.Minute), end, true); early != 0 {
		t.Fatalf("expected 0 on holiday, got %d", early)
	}
}

func TestResolveShiftWindow_Overnight(t *testing.T) {
	date := time.Date(2026, 10, 19, 0, 0, 0, 0, time.UTC)
	// 22:00 to 06:00
	start, end := ResolveShiftWindow(time.UTC, date, 1320, 360)
	if start.Day() != 19 || start.Hour() != 22 {
		t.Fatalf("unexpected start: %v", start)
	}
	if end.Day() != 20 || end.Hour() != 6 {
		t.Fatalf("unexpected end for overnight shift: %v", end)
	}
}
