package httpinfra

import (
	"context"
	"net/http"
	"sync"
	"time"

	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

const featureFlagCacheTTL = time.Minute

// FeatureFlagLookup resolves one runtime flag. Errors fail closed.
type FeatureFlagLookup func(context.Context, string) (bool, error)

// FeatureFlagMiddleware blocks a route group while its runtime flag is disabled.
// It caches the result briefly to avoid a database query on every request.
func FeatureFlagMiddleware(flagKey string, lookup FeatureFlagLookup) Middleware {
	var mu sync.Mutex
	var cached bool
	var cachedEnabled bool
	var cachedAt time.Time

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			now := time.Now()
			mu.Lock()
			if cached && now.Sub(cachedAt) < featureFlagCacheTTL {
				enabled := cachedEnabled
				mu.Unlock()
				if !enabled {
					apperror.WriteError(w, r, apperror.FeatureDisabled(flagKey))
					return
				}
				next.ServeHTTP(w, r)
				return
			}
			mu.Unlock()

			if lookup == nil {
				apperror.WriteError(w, r, apperror.FeatureDisabled(flagKey))
				return
			}
			enabled, err := lookup(r.Context(), flagKey)
			mu.Lock()
			cached = true
			cachedEnabled = err == nil && enabled
			cachedAt = now
			enabled = cachedEnabled
			mu.Unlock()
			if !enabled {
				apperror.WriteError(w, r, apperror.FeatureDisabled(flagKey))
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}
