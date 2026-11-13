package repository

import (
	"errors"
	"testing"

	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestClosurePeriodValidation(t *testing.T) {
	start, end, err := closurePeriodDates(2026, 2)
	if err != nil {
		t.Fatalf("valid period rejected: %v", err)
	}
	if start.Format("2006-01-02") != "2026-02-01" || end.Format("2006-01-02") != "2026-02-28" {
		t.Fatalf("unexpected February bounds: %s - %s", start, end)
	}
	if _, _, err := closurePeriodDates(2026, 13); err == nil {
		t.Fatal("invalid month accepted")
	}
	if _, _, err := validateClosureRequest(2026, 2, 0, "close"); !errors.Is(err, errAttendanceClosureInvalidPeriod) {
		t.Fatalf("expected invalid version error, got %v", err)
	}
	if !errors.Is(api.ErrAttendanceClosureValidation, api.ErrAttendanceClosureValidation) {
		t.Fatal("closure validation sentinel is not stable")
	}
}
