package app

import (
	"context"
	"errors"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
)

// ShutdownTimeout bounds HTTP server and worker drain time.
const ShutdownTimeout = 30 * time.Second

// App owns the HTTP server lifecycle and shutdown hooks.
type App struct {
	server *http.Server
	hooks  []func(context.Context) error
}

// New constructs an application lifecycle owner.
func New(server *http.Server, hooks ...func(context.Context) error) *App {
	return &App{server: server, hooks: hooks}
}

// Server returns the configured HTTP server.
func (a *App) Server() *http.Server {
	return a.server
}

// Run starts the server and drains it on SIGINT/SIGTERM.
func (a *App) Run(ctx context.Context) error {
	if a.server == nil {
		return errors.New("http server is required")
	}

	errCh := make(chan error, 1)
	go func() {
		errCh <- a.server.ListenAndServe()
	}()

	signalCtx, stop := signal.NotifyContext(ctx, syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	select {
	case err := <-errCh:
		if errors.Is(err, http.ErrServerClosed) {
			return nil
		}
		return err
	case <-signalCtx.Done():
		return a.Shutdown(context.Background())
	}
}

// Shutdown stops accepting requests, drains active requests, then runs hooks.
func (a *App) Shutdown(parent context.Context) error {
	ctx, cancel := context.WithTimeout(parent, ShutdownTimeout)
	defer cancel()

	var errs []error
	if a.server != nil {
		if err := a.server.Shutdown(ctx); err != nil {
			errs = append(errs, err)
		}
	}
	for _, hook := range a.hooks {
		if hook == nil {
			continue
		}
		if err := hook(ctx); err != nil {
			errs = append(errs, err)
		}
	}
	return errors.Join(errs...)
}

// NotifySignals returns a context cancelled by SIGINT or SIGTERM.
func NotifySignals(parent context.Context) (context.Context, context.CancelFunc) {
	return signal.NotifyContext(parent, os.Interrupt, syscall.SIGTERM)
}
