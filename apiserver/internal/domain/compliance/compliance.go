package compliance

import (
	"errors"
	"fmt"
	"sort"
	"time"

	"github.com/google/uuid"
)

var (
	ErrHardViolation = errors.New("compliance: hard violation detected")
	ErrSoftViolation = errors.New("compliance: soft violation requires override")
)

type ViolationKind string

const (
	ViolationOverlap         ViolationKind = "overlap"
	ViolationRestPeriod      ViolationKind = "rest_period"
	ViolationWeeklyHours     ViolationKind = "weekly_hours"
	ViolationMonthlyOvertime ViolationKind = "monthly_overtime"
)

type ViolationSeverity string

const (
	SeverityHard ViolationSeverity = "hard"
	SeveritySoft ViolationSeverity = "soft"
)

type Rules struct {
	IsEnabled            bool
	MinRestPeriodHours   int
	MaxWeeklyHours       int
	MaxMonthlyOTHours    int
	AllowManagerOverride bool
}

type Interval struct {
	StartTime            string
	EndTime              string
	BreakDurationMinutes int
	IsBreakAutoDeduct    bool
}

type Roster struct {
	EmploymentID uuid.UUID
	ShiftID      *uuid.UUID
	Date         string
}

type Request struct {
	EmploymentID   uuid.UUID
	ShiftID        uuid.UUID
	Dates          []string
	AllowWarnings  bool
	OverrideReason *string
}

type Violation struct {
	Type             ViolationKind     `json:"violation_type"`
	Severity         ViolationSeverity `json:"severity"`
	Message          string            `json:"message"`
	ConflictingDates []string          `json:"conflicting_dates"`
	ShiftIDs         []uuid.UUID       `json:"shift_ids"`
}

type Result struct {
	Compliant        bool        `json:"compliant"`
	Violations       []Violation `json:"violations"`
	RequiresOverride bool        `json:"requires_override"`
}

func Validate(rules Rules, request Request, existing []Roster, shifts map[uuid.UUID][]Interval) (Result, error) {
	if !rules.IsEnabled {
		return Result{Compliant: true, Violations: nil, RequiresOverride: false}, nil
	}

	dates := uniqueSortedDates(request.Dates)
	if len(dates) == 0 {
		return Result{Compliant: true, Violations: nil, RequiresOverride: false}, nil
	}

	violations := make([]Violation, 0)

	checkOverlaps(request, dates, existing, shifts, &violations)
	checkRestPeriod(request, dates, existing, shifts, rules.MinRestPeriodHours, &violations)
	checkWeeklyHours(request, dates, existing, shifts, rules.MaxWeeklyHours, &violations)
	checkMonthlyOT(request, dates, existing, shifts, rules.MaxMonthlyOTHours, &violations)

	hasHard := false
	hasSoft := false
	for _, v := range violations {
		if v.Severity == SeverityHard {
			hasHard = true
		} else if v.Severity == SeveritySoft {
			hasSoft = true
		}
	}

	if hasHard {
		return Result{Compliant: false, Violations: violations, RequiresOverride: false}, ErrHardViolation
	}

	if hasSoft {
		if request.AllowWarnings && request.OverrideReason != nil && *request.OverrideReason != "" {
			return Result{Compliant: true, Violations: violations, RequiresOverride: false}, nil
		}
		return Result{Compliant: false, Violations: violations, RequiresOverride: true}, ErrSoftViolation
	}

	return Result{Compliant: true, Violations: nil, RequiresOverride: false}, nil
}

func checkOverlaps(req Request, dates []string, existing []Roster, shifts map[uuid.UUID][]Interval, violations *[]Violation) {
	newIntervals := shifts[req.ShiftID]
	for _, date := range dates {
		for _, r := range existing {
			if r.Date != date || r.ShiftID == nil || *r.ShiftID == req.ShiftID {
				continue
			}
			existingShiftID := *r.ShiftID
			existingIntervals := shifts[existingShiftID]
			if len(newIntervals) == 0 || len(existingIntervals) == 0 {
				*violations = append(*violations, Violation{
					Type:             ViolationOverlap,
					Severity:         SeverityHard,
					Message:          "Shift overlap detected",
					ConflictingDates: []string{date},
					ShiftIDs:         []uuid.UUID{req.ShiftID, existingShiftID},
				})
				continue
			}
			overlap := false
			for _, ni := range newIntervals {
				for _, ei := range existingIntervals {
					if intervalsOverlap(ni.StartTime, ni.EndTime, ei.StartTime, ei.EndTime) {
						overlap = true
						break
					}
				}
				if overlap {
					break
				}
			}
			if overlap {
				*violations = append(*violations, Violation{
					Type:             ViolationOverlap,
					Severity:         SeverityHard,
					Message:          "Shift overlap detected",
					ConflictingDates: []string{date},
					ShiftIDs:         []uuid.UUID{req.ShiftID, existingShiftID},
				})
			}
		}
	}
}

