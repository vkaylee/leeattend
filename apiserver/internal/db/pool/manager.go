package pool

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
)

var (
	ErrInvalidURL     = errors.New("db pool: connection url must start with postgres:// or postgresql://")
	ErrPoolClosed     = errors.New("db pool: pool manager is closed")
	ErrContextTimeout = errors.New("db pool: operation timed out")
	ErrAcquireTimeout = errors.New("db pool: connection acquire timed out")
	ErrNilPoolConfig  = errors.New("db pool: configuration is nil")
)

// Config defines connection pool settings for the standalone database.
type Config struct {
	PrimaryURL           string
	PoolerTxURL          string
	PoolerSessURL        string
	ReaderURL            string
	APIMaxConn           int32
	APIAcquireTimeout    time.Duration
	WorkerMaxConn        int32
	WorkerAcquireTimeout time.Duration
	ConnectTimeout       time.Duration
	WorkspaceID          uuid.UUID
}

// DefaultConfig returns safe defaults matching LeeAttend configuration contracts.
func DefaultConfig() Config {
	return Config{
		APIMaxConn:           20,
		APIAcquireTimeout:    5 * time.Second,
		WorkerMaxConn:        5,
		WorkerAcquireTimeout: 30 * time.Second,
		ConnectTimeout:       5 * time.Second,
	}
}

// ValidateDBURL ensures DB connection strings begin with postgres:// or postgresql://.
func ValidateDBURL(u string) error {
	trimmed := strings.TrimSpace(u)
	if !strings.HasPrefix(trimmed, "postgres://") && !strings.HasPrefix(trimmed, "postgresql://") {
		return ErrInvalidURL
	}
	return nil
}

// BuildPgxConfig creates a configured *pgxpool.Config with bounded connection lifecycle.
func BuildPgxConfig(connString string, maxConns int32, acquireTimeout, connectTimeout time.Duration) (*pgxpool.Config, error) {
	if err := ValidateDBURL(connString); err != nil {
		return nil, err
	}
	poolCfg, err := pgxpool.ParseConfig(connString)
	if err != nil {
		return nil, fmt.Errorf("db pool: parse config failed: %w", err)
	}

	if maxConns > 0 {
		poolCfg.MaxConns = maxConns
	}
	if poolCfg.MinConns > poolCfg.MaxConns {
		poolCfg.MinConns = poolCfg.MaxConns
	}

	if acquireTimeout > 0 {
		poolCfg.ConnConfig.ConnectTimeout = acquireTimeout
	}
	if connectTimeout > 0 {
		poolCfg.ConnConfig.ConnectTimeout = connectTimeout
	}

	poolCfg.MaxConnLifetime = 1 * time.Hour
	poolCfg.MaxConnIdleTime = 30 * time.Minute
	return poolCfg, nil
}

// PoolOpener abstracts opening a pgx pool for networkless unit testing.
type PoolOpener func(ctx context.Context, config *pgxpool.Config) (*pgxpool.Pool, error)

// Manager manages the API and worker pools for one standalone organization.
type Manager struct {
	cfg        Config
	poolOpener PoolOpener

	mu         sync.RWMutex
	closed     bool
	systemPool *pgxpool.Pool
	workerPool *pgxpool.Pool
}

// NewManager creates a database pool manager.
func NewManager(cfg Config, opener PoolOpener) *Manager {
	if opener == nil {
		opener = func(ctx context.Context, pcfg *pgxpool.Config) (*pgxpool.Pool, error) {
			return pgxpool.NewWithConfig(ctx, pcfg)
		}
	}
	return &Manager{cfg: cfg, poolOpener: opener}
}

// InitSystemPools initializes system primary/API and worker pools with fail-closed timeout.
func (m *Manager) InitSystemPools(ctx context.Context) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.closed {
		return ErrPoolClosed
	}

	connectTimeout := m.cfg.ConnectTimeout
	if connectTimeout <= 0 {
		connectTimeout = 5 * time.Second
	}

	// 1. System/API Pool (pooler_tx_url or primary_url)
	apiURL := m.cfg.PoolerTxURL
	if apiURL == "" {
		apiURL = m.cfg.PrimaryURL
	}
	if apiURL == "" {
		return fmt.Errorf("db pool: missing api/primary url")
	}
	apiCfg, err := BuildPgxConfig(apiURL, m.cfg.APIMaxConn, m.cfg.APIAcquireTimeout, connectTimeout)
	if err != nil {
		return err
	}

	initCtx, cancel := context.WithTimeout(ctx, connectTimeout)
	defer cancel()

	apiPool, err := m.poolOpener(initCtx, apiCfg)
	if err != nil {
		return fmt.Errorf("db pool: failed to init system api pool: %w", err)
	}
	m.systemPool = apiPool

	// 2. System Worker Pool (pooler_sess_url or fallback)
	workerURL := m.cfg.PoolerSessURL
	if workerURL == "" {
		workerURL = apiURL
	}
	workerCfg, err := BuildPgxConfig(workerURL, m.cfg.WorkerMaxConn, m.cfg.WorkerAcquireTimeout, connectTimeout)
	if err != nil {
		m.systemPool.Close()
		m.systemPool = nil
		return err
	}

	workerCtx, workerCancel := context.WithTimeout(ctx, connectTimeout)
	defer workerCancel()

	workerPool, err := m.poolOpener(workerCtx, workerCfg)
	if err != nil {
		m.systemPool.Close()
		m.systemPool = nil
		return fmt.Errorf("db pool: failed to init system worker pool: %w", err)
	}
	m.workerPool = workerPool

	return nil
}

// SystemPool returns the primary system connection pool.
func (m *Manager) SystemPool() (*pgxpool.Pool, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.closed {
		return nil, ErrPoolClosed
	}
	return m.systemPool, nil
}

// WorkerPool returns the worker connection pool.
func (m *Manager) WorkerPool() (*pgxpool.Pool, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.closed {
		return nil, ErrPoolClosed
	}
	return m.workerPool, nil
}

// GetTenantTxPool returns the shared API pool after validating the singleton workspace.
func (m *Manager) GetTenantTxPool(_ context.Context, workspaceID uuid.UUID) (*pgxpool.Pool, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.closed {
		return nil, ErrPoolClosed
	}
	if m.cfg.WorkspaceID == uuid.Nil || workspaceID != m.cfg.WorkspaceID {
		return nil, fmt.Errorf("db pool: workspace is not configured organization")
	}
	return m.systemPool, nil
}

// GetTenantSessPool returns the shared worker pool after validating the singleton workspace.
func (m *Manager) GetTenantSessPool(_ context.Context, workspaceID uuid.UUID) (*pgxpool.Pool, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if m.closed {
		return nil, ErrPoolClosed
	}
	if m.cfg.WorkspaceID == uuid.Nil || workspaceID != m.cfg.WorkspaceID {
		return nil, fmt.Errorf("db pool: workspace is not configured organization")
	}
	return m.workerPool, nil
}

// Close closes both standalone pools cleanly.
func (m *Manager) Close() {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.closed {
		return
	}
	m.closed = true

	if m.systemPool != nil {
		m.systemPool.Close()
		m.systemPool = nil
	}
	if m.workerPool != nil {
		m.workerPool.Close()
		m.workerPool = nil
	}
}
