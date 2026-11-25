package buckets

import (
	"errors"
	"sort"
	"time"

	"github.com/google/uuid"
)

type RoundingPolicy string

const (
	RoundingPolicyFloor RoundingPolicy = "floor"
	RoundingPolicyRound RoundingPolicy = "round"
	RoundingPolicyCeil  RoundingPolicy = "ceil"
	RoundingPolicyNone  RoundingPolicy = "none"
)

type OtAggregationMode string

const (
	OtAggregate  OtAggregationMode = "aggregate"
	OtPerSegment OtAggregationMode = "per_segment"
)

type WorkBuckets struct{ StandardMinutes, EarlyOTMinutes, LateOTMinutes, BreakOTMinutes int32 }

type BreakPunchMode string
type BreakCreditMode string

type AttendanceExceptionCode string

const (
	BreakPunchModeNone       BreakPunchMode  = "none"
	BreakPunchModeRequired   BreakPunchMode  = "required"
	BreakCreditModeNone      BreakCreditMode = "none"
	BreakCreditModeScheduled BreakCreditMode = "scheduled"
	BreakCreditModeActual    BreakCreditMode = "actual"

	AttendanceExceptionMissingBreakPair     AttendanceExceptionCode = "missing_break_pair"
	AttendanceExceptionUnexpectedBreakPunch AttendanceExceptionCode = "unexpected_break_punch"
)

var ErrInvalidAttendanceCalculation = errors.New("attendance: invalid calculation input")

type BreakPair struct {
	Start time.Time
	End   time.Time
}

type BreakWorkedInterval struct {
	BreakPair
	Status string
}

type AttendanceCalculationInput struct {
	PunchIn                      time.Time
	PunchOut                     time.Time
	BreakPunchMode               BreakPunchMode
	BreakCreditMode              BreakCreditMode
	ScheduledBreakMinutes        int32
	ScheduledBreakStart          *time.Time
	ScheduledBreakEnd            *time.Time
	ActualBreaks                 []BreakPair
	PlannedBreakWorkedMinutes    int32
	ApprovedBreakWorkedIntervals []BreakWorkedInterval
	ApprovedOvertimeMinutes      int32
}

type AttendanceCalculationResult struct {
	GrossMinutes                int32
	ActualBreakMinutes          int32
	ScheduledUnpaidBreakMinutes int32
	PaidBreakWorkMinutes        int32
	ApprovedOvertimeMinutes     int32
	PayableMinutes              int32
	ExceptionCode               AttendanceExceptionCode
}

type BreakDeductionInput struct {
	ActualBreakMinutes, ScheduledBreakMinutes int32
	IsBreakAutoDeduct, IsBreakOvertime        bool
	BreakStart, BreakEnd, PunchIn, PunchOut   *time.Time
	BreakWindows                              []BreakPair
}
type DailyCalcInput struct {
	PunchIn, PunchOut                                time.Time
	ShiftStart, ShiftEnd                             *time.Time
	ScheduledBreakMinutes, ActualBreakMinutes        int32
	ApprovedBreakWorkedMinutes                       int32
	ActualBreakCount                                 int
	HasOpenActualBreak                               bool
	BreakPunchMode                                   BreakPunchMode
	BreakStart, BreakEnd                             *time.Time
	BreakWindows                                     []BreakPair
	WorkUnit                                         int32
	RoundingPolicy                                   RoundingPolicy
	GracePeriodMinutes                               int32
	EarlyArrivalStrategy                             string
	IsEarlyLeavePaid                                 bool
	AuthorizedEarlyLeave                             int32
	AllowEarlyOT, IsBreakAutoDeduct, IsBreakOvertime bool
	OTCeiling                                        *time.Time
	MaxEarlyOTMinutes                                *int32
	SegmentID                                        *uuid.UUID
	OTAggregationMode                                OtAggregationMode
}
type DayBucketResult struct {
	Buckets              WorkBuckets
	DeductedBreakMinutes int32
	ActualBreakMinutes   int32
	BreakProvisional     bool
	BreakExceptionCode   AttendanceExceptionCode
	StandardUnits        int32
	OTUnits              int32
}
type SegmentBucket struct {
	SegmentID *uuid.UUID
	Buckets   WorkBuckets
}
type SegmentCalcResult struct {
	TotalStandardUnits, TotalOTUnits int32
	PerSegmentBuckets                []SegmentBucket
}

func TruncateToMinute(value time.Time) time.Time { return value.Truncate(time.Minute) }

