package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

func TestRegisterRoutePreservesPathValue(t *testing.T) {
	router := chi.NewRouter()
	registerRoute(router, "GET /employees/{employee_id}/contracts/{contract_id}", func(w http.ResponseWriter, r *http.Request) {
		if got := r.PathValue("employee_id"); got != "employee-1" {
			t.Errorf("employee_id = %q, want %q", got, "employee-1")
		}
		if got := r.PathValue("contract_id"); got != "contract-2" {
			t.Errorf("contract_id = %q, want %q", got, "contract-2")
		}
		w.WriteHeader(http.StatusNoContent)
	})

	req := httptest.NewRequest(http.MethodGet, "/employees/employee-1/contracts/contract-2", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusNoContent)
	}
}

func TestRegisterRoutePreservesMethodAndNotFoundBehavior(t *testing.T) {
	router := chi.NewRouter()
	registerRoute(router, "POST /employees", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusCreated)
	})

	tests := []struct {
		name   string
		method string
		path   string
		want   int
	}{
		{name: "registered method", method: http.MethodPost, path: "/employees", want: http.StatusCreated},
		{name: "wrong method", method: http.MethodGet, path: "/employees", want: http.StatusMethodNotAllowed},
		{name: "unknown path", method: http.MethodPost, path: "/unknown", want: http.StatusNotFound},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			req := httptest.NewRequest(test.method, test.path, nil)
			rec := httptest.NewRecorder()
			router.ServeHTTP(rec, req)
			if rec.Code != test.want {
				t.Fatalf("status = %d, want %d", rec.Code, test.want)
			}
		})
	}
}

func TestProtectedRouterRequiresAuthentication(t *testing.T) {
	router := chi.NewRouter()
	protected := router.With(httpinfra.AuthMiddleware("test-secret", nil))
	registerRoute(protected, "GET /protected", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})

	req := httptest.NewRequest(http.MethodGet, "/protected", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusUnauthorized)
	}
}

func TestPublicRouterDoesNotRequireAuthentication(t *testing.T) {
	router := chi.NewRouter()
	router.Get("/health", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusNoContent)
	}
}

func TestPayrollRoutesRetireSalaryBases(t *testing.T) {
	router := chi.NewRouter()
	registerPayrollRoutes(router, api.PayrollHandler{})
	for _, route := range []struct {
		method string
		path   string
	}{
		{http.MethodGet, "/api/v1/salary-bases"},
		{http.MethodPost, "/api/v1/salary-bases"},
		{http.MethodPost, "/api/v1/salary-bases/b0000000-0000-0000-0000-000000000001/members"},
		{http.MethodDelete, "/api/v1/salary-bases/b0000000-0000-0000-0000-000000000001/members/11111111-1111-1111-1111-111111111111"},
	} {
		t.Run(route.method+" "+route.path, func(t *testing.T) {
			rec := httptest.NewRecorder()
			router.ServeHTTP(rec, httptest.NewRequest(route.method, route.path, nil))
			if rec.Code != http.StatusNotFound {
				t.Fatalf("retired route status = %d, want 404", rec.Code)
			}
		})
	}
	for _, method := range []string{http.MethodGet, http.MethodPost} {
		if !router.Match(chi.NewRouteContext(), method, "/api/v1/salary-components") {
			t.Fatalf("retained salary component route missing: %s", method)
		}
	}
}

func TestRouteParamNames(t *testing.T) {
	got := routeParamNames("/employees/{employee_id}/contracts/{contract_id}")
	want := []string{"employee_id", "contract_id"}
	if len(got) != len(want) {
		t.Fatalf("param count = %d, want %d", len(got), len(want))
	}
	for i := range want {
		if got[i] != want[i] {
			t.Errorf("param[%d] = %q, want %q", i, got[i], want[i])
		}
	}
}
