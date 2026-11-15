package repository

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestMarshalKPICalculationInputsIncludesEffectivePolicyWithoutSalary(t *testing.T) {
	policy := &api.WorkPolicyGroupVersion{
		ID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: "2026-06-01",
		WorkCreditPolicyID: uuid.New(), PayBasisPolicyID: uuid.New(), OvertimePolicyID: uuid.New(),
		PayMethod: workpolicy.PayMethodMonthly,
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
	if !strings.Contains(text, policy.ID.String()) || !strings.Contains(text, "2026-06-01") {
		t.Fatalf("snapshot omitted policy version metadata: %s", text)
	}
}

func TestMarshalKPICalculationOutputIncludesImpactFields(t *testing.T) {
	calc := kpiCalculation{
		StandardCredits: decimal.RequireFromString("1.5"),
		BasePay:         decimal.RequireFromString("1000"),
		OvertimePay:     decimal.RequireFromString("100"),
		Deductions:      decimal.RequireFromString("50"),
		NetPay:          decimal.RequireFromString("1050"),
	}
	output, err := json.Marshal(map[string]string{
		"standard_credits": calc.StandardCredits.String(),
		"base_pay":         calc.BasePay.String(),
		"overtime_pay":     calc.OvertimePay.String(),
		"deductions":       calc.Deductions.String(),
		"net_pay":          calc.NetPay.String(),
	})
	if err != nil || !strings.Contains(string(output), "1.5") || !strings.Contains(string(output), "1050") {
		t.Fatalf("expected impact fields in output, got %s, err=%v", output, err)
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

func TestPersistKPICalculationSnapshotsWithoutFormula(t *testing.T) {
	calls := 0
	db := &fakeDBTX{}
	db.execFn = func(_ context.Context, query string, _ ...any) (pgconn.CommandTag, error) {
		calls++
		if strings.Contains(query, "INSERT INTO payroll_calculation_runs") {
			if !strings.Contains(query, "formula_version_id") {
				t.Fatalf("snapshot insert omitted formula version column: %s", query)
			}
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		}
		return pgconn.NewCommandTag("UPDATE 0"), nil
	}
	policy := &api.WorkPolicyGroupVersion{ID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: "2026-06-01", PayMethod: workpolicy.PayMethodHourly}
	if err := persistKPICalculation(context.Background(), db, uuid.New(), uuid.New(), 2026, 6, kpiCalculation{Policy: policy}, nil); err != nil {
		t.Fatalf("persist policy-only snapshot: %v", err)
	}
	if calls != 1 {
		t.Fatalf("expected one policy snapshot write, got %d calls", calls)
	}
}

func TestPersistKPICalculationSnapshotsWithoutFormulaIsAppendOnly(t *testing.T) {
	var query string
	db := &fakeDBTX{execFn: func(_ context.Context, sql string, _ ...any) (pgconn.CommandTag, error) {
		query = sql
		return pgconn.NewCommandTag("INSERT 0 1"), nil
	}}
	if err := persistKPICalculation(context.Background(), db, uuid.New(), uuid.New(), 2026, 6, kpiCalculation{}, nil); err != nil {
		t.Fatalf("persist policy-only snapshot: %v", err)
	}
	if strings.Contains(query, "ON CONFLICT") || strings.Contains(query, "DO UPDATE") {
		t.Fatalf("policy-only snapshot must append without overwrite clause: %s", query)
	}
}

func TestPersistKPICalculationHistoricalRerunPreservesOldPolicySnapshot(t *testing.T) {
	payslipID := uuid.New()
	policyA := &api.WorkPolicyGroupVersion{ID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: "2026-01-01", PayMethod: workpolicy.PayMethodMonthly}
	policyB := &api.WorkPolicyGroupVersion{ID: uuid.New(), GroupID: uuid.New(), EffectiveFrom: "2026-06-01", PayMethod: workpolicy.PayMethodHourly}
	var snapshots [][]byte
	db := &fakeDBTX{execFn: func(_ context.Context, query string, args ...any) (pgconn.CommandTag, error) {
		if !strings.Contains(query, "INSERT INTO payroll_calculation_runs") {
			return pgconn.NewCommandTag("UPDATE 0"), nil
		}
		if len(args) != 8 {
			t.Fatalf("expected policy-only insert args, got %d", len(args))
		}
		input, ok := args[4].([]byte)
		if !ok {
			t.Fatalf("expected input snapshot bytes, got %T", args[4])
		}
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
		t.Fatalf("expected two historical snapshots for one payslip, got %d", len(snapshots))
	}

	var first, second map[string]any
	if err := json.Unmarshal(snapshots[0], &first); err != nil {
		t.Fatalf("decode policy A snapshot: %v", err)
	}
	if err := json.Unmarshal(snapshots[1], &second); err != nil {
		t.Fatalf("decode policy B snapshot: %v", err)
	}
	firstPolicy := first["policy"].(map[string]any)
	secondPolicy := second["policy"].(map[string]any)
	if firstPolicy["version_id"] != policyA.ID.String() {
		t.Fatalf("old snapshot changed policy version: got %v, want %s", firstPolicy["version_id"], policyA.ID)
	}
	if secondPolicy["version_id"] != policyB.ID.String() {
		t.Fatalf("new snapshot did not record policy B: got %v, want %s", secondPolicy["version_id"], policyB.ID)
	}
}
