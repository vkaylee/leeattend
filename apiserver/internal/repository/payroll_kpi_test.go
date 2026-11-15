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

func TestMarshalKPICalculationOutputIncludesStandardCredits(t *testing.T) {
	calc := kpiCalculation{StandardCredits: decimal.RequireFromString("1.5")}
	output, err := json.Marshal(map[string]string{
		"standard_credits": calc.StandardCredits.String(),
	})
	if err != nil || !strings.Contains(string(output), "1.5") {
		t.Fatalf("expected standard credits in output, got %s, err=%v", output, err)
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
