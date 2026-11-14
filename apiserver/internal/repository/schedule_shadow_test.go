package repository

import (
	"testing"
	"time"
)

func TestShadowComparisonScopeNormalizesTimeAndBoundsRange(t *testing.T) {
	start := time.Date(2026, 11, 1, 23, 59, 0, 0, time.FixedZone("test", 7*60*60))
	end := start.AddDate(0, 0, 61)
	if shadowDaysInRange(shadowDateOnly(start), shadowDateOnly(end)) != 62 {
		t.Fatalf("expected 62-day normalized range")
	}
	if shadowDaysInRange(shadowDateOnly(start), shadowDateOnly(end.AddDate(0, 0, 1))) != 63 {
		t.Fatalf("expected range bound to count calendar days")
	}
}

func TestClassifyShadowMismatchDetectsStatusAndIntervals(t *testing.T) {
	legacy := map[string]any{"status": "work", "intervals": []shadowInterval{{SortOrder: 0, Start: "08:00:00", End: "17:00:00"}}}
	resolver := map[string]any{"status": "work", "intervals": []map[string]any{{"sort_order": float64(0), "start": "22:00:00", "end": "06:00:00"}}}
	mismatches := classifyShadowMismatch(legacy, resolver)
	if len(mismatches) != 1 || mismatches[0] != "overnight" {
		t.Fatalf("unexpected mismatch classification: %v", mismatches)
	}

	resolver["status"] = "explicit_off"
	mismatches = classifyShadowMismatch(legacy, resolver)
	if len(mismatches) != 1 || mismatches[0] != "explicit_off" {
		t.Fatalf("unexpected status mismatch classification: %v", mismatches)
	}
}

func TestClassifyShadowMismatchIgnoresNonWorkDetails(t *testing.T) {
	legacy := map[string]any{"status": "missing", "intervals": []shadowInterval{}}
	resolver := map[string]any{"status": "missing", "intervals": []map[string]any{{"sort_order": float64(0), "start": "08:00:00", "end": "17:00:00"}}}
	if mismatches := classifyShadowMismatch(legacy, resolver); len(mismatches) != 0 {
		t.Fatalf("expected no mismatch for non-work statuses, got %v", mismatches)
	}
}

func TestShadowIntervalSignatureSortsAndCompares(t *testing.T) {
	left := shadowIntervalSignature([]map[string]any{{"sort_order": float64(1), "start": "13:00:00", "end": "17:00:00"}, {"sort_order": float64(0), "start": "08:00:00", "end": "12:00:00"}})
	right := shadowIntervalSignature([]shadowInterval{{SortOrder: 0, Start: "08:00:00", End: "12:00:00"}, {SortOrder: 1, Start: "13:00:00", End: "17:00:00"}})
	if !equalShadowIntervals(left, right) {
		t.Fatalf("expected equivalent interval signatures: left=%v right=%v", left, right)
	}
}
