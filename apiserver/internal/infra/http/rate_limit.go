package httpinfra

import (
	"context"
	"net"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

type RateLimitConfig struct {
	MaxReadRequests  uint64
	MaxWriteRequests uint64
	WindowSecs       uint64
}

func DefaultRateLimitConfig() RateLimitConfig {
	return RateLimitConfig{
		MaxReadRequests:  120,
		MaxWriteRequests: 30,
		WindowSecs:       60,
	}
}

type RateLimiterBackend interface {
	CheckRateLimit(context.Context, string, uint64) (uint64, error)
}

type inMemoryBucket struct {
	count     uint64
	expiresAt time.Time
}

type inMemoryRateLimiter struct {
	mu      sync.Mutex
	buckets map[string]inMemoryBucket
}

func newInMemoryRateLimiter() *inMemoryRateLimiter {
	return &inMemoryRateLimiter{buckets: make(map[string]inMemoryBucket)}
}

func (m *inMemoryRateLimiter) check(key string, windowSecs uint64) uint64 {
	m.mu.Lock()
	defer m.mu.Unlock()

	now := time.Now()
	bucket, ok := m.buckets[key]
	if !ok || now.After(bucket.expiresAt) {
		m.buckets[key] = inMemoryBucket{
			count:     1,
			expiresAt: now.Add(time.Duration(windowSecs) * time.Second),
		}
		if len(m.buckets) > 10000 {
			for candidateKey, candidate := range m.buckets {
				if now.After(candidate.expiresAt) {
					delete(m.buckets, candidateKey)
				}
			}
		}
		return 1
	}

	bucket.count++
	m.buckets[key] = bucket
	return bucket.count
}

// RateLimitMiddleware enforces method-based rate limiting with Redis and in-memory fallback.
func RateLimitMiddleware(cfg RateLimitConfig, jwtSecret string, backend RateLimiterBackend) Middleware {
	if cfg.MaxReadRequests == 0 {
		cfg.MaxReadRequests = 120
	}
	if cfg.MaxWriteRequests == 0 {
		cfg.MaxWriteRequests = 30
	}
	if cfg.WindowSecs == 0 {
		cfg.WindowSecs = 60
	}

	fallback := newInMemoryRateLimiter()

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			path := r.Pattern
			if path == "" {
				path = r.URL.Path
			}
			if strings.HasSuffix(path, "/health") || strings.HasSuffix(path, "/health/ready") {
				next.ServeHTTP(w, r)
				return
			}

			identity := resolveRateLimitIdentity(r, jwtSecret)
			tier := rateLimitTier(r.Method)
			limit := cfg.MaxWriteRequests
			if tier == "r" {
				limit = cfg.MaxReadRequests
			}
			key := "rate:" + identity + ":" + tier + ":" + path

			var count uint64
			var checked bool
			if backend != nil {
				candidate, err := backend.CheckRateLimit(r.Context(), key, cfg.WindowSecs)
				if err == nil {
					count = candidate
					checked = true
				}
			}
			if !checked {
				count = fallback.check(key, cfg.WindowSecs)
			}

			if count > limit {
				w.Header().Set("Content-Type", "application/json")
				w.Header().Set("Retry-After", strconv.FormatUint(cfg.WindowSecs, 10))
				w.WriteHeader(http.StatusTooManyRequests)
				_, _ = w.Write([]byte(`{"error":{"code":"RATE_LIMITED","message":"Too many requests. Please try again later."}}`))
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}

func rateLimitTier(method string) string {
	switch method {
	case http.MethodGet, http.MethodHead, http.MethodOptions:
		return "r"
	default:
		return "w"
	}
}

func resolveRateLimitIdentity(r *http.Request, jwtSecret string) string {
	if token := bearerToken(r); token != "" && jwtSecret != "" {
		if userID, err := auth.VerifyToken(token, jwtSecret); err == nil {
			return "u:" + userID.String()
		}
	}

	// Never trust caller-supplied forwarding headers (X-Forwarded-For, X-Real-IP, Forwarded) as rate-limit identity.
	if r.RemoteAddr != "" {
		host, _, err := net.SplitHostPort(r.RemoteAddr)
		if err == nil && host != "" {
			return "ip:" + host
		}
		return "ip:" + r.RemoteAddr
	}
	return "ip:unknown"
}
