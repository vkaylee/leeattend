package httpinfra

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

func TestFeatureFlagMiddlewareBlocksDisabledAndCaches(t *testing.T) {
	lookups := 0
	handler := FeatureFlagMiddleware("ff_ess", func(context.Context, string) (bool, error) {
		lookups++
		return false, nil
	})(http.HandlerFunc(func(http.ResponseWriter, *http.Request) {
		t.Fatal("disabled feature must not reach handler")
	}))

	for range 2 {
		rec := httptest.NewRecorder()
		handler.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/api/v1/ess/profile", nil))
		if rec.Code != http.StatusForbidden {
			t.Fatalf("expected 403, got %d", rec.Code)
		}
		if rec.Header().Get("Content-Type") != "application/json" {
			t.Fatalf("expected JSON error, got %q", rec.Header().Get("Content-Type"))
		}
	}
	if lookups != 1 {
		t.Fatalf("expected one cached lookup, got %d", lookups)
	}
}

func TestFeatureFlagMiddlewareFailsClosedOnLookupError(t *testing.T) {
	handler := FeatureFlagMiddleware("ff_ess", func(context.Context, string) (bool, error) {
		return true, errors.New("database unavailable")
	})(http.HandlerFunc(func(http.ResponseWriter, *http.Request) {
		t.Fatal("lookup errors must not reach handler")
	}))
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/api/v1/ess/profile", nil))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
	status, body := apperror.ToWireEnvelope(apperror.FeatureDisabled("ff_ess"), "en")
	if status != http.StatusForbidden || body.Error.Code != apperror.CodeFeatureDisabled {
		t.Fatalf("unexpected feature-disabled contract: %d %+v", status, body)
	}
}

func TestFeatureFlagMiddlewareAllowsEnabled(t *testing.T) {
	called := false
	handler := FeatureFlagMiddleware("ff_ess", func(context.Context, string) (bool, error) {
		return true, nil
	})(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		called = true
		w.WriteHeader(http.StatusNoContent)
	}))
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/api/v1/ess/profile", nil))
	if !called || rec.Code != http.StatusNoContent {
		t.Fatalf("expected enabled handler call with 204, called=%t status=%d", called, rec.Code)
	}
}
