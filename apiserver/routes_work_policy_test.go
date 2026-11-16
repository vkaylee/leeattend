package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestWorkPolicyRoutesRequireAuthorization(t *testing.T) {
	router := chi.NewRouter()
	registerWorkPolicyRoutes(router, &api.WorkPolicyGroupHandler{})

	for _, test := range []struct {
		method string
		path   string
	}{
		{method: http.MethodGet, path: "/api/v1/payroll/work-policy-groups"},
		{method: http.MethodGet, path: "/api/v1/payroll/work-policy-groups/00000000-0000-0000-0000-000000000001/history"},
		{method: http.MethodGet, path: "/api/v1/payroll/work-policy-groups/00000000-0000-0000-0000-000000000001/versions/00000000-0000-0000-0000-000000000002"},
		{method: http.MethodPost, path: "/api/v1/payroll/work-policy-groups/00000000-0000-0000-0000-000000000001/versions/00000000-0000-0000-0000-000000000002/validate"},
		{method: http.MethodPost, path: "/api/v1/payroll/work-policy-groups"},
		{method: http.MethodPost, path: "/api/v1/payroll/work-policy-groups/00000000-0000-0000-0000-000000000001/versions"},
		{method: http.MethodGet, path: "/api/v1/employments/00000000-0000-0000-0000-000000000001/work-policy?as_of=2026-01-01"},
		{method: http.MethodGet, path: "/api/v1/payroll/work-policy-groups/00000000-0000-0000-0000-000000000001/memberships"},
		{method: http.MethodPost, path: "/api/v1/payroll/work-policy-groups/00000000-0000-0000-0000-000000000001/memberships"},
	} {
		req := httptest.NewRequest(test.method, test.path, nil)
		rec := httptest.NewRecorder()
		router.ServeHTTP(rec, req)
		if rec.Code != http.StatusForbidden {
			t.Errorf("%s %s status = %d, want 403", test.method, test.path, rec.Code)
		}
	}
}
