package pool_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/vkaylee/leeattend/apiserver/internal/db/pool"
)

func TestValidateDBURL(t *testing.T) {
	validCases := []string{
		"postgres://user:pass@localhost:5432/leeattend",
		"postgresql://user:pass@localhost:6543/leeattend?sslmode=disable",
		"  postgres://user:pass@localhost:5432/leeattend  ",
	}
	for _, u := range validCases {
		if err := pool.ValidateDBURL(u); err != nil {
			t.Errorf("expected valid for %s, got: %v", u, err)
		}
	}

	invalidCases := []string{
		"http://localhost:5432",
		"mysql://user:pass@localhost:3306/db",
		"",
		"sqlite://path/to/db",
	}
	for _, u := range invalidCases {
		if err := pool.ValidateDBURL(u); !errors.Is(err, pool.ErrInvalidURL) {
			t.Errorf("expected ErrInvalidURL for %s, got: %v", u, err)
		}
	}
}

func TestBuildPgxConfig(t *testing.T) {
	rawURL := "postgres://user:pass@localhost:5432/testdb"
	cfg, err := pool.BuildPgxConfig(rawURL, 15, 7*time.Second, 3*time.Second)
	if err != nil {
		t.Fatalf("BuildPgxConfig failed: %v", err)
	}

	if cfg.MaxConns != 15 {
		t.Errorf("expected MaxConns=15, got %d", cfg.MaxConns)
	}
	if cfg.ConnConfig.ConnectTimeout != 3*time.Second {
		t.Errorf("expected ConnectTimeout=3s, got %v", cfg.ConnConfig.ConnectTimeout)
	}
}

func TestManagerLifecycleAndTimeout(t *testing.T) {
	openCount := 0
	fakeOpener := func(ctx context.Context, pcfg *pgxpool.Config) (*pgxpool.Pool, error) {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
			openCount++
			return nil, nil
		}
	}

	workspaceID := uuid.New()
	cfg := pool.DefaultConfig()
	cfg.WorkspaceID = workspaceID
	cfg.PrimaryURL = "postgres://user:pass@localhost:5432/standalone"
	cfg.PoolerTxURL = cfg.PrimaryURL
	cfg.PoolerSessURL = cfg.PrimaryURL

	mgr := pool.NewManager(cfg, fakeOpener)
	ctx := context.Background()
	if err := mgr.InitSystemPools(ctx); err != nil {
		t.Fatalf("InitSystemPools failed: %v", err)
	}
	if openCount != 2 {
		t.Fatalf("expected 2 pools opened, got %d", openCount)
	}
	txPool, err := mgr.GetTenantTxPool(ctx, workspaceID)
	if err != nil {
		t.Fatalf("GetTenantTxPool failed: %v", err)
	}
	sessPool, err := mgr.GetTenantSessPool(ctx, workspaceID)
	if err != nil {
		t.Fatalf("GetTenantSessPool failed: %v", err)
	}
	systemPool, _ := mgr.SystemPool()
	workerPool, _ := mgr.WorkerPool()
	if txPool != systemPool || sessPool != workerPool {
		t.Fatal("tenant access did not reuse standalone pools")
	}
	if _, err := mgr.GetTenantTxPool(ctx, uuid.New()); err == nil {
		t.Fatal("expected foreign workspace rejection")
	}
	mgr.Close()
	if _, err := mgr.GetTenantTxPool(ctx, workspaceID); !errors.Is(err, pool.ErrPoolClosed) {
		t.Fatalf("expected ErrPoolClosed, got %v", err)
	}
}

func TestManagerInitTimeout(t *testing.T) {
	timeoutOpener := func(ctx context.Context, _ *pgxpool.Config) (*pgxpool.Pool, error) {
		<-ctx.Done()
		return nil, ctx.Err()
	}
	cfg := pool.DefaultConfig()
	cfg.PrimaryURL = "postgres://user:pass@localhost:5432/standalone"
	if err := pool.NewManager(cfg, timeoutOpener).InitSystemPools(context.Background()); err == nil {
		t.Fatal("expected timeout error")
	}
}
