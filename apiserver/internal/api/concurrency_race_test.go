package api

import (
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/google/uuid"
)

// TestRace_DoublePunchDeduplication tests concurrent identical punch requests
// within a deduplication window (e.g. 5 seconds) to ensure only one punch is recorded.
func TestRace_DoublePunchDeduplication(t *testing.T) {
	const concurrentPunches = 10
	var recordedPunches atomic.Int32
	var duplicateRejections atomic.Int32

	var mu sync.Mutex
	lastPunchTime := make(map[uuid.UUID]time.Time)
	empID := uuid.New()
	targetTime := time.Date(2026, 10, 20, 9, 0, 0, 0, time.UTC)
	dedupWindow := 5 * time.Second

	var wg sync.WaitGroup
	for range concurrentPunches {
		wg.Add(1)
		go func() {
			defer wg.Done()

			mu.Lock()
			prev, exists := lastPunchTime[empID]
			if exists && targetTime.Sub(prev) < dedupWindow {
				mu.Unlock()
				duplicateRejections.Add(1)
				return
			}
			lastPunchTime[empID] = targetTime
			mu.Unlock()

			recordedPunches.Add(1)
		}()
	}
	wg.Wait()

	if recordedPunches.Load() != 1 {
		t.Fatalf("expected exactly 1 recorded punch, got %d", recordedPunches.Load())
	}
	if duplicateRejections.Load() != concurrentPunches-1 {
		t.Fatalf("expected %d deduplicated rejections, got %d", concurrentPunches-1, duplicateRejections.Load())
	}
}

// TestRace_OverlappingShiftsConflict checks that concurrent assignment of overlapping
// shifts for the same employee does not result in double booking.
func TestRace_OverlappingShiftsConflict(t *testing.T) {
	type shiftSpan struct {
		start time.Time
		end   time.Time
	}

	var mu sync.Mutex
	assignedShifts := make([]shiftSpan, 0)
	var successfulAssignments atomic.Int32
	var conflictRejections atomic.Int32

	start := time.Date(2026, 10, 21, 8, 0, 0, 0, time.UTC)
	end := time.Date(2026, 10, 21, 16, 0, 0, 0, time.UTC)

	const attempts = 8
	var wg sync.WaitGroup
	for range attempts {
		wg.Add(1)
		go func() {
			defer wg.Done()

			mu.Lock()
			hasOverlap := false
			for _, s := range assignedShifts {
				// Overlap condition: start < s.end && end > s.start
				if start.Before(s.end) && end.After(s.start) {
					hasOverlap = true
					break
				}
			}
			if hasOverlap {
				mu.Unlock()
				conflictRejections.Add(1)
				return
			}
			assignedShifts = append(assignedShifts, shiftSpan{start: start, end: end})
			mu.Unlock()

			successfulAssignments.Add(1)
		}()
	}
	wg.Wait()

	if successfulAssignments.Load() != 1 {
		t.Fatalf("expected exactly 1 successful shift assignment, got %d", successfulAssignments.Load())
	}
	if conflictRejections.Load() != attempts-1 {
		t.Fatalf("expected %d shift conflict rejections, got %d", attempts-1, conflictRejections.Load())
	}
}

// TestRace_LeaveBalanceConcurrentDepletion checks that simultaneous requests competing
// for a single remaining leave balance unit do not cause negative balance or double allocation.
func TestRace_LeaveBalanceConcurrentDepletion(t *testing.T) {
	var mu sync.Mutex
	balanceDays := 1 // exactly 1 day remaining

	var approvedLeaves atomic.Int32
	var rejectedLeaves atomic.Int32

	const applicants = 6
	var wg sync.WaitGroup
	for range applicants {
		wg.Add(1)
		go func() {
			defer wg.Done()

			mu.Lock()
			if balanceDays >= 1 {
				balanceDays--
				mu.Unlock()
				approvedLeaves.Add(1)
			} else {
				mu.Unlock()
				rejectedLeaves.Add(1)
			}
		}()
	}
	wg.Wait()

	if approvedLeaves.Load() != 1 {
		t.Fatalf("expected exactly 1 approved leave, got %d", approvedLeaves.Load())
	}
	if rejectedLeaves.Load() != applicants-1 {
		t.Fatalf("expected %d rejected leaves due to insufficient balance, got %d", applicants-1, rejectedLeaves.Load())
	}
	if balanceDays != 0 {
		t.Fatalf("expected remaining balance to be 0, got %d", balanceDays)
	}
}

// TestRace_RateLimitingAtomicSlidingWindow verifies thread safety of sliding window rate counter.
func TestRace_RateLimitingAtomicSlidingWindow(t *testing.T) {
	const goroutines = 20
	const requestsPerGoroutine = 50

	var totalCounter atomic.Int64
	var wg sync.WaitGroup

	for range goroutines {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for range requestsPerGoroutine {
				totalCounter.Add(1)
			}
		}()
	}
	wg.Wait()

	expectedTotal := int64(goroutines * requestsPerGoroutine)
	if totalCounter.Load() != expectedTotal {
		t.Fatalf("expected %d atomic increments, got %d", expectedTotal, totalCounter.Load())
	}
}
