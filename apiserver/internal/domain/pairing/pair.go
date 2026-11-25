package pairing

import (
	"sort"
	"time"

	"github.com/google/uuid"
)

type EntryType string

const (
	EntryTypeWork     EntryType = "work"
	EntryTypeBreak    EntryType = "break"
	EntryTypeOvertime EntryType = "overtime"
)

type LogEvent struct {
	ID        uuid.UUID
	Minutes   int32
	PunchTime time.Time
	RawData   []byte
	AllIDs    []uuid.UUID
}
type BreakInterval struct{ StartMinutes, EndMinutes int32 }
type ShiftConfig struct {
	StartMinutes, EndMinutes                   int32
	StartTime, EndTime                         *time.Time
	Breaks                                     []BreakInterval
	CheckInBufferMinutes, MaxSessionGapMinutes *int32
	SegmentID                                  *uuid.UUID
}
type MultiSegmentConfig struct{ Segments []ShiftConfig }
type PairedSession struct {
	StartMinutes    int32
	EndMinutes      *int32
	StartTime       time.Time
	EndTime         *time.Time
	SegmentID       *uuid.UUID
	SourceLogIDs    []uuid.UUID
	EntryType       EntryType
	Note            *string
	IsBreakOvertime bool
	Metadata        map[string]string
}

const defaultBounceDedupeWindow = time.Minute

func Pair(logs []LogEvent, config *ShiftConfig) []PairedSession {
	if config == nil {
		return PairAnchorFirst(logs, nil)
	}
	return PairMulti(logs, &MultiSegmentConfig{Segments: []ShiftConfig{*config}})
}

func PairMulti(logs []LogEvent, config *MultiSegmentConfig) []PairedSession {
	ordered := dedupeLogs(sortedLogs(logs))
	if len(ordered) == 0 {
		return nil
	}
	if config == nil || len(config.Segments) == 0 {
		return PairAnchorFirst(ordered, nil)
	}
	segments := sortSegments(config.Segments)
	used := make([]bool, len(ordered))
	sessions := make([]PairedSession, 0, len(segments))
	for i, segment := range segments {
		var prevSeg, nextSeg *ShiftConfig
		if i > 0 {
			prevSeg = &segments[i-1]
		}
		if i+1 < len(segments) {
			nextSeg = &segments[i+1]
		}
		if session, ok := pairSegmentExclusive(ordered, used, segment, prevSeg, nextSeg); ok {
			sessions = append(sessions, session)
		}
	}
	if len(sessions) > 0 {
		return sessions
	}
	return PairAnchorFirst(ordered, nil)
}

func PairAnchorFirst(logs []LogEvent, maxGap *int) []PairedSession {
	ordered := sortedLogs(logs)
	if len(ordered) == 0 {
		return nil
	}
	gap := int32(960)
	if maxGap != nil && *maxGap >= 0 {
		gap = int32(*maxGap)
	}
	result := make([]PairedSession, 0, (len(ordered)+1)/2)
	for index := 0; index < len(ordered); index += 2 {
		start := ordered[index]
		session := PairedSession{StartMinutes: start.Minutes, StartTime: start.PunchTime, EntryType: EntryTypeWork, Metadata: map[string]string{"pairing_tier": "anchor_first"}}
		if index+1 < len(ordered) && ordered[index+1].Minutes-start.Minutes <= gap {
			session.EndMinutes = new(ordered[index+1].Minutes)
			session.EndTime = new(ordered[index+1].PunchTime)
		} else {
			session.Note = new("missing_punch_out")
		}
		result = append(result, session)
	}
	return result
}

func PairByShiftWindow(logs []LogEvent, shift ShiftConfig) []PairedSession {
	ordered := sortedLogs(logs)
	if len(ordered) == 0 {
		return nil
	}
	buffer := int32(120)
	if shift.CheckInBufferMinutes != nil && *shift.CheckInBufferMinutes > buffer {
		buffer = *shift.CheckInBufferMinutes
	}
	startIndex := -1
	for index := range ordered {
		if ordered[index].Minutes >= shift.StartMinutes-buffer && ordered[index].Minutes <= shift.StartMinutes+buffer {
			startIndex = index
			break
		}
	}
	if startIndex < 0 {
		return nil
	}
	start := ordered[startIndex]
	endIndex := -1
	for index := len(ordered) - 1; index > startIndex; index-- {
		if ordered[index].Minutes > start.Minutes && (shift.EndMinutes == 0 || ordered[index].Minutes <= shift.EndMinutes+buffer) {
			endIndex = index
			break
		}
	}
	if endIndex < 0 {
		return []PairedSession{buildSession(start, nil, "shift_window")}
	}
	end := ordered[endIndex]
	return []PairedSession{buildSession(start, &end, "shift_window")}
}

