package observability

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
)

func TestGenerateRequestID_IsUUIDv7(t *testing.T) {
	idStr := GenerateRequestID()
	u, err := uuid.Parse(idStr)
	if err != nil {
		t.Fatalf("expected valid UUID, got error: %v", err)
	}

	if u.Version() != 7 {
		t.Errorf("expected UUID version 7, got %d", u.Version())
	}
}

func TestValidateOrGenerateRequestID(t *testing.T) {
	// Valid UUID should be preserved
	validUUID := "01946850-891a-7b3b-9a4f-9588fb7cfa02"
	if actual := ValidateOrGenerateRequestID(validUUID); actual != validUUID {
		t.Errorf("expected %q, got %q", validUUID, actual)
	}

	// Empty string should generate new UUIDv7
	emptyRes := ValidateOrGenerateRequestID("")
	if u, err := uuid.Parse(emptyRes); err != nil || u.Version() != 7 {
		t.Errorf("expected valid UUIDv7 for empty input, got %q (err=%v)", emptyRes, err)
	}

	// Malformed input should generate new UUIDv7, not echo invalid value
	malformed := "not-a-valid-uuid-header"
	malformedRes := ValidateOrGenerateRequestID(malformed)
	if malformedRes == malformed {
		t.Errorf("expected invalid header to be replaced, got echoed value")
	}
	if u, err := uuid.Parse(malformedRes); err != nil || u.Version() != 7 {
		t.Errorf("expected valid UUIDv7 for malformed input, got %q (err=%v)", malformedRes, err)
	}
}

func TestContextHelpers(t *testing.T) {
	ctx := context.Background()

	// Initial empty getters
	if GetRequestID(ctx) != "" || GetTraceContext(ctx) != "" || GetOrganizationID(ctx) != "" || GetActorID(ctx) != "" || GetSpanID(ctx) != "" {
		t.Error("expected empty strings for unset context values")
	}
	var nilCtx context.Context
	if GetRequestID(nilCtx) != "" {
		t.Error("expected empty string for nil context")
	}

	reqID := "01946850-891a-7b3b-9a4f-9588fb7cfa02"
	traceCtx := reqID
	wsID := "01946850-891a-7b3b-9a4f-9588fb7cfa03"
	actorID := "01946850-891a-7b3b-9a4f-9588fb7cfa04"
	spanID := "12345678"

	ctx = WithRequestID(ctx, reqID)
	ctx = WithTraceContext(ctx, traceCtx)
	ctx = WithOrganizationID(ctx, wsID)
	ctx = WithActorID(ctx, actorID)
	ctx = WithSpanID(ctx, spanID)

	if GetRequestID(ctx) != reqID {
		t.Errorf("expected %q, got %q", reqID, GetRequestID(ctx))
	}
	if GetTraceContext(ctx) != traceCtx {
		t.Errorf("expected %q, got %q", traceCtx, GetTraceContext(ctx))
	}
	if GetOrganizationID(ctx) != wsID {
		t.Errorf("expected %q, got %q", wsID, GetOrganizationID(ctx))
	}
	if GetActorID(ctx) != actorID {
		t.Errorf("expected %q, got %q", actorID, GetActorID(ctx))
	}
	if GetSpanID(ctx) != spanID {
		t.Errorf("expected %q, got %q", spanID, GetSpanID(ctx))
	}
}

func TestRequestIDMiddleware_PreservesValid(t *testing.T) {
	validUUID := "01946850-891a-7b3b-9a4f-9588fb7cfa02"
	var contextID string
	var headerInside string

	handler := RequestIDMiddleware(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		contextID = GetRequestID(r.Context())
		headerInside = r.Header.Get(HeaderRequestID)
		w.WriteHeader(http.StatusOK)
	}))

	req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	req.Header.Set(HeaderRequestID, validUUID)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	respHeader := rec.Header().Get(HeaderRequestID)
	if respHeader != validUUID {
		t.Errorf("expected response header %q, got %q", validUUID, respHeader)
	}
	if contextID != validUUID {
		t.Errorf("expected context ID %q, got %q", validUUID, contextID)
	}
	if headerInside != validUUID {
		t.Errorf("expected downstream header %q, got %q", validUUID, headerInside)
	}
}

func TestRequestIDMiddleware_GeneratesWhenMissingOrInvalid(t *testing.T) {
	cases := []struct {
		name         string
		headerSent   string
		shouldChange bool
	}{
		{"missing", "", true},
		{"malformed", "not-a-uuid", true},
		{"sql_injection", "' OR '1'='1", true},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			var recordedCtxID string
			handler := RequestIDMiddleware(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				recordedCtxID = GetRequestID(r.Context())
				w.WriteHeader(http.StatusOK)
			}))

			req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
			if tc.headerSent != "" {
				req.Header.Set(HeaderRequestID, tc.headerSent)
			}
			rec := httptest.NewRecorder()

			handler.ServeHTTP(rec, req)

			respHeader := rec.Header().Get(HeaderRequestID)
			if respHeader == "" || respHeader == tc.headerSent {
				t.Errorf("expected newly generated UUID, got %q", respHeader)
			}
			u, err := uuid.Parse(respHeader)
			if err != nil {
				t.Fatalf("expected parseable UUID, got %q", respHeader)
			}
			if u.Version() != 7 {
				t.Errorf("expected UUIDv7, got version %d", u.Version())
			}
			if recordedCtxID != respHeader {
				t.Errorf("context ID (%q) != response header (%q)", recordedCtxID, respHeader)
			}
		})
	}
}
