package queue

import (
	"context"
	"errors"
	"sync"
	"time"
)

type Job struct {
	ID       string
	Kind     string
	Payload  []byte
	Attempts int
}
type Handler func(context.Context, Job) error

type Worker struct {
	jobs        chan Job
	handler     Handler
	maxAttempts int
	backoff     time.Duration
	done        chan struct{}
	closed      chan struct{}
	startOnce   sync.Once
	closeOnce   sync.Once
	seenMu      sync.Mutex
	seen        map[string]struct{}
}

func NewWorker(capacity, maxAttempts int, backoff time.Duration, handler Handler) (*Worker, error) {
	if capacity < 1 || maxAttempts < 1 || handler == nil {
		return nil, errors.New("invalid worker configuration")
	}
	if backoff < 0 {
		return nil, errors.New("backoff cannot be negative")
	}
	w := &Worker{jobs: make(chan Job, capacity), handler: handler, maxAttempts: maxAttempts, backoff: backoff, done: make(chan struct{}), closed: make(chan struct{}), seen: make(map[string]struct{}, capacity)}
	return w, nil
}

func (w *Worker) Start(ctx context.Context) { w.startOnce.Do(func() { go w.run(ctx) }) }

func (w *Worker) Enqueue(ctx context.Context, job Job) error {
	if job.ID == "" || job.Kind == "" {
		return errors.New("job ID and kind are required")
	}
	w.seenMu.Lock()
	if _, exists := w.seen[job.ID]; exists {
		w.seenMu.Unlock()
		return nil
	}
	w.seen[job.ID] = struct{}{}
	w.seenMu.Unlock()
	select {
	case <-w.done:
		return errors.New("worker is closed")
	case <-ctx.Done():
		w.seenMu.Lock()
		delete(w.seen, job.ID)
		w.seenMu.Unlock()
		return ctx.Err()
	case w.jobs <- job:
		return nil
	}
}

func (w *Worker) Shutdown(ctx context.Context) error {
	w.closeOnce.Do(func() { close(w.done) })
	select {
	case <-w.closed:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (w *Worker) run(ctx context.Context) {
	defer close(w.closed)
	for {
		select {
		case <-ctx.Done():
			return
		case <-w.done:
			for {
				select {
				case job := <-w.jobs:
					w.process(ctx, job)
				default:
					return
				}
			}
		case job := <-w.jobs:
			w.process(ctx, job)
		}
	}
}
func (w *Worker) process(ctx context.Context, job Job) {
	for attempt := job.Attempts; attempt < w.maxAttempts; attempt++ {
		job.Attempts = attempt + 1
		if err := w.handler(ctx, job); err == nil {
			return
		}
		if attempt+1 < w.maxAttempts && w.backoff > 0 {
			timer := time.NewTimer(w.backoff * time.Duration(attempt+1))
			select {
			case <-timer.C:
			case <-ctx.Done():
				timer.Stop()
				return
			}
		}
	}
}
