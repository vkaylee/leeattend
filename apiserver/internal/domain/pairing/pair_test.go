package pairing

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestPairAnchorFirst_TwoPunchesWithinGap(t *testing.T) {
	t1 := time.Date(2026, 10, 19, 9, 0, 0, 0, time.UTC)
	t2 := time.Date(2026, 10, 19, 17, 0, 0, 0, time.UTC)
	logs := []LogEvent{
		{ID: uuid.New(), Minutes: 540, PunchTime: t1},
		{ID: uuid.New(), Minutes: 1020, PunchTime: t2},
	}
	sessions := PairAnchorFirst(logs, nil)
	if len(sessions) != 1 {
		t.Fatalf("expected 1 session, got %d", len(sessions))
	}
	s := sessions[0]
	if s.StartMinutes != 540 || s.EndMinutes == nil || *s.EndMinutes != 1020 {
		t.Fatalf("unexpected minutes: %+v", s)
	}
	if s.Note != nil {
		t.Fatalf("unexpected note: %v", *s.Note)
	}
}

func TestPairAnchorFirst_MissingPunchOut(t *testing.T) {
	t1 := time.Date(2026, 10, 19, 9, 0, 0, 0, time.UTC)
	logs := []LogEvent{
		{ID: uuid.New(), Minutes: 540, PunchTime: t1},
	}
	sessions := PairAnchorFirst(logs, nil)
	if len(sessions) != 1 {
		t.Fatalf("expected 1 session, got %d", len(sessions))
	}
	if sessions[0].EndMinutes != nil || sessions[0].Note == nil || *sessions[0].Note != "missing_punch_out" {
		t.Fatalf("expected missing_punch_out note, got %+v", sessions[0])
	}
}

func TestPairByShiftWindow_FindsBoundaryPunches(t *testing.T) {
	t1 := time.Date(2026, 10, 19, 8, 55, 0, 0, time.UTC)
	t2 := time.Date(2026, 10, 19, 17, 5, 0, 0, time.UTC)
	logs := []LogEvent{
		{ID: uuid.New(), Minutes: 535, PunchTime: t1},
		{ID: uuid.New(), Minutes: 1025, PunchTime: t2},
	}
	shift := ShiftConfig{
		StartMinutes: 540,
		EndMinutes:   1020,
	}
	sessions := PairByShiftWindow(logs, shift)
	if len(sessions) != 1 {
		t.Fatalf("expected 1 session, got %d", len(sessions))
	}
	if sessions[0].StartMinutes != 535 || sessions[0].EndMinutes == nil || *sessions[0].EndMinutes != 1025 {
		t.Fatalf("unexpected session: %+v", sessions[0])
	}
}
