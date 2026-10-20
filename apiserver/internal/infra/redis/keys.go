package redis

import (
	"fmt"
	"math"
	"time"

	"github.com/google/uuid"
)

// TTL Policies matching Rust apiserver constants and inventory
const (
	TTLZero                 time.Duration = 0
	TTLAgentPresence        time.Duration = 60 * time.Second
	TTLLoginAttempts        time.Duration = 900 * time.Second
	TTLLoginLockoutBase     time.Duration = 900 * time.Second
	TTLLoginLockoutMax      time.Duration = 86400 * time.Second
	TTLJWTRevokedDefault    time.Duration = 900 * time.Second
	TTLUserRevokedAt        time.Duration = 900 * time.Second
	TTLJWTRotated           time.Duration = 7 * 24 * time.Hour
	TTLPasswordReset        time.Duration = 900 * time.Second
	TTLRateLimitDefault     time.Duration = 60 * time.Second
	TTLDefaultCache         time.Duration = 300 * time.Second
	TTLAuthPermissionsCache time.Duration = 3600 * time.Second
	TTLSystemRolesCache     time.Duration = 1800 * time.Second
	TTLEmailRateMin         time.Duration = 60 * time.Second
	TTLEmailRateHour        time.Duration = 3600 * time.Second
	TTLEmailRateDay         time.Duration = 86400 * time.Second
	TTLJobMetadata          time.Duration = 86400 * time.Second
	TTLReportResultBinary   time.Duration = 3600 * time.Second
	TTLStatsRetention       time.Duration = 7 * 24 * time.Hour
	TTLImportProgress       time.Duration = 86400 * time.Second
	TTLLockHousekeeping     time.Duration = 86400 * time.Second
	TTLIngestDelivery       time.Duration = 1800 * time.Second
	TTLWorkerHeartbeat      time.Duration = 90 * time.Second
)

// CalculateLoginLockoutTTL computes exponential backoff for login lockouts:
// Starts at 900s, increases by 2^(attempts-5) * 900s, capped at 86400s (24h).
func CalculateLoginLockoutTTL(attempts int) time.Duration {
	if attempts <= 5 {
		return TTLLoginLockoutBase
	}
	exponent := float64(attempts - 5)
	multiplier := math.Pow(2, exponent)
	backoffSecs := float64(TTLLoginLockoutBase/time.Second) * multiplier
	if backoffSecs > float64(TTLLoginLockoutMax/time.Second) {
		return TTLLoginLockoutMax
	}
	return time.Duration(backoffSecs) * time.Second
}

// Key Builders — Auth family
func KeyAgentPresence(agentID string) string {
	return fmt.Sprintf("agent:presence:%s", agentID)
}

func KeyLoginAttempts(email string) string {
	return fmt.Sprintf("login:attempts:%s", email)
}

func KeyLoginLockout(email string) string {
	return fmt.Sprintf("login:lockout:%s", email)
}

func KeyJWTRevoked(signature string) string {
	return fmt.Sprintf("jwt:revoked:%s", signature)
}

func KeyUserRevokedAt(userID uuid.UUID) string {
	return fmt.Sprintf("user:revoked_at:%s", userID.String())
}

func KeyJWTRotated(refreshToken string) string {
	return fmt.Sprintf("jwt:rotated:%s", refreshToken)
}

func KeyPasswordReset(token string) string {
	return fmt.Sprintf("pwd_reset:%s", token)
}

func KeyRateLimit(identity, tier, matchedPath string) string {
	return fmt.Sprintf("rate:%s:%s:%s", identity, tier, matchedPath)
}

// Key Builders — Cache family
func KeyCacheWorkspace(workspaceID uuid.UUID) string {
	return fmt.Sprintf("cache:workspace:%s", workspaceID.String())
}

func KeyCacheShifts(workspaceID uuid.UUID) string {
	return fmt.Sprintf("cache:shifts:%s", workspaceID.String())
}

func KeyCacheWorkspaceConfig(workspaceID uuid.UUID) string {
	return fmt.Sprintf("cache:workspace_config:%s", workspaceID.String())
}

func KeyRosterVersion(workspaceID uuid.UUID) string {
	return fmt.Sprintf("roster:version:%s", workspaceID.String())
}

func KeyCacheAuthPermissions(workspaceID, userID uuid.UUID) string {
	return fmt.Sprintf("cache:auth:permissions:%s:%s", workspaceID.String(), userID.String())
}

