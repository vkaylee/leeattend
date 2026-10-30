package scheduler

import (
	"context"
	"errors"
	"log/slog"
	"sync"
	"time"

	"github.com/google/uuid"
)

// WorkspaceSource lists workspaces eligible for scheduled pulls.
type WorkspaceSource interface {
	ListPullWorkspaces(context.Context) ([]uuid.UUID, error)
}

// Puller executes one workspace pull. Implementations own provider access and persistence.
type Puller interface {
	Pull(context.Context, uuid.UUID, string) error
}

// ShadowPuller optionally executes pull without writing attendance records.
type ShadowPuller interface {
	PullShadow(context.Context, uuid.UUID, string) error
}

// Config bounds scheduler cadence, pull timeout, and concurrent workspace pulls.
type Config struct {
	Interval      time.Duration
	Timeout       time.Duration
	MaxConcurrent int
	Shadow        bool
}

// Runner schedules pulls until context cancellation.
type Runner struct {
	Source WorkspaceSource
	Puller Puller
	Config Config
	Logger *slog.Logger
	once   sync.Once
	mu     sync.Mutex
	cancel context.CancelFunc
	done   chan struct{}
}

func (r *Runner) Start(ctx context.Context) {
	r.once.Do(func() {
		ownedCtx, cancel := context.WithCancel(ctx)
		done := make(chan struct{})
		r.mu.Lock()
		r.cancel = cancel
		r.done = done
		r.mu.Unlock()
		go func() {
			defer close(done)
			r.run(ownedCtx)
		}()
	})
}

// Stop cancels scheduled work and waits for its current cycle to finish.
func (r *Runner) Stop(ctx context.Context) error {
	r.mu.Lock()
	cancel, done := r.cancel, r.done
	r.mu.Unlock()
	if done == nil {
		return nil
	}
	cancel()
	select {
	case <-done:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (r *Runner) run(ctx context.Context) {
	interval := r.Config.Interval
	if interval <= 0 {
		interval = time.Minute
	}
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		if err := r.RunOnce(ctx); err != nil && !errors.Is(err, context.Canceled) {
			r.log("scheduled attendance pull failed")
		}
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
		}
	}
}

// RunOnce performs one bounded pull cycle.
func (r *Runner) RunOnce(parent context.Context) error {
	if r.Source == nil || r.Puller == nil {
		return errors.New("attendance pull scheduler: dependencies are required")
	}
	workspaces, err := r.Source.ListPullWorkspaces(parent)
	if err != nil {
		return err
	}
	limit := r.Config.MaxConcurrent
	if limit <= 0 {
		limit = 1
	}
	ctx := parent
	var cancel context.CancelFunc
	if r.Config.Timeout > 0 {
		ctx, cancel = context.WithTimeout(parent, r.Config.Timeout)
		defer cancel()
	}
	sem := make(chan struct{}, limit)
	var wg sync.WaitGroup
	var firstErr error
	var mu sync.Mutex
schedule:
	for _, workspaceID := range workspaces {
		if workspaceID == uuid.Nil {
			continue
		}
		select {
		case sem <- struct{}{}:
		case <-ctx.Done():
			break schedule
		}
		if ctx.Err() != nil {
			break
		}
		workspaceID := workspaceID
		wg.Add(1)
		go func() {
			defer wg.Done()
			defer func() { <-sem }()
			requestID := uuid.NewString()
			var pullErr error
			if r.Config.Shadow {
				if shadow, ok := r.Puller.(ShadowPuller); ok {
					pullErr = shadow.PullShadow(ctx, workspaceID, requestID)
				} else {
					pullErr = errors.New("attendance pull scheduler: shadow mode unsupported")
				}
			} else {
				pullErr = r.Puller.Pull(ctx, workspaceID, requestID)
			}
			if pullErr != nil {
				mu.Lock()
				if firstErr == nil {
					firstErr = pullErr
				}
				mu.Unlock()
				r.log("scheduled attendance workspace pull failed")
				return
			}
			r.log("scheduled attendance workspace pull complete")
		}()
	}
	wg.Wait()
	return firstErr
}

func (r *Runner) log(message string, args ...any) {
	if r.Logger != nil {
		r.Logger.Info(message, args...)
	}
}
