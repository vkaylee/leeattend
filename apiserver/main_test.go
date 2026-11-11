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

func TestOrganizationSettingsRoutesRejectLegacyAlias(t *testing.T) {
	mux := http.NewServeMux()
	get := func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusNoContent) }
	patch := func(w http.ResponseWriter, _ *http.Request) { w.WriteHeader(http.StatusNoContent) }
	registerOrganizationSettingsRoutes(mux, get, patch)

	for _, test := range []struct {
		method string
		path   string
	}{
		{method: http.MethodGet, path: "/api/v1"},
		{method: http.MethodPatch, path: "/api/v1"},
		{method: http.MethodGet, path: "/api/v1/organization/settings"},
		{method: http.MethodPatch, path: "/api/v1/organization/settings"},
	} {
		req := httptest.NewRequest(test.method, test.path, nil)
		rec := httptest.NewRecorder()
		mux.ServeHTTP(rec, req)
		want := http.StatusNoContent
		if test.path != "/api/v1" {
			want = http.StatusNotFound
		}
		if rec.Code != want {
			t.Fatalf("%s %s: expected %d, got %d", test.method, test.path, want, rec.Code)
		}
	}
}

func TestAppVersionConstant(t *testing.T) {
	if Version != "0.1.0" {
		t.Fatalf("unexpected version: %s", Version)
	}
}
