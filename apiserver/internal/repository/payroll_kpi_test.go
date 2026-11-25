package repository

import (
	"context"
	"encoding/json"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestMarshalKPICalculationInputsIncludesEffectivePolicyWithoutSalary(t *testing.T) {
	policy := &workpolicy.ResolvedPolicy{
		VersionID:               uuid.New(),
		GroupID:                 uuid.New(),
		GroupCode:               "STD",
		VersionNo:               1,
		EffectiveFrom:           time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC),
		OvertimePolicyVersionID: uuid.New(),
		PayMethod:               workpolicy.PayMethodMonthly,
	}
	payload, err := marshalKPICalculationInputs(kpiCalculation{
		Inputs: map[string]string{"production.output": "100"},
		Policy: policy,
	})
	if err != nil {
		t.Fatalf("marshal policy snapshot: %v", err)
	}
	if !json.Valid(payload) {
		t.Fatal("expected valid JSON snapshot")
	}
	text := string(payload)
	for _, forbidden := range []string{"salary", "base_pay", "net_pay"} {
		if strings.Contains(text, forbidden) {
			t.Fatalf("snapshot contains forbidden field %q: %s", forbidden, text)
		}
	}
	if !strings.Contains(text, policy.VersionID.String()) || !strings.Contains(text, "2026-06-01") {
		t.Fatalf("snapshot omitted policy version metadata: %s", text)
	}
}

func TestMarshalKPICalculationInputsIncludesPolicySegments(t *testing.T) {
	versionID := uuid.New()
	payload, err := marshalKPICalculationInputs(kpiCalculation{
		PolicySegments: []policySegmentSnapshot{{StartDate: "2026-06-01", EndDate: "2026-06-14", VersionID: versionID}},
	})
	if err != nil {
		t.Fatalf("marshal policy segments: %v", err)
	}
	var decoded map[string]any
	if err := json.Unmarshal(payload, &decoded); err != nil {
		t.Fatalf("decode policy segments: %v", err)
	}
	segments, ok := decoded["policy_segments"].([]any)
	if !ok || len(segments) != 1 || !strings.Contains(string(payload), versionID.String()) {
		t.Fatalf("policy segments missing from snapshot: %s", payload)
	}
}

func TestPersistKPICalculationStoresImpactFields(t *testing.T) {
	var output []byte
	db := &fakeDBTX{execFn: func(_ context.Context, query string, args ...any) (pgconn.CommandTag, error) {
		if strings.Contains(query, "INSERT INTO payroll_calculation_runs") {
			var ok bool
			output, ok = args[5].([]byte)
			if !ok {
				t.Fatalf("expected output snapshot bytes, got %T", args[5])
			}
		}
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}}
	calc := kpiCalculation{
		BasePay:     decimal.RequireFromString("1000"),
		OvertimePay: decimal.RequireFromString("100"),
		Deductions:  decimal.RequireFromString("50"),
		NetPay:      decimal.RequireFromString("1050"),
	}
	if err := persistKPICalculation(context.Background(), db, uuid.New(), uuid.New(), 2026, 6, calc, nil); err != nil {
		t.Fatalf("persist impact snapshot: %v", err)
	}
	for _, field := range []string{`"base_pay":"1000"`, `"overtime_pay":"100"`, `"deductions":"50"`, `"net_pay":"1050"`} {
		if !strings.Contains(string(output), field) {
			t.Fatalf("output snapshot omitted %s: %s", field, output)
		}
	}
}

func TestPersistKPICalculationSnapshotsWithoutFormulaIsAppendOnly(t *testing.T) {
	var query string
	db := &fakeDBTX{execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
		query = sql
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}}
	policy := &workpolicy.ResolvedPolicy{VersionID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC), PayMethod: workpolicy.PayMethodHourly}
	if err := persistKPICalculation(context.Background(), db, uuid.New(), uuid.New(), 2026, 6, kpiCalculation{Policy: policy}, nil); err != nil {
		t.Fatalf("persist policy-only snapshot: %v", err)
	}
	if strings.Contains(query, "ON CONFLICT") || strings.Contains(query, "DO UPDATE") {
		t.Fatalf("policy-only snapshot must append without overwrite clause: %s", query)
	}
}

func TestPersistKPICalculationHistoricalRerunPreservesOldPolicySnapshot(t *testing.T) {
	payslipID := uuid.New()
	policyA := &workpolicy.ResolvedPolicy{VersionID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC), PayMethod: workpolicy.PayMethodMonthly}
	policyB := &workpolicy.ResolvedPolicy{VersionID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC), PayMethod: workpolicy.PayMethodHourly}
	var snapshots [][]byte
	db := &fakeDBTX{execFn: func(_ context.Context, query string, args ...any) (pgconn.CommandTag, error) {
		if !strings.Contains(query, "INSERT INTO payroll_calculation_runs") {
			return pgconn.NewCommandTag("UPDATE 0"), nil
		}
		input := args[4].([]byte)
		snapshots = append(snapshots, append([]byte(nil), input...))
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}}

	if err := persistKPICalculation(context.Background(), db, uuid.New(), payslipID, 2026, 6, kpiCalculation{Policy: policyA}, nil); err != nil {
		t.Fatalf("persist policy A snapshot: %v", err)
	}
	if err := persistKPICalculation(context.Background(), db, uuid.New(), payslipID, 2026, 6, kpiCalculation{Policy: policyB}, nil); err != nil {
		t.Fatalf("persist policy B snapshot: %v", err)
	}
	if len(snapshots) != 2 {
		t.Fatalf("expected two historical snapshots, got %d", len(snapshots))
	}

	var first, second map[string]any
	_ = json.Unmarshal(snapshots[0], &first)
	_ = json.Unmarshal(snapshots[1], &second)
	if first["policy"].(map[string]any)["version_id"] != policyA.VersionID.String() {
		t.Fatalf("old snapshot changed policy version")
	}
	if second["policy"].(map[string]any)["version_id"] != policyB.VersionID.String() {
		t.Fatalf("new snapshot did not record policy B")
	}
}
