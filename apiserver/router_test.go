package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/config"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
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
	registerPayrollRoutes(router, &api.PayrollHandler{})
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

func TestCompatibilityRouteRegistrationsRemainReachable(t *testing.T) {
	router := chi.NewRouter()
	registerAttendanceRoutes(router, &api.AttendanceHandler{}, &api.AttendanceLogsHandler{}, &api.AttendancePolicyHandler{})
	registerLeaveRoutes(router, &api.LeaveHandler{}, &api.HolidayHandler{})

	tests := []struct {
		name   string
		method string
		path   string
	}{
		{"paper overtime bulk legacy path", http.MethodPost, "/api/v1/attendance/bulk-upload-paper-ot"},
		{"paper overtime bulk current path", http.MethodPost, "/api/v1/attendance/paper-forms/overtime/bulk"},
		{"paper overtime employee legacy path", http.MethodPost, "/api/v1/attendance/enter-paper-ot/00000000-0000-0000-0000-000000000001"},
		{"paper overtime employee current path", http.MethodPost, "/api/v1/attendance/paper-forms/overtime/00000000-0000-0000-0000-000000000001"},
		{"holiday put alias", http.MethodPut, "/api/v1/holidays/00000000-0000-0000-0000-000000000001"},
		{"holiday patch alias", http.MethodPatch, "/api/v1/holidays/00000000-0000-0000-0000-000000000001"},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			rec := httptest.NewRecorder()
			router.ServeHTTP(rec, httptest.NewRequest(test.method, test.path, nil))
			if rec.Code == http.StatusNotFound || rec.Code == http.StatusMethodNotAllowed {
				t.Fatalf("compatibility route %s %s unreachable: status %d", test.method, test.path, rec.Code)
			}
		})
	}
}

func TestBuildRouterPreservesPublicProtectedESSAndWebSocketScopes(t *testing.T) {
	cfg := &config.Config{
		JWTSecret:           config.JWTSecret("test-secret-at-least-32-chars-long-for-jwt!!"),
		S3EndpointURL:       "://invalid",
		RateLimitMaxRead:    1000,
		RateLimitMaxWrite:   1000,
		RateLimitWindowSecs: 60,
		AllowedOrigins:      []string{"https://example.test"},
	}
	logger := slog.New(slog.NewTextHandler(httptest.NewRecorder(), nil))
	router := buildRouter(cfg, nil, nil, &api.AuthHandler{}, logger)

	tests := []struct {
		name   string
		method string
		path   string
		want   int
	}{
		{"public health", http.MethodGet, "/api/v1/health", http.StatusOK},
		{"protected payroll", http.MethodGet, "/api/v1/payroll/payslips", http.StatusUnauthorized},
		{"protected ESS", http.MethodGet, "/api/v1/ess/profile", http.StatusUnauthorized},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			rec := httptest.NewRecorder()
			router.ServeHTTP(rec, httptest.NewRequest(test.method, test.path, nil))
			if rec.Code != test.want {
				t.Fatalf("%s %s status = %d, want %d; body=%s", test.method, test.path, rec.Code, test.want, rec.Body.String())
			}
		})
	}
}

func TestBuildRouterReturns404And405ForUnknownAndWrongMethods(t *testing.T) {
	cfg := &config.Config{
		JWTSecret:           config.JWTSecret("test-secret-at-least-32-chars-long-for-jwt!!"),
		S3EndpointURL:       "://invalid",
		RateLimitMaxRead:    1000,
		RateLimitMaxWrite:   1000,
		RateLimitWindowSecs: 60,
	}
	router := buildRouter(cfg, nil, nil, &api.AuthHandler{}, slog.Default())
	for _, test := range []struct {
		name   string
		method string
		path   string
		want   int
	}{
		{"unknown path", http.MethodGet, "/api/v1/route-does-not-exist", http.StatusNotFound},
		{"wrong method", http.MethodPost, "/api/v1/health", http.StatusMethodNotAllowed},
	} {
		t.Run(test.name, func(t *testing.T) {
			rec := httptest.NewRecorder()
			router.ServeHTTP(rec, httptest.NewRequest(test.method, test.path, nil))
			if rec.Code != test.want {
				t.Fatalf("%s %s status = %d, want %d", test.method, test.path, rec.Code, test.want)
			}
		})
	}
}

