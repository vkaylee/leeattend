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
	registerWorkPolicyRoutes(router, api.WorkPolicyGroupHandler{})

	for _, test := range []struct {
		method string
		path   string
	}{
		{method: http.MethodGet, path: "/api/v1/payroll/work-policy-groups"},
		{method: http.MethodPost, path: "/api/v1/payroll/work-policy-groups"},
	} {
		req := httptest.NewRequest(test.method, test.path, nil)
		rec := httptest.NewRecorder()
		router.ServeHTTP(rec, req)
		if rec.Code != http.StatusForbidden {
			t.Errorf("%s %s status = %d, want 403", test.method, test.path, rec.Code)
		}
	}
}
