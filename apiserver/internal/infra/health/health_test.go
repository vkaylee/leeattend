package health

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestHealthLiveness(t *testing.T) {
	h := NewHandler("1.0.0", nil, nil)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	rec := httptest.NewRecorder()

	h.Liveness(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	var res Response
	if err := json.NewDecoder(rec.Body).Decode(&res); err != nil {
		t.Fatalf("failed to decode json: %v", err)
	}

	if res.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", res.Status)
	}
	if res.Version != "1.0.0" {
		t.Errorf("expected 1.0.0, got %s", res.Version)
	}
}

func TestHealthReadiness_NoDependencies(t *testing.T) {
	h := NewHandler("1.0.0", nil, nil)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/health/ready", nil)
	rec := httptest.NewRecorder()

	h.Readiness(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	var res Response
	if err := json.NewDecoder(rec.Body).Decode(&res); err != nil {
		t.Fatalf("failed to decode json: %v", err)
	}

	if res.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", res.Status)
	}
	if res.Checks["database"] != "ok" {
		t.Errorf("expected db ok, got %s", res.Checks["database"])
	}
	if res.Checks["redis"] != "disabled" {
		t.Errorf("expected redis disabled, got %s", res.Checks["redis"])
	}
}

func TestHealthLiveness_ShuttingDown(t *testing.T) {
	h := NewHandler("1.0.0", nil, nil)
	h.SetLive(false)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	rec := httptest.NewRecorder()

	h.Liveness(rec, req)

	if rec.Code != http.StatusServiceUnavailable {
		t.Fatalf("expected 503 during shutdown, got %d", rec.Code)
	}
}
