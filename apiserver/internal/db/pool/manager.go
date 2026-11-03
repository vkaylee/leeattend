package pool

import (
	"context"
	"errors"
	"fmt"
	"net/url"
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
	ErrEmptyBaseURL   = errors.New("db pool: base url is empty")
	ErrAcquireTimeout = errors.New("db pool: connection acquire timed out")
	ErrNilPoolConfig  = errors.New("db pool: configuration is nil")
)

// Config defines connection pool settings for system and standalone workloads.
type Config struct {
	PrimaryURL             string
	PoolerTxURL            string
	PoolerSessURL          string
	TenantPoolerTxURL      string
	TenantPoolerSessURL    string
	ReaderURL              string
	APIMaxConn             int32
	APIAcquireTimeout      time.Duration
	WorkerMaxConn          int32
	WorkerAcquireTimeout   time.Duration
	TenantMaxConn          int32
	TenantIdleEvictTimeout time.Duration
	ConnectTimeout         time.Duration
	Standalone             bool
	StandaloneWorkspaceID  uuid.UUID
}

// DefaultConfig returns safe defaults matching LeeAttend configuration contracts.
func DefaultConfig() Config {
	return Config{
		APIMaxConn:             20,
		APIAcquireTimeout:      5 * time.Second,
		WorkerMaxConn:          5,
		WorkerAcquireTimeout:   30 * time.Second,
		TenantMaxConn:          5,
		TenantIdleEvictTimeout: 30 * time.Minute,
		ConnectTimeout:         5 * time.Second,
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

// TenantDBName computes the canonical isolated PostgreSQL database name for a workspace.
func TenantDBName(workspaceID uuid.UUID) string {
	clean := strings.ReplaceAll(workspaceID.String(), "-", "")
	return "tenant_" + clean
}

// BuildTenantURL swaps database name and optional credentials onto a template connection URL.
func BuildTenantURL(baseURL string, workspaceID uuid.UUID, overrideUser, overridePassword, overrideDBName string) (string, error) {
	if strings.TrimSpace(baseURL) == "" {
		return "", ErrEmptyBaseURL
	}
	parsed, err := url.Parse(baseURL)
	if err != nil {
		return "", fmt.Errorf("db pool: parse tenant base url: %w", err)
	}
	if parsed.Scheme != "postgres" && parsed.Scheme != "postgresql" {
		return "", ErrInvalidURL
	}

	targetDB := TenantDBName(workspaceID)
	if overrideDBName != "" {
		targetDB = overrideDBName
	}
	parsed.Path = "/" + targetDB

	if overrideUser != "" || overridePassword != "" {
		currentUser := ""
		currentPass := ""
		if parsed.User != nil {
			currentUser = parsed.User.Username()
			currentPass, _ = parsed.User.Password()
		}
		userToUse := currentUser
		if overrideUser != "" {
			userToUse = overrideUser
		}
		passToUse := currentPass
		if overridePassword != "" {
			passToUse = overridePassword
		}

		if passToUse != "" {
			parsed.User = url.UserPassword(userToUse, passToUse)
		} else if userToUse != "" {
			parsed.User = url.User(userToUse)
		}
	}

	return parsed.String(), nil
}

// PoolOpener abstracts opening a pgx pool for networkless unit testing.
type PoolOpener func(ctx context.Context, config *pgxpool.Config) (*pgxpool.Pool, error)

// TenantPoolEntry tracks tenant pool with last accessed timestamp for idle management.
type TenantPoolEntry struct {
	Pool       *pgxpool.Pool
	LastAccess time.Time
}

// Manager manages system pools and dynamic workspace tenant pools.
type Manager struct {
	cfg        Config
	poolOpener PoolOpener

	mu         sync.RWMutex
	closed     bool
	systemPool *pgxpool.Pool
	workerPool *pgxpool.Pool
	tenantTx   map[uuid.UUID]*TenantPoolEntry
	tenantSess map[uuid.UUID]*TenantPoolEntry
}

// NewManager creates a database pool manager.
func NewManager(cfg Config, opener PoolOpener) *Manager {
	if opener == nil {
		opener = func(ctx context.Context, pcfg *pgxpool.Config) (*pgxpool.Pool, error) {
			return pgxpool.NewWithConfig(ctx, pcfg)
		}
	}
	return &Manager{
		cfg:        cfg,
		poolOpener: opener,
		tenantTx:   make(map[uuid.UUID]*TenantPoolEntry),
		tenantSess: make(map[uuid.UUID]*TenantPoolEntry),
	}
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

// GetTenantTxPool retrieves or lazily opens a transaction-mode connection pool for workspace.
func (m *Manager) GetTenantTxPool(ctx context.Context, workspaceID uuid.UUID) (*pgxpool.Pool, error) {
	return m.getOrCreateTenantPool(ctx, workspaceID, true)
}

// GetTenantSessPool retrieves or lazily opens a session-mode connection pool for workspace.
func (m *Manager) GetTenantSessPool(ctx context.Context, workspaceID uuid.UUID) (*pgxpool.Pool, error) {
	return m.getOrCreateTenantPool(ctx, workspaceID, false)
}

func (m *Manager) getOrCreateTenantPool(ctx context.Context, workspaceID uuid.UUID, txMode bool) (*pgxpool.Pool, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.closed {
		return nil, ErrPoolClosed
	}
	if m.cfg.Standalone {
		if m.cfg.StandaloneWorkspaceID == uuid.Nil || workspaceID != m.cfg.StandaloneWorkspaceID {
			return nil, fmt.Errorf("db pool: workspace is not standalone organization")
		}
	}

	targetMap := m.tenantTx
	baseURL := m.cfg.TenantPoolerTxURL
	if !txMode {
		targetMap = m.tenantSess
		baseURL = m.cfg.TenantPoolerSessURL
	}
	if baseURL == "" {
		if txMode {
			baseURL = m.cfg.PoolerTxURL
		} else {
			baseURL = m.cfg.PoolerSessURL
		}
	}
	if baseURL == "" {
		baseURL = m.cfg.PrimaryURL
	}
	now := time.Now()
	if entry, exists := targetMap[workspaceID]; exists {
		entry.LastAccess = now
		return entry.Pool, nil
	}
	poolURL := baseURL
	if !m.cfg.Standalone {
		var err error
		poolURL, err = BuildTenantURL(baseURL, workspaceID, "", "", "")
		if err != nil {
			return nil, err
		}
	}
	connectTimeout := m.cfg.ConnectTimeout
	if connectTimeout <= 0 {
		connectTimeout = 5 * time.Second
	}
	poolCfg, err := BuildPgxConfig(poolURL, m.cfg.TenantMaxConn, m.cfg.APIAcquireTimeout, connectTimeout)
	if err != nil {
		return nil, err
	}
	openCtx, cancel := context.WithTimeout(ctx, connectTimeout)
	defer cancel()
	pool, err := m.poolOpener(openCtx, poolCfg)
	if err != nil {
		return nil, fmt.Errorf("db pool: open workspace pool %s failed: %w", workspaceID, err)
	}
	targetMap[workspaceID] = &TenantPoolEntry{Pool: pool, LastAccess: now}
	return pool, nil
}

// EvictIdleTenants closes and clears tenant pools idle longer than maxIdle duration.
func (m *Manager) EvictIdleTenants(maxIdle time.Duration) int {
	if maxIdle <= 0 {
		return 0
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.closed {
		return 0
	}

	now := time.Now()
	evicted := 0

	evictFromMap := func(tmap map[uuid.UUID]*TenantPoolEntry) {
		for id, entry := range tmap {
			if now.Sub(entry.LastAccess) > maxIdle {
				if entry.Pool != nil {
					entry.Pool.Close()
				}
				delete(tmap, id)
				evicted++
			}
		}
	}

	evictFromMap(m.tenantTx)
	evictFromMap(m.tenantSess)
	return evicted
}

// Close closes all open system and tenant pools cleanly.
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

	for id, entry := range m.tenantTx {
		if entry.Pool != nil {
			entry.Pool.Close()
		}
		delete(m.tenantTx, id)
	}

	for id, entry := range m.tenantSess {
		if entry.Pool != nil {
			entry.Pool.Close()
		}
		delete(m.tenantSess, id)
	}
}
