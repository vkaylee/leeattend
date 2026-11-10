package httpinfra

import (
	"context"
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
	contextKeyUserID         contextKey = "user_id"
	contextKeyOrganizationID contextKey = "organization_id"
	contextKeyClaims         contextKey = "jwt_claims"
	contextKeyLocale         contextKey = "locale"
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
					w.Header().Set("Access-Control-Allow-Headers", "Authorization, Content-Type, x-request-id")
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

// RejectRetiredScopedRoutes prevents clients from selecting organization scope by retired URL paths.
func RejectRetiredScopedRoutes() Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if strings.Contains(r.URL.Path, "/workspaces/") {
				http.Error(w, "legacy organization routes are not supported", http.StatusNotFound)
				return
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
			next.ServeHTTP(w, r.WithContext(ctx))
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

// OrganizationIDFromContext retrieves singleton organization UUID.
func OrganizationIDFromContext(ctx context.Context) (uuid.UUID, bool) {
	val, ok := ctx.Value(contextKeyOrganizationID).(uuid.UUID)
	return val, ok
}

// WithOrganizationID attaches the resolved organization UUID to a request context.
func WithOrganizationID(ctx context.Context, id uuid.UUID) context.Context {
	ctx = context.WithValue(ctx, contextKeyOrganizationID, id)
	return observability.WithOrganizationID(ctx, id.String())
}

// WithOrganizationIDForTest is a package test helper for handler contract tests.
func WithOrganizationIDForTest(ctx context.Context, id uuid.UUID) context.Context {
	return WithOrganizationID(ctx, id)
}
