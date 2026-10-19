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
	EntryType       EntryType
	Note            *string
	IsBreakOvertime bool
	Metadata        map[string]string
}

func Pair(logs []LogEvent, config *ShiftConfig) []PairedSession {
	if config == nil {
		return PairAnchorFirst(logs, nil)
	}
	return PairMulti(logs, &MultiSegmentConfig{Segments: []ShiftConfig{*config}})
}

func PairMulti(logs []LogEvent, config *MultiSegmentConfig) []PairedSession {
	if config != nil && len(config.Segments) > 0 {
		for _, segment := range config.Segments {
			if result := PairByShiftWindow(logs, segment); len(result) > 0 {
				return result
			}
		}
	}
	return PairAnchorFirst(logs, nil)
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
	sort.SliceStable(ordered, func(left, right int) bool { return ordered[left].Minutes < ordered[right].Minutes })
	return ordered
}
func buildSession(start LogEvent, end *LogEvent, tier string) PairedSession {
	session := PairedSession{StartMinutes: start.Minutes, StartTime: start.PunchTime, EntryType: EntryTypeWork, Metadata: map[string]string{"pairing_tier": tier}}
	if end != nil {
		session.EndMinutes = new(end.Minutes)
		session.EndTime = new(end.PunchTime)
	}
	return session
}
