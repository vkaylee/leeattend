package repository

import (
	"encoding/json"
	"strings"
	"testing"

	"github.com/google/uuid"
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