func KeyCacheAuthPermissionsPattern(workspaceID uuid.UUID) string {
	return fmt.Sprintf("cache:auth:permissions:%s:*", workspaceID.String())
}

func KeyCacheSystemRoles(userID uuid.UUID) string {
	return fmt.Sprintf("cache:system:roles:%s", userID.String())
}

func KeyCacheEmployment(employmentID uuid.UUID) string {
	return fmt.Sprintf("cache:employment:%s", employmentID.String())
}

func KeyCacheLeaveTypeExists(leaveTypeID uuid.UUID) string {
	return fmt.Sprintf("cache:leave_type_exists:%s", leaveTypeID.String())
}

func KeyCacheEmployeeLeaves(employeeID uuid.UUID) string {
	return fmt.Sprintf("cache:employee_leaves:%s", employeeID.String())
}

func KeySMTPConfig() string {
	return "smtp:config"
}

// Key Builders — Rate limiting family
func KeyEmailRateMin(unixWindow uint64) string {
	return fmt.Sprintf("email:rate:min:%d", unixWindow)
}

func KeyEmailRateHour(unixWindow uint64) string {
	return fmt.Sprintf("email:rate:hour:%d", unixWindow)
}

func KeyEmailRateDay(unixWindow uint64) string {
	return fmt.Sprintf("email:rate:day:%d", unixWindow)
}

// Key Builders — Queue & Worker family
type QueuePriority string

const (
	PriorityHigh   QueuePriority = "high"
	PriorityNormal QueuePriority = "normal"
	PriorityLow    QueuePriority = "low"
)

func KeyWorkerQueue(namespace string, priority QueuePriority) string {
	if namespace == "" {
		return fmt.Sprintf("worker:jobs:%s", priority)
	}
	return fmt.Sprintf("%s:worker:jobs:%s", namespace, priority)
}

func KeyReportMeta(jobID uuid.UUID) string {
	return fmt.Sprintf("report:meta:%s", jobID.String())
}

func KeyReportJobList() string {
	return "report:job_list"
}

func KeyReportCounter(jobID uuid.UUID) string {
	return fmt.Sprintf("report:counter:%s", jobID.String())
}

func KeyReportCounterFailed(jobID uuid.UUID) string {
	return fmt.Sprintf("report:counter:%s:failed", jobID.String())
}

func KeyReportMergeTriggered(jobID uuid.UUID) string {
	return fmt.Sprintf("report:merge_triggered:%s", jobID.String())
}

func KeyReportResults(jobID uuid.UUID) string {
	return fmt.Sprintf("report:results:%s", jobID.String())
}

func KeyReportResultData(jobID uuid.UUID) string {
	return fmt.Sprintf("report:results:%s:data", jobID.String())
}

func KeyReportResultFilename(jobID uuid.UUID) string {
	return fmt.Sprintf("report:results:%s:filename", jobID.String())
}

func KeyReportDLQ() string {
	return "report:dlq"
}

func KeyStatsCompleted(date string) string {
	return fmt.Sprintf("stats:completed:%s", date)
}

func KeyStatsFailed(date string) string {
	return fmt.Sprintf("stats:failed:%s", date)
}

func KeyImportProgress(jobID uuid.UUID) string {
	return fmt.Sprintf("import_progress:%s", jobID.String())
}

func KeyImportErrors(jobID uuid.UUID) string {
	return fmt.Sprintf("import_errors:%s", jobID.String())
}

func KeyImportChunksDone(jobID uuid.UUID) string {
	return fmt.Sprintf("import_chunks_done:%s", jobID.String())
}

func KeyImportTotalChunks(jobID uuid.UUID) string {
	return fmt.Sprintf("import_total_chunks:%s", jobID.String())
}

func KeyLockHousekeeping(date string) string {
	return fmt.Sprintf("lock:housekeeping:%s", date)
}

func KeyIngestDelivery(agentID, msgID string) string {
	return fmt.Sprintf("ingest:delivery:%s:%s", agentID, msgID)
}

func ChannelReportStream(jobID uuid.UUID) string {
	return fmt.Sprintf("report:stream:%s", jobID.String())
}

func KeyMonitorWorker(workerID uuid.UUID) string {
	return fmt.Sprintf("monitor:worker:%s", workerID.String())
}
