package redis_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

func TestValidateRedisURL(t *testing.T) {
	valid := []string{
		"redis://localhost:6379",
		"rediss://default:password@clustercfg.redis.amazonaws.com:6379",
		"redis://:mysecret@127.0.0.1:6379/0",
	}
	for _, u := range valid {
		if err := redis.ValidateRedisURL(u); err != nil {
			t.Errorf("expected valid for %s, got: %v", u, err)
		}
	}

	invalid := []string{
		"http://localhost:6379",
		"tcp://localhost:6379",
		"",
		"postgres://localhost:5432",
	}
	for _, u := range invalid {
		if err := redis.ValidateRedisURL(u); !errors.Is(err, redis.ErrInvalidRedisURL) {
			t.Errorf("expected ErrInvalidRedisURL for %s, got: %v", u, err)
		}
	}
}

func TestRedactURL(t *testing.T) {
	raw := "redis://app_user:secretpassword@redis-host:6379/1"
	redacted := redis.RedactURL(raw)
	if redacted != "redis://app_user:xxxxx@redis-host:6379/1" {
		t.Fatalf("expected redacted password, got: %s", redacted)
	}
}

func TestParseRedisOptions(t *testing.T) {
	cfg := redis.DefaultConfig()
	cfg.URL = "redis://:secret@localhost:6379/2"
	cfg.DialTimeout = 2 * time.Second
	cfg.ReadTimeout = 4 * time.Second
	cfg.WriteTimeout = 4 * time.Second
	cfg.PoolSize = 50

	opts, err := redis.ParseRedisOptions(cfg)
	if err != nil {
		t.Fatalf("ParseRedisOptions failed: %v", err)
	}

	if opts.Addr != "localhost:6379" {
		t.Errorf("expected Addr localhost:6379, got: %s", opts.Addr)
	}
	if opts.Password != "secret" {
		t.Errorf("expected password 'secret', got: %s", opts.Password)
	}
	if opts.DB != 2 {
		t.Errorf("expected DB=2, got: %d", opts.DB)
	}
	if opts.PoolSize != 50 {
		t.Errorf("expected PoolSize=50, got: %d", opts.PoolSize)
	}
	if opts.DialTimeout != 2*time.Second {
		t.Errorf("expected DialTimeout=2s, got: %v", opts.DialTimeout)
	}
}

func TestClientLifecycleWithoutNetwork(t *testing.T) {
	cfg := redis.DefaultConfig()
	cfg.URL = "redis://localhost:6379/0"

	client, err := redis.NewClient(cfg)
	if err != nil {
		t.Fatalf("NewClient failed: %v", err)
	}

	shared, err := client.Shared()
	if err != nil || shared == nil {
		t.Fatalf("expected non-nil shared client, got err: %v", err)
	}

	block, err := client.Blocking()
	if err != nil || block == nil {
		t.Fatalf("expected non-nil blocking client, got err: %v", err)
	}

	// Close client cleanly
	if err := client.Close(); err != nil {
		t.Fatalf("Close failed: %v", err)
	}

	// Subsequent calls return ErrClientClosed
	if _, err := client.Shared(); !errors.Is(err, redis.ErrClientClosed) {
		t.Errorf("expected ErrClientClosed for Shared(), got: %v", err)
	}
	if _, err := client.Blocking(); !errors.Is(err, redis.ErrClientClosed) {
		t.Errorf("expected ErrClientClosed for Blocking(), got: %v", err)
	}
	if err := client.Ping(context.Background()); !errors.Is(err, redis.ErrClientClosed) {
		t.Errorf("expected ErrClientClosed for Ping(), got: %v", err)
	}
}

