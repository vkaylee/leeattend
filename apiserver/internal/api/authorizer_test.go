package api

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"reflect"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type helperAuthorizer struct {
	testAuthorizer
	member bool
	err    error
	calls  []string
}

func (a *helperAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return a.member, a.err
}

func (a *helperAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	a.calls = append(a.calls, permission)
	return permission == "allow"
}

func TestSharedAuthorizerHelpers(t *testing.T) {
	r := httptest.NewRequest("GET", "/", nil)
	withUser := r.WithContext(httpinfra.WithUserIDForTest(r.Context(), uuid.New()))
	a := &helperAuthorizer{member: true, err: errors.New("unavailable")}
	if isMember(r, a) || isMember(withUser, nil) || isMember(withUser, a) {
		t.Fatal("membership must fail closed without identity, authorizer, or on error")
	}
	a.err = nil
	a.member = false
	if isMember(withUser, a) {
		t.Fatal("non-member must be denied")
	}
	a.member = true
	if !isMember(withUser, a) {
		t.Fatal("member must be allowed")
	}
	if hasPermission(r, a, "allow") || hasPermission(withUser, nil, "allow") || hasPermission(withUser, a, "deny") {
		t.Fatal("permission must fail closed")
	}
	if hasAnyPermission(r, a, "allow") || hasAnyPermission(withUser, nil, "allow") || hasAnyPermission(withUser, a) || hasAnyPermission(withUser, a, "deny") {
		t.Fatal("any-permission must fail closed")
	}
	a.calls = nil
	if !hasAnyPermission(withUser, a, "deny", "allow", "unused") || !reflect.DeepEqual(a.calls, []string{"deny", "allow"}) {
		t.Fatalf("permission checks must stop on first grant: %v", a.calls)
	}
}

func TestSharedResponseAndErrorHelpers(t *testing.T) {
	assertErrorResponse := func(t *testing.T, rec *httptest.ResponseRecorder, wantStatus int, wantCode apperror.ErrorCode) {
		t.Helper()
		if rec.Code != wantStatus {
			t.Fatalf("got status %d, want %d", rec.Code, wantStatus)
		}
		if !strings.Contains(rec.Body.String(), string(wantCode)) {
			t.Fatalf("expected error code %s in body %q", wantCode, rec.Body.String())
		}
	}

	t.Run("writeForbidden writes 403 forbidden", func(t *testing.T) {
		rec := httptest.NewRecorder()
		writeForbidden(rec, httptest.NewRequest(http.MethodGet, "/", nil))
		assertErrorResponse(t, rec, http.StatusForbidden, apperror.CodeForbidden)
	})

	t.Run("writeValidation writes 400 validation", func(t *testing.T) {
		rec := httptest.NewRecorder()
		writeValidation(rec, httptest.NewRequest(http.MethodPost, "/", nil))
		assertErrorResponse(t, rec, http.StatusBadRequest, apperror.CodeValidationError)
	})

	t.Run("writeMappedError unwraps AppError before mapper", func(t *testing.T) {
		rec := httptest.NewRecorder()
		called := false
		wrapped := fmt.Errorf("wrap: %w", apperror.Conflict(apperror.KeyConflict))
		writeMappedError(rec, httptest.NewRequest(http.MethodPost, "/", nil), wrapped, func(error) *apperror.AppError {
			called = true
			return apperror.NotFound(apperror.KeyNotFound)
		})
		if called {
			t.Fatal("mapper should not run when error chain already contains AppError")
		}
		assertErrorResponse(t, rec, http.StatusConflict, apperror.CodeConflict)
	})

	t.Run("writeMappedError uses mapper result when non-nil", func(t *testing.T) {
		rec := httptest.NewRecorder()
		sentinel := errors.New("missing")
		writeMappedError(rec, httptest.NewRequest(http.MethodGet, "/", nil), sentinel, func(err error) *apperror.AppError {
			if errors.Is(err, sentinel) {
				return apperror.NotFound(apperror.KeyNotFound)
			}
			return nil
		})
		assertErrorResponse(t, rec, http.StatusNotFound, apperror.CodeNotFound)
	})

	t.Run("writeMappedError falls back to internal error", func(t *testing.T) {
		for _, mapper := range []func(error) *apperror.AppError{nil, func(error) *apperror.AppError { return nil }} {
			rec := httptest.NewRecorder()
			writeMappedError(rec, httptest.NewRequest(http.MethodGet, "/", nil), errors.New("boom"), mapper)
			assertErrorResponse(t, rec, http.StatusInternalServerError, apperror.CodeInternalError)
		}
	})
}
