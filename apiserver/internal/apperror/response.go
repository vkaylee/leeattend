package apperror

import (
	"context"
	"encoding/json"
	"errors"
	"log/slog"
	"net/http"
	"strings"
)

type localeContextKey struct{}

// WithLocale stores a validated language code in context.
func WithLocale(ctx context.Context, lang string) context.Context {
	return context.WithValue(ctx, localeContextKey{}, ResolveLanguage(lang))
}

// GetLocale retrieves the language code from context, defaulting to "vi".
func GetLocale(ctx context.Context) string {
	if ctx == nil {
		return "vi"
	}
	if v, ok := ctx.Value(localeContextKey{}).(string); ok && v != "" {
		return v
	}
	return "vi"
}

// ExtractLocale resolves request language per error-contract.md Section 5.1:
// 1. Context storage
// 2. Query param (?lang=)
// 3. Accept-Language header
// 4. Fallback "vi"
func ExtractLocale(r *http.Request) string {
	if r == nil {
		return "vi"
	}

	// 1. Context storage
	if v, ok := r.Context().Value(localeContextKey{}).(string); ok && v != "" {
		return v
	}

	// 2. Query parameter (?lang=)
	if qLang := r.URL.Query().Get("lang"); qLang != "" {
		trimmed := strings.ToLower(strings.TrimSpace(qLang))
		switch trimmed {
		case "vi", "vi-vn", "vietnamese":
			return "vi"
		case "en", "en-us", "english":
			return "en"
		case "zh", "zh-cn", "chinese":
			return "zh"
		}
	}

	// 3. Accept-Language header
	if header := r.Header.Get("Accept-Language"); header != "" {
		for _, tag := range strings.Split(header, ",") {
			part := strings.TrimSpace(strings.Split(tag, ";")[0])
			part = strings.ToLower(part)
			switch {
			case strings.HasPrefix(part, "vi"):
				return "vi"
			case strings.HasPrefix(part, "en"):
				return "en"
			case strings.HasPrefix(part, "zh"):
				return "zh"
			}
		}
	}

	// 4. Fallback: Vietnamese
	return "vi"
}

// WireErrorEnvelope matches the exact wire JSON envelope from error-contract.md Section 1.
type WireErrorEnvelope struct {
	Error WireErrorBody `json:"error"`
}

type WireErrorBody struct {
	Code    ErrorCode `json:"code"`
	Message string    `json:"message"`
	Retry   any       `json:"retry,omitempty"`
}

// ToWireEnvelope converts any error to HTTP status code and wire JSON envelope.
func ToWireEnvelope(err error, lang string) (int, WireErrorEnvelope) {
	var appErr *AppError
	if !errors.As(err, &appErr) {
		appErr = Internal(err)
	}

	status := appErr.Status
	if status == 0 {
		status = appErr.Code.DefaultStatus()
	}

	envelope := WireErrorEnvelope{
		Error: WireErrorBody{
			Code:    appErr.Code,
			Message: appErr.Translate(lang),
			Retry:   appErr.Retry,
		},
	}
	return status, envelope
}

// WriteError serializes canonical AppError to HTTP response per error contract.
func WriteError(w http.ResponseWriter, r *http.Request, err error) {
	var appErr *AppError
	if !errors.As(err, &appErr) {
		appErr = Internal(err)
	}

	// Log internal server errors (500) with root cause; client message stays masked
	if appErr.Status >= http.StatusInternalServerError {
		ctx := context.Background()
		if r != nil {
			ctx = r.Context()
		}
		slog.ErrorContext(ctx, "Internal API Error",
			"error", appErr.InternalErr,
			"code", string(appErr.Code),
			"status", appErr.Status,
		)
	}

	lang := ExtractLocale(r)
	status, envelope := ToWireEnvelope(appErr, lang)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(envelope)
}

// WriteJSON emits a successful or raw JSON response envelope.
func WriteJSON(w http.ResponseWriter, status int, data any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(data)
}

// WritePrimitiveAuthRequired emits plain-text 401 per error-contract.md Section 1.5.
func WritePrimitiveAuthRequired(w http.ResponseWriter) {
	w.Header().Set("Content-Type", "text/plain; charset=utf-8")
	w.WriteHeader(http.StatusUnauthorized)
	_, _ = w.Write([]byte("Authentication required"))
}
