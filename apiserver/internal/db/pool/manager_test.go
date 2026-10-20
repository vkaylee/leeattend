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

func TestBuildTenantURL(t *testing.T) {
	wsID := uuid.MustParse("a0000000-0000-0000-0000-000000000001")
	base := "postgres://appuser:secret@pgbouncer:6543/template_db?sslmode=disable"

	got, err := pool.BuildTenantURL(base, wsID, "", "", "")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	expectedDB := "tenant_a0000000000000000000000000000001"
	if pool.TenantDBName(wsID) != expectedDB {
		t.Fatalf("TenantDBName mismatch: expected %s, got %s", expectedDB, pool.TenantDBName(wsID))
	}

	expectedURL := "postgres://appuser:secret@pgbouncer:6543/" + expectedDB + "?sslmode=disable"
	if got != expectedURL {
		t.Fatalf("expected %s, got %s", expectedURL, got)
	}

	// Override credentials and DB name
	overridden, err := pool.BuildTenantURL(base, wsID, "custom_user", "custom_pass", "custom_db")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	expectedOverridden := "postgres://custom_user:custom_pass@pgbouncer:6543/custom_db?sslmode=disable"
	if overridden != expectedOverridden {
		t.Fatalf("expected %s, got %s", expectedOverridden, overridden)
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
			// Return nil pool pointer since no real network connection is dialed
			return nil, nil
		}
	}

	cfg := pool.DefaultConfig()
	cfg.PrimaryURL = "postgres://user:pass@localhost:5432/system_db"
	cfg.PoolerTxURL = "postgres://user:pass@localhost:6543/system_db"
	cfg.PoolerSessURL = "postgres://user:pass@localhost:5452/system_db"
	cfg.TenantPoolerTxURL = "postgres://user:pass@localhost:6543/tenant_template"
	cfg.TenantPoolerSessURL = "postgres://user:pass@localhost:5452/tenant_template"

	mgr := pool.NewManager(cfg, fakeOpener)

	ctx := context.Background()
	if err := mgr.InitSystemPools(ctx); err != nil {
		t.Fatalf("InitSystemPools failed: %v", err)
	}
	if openCount != 2 {
		t.Fatalf("expected 2 pools opened (system + worker), got %d", openCount)
	}

	wsID := uuid.New()
	_, err := mgr.GetTenantTxPool(ctx, wsID)
	if err != nil {
		t.Fatalf("GetTenantTxPool failed: %v", err)
	}
	if openCount != 3 {
		t.Fatalf("expected 3 pools opened, got %d", openCount)
	}

	// Repeated call should reuse cache, not re-open
	_, err = mgr.GetTenantTxPool(ctx, wsID)
	if err != nil {
		t.Fatalf("repeat GetTenantTxPool failed: %v", err)
	}
	if openCount != 3 {
		t.Fatalf("expected cache hit with openCount 3, got %d", openCount)
	}

	// Test context deadline cancellation in opener
	timeoutOpener := func(ctx context.Context, pcfg *pgxpool.Config) (*pgxpool.Pool, error) {
		deadlineCtx, cancel := context.WithDeadline(ctx, time.Now().Add(-1*time.Second))
		defer cancel()
		<-deadlineCtx.Done()
		return nil, deadlineCtx.Err()
	}
	timeoutMgr := pool.NewManager(cfg, timeoutOpener)
	err = timeoutMgr.InitSystemPools(context.Background())
	if err == nil {
		t.Fatal("expected timeout error, got nil")
	}

	// Close lifecycle test
	mgr.Close()

	_, err = mgr.SystemPool()
	if !errors.Is(err, pool.ErrPoolClosed) {
		t.Errorf("expected ErrPoolClosed, got: %v", err)
	}
	_, err = mgr.GetTenantTxPool(ctx, wsID)
	if !errors.Is(err, pool.ErrPoolClosed) {
		t.Errorf("expected ErrPoolClosed after Close, got: %v", err)
	}
}

func TestEvictIdleTenants(t *testing.T) {
	fakeOpener := func(ctx context.Context, pcfg *pgxpool.Config) (*pgxpool.Pool, error) {
		return nil, nil
	}

	cfg := pool.DefaultConfig()
	cfg.PrimaryURL = "postgres://user:pass@localhost:5432/system_db"
	mgr := pool.NewManager(cfg, fakeOpener)

	wsID := uuid.New()
	_, err := mgr.GetTenantTxPool(context.Background(), wsID)
	if err != nil {
		t.Fatalf("GetTenantTxPool failed: %v", err)
	}

	// Immediate eviction with 1-hour max idle should not evict anything
	evicted := mgr.EvictIdleTenants(1 * time.Hour)
	if evicted != 0 {
		t.Fatalf("expected 0 evicted, got %d", evicted)
	}

	// Eviction with 0 or negative idle should evict everything older than 0
	// Give a tiny pause so now > lastAccess
	time.Sleep(5 * time.Millisecond)
	evicted = mgr.EvictIdleTenants(1 * time.Millisecond)
	if evicted != 1 {
		t.Fatalf("expected 1 evicted, got %d", evicted)
	}
}
