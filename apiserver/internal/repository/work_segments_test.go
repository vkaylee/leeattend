package repository

import (
	"encoding/json"
	"testing"

	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestApplySegmentDefaults_PreservesExplicitZeroAndDefaultsOmitted(t *testing.T) {
	var zeroReq api.CreateWorkSegmentRequest
	if err := json.Unmarshal([]byte(`{"employment_id":"11111111-1111-1111-1111-111111111111","template_id":"22222222-2222-2222-2222-222222222222","date":"2026-11-20","scheduled_start":"08:00:00","scheduled_end":"17:00:00","segment_type":"on_call","compensation_rate":"0"}`), &zeroReq); err != nil {
		t.Fatalf("unmarshal zero compensation_rate: %v", err)
	}
	applySegmentDefaults(&zeroReq)
	if zeroReq.CompensationRate == nil || !zeroReq.CompensationRate.Equal(decimal.Zero) {
		t.Fatalf("explicit compensation_rate=0 was overwritten to %v", zeroReq.CompensationRate)
	}

	var omittedReq api.CreateWorkSegmentRequest
	if err := json.Unmarshal([]byte(`{"employment_id":"11111111-1111-1111-1111-111111111111","template_id":"22222222-2222-2222-2222-222222222222","date":"2026-11-20","scheduled_start":"08:00:00","scheduled_end":"17:00:00"}`), &omittedReq); err != nil {
		t.Fatalf("unmarshal omitted compensation_rate: %v", err)
	}
	applySegmentDefaults(&omittedReq)
	if omittedReq.CompensationRate == nil || !omittedReq.CompensationRate.Equal(decimal.NewFromInt(1)) {
		t.Fatalf("omitted compensation_rate = %v, want 1", omittedReq.CompensationRate)
	}
}