func TestKeyBuildersAndTTLPolicy(t *testing.T) {
	wsID := uuid.MustParse("b0000000-0000-0000-0000-000000000001")
	userID := uuid.MustParse("c0000000-0000-0000-0000-000000000002")
	jobID := uuid.MustParse("d0000000-0000-0000-0000-000000000003")

	if k := redis.KeyLoginAttempts("user@example.com"); k != "login:attempts:user@example.com" {
		t.Errorf("KeyLoginAttempts mismatch: %s", k)
	}
	if k := redis.KeyLoginLockout("user@example.com"); k != "login:lockout:user@example.com" {
		t.Errorf("KeyLoginLockout mismatch: %s", k)
	}
	if k := redis.KeyJWTRevoked("sig_xyz"); k != "jwt:revoked:sig_xyz" {
		t.Errorf("KeyJWTRevoked mismatch: %s", k)
	}
	if k := redis.KeyUserRevokedAt(userID); k != "user:revoked_at:c0000000-0000-0000-0000-000000000002" {
		t.Errorf("KeyUserRevokedAt mismatch: %s", k)
	}
	if k := redis.KeyJWTRotated("refresh_tok_1"); k != "jwt:rotated:refresh_tok_1" {
		t.Errorf("KeyJWTRotated mismatch: %s", k)
	}
	if k := redis.KeyPasswordReset("reset_tok_2"); k != "pwd_reset:reset_tok_2" {
		t.Errorf("KeyPasswordReset mismatch: %s", k)
	}
	if k := redis.KeyRateLimit("127.0.0.1", "read", "/api/v1/auth"); k != "rate:127.0.0.1:read:/api/v1/auth" {
		t.Errorf("KeyRateLimit mismatch: %s", k)
	}

	// Single-organization cache keys
	if k := redis.KeyCacheShifts(); k != "cache:shifts" {
		t.Errorf("KeyCacheShifts mismatch: %s", k)
	}
	if k := redis.KeyRosterVersion(); k != "roster:version" {
		t.Errorf("KeyRosterVersion mismatch: %s", k)
	}
	if k := redis.KeyCacheAuthPermissions(userID); k != "cache:auth:permissions:c0000000-0000-0000-0000-000000000002" {
		t.Errorf("KeyCacheAuthPermissions mismatch: %s", k)
	}
	if k := redis.KeyCacheAuthPermissionsPattern(); k != "cache:auth:permissions:*" {
		t.Errorf("KeyCacheAuthPermissionsPattern mismatch: %s", k)
	}
	if k := redis.KeyCacheEmployment(userID); k != "cache:employment:c0000000-0000-0000-0000-000000000002" {
		t.Errorf("KeyCacheEmployment mismatch: %s", k)
	}
	if k := redis.KeyCacheLeaveTypeExists(wsID); k != "cache:leave_type_exists:b0000000-0000-0000-0000-000000000001" {
		t.Errorf("KeyCacheLeaveTypeExists mismatch: %s", k)
	}
	if k := redis.KeyCacheEmployeeLeaves(userID); k != "cache:employee_leaves:c0000000-0000-0000-0000-000000000002" {
		t.Errorf("KeyCacheEmployeeLeaves mismatch: %s", k)
	}
	if k := redis.KeySMTPConfig(); k != "smtp:config" {
		t.Errorf("KeySMTPConfig mismatch: %s", k)
	}

	// Email rate keys
	if k := redis.KeyEmailRateMin(123456); k != "email:rate:min:123456" {
		t.Errorf("KeyEmailRateMin mismatch: %s", k)
	}
	if k := redis.KeyEmailRateHour(123456); k != "email:rate:hour:123456" {
		t.Errorf("KeyEmailRateHour mismatch: %s", k)
	}
	if k := redis.KeyEmailRateDay(123456); k != "email:rate:day:123456" {
		t.Errorf("KeyEmailRateDay mismatch: %s", k)
	}

	// Queue and report keys
	if k := redis.KeyWorkerQueue("", redis.PriorityHigh); k != "worker:jobs:high" {
		t.Errorf("KeyWorkerQueue mismatch: %s", k)
	}
	if k := redis.KeyWorkerQueue("prod", redis.PriorityNormal); k != "prod:worker:jobs:normal" {
		t.Errorf("KeyWorkerQueue mismatch: %s", k)
	}
	if k := redis.KeyReportMeta(jobID); k != "report:meta:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyReportMeta mismatch: %s", k)
	}
	if k := redis.KeyReportJobList(); k != "report:job_list" {
		t.Errorf("KeyReportJobList mismatch: %s", k)
	}
	if k := redis.KeyReportCounter(jobID); k != "report:counter:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyReportCounter mismatch: %s", k)
	}
	if k := redis.KeyReportCounterFailed(jobID); k != "report:counter:d0000000-0000-0000-0000-000000000003:failed" {
		t.Errorf("KeyReportCounterFailed mismatch: %s", k)
	}
	if k := redis.KeyReportMergeTriggered(jobID); k != "report:merge_triggered:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyReportMergeTriggered mismatch: %s", k)
	}
	if k := redis.KeyReportResults(jobID); k != "report:results:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyReportResults mismatch: %s", k)
	}
	if k := redis.KeyReportResultData(jobID); k != "report:results:d0000000-0000-0000-0000-000000000003:data" {
		t.Errorf("KeyReportResultData mismatch: %s", k)
	}
	if k := redis.KeyReportResultFilename(jobID); k != "report:results:d0000000-0000-0000-0000-000000000003:filename" {
		t.Errorf("KeyReportResultFilename mismatch: %s", k)
	}
	if k := redis.KeyReportDLQ(); k != "report:dlq" {
		t.Errorf("KeyReportDLQ mismatch: %s", k)
	}
	if k := redis.KeyStatsCompleted("2026-10-19"); k != "stats:completed:2026-10-19" {
		t.Errorf("KeyStatsCompleted mismatch: %s", k)
	}
	if k := redis.KeyStatsFailed("2026-10-19"); k != "stats:failed:2026-10-19" {
		t.Errorf("KeyStatsFailed mismatch: %s", k)
	}
	if k := redis.KeyImportProgress(jobID); k != "import_progress:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyImportProgress mismatch: %s", k)
	}
	if k := redis.KeyImportErrors(jobID); k != "import_errors:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyImportErrors mismatch: %s", k)
	}
	if k := redis.KeyImportChunksDone(jobID); k != "import_chunks_done:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyImportChunksDone mismatch: %s", k)
	}
	if k := redis.KeyImportTotalChunks(jobID); k != "import_total_chunks:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyImportTotalChunks mismatch: %s", k)
	}
	if k := redis.KeyLockHousekeeping("2026-10-19"); k != "lock:housekeeping:2026-10-19" {
		t.Errorf("KeyLockHousekeeping mismatch: %s", k)
	}
	if k := redis.ChannelReportStream(jobID); k != "report:stream:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("ChannelReportStream mismatch: %s", k)
	}
	if k := redis.KeyMonitorWorker(jobID); k != "monitor:worker:d0000000-0000-0000-0000-000000000003" {
		t.Errorf("KeyMonitorWorker mismatch: %s", k)
	}
}