func FindEarliestInWindow(logs []LogEvent, startMinutes, buffer int) *LogEvent {
	start := int32(startMinutes)
	window := int32(buffer)
	for index := range logs {
		if logs[index].Minutes >= start-window && logs[index].Minutes <= start+window {
			return &logs[index]
		}
	}
	return nil
}
func FindLastOut(logs []LogEvent, startMinutes int, startID uuid.UUID) *LogEvent {
	start := int32(startMinutes)
	for index := len(logs) - 1; index >= 0; index-- {
		if logs[index].ID != startID && logs[index].Minutes > start {
			return &logs[index]
		}
	}
	return nil
}
func CombineNotes(start, end *string) *string {
	if start == nil && end == nil {
		return nil
	}
	if start == nil {
		return end
	}
	if end == nil {
		return start
	}
	combined := *start + "\n" + *end
	return &combined
}
func FinalizeNote(note *string, missingOut bool) *string {
	if !missingOut {
		return note
	}
	return CombineNotes(note, new("missing_punch_out"))
}
func CollectLogIDs(logs []LogEvent, start int32, end *int32) []uuid.UUID {
	result := make([]uuid.UUID, 0)
	for _, log := range logs {
		if log.Minutes < start {
			continue
		}
		if end != nil && log.Minutes > *end {
			continue
		}
		result = append(result, log.ID)
	}
	return result
}

func sortedLogs(logs []LogEvent) []LogEvent {
	ordered := append([]LogEvent(nil), logs...)
	sort.SliceStable(ordered, func(left, right int) bool {
		if !ordered[left].PunchTime.IsZero() && !ordered[right].PunchTime.IsZero() && !ordered[left].PunchTime.Equal(ordered[right].PunchTime) {
			return ordered[left].PunchTime.Before(ordered[right].PunchTime)
		}
		return ordered[left].Minutes < ordered[right].Minutes
	})
	return ordered
}

func dedupeLogs(ordered []LogEvent) []LogEvent {
	if len(ordered) <= 1 {
		return ordered
	}
	result := make([]LogEvent, 0, len(ordered))
	for _, ev := range ordered {
		if len(result) == 0 {
			ev.AllIDs = appendLogIDs(ev.AllIDs, ev.ID)
			result = append(result, ev)
			continue
		}
		last := &result[len(result)-1]
		if isBounceDuplicate(*last, ev) {
			last.AllIDs = appendLogIDs(last.AllIDs, ev.ID)
			continue
		}
		ev.AllIDs = appendLogIDs(ev.AllIDs, ev.ID)
		result = append(result, ev)
	}
	return result
}

func isBounceDuplicate(prev, curr LogEvent) bool {
	if !prev.PunchTime.IsZero() && !curr.PunchTime.IsZero() {
		delta := curr.PunchTime.Sub(prev.PunchTime)
		return delta >= 0 && delta <= defaultBounceDedupeWindow
	}
	return curr.Minutes == prev.Minutes
}

func appendLogIDs(ids []uuid.UUID, id uuid.UUID) []uuid.UUID {
	if id == uuid.Nil {
		return ids
	}
	for _, existing := range ids {
		if existing == id {
			return ids
		}
	}
	return append(ids, id)
}

func sortSegments(segments []ShiftConfig) []ShiftConfig {
	ordered := append([]ShiftConfig(nil), segments...)
	sort.SliceStable(ordered, func(i, j int) bool {
		if ordered[i].StartTime != nil && ordered[j].StartTime != nil && !ordered[i].StartTime.Equal(*ordered[j].StartTime) {
			return ordered[i].StartTime.Before(*ordered[j].StartTime)
		}
		return ordered[i].StartMinutes < ordered[j].StartMinutes
	})
	return ordered
}

func segmentBuffer(shift ShiftConfig) time.Duration {
	minutes := int32(120)
	if shift.CheckInBufferMinutes != nil && *shift.CheckInBufferMinutes > 0 {
		minutes = *shift.CheckInBufferMinutes
	}
	return time.Duration(minutes) * time.Minute
}

func pairSegmentExclusive(ordered []LogEvent, used []bool, seg ShiftConfig, prevSeg, nextSeg *ShiftConfig) (PairedSession, bool) {
	if seg.StartTime != nil && seg.EndTime != nil {
		return pairSegmentByTime(ordered, used, seg, prevSeg, nextSeg)
	}
	return pairSegmentByMinutes(ordered, used, seg, prevSeg, nextSeg)
}

func pairSegmentByTime(ordered []LogEvent, used []bool, seg ShiftConfig, prevSeg, nextSeg *ShiftConfig) (PairedSession, bool) {
	minStart, maxStart, maxEnd := timeWindowBounds(seg, prevSeg, nextSeg)
	startIdx := findFirstLogInTimeRange(ordered, used, minStart, maxStart)
	if startIdx < 0 {
		return PairedSession{}, false
	}
	endIdx := findLastLogBeforeTime(ordered, used, startIdx, maxEnd)
	return finalizeSegmentSession(ordered, used, startIdx, endIdx, seg), true
}