func checkRestPeriod(req Request, dates []string, existing []Roster, shifts map[uuid.UUID][]Interval, minRestHours int, violations *[]Violation) {
	if minRestHours <= 0 {
		return
	}
	minRestMinutes := minRestHours * 60
	newIntervals := shifts[req.ShiftID]
	if len(newIntervals) == 0 {
		return
	}

	newStartMinutes, newEndMinutes := shiftStartEndMinutes(newIntervals)

	for _, dateStr := range dates {
		currentDate, err := time.Parse("2006-01-02", dateStr)
		if err != nil {
			continue
		}
		prevDateStr := currentDate.AddDate(0, 0, -1).Format("2006-01-02")
		nextDateStr := currentDate.AddDate(0, 0, 1).Format("2006-01-02")

		for _, r := range existing {
			if r.ShiftID == nil {
				continue
			}
			if r.Date == prevDateStr {
				prevIntervals := shifts[*r.ShiftID]
				if len(prevIntervals) > 0 {
					_, prevEnd := shiftStartEndMinutes(prevIntervals)
					gap := restGapMinutes(prevEnd, newStartMinutes)
					if gap < minRestMinutes {
						*violations = append(*violations, Violation{
							Type:             ViolationRestPeriod,
							Severity:         SeveritySoft,
							Message:          fmt.Sprintf("Insufficient rest: %dh gap, %dh required", gap/60, minRestHours),
							ConflictingDates: []string{prevDateStr, dateStr},
							ShiftIDs:         []uuid.UUID{req.ShiftID, *r.ShiftID},
						})
					}
				}
			}
			if r.Date == nextDateStr {
				nextIntervals := shifts[*r.ShiftID]
				if len(nextIntervals) > 0 {
					nextStart, _ := shiftStartEndMinutes(nextIntervals)
					gap := restGapMinutes(newEndMinutes, nextStart)
					if gap < minRestMinutes {
						*violations = append(*violations, Violation{
							Type:             ViolationRestPeriod,
							Severity:         SeveritySoft,
							Message:          fmt.Sprintf("Insufficient rest: %dh gap, %dh required", gap/60, minRestHours),
							ConflictingDates: []string{dateStr, nextDateStr},
							ShiftIDs:         []uuid.UUID{req.ShiftID, *r.ShiftID},
						})
					}
				}
			}
		}
	}
}

func checkWeeklyHours(req Request, dates []string, existing []Roster, shifts map[uuid.UUID][]Interval, maxWeeklyHours int, violations *[]Violation) {
	if maxWeeklyHours <= 0 {
		return
	}
	maxWeeklyMinutes := maxWeeklyHours * 60
	weeksEvaluated := make(map[string]struct{})

	for _, dateStr := range dates {
		d, err := time.Parse("2006-01-02", dateStr)
		if err != nil {
			continue
		}
		mon := mondayOfWeek(d)
		monStr := mon.Format("2006-01-02")
		if _, ok := weeksEvaluated[monStr]; ok {
			continue
		}
		weeksEvaluated[monStr] = struct{}{}
		sun := mon.AddDate(0, 0, 6)

		totalMinutes := 0
		for _, r := range existing {
			if r.ShiftID == nil {
				continue
			}
			rd, err := time.Parse("2006-01-02", r.Date)
			if err != nil || rd.Before(mon) || rd.After(sun) {
				continue
			}
			for _, iv := range shifts[*r.ShiftID] {
				totalMinutes += intervalMinutes(iv)
			}
		}

		for _, reqDateStr := range dates {
			rd, err := time.Parse("2006-01-02", reqDateStr)
			if err != nil || rd.Before(mon) || rd.After(sun) {
				continue
			}
			for _, iv := range shifts[req.ShiftID] {
				totalMinutes += intervalMinutes(iv)
			}
		}

		if totalMinutes > maxWeeklyMinutes {
			*violations = append(*violations, Violation{
				Type:             ViolationWeeklyHours,
				Severity:         SeveritySoft,
				Message:          fmt.Sprintf("Weekly hours exceeded: %dh, limit: %dh", totalMinutes/60, maxWeeklyHours),
				ConflictingDates: []string{monStr, sun.Format("2006-01-02")},
				ShiftIDs:         []uuid.UUID{req.ShiftID},
			})
		}
	}
}