func CalculateAttendance(input AttendanceCalculationInput) (AttendanceCalculationResult, error) {
	if input.PunchIn.IsZero() || input.PunchOut.IsZero() || !input.PunchOut.After(input.PunchIn) || input.ScheduledBreakMinutes < 0 || input.PlannedBreakWorkedMinutes < 0 || input.ApprovedOvertimeMinutes < 0 {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	if input.BreakPunchMode != BreakPunchModeNone && input.BreakPunchMode != BreakPunchModeRequired {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	if input.BreakCreditMode != BreakCreditModeNone && input.BreakCreditMode != BreakCreditModeScheduled && input.BreakCreditMode != BreakCreditModeActual {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	if input.BreakPunchMode == BreakPunchModeRequired && input.BreakCreditMode != BreakCreditModeActual {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	if input.BreakPunchMode == BreakPunchModeNone && input.BreakCreditMode == BreakCreditModeActual {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	if (input.ScheduledBreakStart == nil) != (input.ScheduledBreakEnd == nil) {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	if input.ScheduledBreakStart != nil && (!input.ScheduledBreakEnd.After(*input.ScheduledBreakStart) || input.ScheduledBreakStart.Before(input.PunchIn) || input.ScheduledBreakEnd.After(input.PunchOut)) {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	approvedIntervals := make([]BreakWorkedInterval, 0, len(input.ApprovedBreakWorkedIntervals))
	for _, interval := range input.ApprovedBreakWorkedIntervals {
		switch interval.Status {
		case "approved":
			approvedIntervals = append(approvedIntervals, interval)
		case "pending", "rejected":
			continue
		default:
			return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
		}
	}
	if len(approvedIntervals) > 0 && (input.BreakPunchMode != BreakPunchModeNone || input.BreakCreditMode != BreakCreditModeScheduled || input.ScheduledBreakStart == nil) {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}

	grossMinutes := int32(input.PunchOut.Sub(input.PunchIn) / time.Minute)
	if input.ScheduledBreakMinutes > grossMinutes {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	result := AttendanceCalculationResult{
		GrossMinutes:            grossMinutes,
		ApprovedOvertimeMinutes: input.ApprovedOvertimeMinutes,
	}
	pairs := append([]BreakPair(nil), input.ActualBreaks...)
	sort.Slice(pairs, func(i, j int) bool { return pairs[i].Start.Before(pairs[j].Start) })
	for i, pair := range pairs {
		if pair.Start.Before(input.PunchIn) || pair.End.After(input.PunchOut) || !pair.End.After(pair.Start) || (i > 0 && pair.Start.Before(pairs[i-1].End)) {
			return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
		}
		result.ActualBreakMinutes += int32(pair.End.Sub(pair.Start) / time.Minute)
	}
	approvedBreakWorkedMinutes := int32(0)
	sort.Slice(approvedIntervals, func(i, j int) bool { return approvedIntervals[i].Start.Before(approvedIntervals[j].Start) })
	for i, interval := range approvedIntervals {
		if interval.Start.Before(*input.ScheduledBreakStart) || interval.End.After(*input.ScheduledBreakEnd) || !interval.End.After(interval.Start) || (i > 0 && interval.Start.Before(approvedIntervals[i-1].End)) {
			return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
		}
		for _, actual := range pairs {
			if interval.Start.Before(actual.End) && actual.Start.Before(interval.End) {
				return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
			}
		}
		approvedBreakWorkedMinutes += int32(interval.End.Sub(interval.Start) / time.Minute)
	}

	switch input.BreakPunchMode {
	case BreakPunchModeRequired:
		if len(pairs) == 0 {
			result.ExceptionCode = AttendanceExceptionMissingBreakPair
		}
	case BreakPunchModeNone:
		if len(pairs) > 0 {
			result.ExceptionCode = AttendanceExceptionUnexpectedBreakPunch
		}
		if input.BreakCreditMode == BreakCreditModeScheduled {
			result.ScheduledUnpaidBreakMinutes = input.ScheduledBreakMinutes
		}
	}
	if input.PlannedBreakWorkedMinutes+approvedBreakWorkedMinutes > result.ScheduledUnpaidBreakMinutes {
		return AttendanceCalculationResult{}, ErrInvalidAttendanceCalculation
	}
	result.ScheduledUnpaidBreakMinutes -= input.PlannedBreakWorkedMinutes + approvedBreakWorkedMinutes
	result.PaidBreakWorkMinutes = input.PlannedBreakWorkedMinutes + approvedBreakWorkedMinutes
	actualDeduction := int32(0)
	if input.BreakPunchMode == BreakPunchModeRequired {
		actualDeduction = result.ActualBreakMinutes
	}
	result.PayableMinutes = result.GrossMinutes - actualDeduction - result.ScheduledUnpaidBreakMinutes
	if result.PayableMinutes < 0 {
		result.PayableMinutes = 0
	}
	return result, nil
}

func CalculateBreakDeduction(input BreakDeductionInput) int32 {
	if input.IsBreakOvertime || !input.IsBreakAutoDeduct {
		return 0
	}
	return calculateBreakOverlap(input)
}

func CalculateBreakOvertime(input BreakDeductionInput) int32 {
	if !input.IsBreakOvertime {
		return 0
	}
	return calculateBreakOverlap(input)
}

func calculateBreakOverlap(input BreakDeductionInput) int32 {
	if input.PunchIn == nil || input.PunchOut == nil {
		return input.ScheduledBreakMinutes
	}
	windows := input.BreakWindows
	if len(windows) == 0 && input.BreakStart != nil && input.BreakEnd != nil {
		windows = []BreakPair{{Start: *input.BreakStart, End: *input.BreakEnd}}
	}
	if len(windows) == 0 {
		return input.ScheduledBreakMinutes
	}
	merged := mergeBreakPairs(windows)
	var total int32
	for _, window := range merged {
		start := maxTime(window.Start, *input.PunchIn)
		end := minTime(window.End, *input.PunchOut)
		if end.After(start) {
			total += int32(end.Sub(start) / time.Minute)
		}
	}
	return total
}

func mergeBreakPairs(windows []BreakPair) []BreakPair {
	valid := make([]BreakPair, 0, len(windows))
	for _, w := range windows {
		if w.End.After(w.Start) {
			valid = append(valid, w)
		}
	}
	if len(valid) <= 1 {
		return valid
	}
	sort.Slice(valid, func(i, j int) bool {
		if valid[i].Start.Equal(valid[j].Start) {
			return valid[i].End.Before(valid[j].End)
		}
		return valid[i].Start.Before(valid[j].Start)
	})
	merged := []BreakPair{valid[0]}
	for _, w := range valid[1:] {
		last := &merged[len(merged)-1]
		if !w.Start.After(last.End) {
			if w.End.After(last.End) {
				last.End = w.End
			}
			continue
		}
		merged = append(merged, w)
	}
	return merged
}

func CalculateUnits(minutes, unitSize int32, policy RoundingPolicy) int32 {
	if minutes <= 0 || unitSize <= 0 {
		return 0
	}
	switch policy {
	case RoundingPolicyCeil:
		return (minutes + unitSize - 1) / unitSize
	case RoundingPolicyRound:
		return (minutes + unitSize/2) / unitSize
	case RoundingPolicyFloor, RoundingPolicyNone:
		fallthrough
	default:
		return minutes / unitSize
	}
}

func CalculateDailyUnits(input DailyCalcInput) (int32, int32) {
	buckets := CalculateBuckets(input)
	return CalculateUnits(buckets.StandardMinutes, input.WorkUnit, input.RoundingPolicy), CalculateUnits(buckets.EarlyOTMinutes+buckets.LateOTMinutes+buckets.BreakOTMinutes, input.WorkUnit, input.RoundingPolicy)
}

func CalculateBuckets(input DailyCalcInput) WorkBuckets {
	buckets, _ := calculateSegmentBuckets(input)
	return buckets
}

func resolveSegmentBreak(input DailyCalcInput) (int32, int32) {
	if input.BreakPunchMode == BreakPunchModeRequired {
		if input.ActualBreakMinutes > 0 {
			return input.ActualBreakMinutes, 0
		}
		return 0, 0
	}
	breakInput := BreakDeductionInput{ActualBreakMinutes: input.ActualBreakMinutes, ScheduledBreakMinutes: input.ScheduledBreakMinutes, IsBreakAutoDeduct: input.IsBreakAutoDeduct, IsBreakOvertime: input.IsBreakOvertime, BreakStart: input.BreakStart, BreakEnd: input.BreakEnd, BreakWindows: input.BreakWindows, PunchIn: &input.PunchIn, PunchOut: &input.PunchOut}
	deduction := CalculateBreakDeduction(breakInput)
	if input.ApprovedBreakWorkedMinutes > 0 {
		deduction = max(0, deduction-input.ApprovedBreakWorkedMinutes)
	}
	return deduction, CalculateBreakOvertime(breakInput)
}

func calculateLateOT(input DailyCalcInput) int32 {
	if input.ShiftEnd == nil || !input.PunchOut.After(*input.ShiftEnd) {
		return 0
	}
	if input.OTCeiling != nil && input.PunchOut.After(*input.OTCeiling) {
		return int32(input.OTCeiling.Sub(*input.ShiftEnd) / time.Minute)
	}
	return int32(input.PunchOut.Sub(*input.ShiftEnd) / time.Minute)
}

func calculateSegmentBuckets(input DailyCalcInput) (WorkBuckets, int32) {
	if !input.PunchOut.After(input.PunchIn) {
		return WorkBuckets{}, 0
	}
	standardStart, standardEnd := input.PunchIn, input.PunchOut
	if input.ShiftStart != nil {
		standardStart = maxTime(standardStart, *input.ShiftStart)
	}
	if input.ShiftEnd != nil {
		standardEnd = minTime(standardEnd, *input.ShiftEnd)
	}
	breakDeduction, breakOvertime := resolveSegmentBreak(input)
	standardMinutes := max(0, int32(maxDuration(standardEnd.Sub(standardStart))/time.Minute)-breakDeduction)
	early := applyEarlyOTRules(rawEarlyMinutes(input), input.GracePeriodMinutes, input.MaxEarlyOTMinutes)
	late := calculateLateOT(input)
	return WorkBuckets{StandardMinutes: standardMinutes, EarlyOTMinutes: early, LateOTMinutes: late, BreakOTMinutes: breakOvertime}, breakDeduction
}

func rawEarlyMinutes(input DailyCalcInput) int32 {
	if !input.AllowEarlyOT || input.ShiftStart == nil || !input.PunchIn.Before(*input.ShiftStart) {
		return 0
	}
	return int32((*input.ShiftStart).Sub(input.PunchIn) / time.Minute)
}

func applyEarlyOTRules(rawMinutes, graceMinutes int32, maxEarly *int32) int32 {
	if rawMinutes <= 0 {
		return 0
	}
	early := max(0, rawMinutes-max(0, graceMinutes))
	if maxEarly != nil && early > *maxEarly {
		early = *maxEarly
	}
	return early
}

func applyBreakException(result *DayBucketResult, input DailyCalcInput) {
	if input.BreakPunchMode == BreakPunchModeRequired {
		if input.ActualBreakCount == 0 || input.HasOpenActualBreak {
			result.BreakProvisional = true
			result.BreakExceptionCode = AttendanceExceptionMissingBreakPair
		}
		return
	}
	if (input.ActualBreakCount > 0 || input.ActualBreakMinutes > 0) && result.BreakExceptionCode == "" {
		result.BreakExceptionCode = AttendanceExceptionUnexpectedBreakPunch
	}
}

func CalculateDayBuckets(inputs []DailyCalcInput, mode OtAggregationMode) DayBucketResult {
	if len(inputs) == 0 {
		return DayBucketResult{}
	}
	if mode == "" {
		mode = OtAggregate
	}
	var result DayBucketResult
	var rawEarlyTotal int32
	for _, input := range inputs {
		seg, deducted := calculateSegmentBuckets(input)
		result.DeductedBreakMinutes += deducted
		result.ActualBreakMinutes += max(0, input.ActualBreakMinutes)
		applyBreakException(&result, input)
		result.Buckets.StandardMinutes += seg.StandardMinutes
		result.Buckets.LateOTMinutes += seg.LateOTMinutes
		result.Buckets.BreakOTMinutes += seg.BreakOTMinutes
		if mode == OtPerSegment {
			result.Buckets.EarlyOTMinutes += seg.EarlyOTMinutes
			result.StandardUnits += CalculateUnits(seg.StandardMinutes, input.WorkUnit, input.RoundingPolicy)
			result.OTUnits += CalculateUnits(seg.EarlyOTMinutes+seg.LateOTMinutes+seg.BreakOTMinutes, input.WorkUnit, input.RoundingPolicy)
		} else {
			rawEarlyTotal += rawEarlyMinutes(input)
		}
	}
	if mode != OtPerSegment {
		first := inputs[0]
		result.Buckets.EarlyOTMinutes = applyEarlyOTRules(rawEarlyTotal, first.GracePeriodMinutes, first.MaxEarlyOTMinutes)
		result.StandardUnits = CalculateUnits(result.Buckets.StandardMinutes, first.WorkUnit, first.RoundingPolicy)
		result.OTUnits = CalculateUnits(result.Buckets.EarlyOTMinutes+result.Buckets.LateOTMinutes+result.Buckets.BreakOTMinutes, first.WorkUnit, first.RoundingPolicy)
	}
	return result
}

func maxTime(a, b time.Time) time.Time {
	if a.After(b) {
		return a
	}
	return b
}
func minTime(a, b time.Time) time.Time {
	if a.Before(b) {
		return a
	}
	return b
}
func maxDuration(value time.Duration) time.Duration {
	if value < 0 {
		return 0
	}
	return value
}
