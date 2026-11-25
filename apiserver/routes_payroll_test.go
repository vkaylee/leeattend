package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestPayrollRoutesRegisterPayrollPeriodEndpoints(t *testing.T) {
	router := chi.NewRouter()
	registerPayrollRoutes(router, &api.PayrollHandler{})

	employerID := "00000000-0000-0000-0000-000000000001"
	for _, test := range []struct {
		name   string
		method string
		path   string
	}{
		{name: "get", method: http.MethodGet, path: "/api/v1/payroll/periods/2026/1?employer_id=" + employerID},
		{name: "lock", method: http.MethodPost, path: "/api/v1/payroll/periods/2026/1/lock?employer_id=" + employerID},
		{name: "reopen", method: http.MethodPost, path: "/api/v1/payroll/periods/2026/1/reopen?employer_id=" + employerID},
	} {
		t.Run(test.name, func(t *testing.T) {
			req := httptest.NewRequest(test.method, test.path, nil)
			rec := httptest.NewRecorder()
			router.ServeHTTP(rec, req)
			if rec.Code == http.StatusNotFound {
				t.Fatalf("payroll period route is not registered: got %d", rec.Code)
			}
			if rec.Code != http.StatusForbidden {
				t.Fatalf("payroll period route = %d, want 403 before authorization", rec.Code)
			}
		})
	}
}

func TestPayrollRoutesRegisterSalaryManagement(t *testing.T) {
	router := chi.NewRouter()
	registerPayrollRoutes(router, &api.PayrollHandler{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/salary-management", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want 403 from registered protected endpoint", rec.Code)
	}
}

func TestPayrollRoutesRegisterWorkPolicyBatchSimulation(t *testing.T) {
	router := chi.NewRouter()
	registerPayrollRoutes(router, &api.PayrollHandler{})

	req := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/work-policy-groups/00000000-0000-0000-0000-000000000001/versions/00000000-0000-0000-0000-000000000002/simulate", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want 403 from registered protected simulate endpoint", rec.Code)
	}
}