func timeWindowBounds(seg ShiftConfig, prevSeg, nextSeg *ShiftConfig) (time.Time, time.Time, time.Time) {
	buf := segmentBuffer(seg)
	minStart := seg.StartTime.Add(-buf)
	if prevSeg != nil && prevSeg.EndTime != nil {
		if mid := midpointTime(*prevSeg.EndTime, *seg.StartTime); mid.After(minStart) {
			minStart = mid
		}
	}
	maxStart := seg.StartTime.Add(buf)
	if maxStart.After(*seg.EndTime) {
		maxStart = *seg.EndTime
	}
	maxEnd := seg.EndTime.Add(buf)
	if nextSeg != nil && nextSeg.StartTime != nil {
		if mid := midpointTime(*seg.EndTime, *nextSeg.StartTime); mid.Before(maxEnd) {
			maxEnd = mid
		}
	}
	return minStart, maxStart, maxEnd
}

func findFirstLogInTimeRange(ordered []LogEvent, used []bool, minStart, maxStart time.Time) int {
	for i, ev := range ordered {
		if !used[i] && !ev.PunchTime.Before(minStart) && !ev.PunchTime.After(maxStart) {
			return i
		}
	}
	return -1
}

func findLastLogBeforeTime(ordered []LogEvent, used []bool, startIdx int, maxEnd time.Time) int {
	for i := len(ordered) - 1; i > startIdx; i-- {
		if !used[i] && ordered[i].PunchTime.After(ordered[startIdx].PunchTime) && !ordered[i].PunchTime.After(maxEnd) {
			return i
		}
	}
	return -1
}

func pairSegmentByMinutes(ordered []LogEvent, used []bool, seg ShiftConfig, prevSeg, nextSeg *ShiftConfig) (PairedSession, bool) {
	minStart, maxStart, maxEnd := minuteWindowBounds(seg, prevSeg, nextSeg)
	startIdx := findFirstLogInMinuteRange(ordered, used, minStart, maxStart)
	if startIdx < 0 {
		return PairedSession{}, false
	}
	endIdx := findLastLogBeforeMinute(ordered, used, startIdx, seg.EndMinutes, maxEnd)
	return finalizeSegmentSession(ordered, used, startIdx, endIdx, seg), true
}

func minuteWindowBounds(seg ShiftConfig, prevSeg, nextSeg *ShiftConfig) (int32, int32, int32) {
	buf := int32(segmentBuffer(seg) / time.Minute)
	minStart := seg.StartMinutes - buf
	if prevSeg != nil && prevSeg.EndMinutes > 0 {
		if mid := (prevSeg.EndMinutes + seg.StartMinutes) / 2; mid > minStart {
			minStart = mid
		}
	}
	maxStart := seg.StartMinutes + buf
	maxEnd := seg.EndMinutes + buf
	if nextSeg != nil {
		if mid := (seg.EndMinutes + nextSeg.StartMinutes) / 2; mid < maxEnd {
			maxEnd = mid
		}
	}
	return minStart, maxStart, maxEnd
}

func findFirstLogInMinuteRange(ordered []LogEvent, used []bool, minStart, maxStart int32) int {
	for i, ev := range ordered {
		if !used[i] && ev.Minutes >= minStart && ev.Minutes <= maxStart {
			return i
		}
	}
	return -1
}

func findLastLogBeforeMinute(ordered []LogEvent, used []bool, startIdx int, segEnd, maxEnd int32) int {
	for i := len(ordered) - 1; i > startIdx; i-- {
		if used[i] || ordered[i].Minutes <= ordered[startIdx].Minutes {
			continue
		}
		if segEnd != 0 && ordered[i].Minutes > maxEnd {
			continue
		}
		return i
	}
	return -1
}

func finalizeSegmentSession(ordered []LogEvent, used []bool, startIdx, endIdx int, seg ShiftConfig) PairedSession {
	lastConsumed := startIdx
	var end *LogEvent
	if endIdx > startIdx {
		end = &ordered[endIdx]
		lastConsumed = endIdx
	}
	sourceIDs := make([]uuid.UUID, 0, lastConsumed-startIdx+1)
	for i := startIdx; i <= lastConsumed; i++ {
		used[i] = true
		if len(ordered[i].AllIDs) > 0 {
			sourceIDs = append(sourceIDs, ordered[i].AllIDs...)
		} else if ordered[i].ID != uuid.Nil {
			sourceIDs = append(sourceIDs, ordered[i].ID)
		}
	}
	session := buildSession(ordered[startIdx], end, "shift_window")
	session.SegmentID = seg.SegmentID
	session.SourceLogIDs = sourceIDs
	if end == nil {
		session.Note = FinalizeNote(session.Note, true)
	}
	return session
}

func midpointTime(a, b time.Time) time.Time {
	return a.Add(b.Sub(a) / 2)
}

func buildSession(start LogEvent, end *LogEvent, tier string) PairedSession {
	session := PairedSession{StartMinutes: start.Minutes, StartTime: start.PunchTime, EntryType: EntryTypeWork, Metadata: map[string]string{"pairing_tier": tier}}
	if end != nil {
		session.EndMinutes = new(end.Minutes)
		session.EndTime = new(end.PunchTime)
	}
	return session
}
