package observability

import (
	"context"
	"net/http"

	"github.com/google/uuid"
)

// HeaderRequestID is the standard HTTP header used for distributed request correlation.
const HeaderRequestID = "x-request-id"

type contextKey string

const (
	requestIDKey    contextKey = "request_id"
	traceContextKey contextKey = "trace_context"
	actorIDKey      contextKey = "actor_id"
	spanIDKey       contextKey = "span_id"
)

// GenerateRequestID generates a new UUIDv7 string.
// Falls back to UUIDv4 if time-based v7 generation encounters clock error.
func GenerateRequestID() string {
	id, err := uuid.NewV7()
	if err != nil {
		return uuid.New().String()
	}
	return id.String()
}

// ValidateOrGenerateRequestID returns the incoming string if it is a valid UUID,
// or generates a new UUIDv7 if it is missing, malformed, or non-UUID.
func ValidateOrGenerateRequestID(raw string) string {
	if raw != "" {
		if u, err := uuid.Parse(raw); err == nil {
			return u.String()
		}
	}
	return GenerateRequestID()
}

// WithRequestID stores the request ID in context.
func WithRequestID(ctx context.Context, id string) context.Context {
	return context.WithValue(ctx, requestIDKey, id)
}

// GetRequestID retrieves the request ID from context, or empty string if unset.
func GetRequestID(ctx context.Context) string {
	if ctx == nil {
		return ""
	}
	if v, ok := ctx.Value(requestIDKey).(string); ok {
		return v
	}
	return ""
}

// WithTraceContext stores the trace context correlation string in context.
func WithTraceContext(ctx context.Context, tc string) context.Context {
	return context.WithValue(ctx, traceContextKey, tc)
}

// GetTraceContext retrieves the trace context correlation string from context.
func GetTraceContext(ctx context.Context) string {
	if ctx == nil {
		return ""
	}
	if v, ok := ctx.Value(traceContextKey).(string); ok {
		return v
	}
	return ""
}

// WithActorID stores the authenticated actor/user ID in context.
func WithActorID(ctx context.Context, id string) context.Context {
	return context.WithValue(ctx, actorIDKey, id)
}

// GetActorID retrieves the actor ID from context, or empty string if unset.
func GetActorID(ctx context.Context) string {
	if ctx == nil {
		return ""
	}
	if v, ok := ctx.Value(actorIDKey).(string); ok {
		return v
	}
	return ""
}

// WithSpanID stores the tracing span ID in context.
func WithSpanID(ctx context.Context, id string) context.Context {
	return context.WithValue(ctx, spanIDKey, id)
}

// GetSpanID retrieves the span ID from context, or empty string if unset.
func GetSpanID(ctx context.Context) string {
	if ctx == nil {
		return ""
	}
	if v, ok := ctx.Value(spanIDKey).(string); ok {
		return v
	}
	return ""
}

// RequestIDMiddleware is an outermost HTTP middleware that ensures every request and
// response carries an x-request-id header and propagates the ID in context.
//
// Matches Rust apiserver/src/mw/request_id.rs:
// 1. Extracts x-request-id from incoming header; if valid UUID retains it, otherwise generates UUIDv7.
// 2. Downstream request header is rewritten to canonical UUID.
// 3. Request context is populated with request_id and trace_context.
// 4. Response header is populated with x-request-id.
func RequestIDMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		incoming := r.Header.Get(HeaderRequestID)
		reqID := ValidateOrGenerateRequestID(incoming)

		// Set canonical request ID on downstream request
		r.Header.Set(HeaderRequestID, reqID)

		// Set on response header
		w.Header().Set(HeaderRequestID, reqID)

		// Propagate in context
		ctx := WithRequestID(r.Context(), reqID)
		ctx = WithTraceContext(ctx, reqID)

		next.ServeHTTP(w, r.WithContext(ctx))
	})
}
