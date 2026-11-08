package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type fixtureRevocationChecker struct {
	revokedSigs map[string]bool
	userCutoffs map[uuid.UUID]time.Time
}

func (c *fixtureRevocationChecker) IsSignatureRevoked(_ context.Context, sig string) (bool, error) {
	return c.revokedSigs[sig], nil
}

func (c *fixtureRevocationChecker) UserRevokedAt(_ context.Context, uid uuid.UUID) (time.Time, error) {
	return c.userCutoffs[uid], nil
}
func stringPointer(value string) *string { return &value }

type fixtureRateLimiter struct {
	counts map[string]uint64
}

func (f *fixtureRateLimiter) CheckRateLimit(_ context.Context, key string, _ uint64) (uint64, error) {
	if f.counts == nil {
		f.counts = make(map[string]uint64)
	}
	f.counts[key]++
	return f.counts[key], nil
}

func setupFixtureServer(t *testing.T, jwtSecret string, revocation *fixtureRevocationChecker, limiter *fixtureRateLimiter) http.Handler {
	t.Helper()

	repo := &authTestRepository{
		user: service.UserRecord{
			ID:           uuid.MustParse("018d0000-0000-7000-8000-000000000010"),
			Email:        "member@example.com",
			PasswordHash: "$argon2id$v=19$m=65536,t=1,p=4$fake$fake",
			FullName:     stringPointer("Member"),
		},
		sessions: make(map[string]service.SessionRecord),
	}

	authHandler := AuthHandler{
		Service: service.AuthService{
			Repository: repo,
			JWTSecret:  jwtSecret,
		},
	}

	workspaceHandler := WorkspaceHandler{
		Repository: nil,
		Authorizer: testAuthorizer{
			permissions: map[string]bool{
				"018d0000-0000-7000-8000-000000000020:018d0000-0000-7000-8000-000000000001:ess:profile": true,
			},
		},
	}

	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/v1/me", authHandler.Me)
	mux.Handle("PATCH /api/v1", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r = r.WithContext(httpinfra.WithOrganizationIDForTest(r.Context(), uuid.MustParse("018d0000-0000-7000-8000-000000000001")))
		workspaceHandler.Patch(w, r)
	}))
	mux.HandleFunc("GET /api/v1/test-route", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	publicPrefixes := []string{"/api/v1/health"}
	rateLimitConfig := httpinfra.RateLimitConfig{
		MaxReadRequests:  1000,
		MaxWriteRequests: 1,
		WindowSecs:       10,
	}

	return httpinfra.Chain(
		httpinfra.RequestIDMiddleware(),
		httpinfra.RateLimitMiddleware(rateLimitConfig, jwtSecret, limiter),
		httpinfra.LocaleMiddleware(),
		httpinfra.AuthMiddlewareWithRevocation(jwtSecret, publicPrefixes, revocation),
	)(mux)
}

type testAuthorizer struct {
	permissions map[string]bool
}

func (a testAuthorizer) HasPermission(_ context.Context, userID, workspaceID uuid.UUID, perm string) bool {
	key := userID.String() + ":" + workspaceID.String() + ":" + perm
	return a.permissions[key]
}

