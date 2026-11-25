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

func TestPairMulti_PairsAllSplitSegmentsWithoutReusingPunches(t *testing.T) {
	day := time.Date(2026, 11, 20, 0, 0, 0, 0, time.UTC)
	seg1ID, seg2ID := uuid.New(), uuid.New()
	seg1Start, seg1End := day.Add(8*time.Hour), day.Add(12*time.Hour)
	seg2Start, seg2End := day.Add(14*time.Hour), day.Add(18*time.Hour)
	// Out-of-order punches + a duplicate bounce scan 20s after morning check-in.
	logs := []LogEvent{
		{ID: uuid.New(), Minutes: 1080, PunchTime: day.Add(18 * time.Hour)},
		{ID: uuid.New(), Minutes: 720, PunchTime: day.Add(12 * time.Hour)},
		{ID: uuid.New(), Minutes: 480, PunchTime: day.Add(8*time.Hour + 20*time.Second)},
		{ID: uuid.New(), Minutes: 840, PunchTime: day.Add(14 * time.Hour)},
		{ID: uuid.New(), Minutes: 480, PunchTime: day.Add(8 * time.Hour)},
	}
	sessions := PairMulti(logs, &MultiSegmentConfig{Segments: []ShiftConfig{
		{StartMinutes: 480, EndMinutes: 720, StartTime: &seg1Start, EndTime: &seg1End, SegmentID: &seg1ID},
		{StartMinutes: 840, EndMinutes: 1080, StartTime: &seg2Start, EndTime: &seg2End, SegmentID: &seg2ID},
	}})
	if len(sessions) != 2 {
		t.Fatalf("expected 2 split-shift sessions, got %d: %+v", len(sessions), sessions)
	}
	assertPairedSegment(t, sessions[0], seg1ID, day.Add(8*time.Hour), day.Add(12*time.Hour))
	assertPairedSegment(t, sessions[1], seg2ID, day.Add(14*time.Hour), day.Add(18*time.Hour))
}

func assertPairedSegment(t *testing.T, s PairedSession, wantSeg uuid.UUID, wantStart, wantEnd time.Time) {
	t.Helper()
	if s.SegmentID == nil || *s.SegmentID != wantSeg {
		t.Fatalf("expected segment %s, got %v", wantSeg, s.SegmentID)
	}
	if !s.StartTime.Equal(wantStart) || s.EndTime == nil || !s.EndTime.Equal(wantEnd) {
		t.Fatalf("unexpected session bounds: %+v", s)
	}
}

func TestPairMulti_SplitShiftMissingMorningOutDoesNotStealAfternoonIn(t *testing.T) {
	day := time.Date(2026, 11, 20, 0, 0, 0, 0, time.UTC)
	seg1ID, seg2ID := uuid.New(), uuid.New()
	seg1Start, seg1End := day.Add(8*time.Hour), day.Add(12*time.Hour)
	seg2Start, seg2End := day.Add(14*time.Hour), day.Add(18*time.Hour)
	logs := []LogEvent{
		{ID: uuid.New(), Minutes: 485, PunchTime: day.Add(8*time.Hour + 5*time.Minute)},
		{ID: uuid.New(), Minutes: 840, PunchTime: day.Add(14 * time.Hour)},
		{ID: uuid.New(), Minutes: 1080, PunchTime: day.Add(18 * time.Hour)},
	}
	sessions := PairMulti(logs, &MultiSegmentConfig{Segments: []ShiftConfig{
		{StartMinutes: 480, EndMinutes: 720, StartTime: &seg1Start, EndTime: &seg1End, SegmentID: &seg1ID},
		{StartMinutes: 840, EndMinutes: 1080, StartTime: &seg2Start, EndTime: &seg2End, SegmentID: &seg2ID},
	}})
	if len(sessions) != 2 {
		t.Fatalf("expected 2 sessions (open morning + closed afternoon), got %d: %+v", len(sessions), sessions)
	}
	if sessions[0].EndTime != nil || sessions[0].Note == nil || *sessions[0].Note != "missing_punch_out" {
		t.Fatalf("expected open morning session with missing_punch_out, got %+v", sessions[0])
	}
	if sessions[1].EndTime == nil || !sessions[1].StartTime.Equal(day.Add(14*time.Hour)) || !sessions[1].EndTime.Equal(day.Add(18*time.Hour)) {
		t.Fatalf("expected afternoon session 14:00-18:00 intact, got %+v", sessions[1])
	}
}

func TestPairMulti_OvernightAbsoluteTimeWindow(t *testing.T) {
	day := time.Date(2026, 11, 20, 0, 0, 0, 0, time.UTC)
	segID := uuid.New()
	segStart := day.Add(22 * time.Hour)
	segEnd := day.Add(30 * time.Hour) // 06:00 next day
	logs := []LogEvent{
		{ID: uuid.New(), Minutes: 1315, PunchTime: day.Add(21*time.Hour + 55*time.Minute)},
		{ID: uuid.New(), Minutes: 365, PunchTime: day.Add(30*time.Hour + 5*time.Minute)},
	}
	sessions := PairMulti(logs, &MultiSegmentConfig{Segments: []ShiftConfig{
		{StartMinutes: 1320, EndMinutes: 360, StartTime: &segStart, EndTime: &segEnd, SegmentID: &segID},
	}})
	if len(sessions) != 1 {
		t.Fatalf("expected 1 overnight session, got %d", len(sessions))
	}
	if sessions[0].EndTime == nil || !sessions[0].EndTime.Equal(day.Add(30*time.Hour+5*time.Minute)) {
		t.Fatalf("expected overnight end time next morning, got %+v", sessions[0])
	}
}
