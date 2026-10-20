package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/vkaylee/leeattend/apiserver/internal/infra/health"
)

func TestHealthRoutesMounted(t *testing.T) {
	mux := http.NewServeMux()
	h := health.NewHandler(Version, nil, nil)
	h.SetLive(true)
	mux.HandleFunc("GET /health", h.Liveness)
	mux.HandleFunc("GET /health/ready", h.Readiness)
	mux.HandleFunc("GET /api/v1/health", h.Liveness)

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	rec = httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestAppVersionConstant(t *testing.T) {
	if Version != "0.1.0" {
		t.Fatalf("unexpected version: %s", Version)
	}
}
