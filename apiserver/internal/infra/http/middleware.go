package httpinfra

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/observability"
)

type contextKey string

const (
	contextKeyUserID      contextKey = "user_id"
	contextKeyWorkspaceID contextKey = "workspace_id"
	contextKeyClaims      contextKey = "jwt_claims"
	contextKeyLocale      contextKey = "locale"
)

// Middleware wraps an http.Handler with cross-cutting behavior.
type Middleware func(http.Handler) http.Handler

// Chain combines multiple middlewares from outermost to innermost.
func Chain(middlewares ...Middleware) Middleware {
	return func(final http.Handler) http.Handler {
		for i := len(middlewares) - 1; i >= 0; i-- {
			final = middlewares[i](final)
		}
		return final
	}
}

// RequestIDMiddleware injects or propagates a valid UUID x-request-id.
func RequestIDMiddleware() Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			rawID := r.Header.Get("x-request-id")
			reqID := rawID
			if _, err := uuid.Parse(rawID); err != nil {
				reqID = observability.GenerateRequestID()
			}
			w.Header().Set("x-request-id", reqID)
			ctx := observability.WithRequestID(r.Context(), reqID)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// LocaleMiddleware detects request locale (?lang= or Accept-Language) with "vi" default.
func LocaleMiddleware() Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			locale := apperror.ExtractLocale(r)
			ctx := context.WithValue(r.Context(), contextKeyLocale, locale)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// CORSMiddleware handles cross-origin requests.
func CORSMiddleware(allowedOrigins []string) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			origin := r.Header.Get("Origin")
			if origin != "" {
				allowed := false
				for _, o := range allowedOrigins {
					if o == "*" || o == origin {
						allowed = true
						break
					}
				}
				if allowed {
					w.Header().Set("Access-Control-Allow-Origin", origin)
					w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
					w.Header().Set("Access-Control-Allow-Headers", "Authorization, Content-Type, x-request-id, X-Workspace-Id")
					w.Header().Set("Access-Control-Allow-Credentials", "true")
				}
			}

			if r.Method == http.MethodOptions {
				w.WriteHeader(http.StatusOK)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// BodyLimitMiddleware enforces maximum request body size (default 2MB).
func BodyLimitMiddleware(maxBytes int64) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Body != nil && maxBytes > 0 {
				r.Body = http.MaxBytesReader(w, r.Body, maxBytes)
			}
			next.ServeHTTP(w, r)
		})
	}
}

// TokenRevocationChecker checks Redis-backed revocation state without coupling middleware to Redis.
type TokenRevocationChecker interface {
	IsSignatureRevoked(context.Context, string) (bool, error)
	UserRevokedAt(context.Context, uuid.UUID) (time.Time, error)
}

// AuthMiddleware extracts bearer, cookie, or query tokens and validates revocation state.
func AuthMiddleware(jwtSecret string, publicPrefixes []string) Middleware {
	return AuthMiddlewareWithRevocation(jwtSecret, publicPrefixes, nil)
}

