package timezone

import (
	"testing"
	"time"
)

func TestValidateAndParseTimezone(t *testing.T) {
	loc, err := ValidateTZ("Asia/Ho_Chi_Minh")
	if err != nil || loc == nil {
		t.Fatalf("expected valid timezone, got %v", err)
	}
	if ParseTimezone("not/a-zone") != time.UTC {
		t.Fatal("invalid timezone must degrade to UTC")
	}
	if _, err := ValidateTZ("not/a-zone"); err == nil {
		t.Fatal("expected strict validation error")
	}
}

func TestStartOfDayUTC_UsesCalendarBoundaries(t *testing.T) {
	loc := time.FixedZone("UTC+7", 7*60*60)
	date := time.Date(2026, 10, 19, 14, 30, 0, 0, loc)
	start := StartOfDayUTC(loc, date)
	next := StartOfNextDayUTC(loc, date)
	if start.Hour() != 17 || start.Day() != 18 || next.Sub(start) != 24*time.Hour {
		t.Fatalf("unexpected boundaries: %v %v", start, next)
	}
}
