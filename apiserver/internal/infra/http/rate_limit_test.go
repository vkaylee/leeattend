package httpinfra

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

type mockRateLimiterBackend struct {
	counts    map[string]uint64
	shouldErr bool
}

func (m *mockRateLimiterBackend) CheckRateLimit(_ context.Context, key string, _ uint64) (uint64, error) {
	if m.shouldErr {
		return 0, errors.New("redis timeout")
	}
	if m.counts == nil {
		m.counts = make(map[string]uint64)
	}
	m.counts[key]++
	return m.counts[key], nil
}

func TestRateLimitMiddleware_HealthBypass(t *testing.T) {
	backend := &mockRateLimiterBackend{
		counts: map[string]uint64{
			"rate:ip:unknown:r:/api/v1/health": 9999,
		},
	}
	cfg := RateLimitConfig{MaxReadRequests: 1, MaxWriteRequests: 1, WindowSecs: 10}
	mw := RateLimitMiddleware(cfg, "secret", backend)

	handler := mw(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for health endpoint, got %d", rec.Code)
	}
}

func TestRateLimitMiddleware_ExceededReturns429(t *testing.T) {
	backend := &mockRateLimiterBackend{
		counts: map[string]uint64{
			"rate:ip:192.0.2.1:r:/api/v1/test-route": 2, // seeded 2 hits for httptest remote address
		},
	}
	cfg := RateLimitConfig{MaxReadRequests: 2, MaxWriteRequests: 1, WindowSecs: 10}
	mw := RateLimitMiddleware(cfg, "secret", backend)

	handler := mw(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/api/v1/test-route", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req) // 3rd hit -> exceeds limit 2

	if rec.Code != http.StatusTooManyRequests {
		t.Fatalf("expected 429 Too Many Requests, got %d", rec.Code)
	}
	if rec.Header().Get("Retry-After") != "10" {
		t.Fatalf("expected Retry-After: 10, got %q", rec.Header().Get("Retry-After"))
	}
	if rec.Header().Get("Content-Type") != "application/json" {
		t.Fatalf("expected Content-Type: application/json, got %q", rec.Header().Get("Content-Type"))
	}
	expectedBody := `{"error":{"code":"RATE_LIMITED","message":"Too many requests. Please try again later."}}`
	if rec.Body.String() != expectedBody {
		t.Fatalf("expected body %q, got %q", expectedBody, rec.Body.String())
	}
}

func TestRateLimitMiddleware_IdentityResolution(t *testing.T) {
	secret := "jwt-secret-with-sufficient-length-32-bytes!!"
	userID := uuid.New()
	token, err := auth.GenerateToken(ids.UserID(userID), secret, 15*time.Minute)
	if err != nil {
		t.Fatal(err)
	}

	backend := &mockRateLimiterBackend{}
	cfg := RateLimitConfig{MaxReadRequests: 10, MaxWriteRequests: 10, WindowSecs: 60}
	mw := RateLimitMiddleware(cfg, secret, backend)

	handler := mw(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	// 1. Authenticated via JWT
	reqAuth := httptest.NewRequest(http.MethodGet, "/api/v1/data", nil)
	reqAuth.Header.Set("Authorization", "Bearer "+token)
	recAuth := httptest.NewRecorder()
	handler.ServeHTTP(recAuth, reqAuth)
	if backend.counts["rate:u:"+userID.String()+":r:/api/v1/data"] != 1 {
		t.Fatalf("expected key for authenticated user, got %+v", backend.counts)
	}

	// 2. Unauthenticated with X-Forwarded-For
	reqXFF := httptest.NewRequest(http.MethodPost, "/api/v1/data", nil)
	reqXFF.RemoteAddr = ""
	reqXFF.Header.Set("X-Forwarded-For", "203.0.113.195, 10.0.0.1")
	recXFF := httptest.NewRecorder()
	handler.ServeHTTP(recXFF, reqXFF)
	if backend.counts["rate:ip:203.0.113.195:w:/api/v1/data"] != 1 {
		t.Fatalf("expected key for XFF IP, got %+v", backend.counts)
	}
}

func TestRateLimitMiddleware_FallbackOnBackendError(t *testing.T) {
	backend := &mockRateLimiterBackend{shouldErr: true}
	cfg := RateLimitConfig{MaxReadRequests: 2, MaxWriteRequests: 1, WindowSecs: 10}
	mw := RateLimitMiddleware(cfg, "secret", backend)

	handler := mw(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))

	// Hit 1: allowed via in-memory fallback
	req1 := httptest.NewRequest(http.MethodGet, "/api/v1/test", nil)
	rec1 := httptest.NewRecorder()
	handler.ServeHTTP(rec1, req1)
	if rec1.Code != http.StatusOK {
		t.Fatalf("expected 200 on first hit, got %d", rec1.Code)
	}

	// Hit 2: allowed
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1/test", nil)
	rec2 := httptest.NewRecorder()
	handler.ServeHTTP(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("expected 200 on second hit, got %d", rec2.Code)
	}

	// Hit 3: rate limited
	req3 := httptest.NewRequest(http.MethodGet, "/api/v1/test", nil)
	rec3 := httptest.NewRecorder()
	handler.ServeHTTP(rec3, req3)
	if rec3.Code != http.StatusTooManyRequests {
		t.Fatalf("expected 429 on third hit, got %d", rec3.Code)
	}
}
