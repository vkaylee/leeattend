package observability

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"os"
	"sync"
	"time"
)

// LevelTrace represents the TRACE logging level below DEBUG.
const LevelTrace = slog.Level(-8)

// HandlerOptions contains configuration options for the JSON log handler.
type HandlerOptions struct {
	Level slog.Leveler
}

// JSONHandler implements slog.Handler with frozen production envelope and PII masking.
//
// Matches Rust apiserver/src/utils/logging.rs ProductionJsonFormatter:
// - Fixed keys: timestamp (UTC millisecond RFC3339), level (uppercase string), message, service ("apiserver")
// - Correlation keys: request_id, actor_id, span_id (omitted if absent)
// - On ERROR level: extracts error.type if error attribute exists
// - All non-envelope attributes are nested under "fields" with centralized PII masking applied.
type JSONHandler struct {
	w      io.Writer
	mu     *sync.Mutex
	level  slog.Leveler
	attrs  []slog.Attr
	groups []string
}

// NewJSONHandler creates a new JSONHandler writing to w.
func NewJSONHandler(w io.Writer, opts *HandlerOptions) *JSONHandler {
	if w == nil {
		w = os.Stdout
	}
	var lvl slog.Leveler = slog.LevelInfo
	if opts != nil && opts.Level != nil {
		lvl = opts.Level
	}
	return &JSONHandler{
		w:     w,
		mu:    &sync.Mutex{},
		level: lvl,
	}
}

// Enabled reports whether the handler emits log records at the given level.
func (h *JSONHandler) Enabled(_ context.Context, l slog.Level) bool {
	minLvl := slog.LevelInfo
	if h.level != nil {
		minLvl = h.level.Level()
	}
	return l >= minLvl
}

// WithAttrs returns a new JSONHandler with the given attributes appended.
func (h *JSONHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	newAttrs := make([]slog.Attr, len(h.attrs)+len(attrs))
	copy(newAttrs, h.attrs)
	copy(newAttrs[len(h.attrs):], attrs)
	return &JSONHandler{
		w:      h.w,
		mu:     h.mu,
		level:  h.level,
		attrs:  newAttrs,
		groups: h.groups,
	}
}

// WithGroup returns a new JSONHandler with the given group name appended.
func (h *JSONHandler) WithGroup(name string) slog.Handler {
	if name == "" {
		return h
	}
	newGroups := make([]string, len(h.groups)+1)
	copy(newGroups, h.groups)
	newGroups[len(h.groups)] = name
	return &JSONHandler{
		w:      h.w,
		mu:     h.mu,
		level:  h.level,
		attrs:  h.attrs,
		groups: newGroups,
	}
}

type logEnvelope struct {
	Timestamp string         `json:"timestamp"`
	Level     string         `json:"level"`
	Message   string         `json:"message"`
	Service   string         `json:"service"`
	RequestID string         `json:"request_id,omitempty"`
	ActorID   string         `json:"actor_id,omitempty"`
	SpanID    string         `json:"span_id,omitempty"`
	ErrorType any            `json:"error.type,omitempty"`
	Fields    map[string]any `json:"fields,omitempty"`
}

// Handle formats and outputs a log record.
func (h *JSONHandler) Handle(ctx context.Context, r slog.Record) error {
	// Format level string
	var levelStr string
	switch {
	case r.Level >= slog.LevelError:
		levelStr = "ERROR"
	case r.Level >= slog.LevelWarn:
		levelStr = "WARN"
	case r.Level >= slog.LevelInfo:
		levelStr = "INFO"
	case r.Level >= slog.LevelDebug:
		levelStr = "DEBUG"
	default:
		levelStr = "TRACE"
	}

	// Format timestamp as UTC ISO-8601 with milliseconds
	ts := r.Time.UTC()
	if ts.IsZero() {
		ts = time.Now().UTC()
	}
	timestampStr := ts.Format("2006-01-02T15:04:05.000Z")

	envelope := logEnvelope{
		Timestamp: timestampStr,
		Level:     levelStr,
		Message:   r.Message,
		Service:   "apiserver",
		RequestID: GetRequestID(ctx),
		ActorID:   GetActorID(ctx),
		SpanID:    GetSpanID(ctx),
	}

	fields := make(map[string]any)

	addAttr := func(a slog.Attr) {
		if a.Equal(slog.Attr{}) {
			return
		}
		key := a.Key
		val := a.Value.Resolve()

		switch key {
		case "request_id":
			envelope.RequestID = val.String()
		case "actor_id":
			envelope.ActorID = val.String()
		case "span_id":
			envelope.SpanID = val.String()
		case "service":
			envelope.Service = val.String()
		case "message":
			envelope.Message = val.String()
		default:
			// Process non-envelope field with PII masking
			switch val.Kind() {
			case slog.KindString:
				fields[key] = MaskPIIValue(key, val.String())
			case slog.KindAny:
				if err, ok := val.Any().(error); ok && err != nil {
					fields[key] = MaskPIIValue(key, err.Error())
				} else if str, ok := val.Any().(fmt.Stringer); ok && str != nil {
					fields[key] = MaskPIIValue(key, str.String())
				} else {
					fields[key] = val.Any()
				}
			default:
				fields[key] = val.Any()
			}
		}
	}

	// 1. Static attributes from handler
	for _, a := range h.attrs {
		addAttr(a)
	}

	// 2. Record attributes
	r.Attrs(func(a slog.Attr) bool {
		addAttr(a)
		return true
	})

	// On ERROR events, extract error.type if present
	if levelStr == "ERROR" {
		if errVal, ok := fields["error"]; ok {
			envelope.ErrorType = errVal
		} else if errVal, ok := fields["error.type"]; ok {
			envelope.ErrorType = errVal
		}
	}

	if len(fields) > 0 {
		envelope.Fields = fields
	}

	data, err := json.Marshal(envelope)
	if err != nil {
		return err
	}

	h.mu.Lock()
	defer h.mu.Unlock()
	_, err = h.w.Write(append(data, '\n'))
	return err
}

// NewLogger creates a new slog.Logger configured with the structured JSON handler.
func NewLogger(w io.Writer, minLevel slog.Level) *slog.Logger {
	handler := NewJSONHandler(w, &HandlerOptions{Level: minLevel})
	return slog.New(handler)
}

// InitProductionLogger initializes a production-grade logger (INFO level) and sets as default.
func InitProductionLogger(w io.Writer) *slog.Logger {
	logger := NewLogger(w, slog.LevelInfo)
	slog.SetDefault(logger)
	return logger
}

// InitDevelopmentLogger initializes a development-grade logger (DEBUG level) and sets as default.
func InitDevelopmentLogger(w io.Writer) *slog.Logger {
	logger := NewLogger(w, slog.LevelDebug)
	slog.SetDefault(logger)
	return logger
}