func TestAuthFixtures_Parity(t *testing.T) {
	secret := "test-secret-at-least-32-chars-long-for-jwt!!"
	revocation := &fixtureRevocationChecker{
		revokedSigs: make(map[string]bool),
		userCutoffs: make(map[uuid.UUID]time.Time),
	}
	limiter := &fixtureRateLimiter{
		counts: map[string]uint64{
			"rate:ip:unknown:r:/api/v1/test-route": 1000, // seeded 1000 hits
		},
	}
	server := setupFixtureServer(t, secret, revocation, limiter)

	t.Run("AUTH-001: Missing token returns 401 Authentication required", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/me", nil)
		rec := httptest.NewRecorder()
		server.ServeHTTP(rec, req)

		if rec.Code != http.StatusUnauthorized {
			t.Fatalf("expected 401, got %d", rec.Code)
		}
		if !strings.HasPrefix(rec.Header().Get("Content-Type"), "text/plain") {
			t.Fatalf("expected text/plain, got %q", rec.Header().Get("Content-Type"))
		}
		if strings.TrimSpace(rec.Body.String()) != "Authentication required" {
			t.Fatalf("expected 'Authentication required', got %q", rec.Body.String())
		}
	})

	t.Run("AUTH-002: Invalid token signature returns 401", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/me", nil)
		req.Header.Set("Authorization", "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.e30.tampered_invalid_signature")
		rec := httptest.NewRecorder()
		server.ServeHTTP(rec, req)

		if rec.Code != http.StatusUnauthorized {
			t.Fatalf("expected 401, got %d", rec.Code)
		}
		if strings.TrimSpace(rec.Body.String()) != "Authentication required" {
			t.Fatalf("expected 'Authentication required', got %q", rec.Body.String())
		}
	})

	t.Run("AUTH-003: Expired token returns 401", func(t *testing.T) {
		now := time.Now().UTC()
		expiredToken, err := auth.GenerateTokenWithTimes(uuid.New(), secret, now.Add(-20*time.Minute), now.Add(-10*time.Minute))
		if err != nil {
			t.Fatal(err)
		}
		req := httptest.NewRequest(http.MethodGet, "/api/v1/me", nil)
		req.Header.Set("Authorization", "Bearer "+expiredToken)
		rec := httptest.NewRecorder()
		server.ServeHTTP(rec, req)

		if rec.Code != http.StatusUnauthorized {
			t.Fatalf("expected 401, got %d", rec.Code)
		}
		if strings.TrimSpace(rec.Body.String()) != "Authentication required" {
			t.Fatalf("expected 'Authentication required', got %q", rec.Body.String())
		}
	})

	t.Run("AUTH-004: Revoked token signature in blocklist returns 401", func(t *testing.T) {
		validToken, err := auth.GenerateToken(uuid.New(), secret, 15*time.Minute)
		if err != nil {
			t.Fatal(err)
		}
		sig := auth.ExtractTokenSignature(validToken)
		revocation.revokedSigs[sig] = true

		req := httptest.NewRequest(http.MethodGet, "/api/v1/me", nil)
		req.Header.Set("Authorization", "Bearer "+validToken)
		rec := httptest.NewRecorder()
		server.ServeHTTP(rec, req)

		if rec.Code != http.StatusUnauthorized {
			t.Fatalf("expected 401 for revoked token, got %d", rec.Code)
		}
		if strings.TrimSpace(rec.Body.String()) != "Authentication required" {
			t.Fatalf("expected 'Authentication required', got %q", rec.Body.String())
		}
	})

	t.Run("AUTH-006: Missing permission on protected route returns 403 FORBIDDEN", func(t *testing.T) {
		memberID := uuid.MustParse("018d0000-0000-7000-8000-000000000020")
		token, err := auth.GenerateToken(memberID, secret, 15*time.Minute)
		if err != nil {
			t.Fatal(err)
		}
		req := httptest.NewRequest(http.MethodPatch, "/api/v1", strings.NewReader(`{"name":"Attempted Rename"}`))
		req.Header.Set("Authorization", "Bearer "+token)
		req.Header.Set("Content-Type", "application/json")
		rec := httptest.NewRecorder()
		server.ServeHTTP(rec, req)

		if rec.Code != http.StatusForbidden {
			t.Fatalf("expected 403, got %d", rec.Code)
		}
		var errResp struct {
			Error struct {
				Code    string `json:"code"`
				Message string `json:"message"`
			} `json:"error"`
		}
		if err := json.NewDecoder(rec.Body).Decode(&errResp); err != nil {
			t.Fatalf("decode response: %v", err)
		}
		if errResp.Error.Code != "FORBIDDEN" {
			t.Fatalf("expected error code FORBIDDEN, got %q", errResp.Error.Code)
		}
	})

	t.Run("AUTH-008: Rate limit trip returns 429 with Retry-After", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/test-route", nil)
		req.RemoteAddr = ""
		req.Header.Set("X-Forwarded-For", "unknown")
		rec := httptest.NewRecorder()
		server.ServeHTTP(rec, req)

		if rec.Code != http.StatusTooManyRequests {
			t.Fatalf("expected 429, got %d: %s", rec.Code, rec.Body.String())
		}
		if rec.Header().Get("Retry-After") != "10" {
			t.Fatalf("expected Retry-After: 10, got %q", rec.Header().Get("Retry-After"))
		}
		var errResp struct {
			Error struct {
				Code    string `json:"code"`
				Message string `json:"message"`
			} `json:"error"`
		}
		if err := json.NewDecoder(rec.Body).Decode(&errResp); err != nil {
			t.Fatalf("decode response: %v", err)
		}
		if errResp.Error.Code != "RATE_LIMITED" {
			t.Fatalf("expected code RATE_LIMITED, got %q", errResp.Error.Code)
		}
	})
}
