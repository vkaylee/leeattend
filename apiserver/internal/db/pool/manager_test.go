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

func TestStandaloneTenantPoolsReuseOwnedPools(t *testing.T) {
	workspaceID := uuid.New()
	openCount := 0
	opener := func(ctx context.Context, cfg *pgxpool.Config) (*pgxpool.Pool, error) {
		openCount++
		return pgxpool.NewWithConfig(ctx, cfg)
	}
	cfg := pool.DefaultConfig()
	cfg.Standalone = true
	cfg.StandaloneWorkspaceID = workspaceID
	cfg.PrimaryURL = "postgres://user:pass@localhost:5432/standalone"
	cfg.PoolerTxURL = "postgres://user:pass@localhost:6543/standalone"
	cfg.PoolerSessURL = "postgres://user:pass@localhost:5452/standalone"

	mgr := pool.NewManager(cfg, opener)
	if err := mgr.InitSystemPools(context.Background()); err != nil {
		t.Fatalf("InitSystemPools failed: %v", err)
	}
	defer mgr.Close()

	systemPool, err := mgr.SystemPool()
	if err != nil {
		t.Fatalf("SystemPool failed: %v", err)
	}
	workerPool, err := mgr.WorkerPool()
	if err != nil {
		t.Fatalf("WorkerPool failed: %v", err)
	}
	txPool, err := mgr.GetTenantTxPool(context.Background(), workspaceID)
	if err != nil {
		t.Fatalf("GetTenantTxPool failed: %v", err)
	}
	sessPool, err := mgr.GetTenantSessPool(context.Background(), workspaceID)
	if err != nil {
		t.Fatalf("GetTenantSessPool failed: %v", err)
	}
	if txPool != systemPool || sessPool != workerPool {
		t.Fatal("standalone tenant pools did not reuse owned pools")
	}
	if openCount != 2 {
		t.Fatalf("expected only system and worker pools, got %d opens", openCount)
	}
	if evicted := mgr.EvictIdleTenants(time.Nanosecond); evicted != 0 {
		t.Fatalf("expected no standalone tenant pool eviction, got %d", evicted)
	}
	if got, err := mgr.GetTenantTxPool(context.Background(), workspaceID); err != nil || got != systemPool {
		t.Fatalf("shared pool changed after eviction: pool=%p err=%v", got, err)
	}
	if _, err := mgr.GetTenantTxPool(context.Background(), uuid.New()); err == nil {
		t.Fatal("expected standalone workspace rejection")
	}
}

func TestSaaSTenantPoolsDeriveAndIsolateWorkspaceDatabases(t *testing.T) {
	var databases []string
	opener := func(ctx context.Context, cfg *pgxpool.Config) (*pgxpool.Pool, error) {
		databases = append(databases, cfg.ConnConfig.Database)
		return pgxpool.NewWithConfig(ctx, cfg)
	}
	cfg := pool.DefaultConfig()
	cfg.TenantPoolerTxURL = "postgres://user:pass@localhost:6543/template"
	mgr := pool.NewManager(cfg, opener)
	defer mgr.Close()

	firstID, secondID := uuid.New(), uuid.New()
	firstPool, err := mgr.GetTenantTxPool(context.Background(), firstID)
	if err != nil {
		t.Fatalf("first tenant pool failed: %v", err)
	}
	secondPool, err := mgr.GetTenantTxPool(context.Background(), secondID)
	if err != nil {
		t.Fatalf("second tenant pool failed: %v", err)
	}
	if firstPool == secondPool {
		t.Fatal("SaaS tenant pools are not isolated")
	}
	if len(databases) != 2 || databases[0] != pool.TenantDBName(firstID) || databases[1] != pool.TenantDBName(secondID) {
		t.Fatalf("unexpected tenant databases: %v", databases)
	}
}
