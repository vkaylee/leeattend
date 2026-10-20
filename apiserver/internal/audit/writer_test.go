package audit

import (
	"context"
	"sync"
	"testing"
	"time"
)

type memorySink struct {
	mu     sync.Mutex
	events []Event
}

func (s *memorySink) Write(_ context.Context, event Event) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.events = append(s.events, event)
	return nil
}

func TestWriterRedactsRestrictedDetailsAndDrains(t *testing.T) {
	sink := &memorySink{}
	writer, err := NewWriter(sink, 2)
	if err != nil {
		t.Fatal(err)
	}
	if err := writer.Emit(context.Background(), Event{Name: "employee.read", Outcome: "success", Details: map[string]string{"email": "person@example.com", "count": "1"}}); err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	if err := writer.Shutdown(ctx); err != nil {
		t.Fatal(err)
	}
	sink.mu.Lock()
	defer sink.mu.Unlock()
	if len(sink.events) != 1 || sink.events[0].Details["email"] != "[REDACTED]" || sink.events[0].Details["count"] != "1" {
		t.Fatalf("unexpected events: %+v", sink.events)
	}
}
