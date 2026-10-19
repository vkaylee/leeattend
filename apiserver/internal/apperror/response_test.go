package apperror

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestWriteError(t *testing.T) {
	t.Run("writes json envelope with proper status and content type", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/test", nil)
		rec := httptest.NewRecorder()

		WriteError(rec, req, NotFound(KeyResourceNotFound))

		if rec.Code != http.StatusNotFound {
			t.Errorf("got status %d, want 404", rec.Code)
		}
		if ct := rec.Header().Get("Content-Type"); ct != "application/json" {
			t.Errorf("got content type %s, want application/json", ct)
		}

		var body WireErrorEnvelope
		if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
			t.Fatalf("failed to parse json: %v", err)
		}
		if body.Error.Code != CodeNotFound {
			t.Errorf("got code %s, want NOT_FOUND", body.Error.Code)
		}
		if body.Error.Message != "Không tìm thấy tài nguyên" {
			t.Errorf("got message %s", body.Error.Message)
		}
	})

	t.Run("converts non-AppError to 500 masked internal error", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/test?lang=en", nil)
		rec := httptest.NewRecorder()

		WriteError(rec, req, errors.New("raw unexpected failure"))

		if rec.Code != http.StatusInternalServerError {
			t.Errorf("got status %d, want 500", rec.Code)
		}
		var body WireErrorEnvelope
		_ = json.Unmarshal(rec.Body.Bytes(), &body)
		if body.Error.Code != CodeInternalError {
			t.Errorf("got code %s, want INTERNAL_ERROR", body.Error.Code)
		}
		if body.Error.Message != "An internal server error occurred." {
			t.Errorf("got message %s", body.Error.Message)
		}
	})

	t.Run("resolves locale from Accept-Language header", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/test", nil)
		req.Header.Set("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
		rec := httptest.NewRecorder()

		WriteError(rec, req, Forbidden(KeyForbidden))

		var body WireErrorEnvelope
		_ = json.Unmarshal(rec.Body.Bytes(), &body)
		if body.Error.Message != "权限拒绝" {
			t.Errorf("got message %s, want Chinese translation", body.Error.Message)
		}
	})

	t.Run("resolves locale from context", func(t *testing.T) {
		ctx := WithLocale(context.Background(), "en")
		req := httptest.NewRequest("GET", "/test", nil).WithContext(ctx)
		rec := httptest.NewRecorder()

		WriteError(rec, req, Forbidden(KeyForbidden))

		var body WireErrorEnvelope
		_ = json.Unmarshal(rec.Body.Bytes(), &body)
		if body.Error.Message != "Permission denied" {
			t.Errorf("got message %s, want English translation", body.Error.Message)
		}
	})
}

const KeyResourceNotFound = "error.not_found"

func TestWritePrimitiveAuthRequired(t *testing.T) {
	rec := httptest.NewRecorder()
	WritePrimitiveAuthRequired(rec)

	if rec.Code != http.StatusUnauthorized {
		t.Errorf("got status %d, want 401", rec.Code)
	}
	if ct := rec.Header().Get("Content-Type"); ct != "text/plain; charset=utf-8" {
		t.Errorf("got content type %s, want text/plain; charset=utf-8", ct)
	}
	if body := rec.Body.String(); body != "Authentication required" {
		t.Errorf("got body %q, want 'Authentication required'", body)
	}
}

func TestWriteJSON(t *testing.T) {
	rec := httptest.NewRecorder()
	payload := map[string]string{"status": "ok"}
	WriteJSON(rec, http.StatusOK, payload)

	if rec.Code != http.StatusOK {
		t.Errorf("got status %d, want 200", rec.Code)
	}
	if ct := rec.Header().Get("Content-Type"); ct != "application/json" {
		t.Errorf("got content type %s, want application/json", ct)
	}
	var res map[string]string
	_ = json.Unmarshal(rec.Body.Bytes(), &res)
	if res["status"] != "ok" {
		t.Errorf("got status %s, want ok", res["status"])
	}
}
