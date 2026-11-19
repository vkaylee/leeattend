package buckets

import (
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
type BreakDeductionInput struct {
	ActualBreakMinutes, ScheduledBreakMinutes int32
	IsBreakAutoDeduct, IsBreakOvertime        bool
	BreakStart, BreakEnd, PunchIn, PunchOut   *time.Time
}
type DailyCalcInput struct {
	PunchIn, PunchOut                                time.Time
	ShiftStart, ShiftEnd                             *time.Time
	ScheduledBreakMinutes, ActualBreakMinutes        int32
	BreakStart, BreakEnd                             *time.Time
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
type SegmentBucket struct {
	SegmentID *uuid.UUID
	Buckets   WorkBuckets
}
type SegmentCalcResult struct {
	TotalStandardUnits, TotalOTUnits int32
	PerSegmentBuckets                []SegmentBucket
}

func TruncateToMinute(value time.Time) time.Time { return value.Truncate(time.Minute) }

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
	if input.BreakStart == nil || input.BreakEnd == nil || input.PunchIn == nil || input.PunchOut == nil {
		return input.ScheduledBreakMinutes
	}
	start := maxTime(*input.BreakStart, *input.PunchIn)
	end := minTime(*input.BreakEnd, *input.PunchOut)
	if !end.After(start) {
		return 0
	}
	return int32(end.Sub(start) / time.Minute)
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
	if !input.PunchOut.After(input.PunchIn) {
		return WorkBuckets{}
	}
	start, end := input.PunchIn, input.PunchOut
	standardStart, standardEnd := start, end
	if input.ShiftStart != nil {
		standardStart = maxTime(standardStart, *input.ShiftStart)
	}
	if input.ShiftEnd != nil {
		standardEnd = minTime(standardEnd, *input.ShiftEnd)
	}
	breakInput := BreakDeductionInput{ActualBreakMinutes: input.ActualBreakMinutes, ScheduledBreakMinutes: input.ScheduledBreakMinutes, IsBreakAutoDeduct: input.IsBreakAutoDeduct, IsBreakOvertime: input.IsBreakOvertime, BreakStart: input.BreakStart, BreakEnd: input.BreakEnd, PunchIn: &input.PunchIn, PunchOut: &input.PunchOut}
	breakDeduction := CalculateBreakDeduction(breakInput)
	breakOvertime := CalculateBreakOvertime(breakInput)
	standardMinutes := int32(maxDuration(standardEnd.Sub(standardStart))/time.Minute) - breakDeduction
	if standardMinutes < 0 {
		standardMinutes = 0
	}
	var early, late int32
	if input.AllowEarlyOT && input.ShiftStart != nil && input.PunchIn.Before(*input.ShiftStart) {
		early = int32((*input.ShiftStart).Sub(input.PunchIn) / time.Minute)
		if input.GracePeriodMinutes > 0 {
			early -= input.GracePeriodMinutes
			if early < 0 {
				early = 0
			}
		}
		if input.MaxEarlyOTMinutes != nil && early > *input.MaxEarlyOTMinutes {
			early = *input.MaxEarlyOTMinutes
		}
	}
	if input.ShiftEnd != nil && input.PunchOut.After(*input.ShiftEnd) {
		late = int32(input.PunchOut.Sub(*input.ShiftEnd) / time.Minute)
		if input.OTCeiling != nil && input.PunchOut.After(*input.OTCeiling) {
			late = int32(input.OTCeiling.Sub(*input.ShiftEnd) / time.Minute)
		}
	}
	return WorkBuckets{StandardMinutes: standardMinutes, EarlyOTMinutes: early, LateOTMinutes: late, BreakOTMinutes: breakOvertime}
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
