package observability

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"log/slog"
	"strings"
	"testing"
)

func parseSingleJSON(t *testing.T, line string) map[string]any {
	t.Helper()
	var res map[string]any
	if err := json.Unmarshal([]byte(strings.TrimSpace(line)), &res); err != nil {
		t.Fatalf("failed to parse JSON log %q: %v", line, err)
	}
	return res
}

func TestJSONHandler_BasicEnvelope(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogger(buf, slog.LevelInfo)

	logger.Info("server starting up")

	output := buf.String()
	if !strings.HasSuffix(output, "\n") {
		t.Errorf("expected newline ending, got %q", output)
	}

	entry := parseSingleJSON(t, output)

	if entry["service"] != "apiserver" {
		t.Errorf("expected service 'apiserver', got %v", entry["service"])
	}
	if entry["level"] != "INFO" {
		t.Errorf("expected level 'INFO', got %v", entry["level"])
	}
	if entry["message"] != "server starting up" {
		t.Errorf("expected message 'server starting up', got %v", entry["message"])
	}
	if entry["timestamp"] == nil || entry["timestamp"] == "" {
		t.Error("expected non-empty timestamp")
	}

	// Should omit empty correlation fields
	if _, ok := entry["request_id"]; ok {
		t.Errorf("unexpected request_id when unset in context")
	}
	if _, ok := entry["organization_id"]; ok {
		t.Errorf("unexpected organization_id when unset in context")
	}
	if _, ok := entry["actor_id"]; ok {
		t.Errorf("unexpected actor_id when unset in context")
	}
	if _, ok := entry["span_id"]; ok {
		t.Errorf("unexpected span_id when unset in context")
	}
	if _, ok := entry["fields"]; ok {
		t.Errorf("unexpected fields object when no custom attrs passed")
	}
}

func TestJSONHandler_ContextCorrelationFields(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogger(buf, slog.LevelInfo)

	ctx := context.Background()
	reqID := "01946850-891a-7b3b-9a4f-9588fb7cfa02"
	organizationID := "01946850-891a-7b3b-9a4f-9588fb7cfa03"
	actorID := "01946850-891a-7b3b-9a4f-9588fb7cfa04"
	spanID := "987654321"

	ctx = WithRequestID(ctx, reqID)
	ctx = WithOrganizationID(ctx, organizationID)
	ctx = WithActorID(ctx, actorID)
	ctx = WithSpanID(ctx, spanID)

	logger.InfoContext(ctx, "processing request")

	entry := parseSingleJSON(t, buf.String())

	if entry["request_id"] != reqID {
		t.Errorf("expected request_id %q, got %v", reqID, entry["request_id"])
	}
	if entry["organization_id"] != organizationID {
		t.Errorf("expected organization_id %q, got %v", organizationID, entry["organization_id"])
	}
	if entry["actor_id"] != actorID {
		t.Errorf("expected actor_id %q, got %v", actorID, entry["actor_id"])
	}
	if entry["span_id"] != spanID {
		t.Errorf("expected span_id %q, got %v", spanID, entry["span_id"])
	}
}

func TestJSONHandler_ErrorLevelAndErrorTypeExtraction(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogger(buf, slog.LevelInfo)

	logger.Error("database connection failed", "error", errors.New("timeout connecting to pool"))

	entry := parseSingleJSON(t, buf.String())

	if entry["level"] != "ERROR" {
		t.Errorf("expected level 'ERROR', got %v", entry["level"])
	}
	if entry["error.type"] != "timeout connecting to pool" {
		t.Errorf("expected error.type 'timeout connecting to pool', got %v", entry["error.type"])
	}

	fields, ok := entry["fields"].(map[string]any)
	if !ok {
		t.Fatalf("expected fields map, got %T", entry["fields"])
	}
	if fields["error"] != "timeout connecting to pool" {
		t.Errorf("expected fields.error preserved, got %v", fields["error"])
	}
}

func TestJSONHandler_PIIMaskingInFields(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogger(buf, slog.LevelDebug)

	logger.Debug("user registration event",
		"user_email", "john.doe@example.com",
		"phone_number", "0987654321",
		"bank_account", "1234567890",
		"password", "secretPassword123",
		"jwt_token", "eyJhbGciOi...",
		"attempt_count", 3,
		"is_active", true,
	)

	entry := parseSingleJSON(t, buf.String())
	fields, ok := entry["fields"].(map[string]any)
	if !ok {
		t.Fatalf("expected fields map, got %T", entry["fields"])
	}

	if fields["user_email"] != "joh***@example.com" {
		t.Errorf("expected masked email, got %v", fields["user_email"])
	}
	if fields["phone_number"] != "098****321" {
		t.Errorf("expected masked phone, got %v", fields["phone_number"])
	}
	if fields["bank_account"] != "******7890" {
		t.Errorf("expected masked bank account, got %v", fields["bank_account"])
	}
	if fields["password"] != "[REDACTED]" {
		t.Errorf("expected redacted password, got %v", fields["password"])
	}
	if fields["jwt_token"] != "[REDACTED]" {
		t.Errorf("expected redacted token, got %v", fields["jwt_token"])
	}
	// Non-string types preserved
	if fields["attempt_count"] != float64(3) { // json numbers are float64 in unmarshal
		t.Errorf("expected count 3, got %v", fields["attempt_count"])
	}
	if fields["is_active"] != true {
		t.Errorf("expected boolean true, got %v", fields["is_active"])
	}
}

func TestJSONHandler_LevelFiltering(t *testing.T) {
	buf := &bytes.Buffer{}
	logger := NewLogger(buf, slog.LevelInfo)

	logger.Debug("this debug log should be filtered out")
	if buf.Len() != 0 {
		t.Errorf("expected no output for DEBUG when min level is INFO, got %s", buf.String())
	}

	logger.Warn("this warning should appear")
	if buf.Len() == 0 {
		t.Error("expected output for WARN log")
	}
	entry := parseSingleJSON(t, buf.String())
	if entry["level"] != "WARN" {
		t.Errorf("expected level 'WARN', got %v", entry["level"])
	}
}

func TestJSONHandler_WithAttrs(t *testing.T) {
	buf := &bytes.Buffer{}
	baseLogger := NewLogger(buf, slog.LevelInfo)
	subLogger := baseLogger.With("component", "auth", "secret_key", "secret123")

	subLogger.Info("token generated", "user_id", "u-1234")

	entry := parseSingleJSON(t, buf.String())
	fields, ok := entry["fields"].(map[string]any)
	if !ok {
		t.Fatalf("expected fields map, got %T", entry["fields"])
	}

	if fields["component"] != "auth" {
		t.Errorf("expected component 'auth', got %v", fields["component"])
	}
	if fields["secret_key"] != "[REDACTED]" {
		t.Errorf("expected secret_key to be [REDACTED], got %v", fields["secret_key"])
	}
	if fields["user_id"] != "u-1234" {
		t.Errorf("expected user_id 'u-1234', got %v", fields["user_id"])
	}
}
