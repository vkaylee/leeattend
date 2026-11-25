package compliance

import (
	"errors"
	"fmt"
	"sort"
	"strings"
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
	StartDayOffset       int
	EndDayOffset         int
}

type Roster struct {
	EmploymentID uuid.UUID
	ShiftID      *uuid.UUID
	Date         string
	Intervals    []Interval
}

type Request struct {
	EmploymentID    uuid.UUID
	ShiftID         uuid.UUID
	Dates           []string
	IntervalsByDate map[string][]Interval
	CandidateDays   []Roster
	AllowWarnings   bool
	OverrideReason  *string
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

type timeWindow struct {
	start time.Time
	end   time.Time
}

func Validate(rules Rules, request Request, existing []Roster, shifts map[uuid.UUID][]Interval) (Result, error) {
	if !rules.IsEnabled {
		return Result{Compliant: true}, nil
	}

	candidates := buildCandidates(request, shifts)
	if len(candidates) == 0 {
		return Result{Compliant: true}, nil
	}

	violations := make([]Violation, 0)
	checkOverlaps(candidates, existing, shifts, &violations)
	checkRestPeriod(candidates, existing, shifts, rules.MinRestPeriodHours, &violations)
	checkWeeklyHours(candidates, existing, shifts, rules.MaxWeeklyHours, &violations)
	checkMonthlyOT(candidates, existing, shifts, rules.MaxMonthlyOTHours, &violations)

	return evaluateComplianceResult(rules, request, violations)
}

func evaluateComplianceResult(rules Rules, request Request, violations []Violation) (Result, error) {
	hasHard, hasSoft := classifyViolations(violations)
	if hasHard {
		return Result{Compliant: false, Violations: violations, RequiresOverride: false}, ErrHardViolation
	}
	if !hasSoft {
		return Result{Compliant: true}, nil
	}
	if rules.AllowManagerOverride && request.AllowWarnings && request.OverrideReason != nil && strings.TrimSpace(*request.OverrideReason) != "" {
		return Result{Compliant: true, Violations: violations, RequiresOverride: false}, nil
	}
	if !rules.AllowManagerOverride {
		return Result{Compliant: false, Violations: violations, RequiresOverride: false}, ErrHardViolation
	}
	return Result{Compliant: false, Violations: violations, RequiresOverride: true}, ErrSoftViolation
}

func classifyViolations(violations []Violation) (hasHard, hasSoft bool) {
	for _, v := range violations {
		if v.Severity == SeverityHard {
			hasHard = true
		} else if v.Severity == SeveritySoft {
			hasSoft = true
		}
	}
	return hasHard, hasSoft
}

func buildCandidates(request Request, shifts map[uuid.UUID][]Interval) []Roster {
	if len(request.CandidateDays) > 0 {
		out := make([]Roster, 0, len(request.CandidateDays))
		for _, item := range request.CandidateDays {
			c := item
			if c.EmploymentID == uuid.Nil {
				c.EmploymentID = request.EmploymentID
			}
			if len(c.Intervals) == 0 && c.ShiftID != nil {
				c.Intervals = shifts[*c.ShiftID]
			}
			out = append(out, c)
		}
		sort.Slice(out, func(i, j int) bool { return out[i].Date < out[j].Date })
		return out
	}
	dates := uniqueSortedDates(request.Dates)
	out := make([]Roster, 0, len(dates))
	for _, date := range dates {
		shiftID := request.ShiftID
		ivs := request.IntervalsByDate[date]
		if len(ivs) == 0 {
			ivs = shifts[shiftID]
		}
		out = append(out, Roster{
			EmploymentID: request.EmploymentID,
			ShiftID:      &shiftID,
			Date:         date,
			Intervals:    ivs,
		})
	}
	return out
}

func resolveIntervals(r Roster, shifts map[uuid.UUID][]Interval) []Interval {
	if len(r.Intervals) > 0 {
		return r.Intervals
	}
	if r.ShiftID != nil {
		return shifts[*r.ShiftID]
	}
	return nil
}

func hasShiftData(r Roster) bool {
	return r.ShiftID != nil || len(r.Intervals) > 0
}

func isSameEmploymentSameShiftSameDay(a, b Roster) bool {
	if a.EmploymentID != b.EmploymentID || a.Date != b.Date {
		return false
	}
	return a.ShiftID != nil && b.ShiftID != nil && *a.ShiftID == *b.ShiftID
}

func collectShiftIDs(a, b *uuid.UUID) []uuid.UUID {
	out := make([]uuid.UUID, 0, 2)
	if a != nil && *a != uuid.Nil {
		out = append(out, *a)
	}
	if b != nil && *b != uuid.Nil && (len(out) == 0 || out[0] != *b) {
		out = append(out, *b)
	}
	return out
}

func orderedDates(a, b string) []string {
	if a == b {
		return []string{a}
	}
	if a < b {
		return []string{a, b}
	}
	return []string{b, a}
}

func compareOverlapPair(left, right Roster, shifts map[uuid.UUID][]Interval) *Violation {
	if isSameEmploymentSameShiftSameDay(left, right) {
		return nil
	}
	leftIvs := resolveIntervals(left, shifts)
	rightIvs := resolveIntervals(right, shifts)
	if left.Date == right.Date && (len(leftIvs) == 0 || len(rightIvs) == 0) {
		if hasShiftData(left) && hasShiftData(right) {
			return &Violation{
				Type:             ViolationOverlap,
				Severity:         SeverityHard,
				Message:          "Shift overlap detected",
				ConflictingDates: []string{left.Date},
				ShiftIDs:         collectShiftIDs(left.ShiftID, right.ShiftID),
			}
		}
		return nil
	}
	if !windowsOverlap(rosterWindows(left.Date, leftIvs), rosterWindows(right.Date, rightIvs)) {
		return nil
	}
	return &Violation{
		Type:             ViolationOverlap,
		Severity:         SeverityHard,
		Message:          "Shift overlap detected",
		ConflictingDates: orderedDates(left.Date, right.Date),
		ShiftIDs:         collectShiftIDs(left.ShiftID, right.ShiftID),
	}
}

func checkOverlaps(candidates []Roster, existing []Roster, shifts map[uuid.UUID][]Interval, violations *[]Violation) {
	for i, cand := range candidates {
		for _, r := range existing {
			if !hasShiftData(r) {
				continue
			}
			if v := compareOverlapPair(cand, r, shifts); v != nil {
				*violations = append(*violations, *v)
			}
		}
		for j := i + 1; j < len(candidates); j++ {
			if v := compareOverlapPair(cand, candidates[j], shifts); v != nil {
				*violations = append(*violations, *v)
			}
		}
	}
}

func nonOverlappingRestGapMinutes(left, right []timeWindow) (int, bool) {
	if len(left) == 0 || len(right) == 0 || windowsOverlap(left, right) {
		return 0, false
	}
	lStart, lEnd := windowSpan(left)
	rStart, rEnd := windowSpan(right)
	if !rEnd.After(lStart) {
		return int(lStart.Sub(rEnd).Minutes()), true
	}
	if !lEnd.After(rStart) {
		return int(rStart.Sub(lEnd).Minutes()), true
	}
	return 0, false
}

func compareRestPair(left, right Roster, shifts map[uuid.UUID][]Interval, minRestHours int) *Violation {
	if isSameEmploymentSameShiftSameDay(left, right) {
		return nil
	}
	leftDate, errL := time.Parse("2006-01-02", left.Date)
	rightDate, errR := time.Parse("2006-01-02", right.Date)
	if errL != nil || errR != nil {
		return nil
	}
	dayDiff := int(leftDate.Sub(rightDate) / (24 * time.Hour))
	if dayDiff < -1 || dayDiff > 1 {
		return nil
	}
	gap, ok := nonOverlappingRestGapMinutes(
		rosterWindows(left.Date, resolveIntervals(left, shifts)),
		rosterWindows(right.Date, resolveIntervals(right, shifts)),
	)
	if !ok || gap >= minRestHours*60 {
		return nil
	}
	return &Violation{
		Type:             ViolationRestPeriod,
		Severity:         SeveritySoft,
		Message:          fmt.Sprintf("Insufficient rest: %dh gap, %dh required", gap/60, minRestHours),
		ConflictingDates: orderedDates(left.Date, right.Date),
		ShiftIDs:         collectShiftIDs(left.ShiftID, right.ShiftID),
	}
}

func checkRestPeriod(candidates []Roster, existing []Roster, shifts map[uuid.UUID][]Interval, minRestHours int, violations *[]Violation) {
	if minRestHours <= 0 {
		return
	}
	for i, cand := range candidates {
		for _, r := range existing {
			if !hasShiftData(r) {
				continue
			}
			if v := compareRestPair(cand, r, shifts, minRestHours); v != nil {
				*violations = append(*violations, *v)
			}
		}
		for j := i + 1; j < len(candidates); j++ {
			if v := compareRestPair(cand, candidates[j], shifts, minRestHours); v != nil {
				*violations = append(*violations, *v)
			}
		}
	}
}

func candidateDayKeys(candidates []Roster) map[string]struct{} {
	keys := make(map[string]struct{}, len(candidates))
	for _, c := range candidates {
		keys[c.EmploymentID.String()+":"+c.Date] = struct{}{}
	}
	return keys
}

func rosterIntervalsMinutes(r Roster, shifts map[uuid.UUID][]Interval) int {
	total := 0
	for _, iv := range resolveIntervals(r, shifts) {
		total += intervalMinutes(iv)
	}
	return total
}

func appendUniqueShiftID(shiftIDs []uuid.UUID, seen map[uuid.UUID]struct{}, shiftID *uuid.UUID) []uuid.UUID {
	if shiftID == nil || *shiftID == uuid.Nil {
		return shiftIDs
	}
	if _, ok := seen[*shiftID]; ok {
		return shiftIDs
	}
	seen[*shiftID] = struct{}{}
	return append(shiftIDs, *shiftID)
}

func rosterInRange(r Roster, start, end time.Time, skipKeys map[string]struct{}) bool {
	if !hasShiftData(r) {
		return false
	}
	if _, replaced := skipKeys[r.EmploymentID.String()+":"+r.Date]; replaced {
		return false
	}
	rd, err := time.Parse("2006-01-02", r.Date)
	return err == nil && !rd.Before(start) && !rd.After(end)
}

func sumRangeMinutes(rosters []Roster, shifts map[uuid.UUID][]Interval, start, end time.Time, skipKeys map[string]struct{}) (int, []uuid.UUID) {
	total := 0
	seen := make(map[uuid.UUID]struct{})
	shiftIDs := make([]uuid.UUID, 0, 2)
	for _, r := range rosters {
		if !rosterInRange(r, start, end, skipKeys) {
			continue
		}
		total += rosterIntervalsMinutes(r, shifts)
		shiftIDs = appendUniqueShiftID(shiftIDs, seen, r.ShiftID)
	}
	return total, shiftIDs
}

func checkWeeklyHours(candidates []Roster, existing []Roster, shifts map[uuid.UUID][]Interval, maxWeeklyHours int, violations *[]Violation) {
	if maxWeeklyHours <= 0 {
		return
	}
	maxWeeklyMinutes := maxWeeklyHours * 60
	weeksEvaluated := make(map[string]struct{})
	replacedKeys := candidateDayKeys(candidates)

	for _, cand := range candidates {
		d, err := time.Parse("2006-01-02", cand.Date)
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

		existingMins, _ := sumRangeMinutes(existing, shifts, mon, sun, replacedKeys)
		candMins, shiftIDs := sumRangeMinutes(candidates, shifts, mon, sun, nil)
		totalMinutes := existingMins + candMins

		if totalMinutes > maxWeeklyMinutes {
			*violations = append(*violations, Violation{
				Type:             ViolationWeeklyHours,
				Severity:         SeveritySoft,
				Message:          fmt.Sprintf("Weekly hours exceeded: %dh, limit: %dh", totalMinutes/60, maxWeeklyHours),
				ConflictingDates: []string{monStr, sun.Format("2006-01-02")},
				ShiftIDs:         shiftIDs,
			})
		}
	}
}

func addDailyMinutesInRange(dailyTotals map[string]int, rosters []Roster, shifts map[uuid.UUID][]Interval, start, end time.Time, skipKeys map[string]struct{}) []uuid.UUID {
	seen := make(map[uuid.UUID]struct{})
	shiftIDs := make([]uuid.UUID, 0, 2)
	for _, r := range rosters {
		if !rosterInRange(r, start, end, skipKeys) {
			continue
		}
		dailyTotals[r.Date] += rosterIntervalsMinutes(r, shifts)
		shiftIDs = appendUniqueShiftID(shiftIDs, seen, r.ShiftID)
	}
	return shiftIDs
}

func checkMonthlyOT(candidates []Roster, existing []Roster, shifts map[uuid.UUID][]Interval, maxMonthlyOTHours int, violations *[]Violation) {
	if maxMonthlyOTHours <= 0 {
		return
	}
	maxMonthlyOTMinutes := maxMonthlyOTHours * 60
	monthsEvaluated := make(map[string]struct{})
	replacedKeys := candidateDayKeys(candidates)

	for _, cand := range candidates {
		d, err := time.Parse("2006-01-02", cand.Date)
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

		dailyTotals := make(map[string]int)
		addDailyMinutesInRange(dailyTotals, existing, shifts, monthStart, monthEnd, replacedKeys)
		shiftIDs := addDailyMinutesInRange(dailyTotals, candidates, shifts, monthStart, monthEnd, nil)

		totalOTMinutes := 0
		for _, mins := range dailyTotals {
			if mins > 480 {
				totalOTMinutes += mins - 480
			}
		}

		if totalOTMinutes > maxMonthlyOTMinutes {
			*violations = append(*violations, Violation{
				Type:             ViolationMonthlyOvertime,
				Severity:         SeveritySoft,
				Message:          fmt.Sprintf("Monthly OT exceeded: %dh OT, limit: %dh", totalOTMinutes/60, maxMonthlyOTHours),
				ConflictingDates: []string{monthStart.Format("2006-01-02"), monthEnd.Format("2006-01-02")},
				ShiftIDs:         shiftIDs,
			})
		}
	}
}

func rosterWindows(dateStr string, intervals []Interval) []timeWindow {
	base, err := time.Parse("2006-01-02", dateStr)
	if err != nil || len(intervals) == 0 {
		return nil
	}
	out := make([]timeWindow, 0, len(intervals))
	for _, iv := range intervals {
		sh, sm, ss, okS := parseClock(iv.StartTime)
		eh, em, es, okE := parseClock(iv.EndTime)
		if !okS || !okE {
			continue
		}
		startDay := base.AddDate(0, 0, iv.StartDayOffset)
		endDay := base.AddDate(0, 0, iv.EndDayOffset)
		start := time.Date(startDay.Year(), startDay.Month(), startDay.Day(), sh, sm, ss, 0, time.UTC)
		end := time.Date(endDay.Year(), endDay.Month(), endDay.Day(), eh, em, es, 0, time.UTC)
		if iv.EndDayOffset == iv.StartDayOffset && !end.After(start) {
			end = end.AddDate(0, 0, 1)
		}
		if end.After(start) {
			out = append(out, timeWindow{start: start, end: end})
		}
	}
	return out
}

func windowsOverlap(left, right []timeWindow) bool {
	for _, l := range left {
		for _, r := range right {
			if l.start.Before(r.end) && r.start.Before(l.end) {
				return true
			}
		}
	}
	return false
}

func windowSpan(windows []timeWindow) (time.Time, time.Time) {
	minStart := windows[0].start
	maxEnd := windows[0].end
	for _, w := range windows[1:] {
		if w.start.Before(minStart) {
			minStart = w.start
		}
		if w.end.After(maxEnd) {
			maxEnd = w.end
		}
	}
	return minStart, maxEnd
}

func intervalMinutes(iv Interval) int {
	sh, sm, _, okS := parseClock(iv.StartTime)
	eh, em, _, okE := parseClock(iv.EndTime)
	if !okS || !okE {
		return 0
	}
	startMins := iv.StartDayOffset*1440 + sh*60 + sm
	endMins := iv.EndDayOffset*1440 + eh*60 + em
	duration := endMins - startMins
	if iv.EndDayOffset == iv.StartDayOffset && duration <= 0 {
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

func parseClock(raw string) (hour, minute, second int, ok bool) {
	trimmed := strings.TrimSpace(raw)
	for _, layout := range []string{"15:04:05", "15:04"} {
		if parsed, err := time.Parse(layout, trimmed); err == nil {
			return parsed.Hour(), parsed.Minute(), parsed.Second(), true
		}
	}
	return 0, 0, 0, false
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
