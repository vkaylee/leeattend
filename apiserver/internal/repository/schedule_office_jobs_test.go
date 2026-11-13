package repository

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestHorizonCoversTodayThroughNextCalendarMonth(t *testing.T) {
	start, end := horizon(time.Date(2026, 12, 15, 10, 30, 0, 0, time.FixedZone("test", 7*60*60)))
	if got := start.Format("2006-01-02"); got != "2026-12-15" {
		t.Fatalf("start = %s", got)
	}
	if got := end.Format("2006-01-02"); got != "2027-01-31" {
		t.Fatalf("end = %s", got)
	}
}

func TestUnsupportedScheduleOfficePolicySourceFailsClosed(t *testing.T) {
	_, err := (UnsupportedScheduleOfficePolicySource{}).ApprovedEmployers(context.Background(), nil, time.Now())
	if !errors.Is(err, ErrScheduleOfficePolicyUnavailable) {
		t.Fatalf("expected typed unavailable error, got %v", err)
	}
}

func TestValidateScheduleOfficeEventRejectsNonOfficeAndMalformedDate(t *testing.T) {
	event := ScheduleOfficeCoverageEvent{ApprovalInstanceID: uuid.New(), EmployerID: uuid.New(), EmploymentID: uuid.New(), BusinessDate: "2026-02-30", Mode: "office"}
	if err := validateScheduleOfficeEvent(event); err == nil {
		t.Fatal("expected invalid date error")
	}
	event.BusinessDate = "2026-02-01"
	event.Mode = "rotating"
	if err := validateScheduleOfficeEvent(event); err == nil {
		t.Fatal("expected non-office error")
	}
}
