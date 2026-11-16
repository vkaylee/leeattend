package redis

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"strings"
	"sync"
	"time"

	"github.com/google/uuid"
	goredis "github.com/redis/go-redis/v9"
)

var (
	ErrInvalidRedisURL = errors.New("redis: connection url must start with redis:// or rediss://")
	ErrClientClosed    = errors.New("redis: client is closed")
	ErrNilRedisClient  = errors.New("redis: redis client is nil")
)

// FailurePolicy defines explicit degradation semantics when Redis is unavailable.
type FailurePolicy string

const (
	// FailOpen indicates non-fatal Redis degradation; request or routine continues with fallback.
	FailOpen FailurePolicy = "fail-open"
	// FailClosed indicates fatal Redis degradation; operation halts and bubbles error.
	FailClosed FailurePolicy = "fail-closed"
)

// PolicyMap documents failure policy per key family or operation.
var PolicyMap = map[string]FailurePolicy{
	"token_revocation_check": FailOpen,
	"login_attempt_counter":  FailOpen,
	"rate_limiting":          FailOpen,
	"email_rate_counter":     FailOpen,
	"roster_version_check":   FailOpen,
	"smtp_config_cache":      FailOpen,
	"password_reset":         FailClosed,
	"global_user_revocation": FailClosed,
	"job_queue":              FailClosed,
	"report_generation":      FailClosed,
	"attendance_ingestion":   FailClosed,
}

// Config encapsulates Redis client options with fail-closed bounds.
type Config struct {
	URL          string
	DialTimeout  time.Duration
	ReadTimeout  time.Duration
	WriteTimeout time.Duration
	PoolSize     int
	MinIdleConns int
}

// DefaultConfig provides conservative connection timeouts.
func DefaultConfig() Config {
	return Config{
		DialTimeout:  5 * time.Second,
		ReadTimeout:  3 * time.Second,
		WriteTimeout: 3 * time.Second,
		PoolSize:     20,
		MinIdleConns: 5,
	}
}

// ValidateRedisURL checks URL scheme prefix.
func ValidateRedisURL(rawURL string) error {
	trimmed := strings.TrimSpace(rawURL)
	if !strings.HasPrefix(trimmed, "redis://") && !strings.HasPrefix(trimmed, "rediss://") {
		return ErrInvalidRedisURL
	}
	return nil
}

// ParseRedisOptions parses and builds goredis.Options from a connection string.
func ParseRedisOptions(cfg Config) (*goredis.Options, error) {
	if err := ValidateRedisURL(cfg.URL); err != nil {
		return nil, err
	}
	opts, err := goredis.ParseURL(cfg.URL)
	if err != nil {
		return nil, fmt.Errorf("redis: parse url: %w", err)
	}

	if cfg.DialTimeout > 0 {
		opts.DialTimeout = cfg.DialTimeout
	}
	if cfg.ReadTimeout > 0 {
		opts.ReadTimeout = cfg.ReadTimeout
	}
	if cfg.WriteTimeout > 0 {
		opts.WriteTimeout = cfg.WriteTimeout
	}
	if cfg.PoolSize > 0 {
		opts.PoolSize = cfg.PoolSize
	}
	if cfg.MinIdleConns > 0 {
		opts.MinIdleConns = cfg.MinIdleConns
	}

	return opts, nil
}

// Client abstracts both standard fast non-blocking client and isolated blocking/pubsub clients.
type Client struct {
	cfg Config

	mu           sync.RWMutex
	closed       bool
	sharedClient *goredis.Client
	blockClient  *goredis.Client
}

// NewClient creates a new Client wrapping go-redis with dual-client isolation for BLPOP.
func NewClient(cfg Config) (*Client, error) {
	opts, err := ParseRedisOptions(cfg)
	if err != nil {
		return nil, err
	}

	shared := goredis.NewClient(opts)

	// Clone options for dedicated blocking operations (e.g. BLPOP) with independent pool
	blockOpts := *opts
	blockOpts.PoolSize = 10
	blockOpts.MinIdleConns = 2
	block := goredis.NewClient(&blockOpts)

	return &Client{
		cfg:          cfg,
		sharedClient: shared,
		blockClient:  block,
	}, nil
}

// NewClientWithUnderlying allows injection of pre-instantiated clients for test harnesses.
func NewClientWithUnderlying(cfg Config, shared, block *goredis.Client) *Client {
	return &Client{
		cfg:          cfg,
		sharedClient: shared,
		blockClient:  block,
	}
}

// Shared returns the general-purpose non-blocking client.
func (c *Client) Shared() (*goredis.Client, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	if c.closed {
		return nil, ErrClientClosed
	}
	return c.sharedClient, nil
}

// Blocking returns the dedicated client reserved for BLPOP and long-polling.
func (c *Client) Blocking() (*goredis.Client, error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	if c.closed {
		return nil, ErrClientClosed
	}
	return c.blockClient, nil
}

// Ping verifies connectivity on the shared client with context timeout.
func (c *Client) WriteWorkerHeartbeat(ctx context.Context, workerID uuid.UUID, status string, taskType *string, lastSeenAt time.Time) error {
	client, err := c.Shared()
	if err != nil {
		return err
	}
	payload, err := json.Marshal(struct {
		ID              uuid.UUID `json:"id"`
		Host            string    `json:"host"`
		Status          string    `json:"status"`
		CurrentTaskType *string   `json:"current_task_type"`
		LastSeenAt      time.Time `json:"last_seen_at"`
	}{ID: workerID, Host: "apiserver", Status: status, CurrentTaskType: taskType, LastSeenAt: lastSeenAt.UTC()})
	if err != nil {
		return err
	}
	return client.Set(ctx, KeyMonitorWorker(workerID), payload, TTLWorkerHeartbeat).Err()
}

func (c *Client) DeleteWorkerHeartbeat(ctx context.Context, workerID uuid.UUID) error {
	client, err := c.Shared()
	if err != nil {
		return err
	}
	return client.Del(ctx, KeyMonitorWorker(workerID)).Err()
}

func (c *Client) Ping(ctx context.Context) error {
	client, err := c.Shared()
	if err != nil {
		return err
	}
	if client == nil {
		return ErrNilRedisClient
	}
	return client.Ping(ctx).Err()
}

// Close shuts down both shared and blocking connections.
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed {
		return nil
	}
	c.closed = true

	var errs []error
	if c.sharedClient != nil {
		if err := c.sharedClient.Close(); err != nil {
			errs = append(errs, err)
		}
	}
	if c.blockClient != nil {
		if err := c.blockClient.Close(); err != nil {
			errs = append(errs, err)
		}
	}
	if len(errs) > 0 {
		return fmt.Errorf("redis: close failed: %v", errs)
	}
	return nil
}

// RedactURL masks password/credentials in Redis URLs for safe logging.
func RedactURL(rawURL string) string {
	u, err := url.Parse(rawURL)
	if err != nil {
		return "[REDACTED_URL]"
	}
	return u.Redacted()
}
