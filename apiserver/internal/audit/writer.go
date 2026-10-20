package audit

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"
)

var ErrClosed = errors.New("audit writer is closed")

// Event contains only fields safe for structured audit storage.
type Event struct {
	Name      string
	ActorID   string
	Workspace string
	Resource  string
	Outcome   string
	Details   map[string]string
	At        time.Time
}

type Sink interface {
	Write(context.Context, Event) error
}

type Writer struct {
	sink   Sink
	queue  chan Event
	done   chan struct{}
	closed chan struct{}
}

func NewWriter(sink Sink, capacity int) (*Writer, error) {
	if sink == nil {
		return nil, errors.New("audit sink is required")
	}
	if capacity < 1 {
		return nil, errors.New("audit capacity must be positive")
	}
	w := &Writer{sink: sink, queue: make(chan Event, capacity), done: make(chan struct{}), closed: make(chan struct{})}
	go w.run()
	return w, nil
}

func (w *Writer) Emit(ctx context.Context, event Event) error {
	if w == nil {
		return errors.New("audit writer is required")
	}
	if event.Name == "" || event.Outcome == "" {
		return errors.New("audit name and outcome are required")
	}
	event = sanitize(event)
	select {
	case <-w.done:
		return ErrClosed
	case <-ctx.Done():
		return ctx.Err()
	case w.queue <- event:
		return nil
	default:
		return errors.New("audit queue is full")
	}
}

func (w *Writer) Shutdown(ctx context.Context) error {
	if w == nil {
		return nil
	}
	select {
	case <-w.done:
	default:
		close(w.done)
	}
	select {
	case <-w.closed:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (w *Writer) run() {
	defer close(w.closed)
	for {
		select {
		case event := <-w.queue:
			_ = w.sink.Write(context.Background(), event)
		case <-w.done:
			for {
				select {
				case event := <-w.queue:
					_ = w.sink.Write(context.Background(), event)
				default:
					return
				}
			}
		}
	}
}

func sanitize(event Event) Event {
	if event.At.IsZero() {
		event.At = time.Unix(0, 0).UTC()
	}
	event.Details = redact(event.Details)
	return event
}

func redact(details map[string]string) map[string]string {
	if len(details) == 0 {
		return nil
	}
	out := make(map[string]string, len(details))
	for key, value := range details {
		lower := strings.ToLower(key)
		if strings.Contains(lower, "password") || strings.Contains(lower, "secret") || strings.Contains(lower, "token") || strings.Contains(lower, "email") || strings.Contains(lower, "phone") {
			out[key] = "[REDACTED]"
			continue
		}
		out[key] = fmt.Sprint(value)
	}
	return out
}