func TestCalculateLoginLockoutTTL(t *testing.T) {
	// <= 5 attempts: 900s
	if ttl := redis.CalculateLoginLockoutTTL(3); ttl != 900*time.Second {
		t.Errorf("expected 900s for 3 attempts, got: %v", ttl)
	}
	if ttl := redis.CalculateLoginLockoutTTL(5); ttl != 900*time.Second {
		t.Errorf("expected 900s for 5 attempts, got: %v", ttl)
	}

	// 6 attempts: 2^(6-5) * 900s = 1800s
	if ttl := redis.CalculateLoginLockoutTTL(6); ttl != 1800*time.Second {
		t.Errorf("expected 1800s for 6 attempts, got: %v", ttl)
	}

	// 7 attempts: 2^(7-5) * 900s = 3600s
	if ttl := redis.CalculateLoginLockoutTTL(7); ttl != 3600*time.Second {
		t.Errorf("expected 3600s for 7 attempts, got: %v", ttl)
	}

	// 12 attempts: 2^7 * 900 = 128 * 900 = 115200s > cap (86400s) -> 86400s
	if ttl := redis.CalculateLoginLockoutTTL(12); ttl != 86400*time.Second {
		t.Errorf("expected 86400s cap for 12 attempts, got: %v", ttl)
	}
}

func TestFailurePolicyMapping(t *testing.T) {
	if redis.PolicyMap["rate_limiting"] != redis.FailOpen {
		t.Errorf("rate_limiting must be FailOpen")
	}
	if redis.PolicyMap["token_revocation_check"] != redis.FailOpen {
		t.Errorf("token_revocation_check must be FailOpen")
	}
	if redis.PolicyMap["password_reset"] != redis.FailClosed {
		t.Errorf("password_reset must be FailClosed")
	}
	if redis.PolicyMap["job_queue"] != redis.FailClosed {
		t.Errorf("job_queue must be FailClosed")
	}
}