func AuthMiddlewareWithRevocation(jwtSecret string, publicPrefixes []string, revocations TokenRevocationChecker) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			for _, prefix := range publicPrefixes {
				if strings.HasPrefix(r.URL.Path, prefix) {
					next.ServeHTTP(w, r)
					return
				}
			}
			token := bearerToken(r)
			if token == "" {
				apperror.WritePrimitiveAuthRequired(w)
				return
			}
			claims, err := auth.VerifyTokenClaims(token, jwtSecret)
			if err != nil {
				apperror.WritePrimitiveAuthRequired(w)
				return
			}
			userID, err := uuid.Parse(claims.Subject)
			if err != nil {
				apperror.WritePrimitiveAuthRequired(w)
				return
			}
			if revocations != nil {
				revoked, checkErr := revocations.IsSignatureRevoked(r.Context(), auth.ExtractTokenSignature(token))
				if checkErr == nil && revoked {
					apperror.WritePrimitiveAuthRequired(w)
					return
				}
				if claims.IssuedAt != nil {
					revokedAt, checkErr := revocations.UserRevokedAt(r.Context(), userID)
					if checkErr == nil && !revokedAt.IsZero() && !claims.IssuedAt.Time.After(revokedAt) {
						apperror.WritePrimitiveAuthRequired(w)
						return
					}
				}
			}
			ctx := context.WithValue(r.Context(), contextKeyUserID, userID)
			ctx = context.WithValue(ctx, contextKeyClaims, claims)
			ctx = observability.WithActorID(ctx, userID.String())
			if wsHeader := r.Header.Get("X-Workspace-Id"); wsHeader != "" {
				if wsID, parseErr := uuid.Parse(wsHeader); parseErr == nil {
					ctx = context.WithValue(ctx, contextKeyWorkspaceID, wsID)
					ctx = observability.WithWorkspaceID(ctx, wsID.String())
				}
			}
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// StandaloneWorkspaceMiddleware rejects workspace IDs other than the one local organization.
// It preserves workspace IDs in existing routes while preventing tenant switching.
func StandaloneWorkspaceMiddleware(workspaceID uuid.UUID) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if workspaceID == uuid.Nil {
				apperror.WriteError(w, r, apperror.Internal(errors.New("standalone workspace is not configured")))
				return
			}
			if raw := r.PathValue("ws_id"); raw != "" {
				pathID, err := uuid.Parse(raw)
				if err != nil || pathID != workspaceID {
					apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
					return
				}
			}
			if raw := r.Header.Get("X-Workspace-Id"); raw != "" {
				headerID, err := uuid.Parse(raw)
				if err != nil || headerID != workspaceID {
					apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
					return
				}
			}
			next.ServeHTTP(w, r)
		})
	}
}

// StandaloneSaaSRouteMiddleware blocks control-plane and self-registration routes.
func StandaloneSaaSRouteMiddleware() Middleware {
	blocked := map[string]bool{
		http.MethodPost + " /api/v1/workspaces":    true,
		http.MethodGet + " /api/v1/workspaces":     true,
		http.MethodPost + " /api/v1/auth/register": true,
		http.MethodPost + " /auth/register":        true,
	}
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if blocked[r.Method+" "+r.URL.Path] || strings.HasPrefix(r.URL.Path, "/api/v1/system/tenants") || strings.HasPrefix(r.URL.Path, "/api/v1/system/db-clusters") {
				apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}

func bearerToken(r *http.Request) string {
	if header := r.Header.Get("Authorization"); strings.HasPrefix(header, "Bearer ") {
		return strings.TrimSpace(strings.TrimPrefix(header, "Bearer "))
	}
	if cookie, err := r.Cookie("access_token"); err == nil && cookie.Value != "" {
		return cookie.Value
	}
	return ""
}

// UserIDFromContext retrieves authenticated user UUID.
func UserIDFromContext(ctx context.Context) (uuid.UUID, bool) {
	val, ok := ctx.Value(contextKeyUserID).(uuid.UUID)
	return val, ok
}

// WithUserIDForTest is a package test helper for handler contract tests.
func WithUserIDForTest(ctx context.Context, id uuid.UUID) context.Context {
	return context.WithValue(ctx, contextKeyUserID, id)
}

// WorkspaceIDFromContext retrieves workspace UUID.
func WorkspaceIDFromContext(ctx context.Context) (uuid.UUID, bool) {
	val, ok := ctx.Value(contextKeyWorkspaceID).(uuid.UUID)
	return val, ok
}

// WithWorkspaceIDForTest is a package test helper for handler contract tests.
func WithWorkspaceIDForTest(ctx context.Context, id uuid.UUID) context.Context {
	return context.WithValue(ctx, contextKeyWorkspaceID, id)
}
