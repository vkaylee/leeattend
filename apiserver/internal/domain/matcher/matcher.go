package matcher

import (
	"time"
)

// ComputeLateMinutes calculates tardiness in minutes. If isHoliday is true or
// arrival is within grace period, late minutes is 0.
func ComputeLateMinutes(punchIn time.Time, shiftStart time.Time, graceMinutes int, isHoliday bool) int64 {
	if isHoliday {
		return 0
	}
	graceThreshold := shiftStart.Add(time.Duration(graceMinutes) * time.Minute)
	if !punchIn.After(graceThreshold) {
		return 0
	}
	diff := punchIn.Sub(shiftStart)
	if diff <= 0 {
		return 0
	}
	return int64(diff / time.Minute)
}

// ComputeEarlyMinutes calculates early departure in minutes. If isHoliday is true or
// departure is on/after shift end, early minutes is 0.
func ComputeEarlyMinutes(punchOut time.Time, shiftEnd time.Time, isHoliday bool) int64 {
	if isHoliday {
		return 0
	}
	if !punchOut.Before(shiftEnd) {
		return 0
	}
	diff := shiftEnd.Sub(punchOut)
	if diff <= 0 {
		return 0
	}
	return int64(diff / time.Minute)
}

// DetectAbsence returns true if late or early minutes indicate a partial absence/violation.
func DetectAbsence(lateMinutes, earlyMinutes int64) bool {
	return lateMinutes > 0 || earlyMinutes > 0
}

// ResolveShiftWindow converts minute offsets into local boundary times,
// advancing to the next calendar day when shift ends on or before start time (overnight).
func ResolveShiftWindow(loc *time.Location, date time.Time, startMinutes, endMinutes int32) (time.Time, time.Time) {
	if loc == nil {
		loc = time.UTC
	}
	y, m, d := date.Date()
	midnight := time.Date(y, m, d, 0, 0, 0, 0, loc)
	start := midnight.Add(time.Duration(startMinutes) * time.Minute)
	end := midnight.Add(time.Duration(endMinutes) * time.Minute)
	if endMinutes <= startMinutes {
		end = end.Add(24 * time.Hour)
	}
	return start, end
}