func checkMonthlyOT(req Request, dates []string, existing []Roster, shifts map[uuid.UUID][]Interval, maxMonthlyOTHours int, violations *[]Violation) {
	if maxMonthlyOTHours <= 0 {
		return
	}
	maxMonthlyOTMinutes := maxMonthlyOTHours * 60
	monthsEvaluated := make(map[string]struct{})

	for _, dateStr := range dates {
		d, err := time.Parse("2006-01-02", dateStr)
		if err != nil {
			continue
		}
		monthKey := d.Format("2006-01")
		if _, ok := monthsEvaluated[monthKey]; ok {
			continue
		}
		monthsEvaluated[monthKey] = struct{}{}

		monthStart := time.Date(d.Year(), d.Month(), 1, 0, 0, 0, 0, time.UTC)
		monthEnd := monthStart.AddDate(0, 1, -1)

		totalOTMinutes := 0

		for _, r := range existing {
			if r.ShiftID == nil {
				continue
			}
			rd, err := time.Parse("2006-01-02", r.Date)
			if err != nil || rd.Before(monthStart) || rd.After(monthEnd) {
				continue
			}
			dailyMinutes := 0
			for _, iv := range shifts[*r.ShiftID] {
				dailyMinutes += intervalMinutes(iv)
			}
			if dailyMinutes > 480 {
				totalOTMinutes += (dailyMinutes - 480)
			}
		}

		for _, reqDateStr := range dates {
			rd, err := time.Parse("2006-01-02", reqDateStr)
			if err != nil || rd.Before(monthStart) || rd.After(monthEnd) {
				continue
			}
			dailyMinutes := 0
			for _, iv := range shifts[req.ShiftID] {
				dailyMinutes += intervalMinutes(iv)
			}
			if dailyMinutes > 480 {
				totalOTMinutes += (dailyMinutes - 480)
			}
		}

		if totalOTMinutes > maxMonthlyOTMinutes {
			*violations = append(*violations, Violation{
				Type:             ViolationMonthlyOvertime,
				Severity:         SeveritySoft,
				Message:          fmt.Sprintf("Monthly OT exceeded: %dh OT, limit: %dh", totalOTMinutes/60, maxMonthlyOTHours),
				ConflictingDates: []string{monthStart.Format("2006-01-02"), monthEnd.Format("2006-01-02")},
				ShiftIDs:         []uuid.UUID{req.ShiftID},
			})
		}
	}
}

func shiftStartEndMinutes(intervals []Interval) (int, int) {
	if len(intervals) == 0 {
		return 0, 0
	}
	minStart := 1440
	maxEnd := 0
	for _, iv := range intervals {
		sm, em := parseIntervalMinutes(iv.StartTime, iv.EndTime)
		if sm < minStart {
			minStart = sm
		}
		if em <= sm {
			em += 1440
		}
		if em > maxEnd {
			maxEnd = em
		}
	}
	return minStart, maxEnd
}

func restGapMinutes(prevEndMinutes, nextStartMinutes int) int {
	pe := prevEndMinutes
	if pe < 720 {
		pe += 1440
	}
	nextStartAdjusted := nextStartMinutes + 1440
	gap := nextStartAdjusted - pe
	if gap < 0 {
		return 0
	}
	return gap
}

func intervalMinutes(iv Interval) int {
	sm, em := parseIntervalMinutes(iv.StartTime, iv.EndTime)
	duration := em - sm
	if duration <= 0 {
		duration += 1440
	}
	if iv.IsBreakAutoDeduct && iv.BreakDurationMinutes > 0 {
		duration -= iv.BreakDurationMinutes
	}
	if duration < 0 {
		return 0
	}
	return duration
}

func intervalsOverlap(s1, e1, s2, e2 string) bool {
	a := intervalSegments(s1, e1)
	b := intervalSegments(s2, e2)
	for _, left := range a {
		for _, right := range b {
			if left[0] < right[1] && right[0] < left[1] {
				return true
			}
		}
	}
	return false
}

func intervalSegments(start, end string) [][2]int {
	sm, em := parseIntervalMinutes(start, end)
	if em > sm {
		return [][2]int{{sm, em}}
	}
	return [][2]int{{sm, 1440}, {0, em}}
}

func parseIntervalMinutes(start, end string) (int, int) {
	t1, err1 := time.Parse("15:04:05", start)
	t2, err2 := time.Parse("15:04:05", end)
	if err1 != nil || err2 != nil {
		return 0, 0
	}
	sm := t1.Hour()*60 + t1.Minute()
	em := t2.Hour()*60 + t2.Minute()
	return sm, em
}

func uniqueSortedDates(raw []string) []string {
	set := make(map[string]struct{}, len(raw))
	out := make([]string, 0, len(raw))
	for _, d := range raw {
		if _, ok := set[d]; !ok {
			set[d] = struct{}{}
			out = append(out, d)
		}
	}
	sort.Strings(out)
	return out
}

func mondayOfWeek(t time.Time) time.Time {
	offset := int(t.Weekday()+6) % 7
	return t.AddDate(0, 0, -offset)
}
