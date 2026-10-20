package app

import (
	"context"
	"net/http"
	"testing"
	"time"
)

func TestAppShutdown_DrainsAndRunsHooks(t *testing.T) {
	server := &http.Server{Addr: "127.0.0.1:0"}
	hookRan := false

	app := New(server, func(ctx context.Context) error {
		hookRan = true
		return nil
	})

	if app.Server() != server {
		t.Fatal("expected server match")
	}

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	if err := app.Shutdown(ctx); err != nil {
		t.Fatalf("unexpected shutdown error: %v", err)
	}

	if !hookRan {
		t.Fatal("expected shutdown hook to execute")
	}
}