func TestBuildRouterReadinessNilDependenciesRemainDegradedOnlyByRedis(t *testing.T) {
	cfg := &config.Config{
		JWTSecret:           config.JWTSecret("test-secret-at-least-32-chars-long-for-jwt!!"),
		S3EndpointURL:       "://invalid",
		RateLimitMaxRead:    1000,
		RateLimitMaxWrite:   1000,
		RateLimitWindowSecs: 60,
	}
	router := buildRouter(cfg, nil, nil, &api.AuthHandler{}, slog.Default())
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/api/v1/health/ready", nil))
	if rec.Code != http.StatusOK {
		t.Fatalf("readiness status = %d, want 200: %s", rec.Code, rec.Body.String())
	}
	var body struct {
		Checks map[string]string `json:"checks"`
	}
	if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
		t.Fatalf("decode readiness response: %v", err)
	}
	if body.Checks["database"] != "ok" || body.Checks["redis"] != "disabled" {
		t.Fatalf("readiness checks = %#v, want database=ok redis=disabled", body.Checks)
	}
}

func TestBuildRouterEmployeeMatchAuthAndRateLimit(t *testing.T) {
	const jwtSecret = "test-secret-at-least-32-chars-long-for-jwt!!"
	cfg := &config.Config{
		JWTSecret:                config.JWTSecret(jwtSecret),
		IdentityLookupHMACSecret: config.Secret("0123456789abcdef0123456789abcdef"),
		S3EndpointURL:            "://invalid",
		RateLimitMaxRead:         120,
		RateLimitMaxWrite:        30,
		RateLimitWindowSecs:      60,
	}
	router := buildRouter(cfg, nil, nil, &api.AuthHandler{}, slog.Default())

	unauthRec := httptest.NewRecorder()
	router.ServeHTTP(unauthRec, httptest.NewRequest(http.MethodPost, "/api/v1/employees/match", bytes.NewBufferString(`{}`)))
	if unauthRec.Code != http.StatusUnauthorized {
		t.Fatalf("unauthenticated match status = %d, want 401", unauthRec.Code)
	}

	unauthTenureRec := httptest.NewRecorder()
	router.ServeHTTP(unauthTenureRec, httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+uuid.NewString()+"/tenures", bytes.NewBufferString(`{}`)))
	if unauthTenureRec.Code != http.StatusUnauthorized {
		t.Fatalf("unauthenticated create tenure status = %d, want 401", unauthTenureRec.Code)
	}

	assertActorMatchRateLimit(t, router, jwtSecret, false)
	assertActorMatchRateLimit(t, router, jwtSecret, true)
}

func assertActorMatchRateLimit(t *testing.T, router http.Handler, jwtSecret string, useCookie bool) {
	t.Helper()
	token, err := auth.GenerateToken(ids.UserID(uuid.New()), jwtSecret, 15*time.Minute)
	if err != nil {
		t.Fatalf("GenerateToken: %v", err)
	}
	for i := 1; i <= 31; i++ {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/match", bytes.NewBufferString(`{}`))
		if useCookie {
			req.AddCookie(&http.Cookie{Name: "access_token", Value: token})
		} else {
			req.Header.Set("Authorization", "Bearer "+token)
		}
		// Rotate spoofed forwarding headers on every attempt to prove identity cannot be forged.
		req.Header.Set("X-Forwarded-For", fmt.Sprintf("203.0.113.%d", i))
		req.Header.Set("X-Real-IP", fmt.Sprintf("198.51.100.%d", i))
		req.Header.Set("Forwarded", fmt.Sprintf("for=192.0.2.%d", i))

		rec := httptest.NewRecorder()
		router.ServeHTTP(rec, req)
		if i <= 30 && rec.Code == http.StatusTooManyRequests {
			t.Fatalf("useCookie=%v request %d hit 429 early", useCookie, i)
		}
		if i == 31 && rec.Code != http.StatusTooManyRequests {
			t.Fatalf("useCookie=%v request 31 status = %d, want 429", useCookie, rec.Code)
		}
		if i == 31 && rec.Header().Get("Retry-After") != "60" {
			t.Fatalf("useCookie=%v Retry-After = %q, want 60", useCookie, rec.Header().Get("Retry-After"))
		}
	}
}
