package queue

import (
	"context"
	"errors"
	"sync/atomic"
	"testing"
	"time"
)

func TestWorker_ExecutesAndRetries(t *testing.T) {
	var attempts atomic.Int32
	worker, err := NewWorker(10, 3, 5*time.Millisecond, func(ctx context.Context, job Job) error {
		if attempts.Add(1) < 2 {
			return errors.New("transient error")
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	worker.Start(ctx)

	if err := worker.Enqueue(ctx, Job{ID: "j1", Kind: "test"}); err != nil {
		t.Fatal(err)
	}

	// Wait for execution
	time.Sleep(50 * time.Millisecond)
	if err := worker.Shutdown(ctx); err != nil {
		t.Fatal(err)
	}
	if attempts.Load() != 2 {
		t.Fatalf("expected 2 attempts, got %d", attempts.Load())
	}
}

func TestWorker_DuplicateJobIDRunsOnce(t *testing.T) {
	var attempts atomic.Int32
	worker, err := NewWorker(4, 1, 0, func(context.Context, Job) error { attempts.Add(1); return nil })
	if err != nil {
		t.Fatal(err)
	}
	ctx := context.Background()
	worker.Start(ctx)
	job := Job{ID: "same", Kind: "test"}
	if err := worker.Enqueue(ctx, job); err != nil {
		t.Fatal(err)
	}
	if err := worker.Enqueue(ctx, job); err != nil {
		t.Fatal(err)
	}
	time.Sleep(20 * time.Millisecond)
	if err := worker.Shutdown(ctx); err != nil {
		t.Fatal(err)
	}
	if got := attempts.Load(); got != 1 {
		t.Fatalf("expected one handler call, got %d", got)
	}
}
