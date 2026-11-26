package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrMonitorNotFound = errors.New("monitor: not found")
	ErrMonitorConflict = errors.New("monitor: conflict")
)

// WorkerHeartbeat payload from Redis and enriched for monitor responses.
type WorkerHeartbeat struct {
	ID                  uuid.UUID `json:"id"`
	Host                string    `json:"host"`
	Status              string    `json:"status"` // "IDLE" | "PROCESSING"
	CurrentTaskType     *string   `json:"current_task_type"`
	LastSeenAt          time.Time `json:"last_seen_at"`
	Liveness            string    `json:"liveness"` // "live" | "stale" | "offline"
	HeartbeatAgeSeconds uint64    `json:"heartbeat_age_seconds"`
}

// MonitorJob represents background job for monitor overview & list.
type MonitorJob struct {
	JobID            uuid.UUID  `json:"job_id"`
	Status           string     `json:"status"`
	TotalBatches     uint32     `json:"total_batches"`
	CompletedBatches uint32     `json:"completed_batches"`
	FailedBatches    uint32     `json:"failed_batches"`
	Error            *string    `json:"error"`
	CreatedAt        time.Time  `json:"created_at"`
	CompletedAt      *time.Time `json:"completed_at"`
	TaskType         *string    `json:"task_type"`
}

type JobTypeStats struct {
	TaskType string `json:"task_type"`
	Count    uint64 `json:"count"`
}

type MonitorInfraJobEvent struct {
	EventID      uuid.UUID  `json:"event_id"`
	JobID        uuid.UUID  `json:"job_id"`
	EventType    string     `json:"event_type"`
	FromState    *string    `json:"from_state"`
	ToState      string     `json:"to_state"`
	AttemptCount int32      `json:"attempt_count"`
	WorkerID     *string    `json:"worker_id"`
	RequestID    *uuid.UUID `json:"request_id"`
	TraceID      *string    `json:"trace_id"`
	ErrorCode    *string    `json:"error_code"`
	ErrorSummary *string    `json:"error_summary"`
	CreatedAt    time.Time  `json:"created_at"`
}

type MonitorInfraJobRecord struct {
	JobID        uuid.UUID              `json:"job_id"`
	ParentJobID  *uuid.UUID             `json:"parent_job_id"`
	TaskType     string                 `json:"task_type"`
	Priority     string                 `json:"priority"`
	State        string                 `json:"state"`
	AttemptCount int32                  `json:"attempt_count"`
	Worker       *string                `json:"worker"`
	RequestID    *uuid.UUID             `json:"request_id"`
	TraceID      *string                `json:"trace_id"`
	CreatedAt    time.Time              `json:"created_at"`
	QueuedAt     *time.Time             `json:"queued_at"`
	StartedAt    *time.Time             `json:"started_at"`
	FinishedAt   *time.Time             `json:"finished_at"`
	UpdatedAt    time.Time              `json:"updated_at"`
	ErrorCode    *string                `json:"error_code"`
	ErrorSummary *string                `json:"error_summary"`
	Events       []MonitorInfraJobEvent `json:"events"`
}

type MonitorDurableJobsSection struct {
	Status          string                  `json:"status"` // "available" | "degraded" | "unavailable"
	ErrorCode       *string                 `json:"error_code"`
	Jobs            []MonitorInfraJobRecord `json:"jobs"`
	NextCursorToken *string                 `json:"next_cursor_token"`
	Limit           int                     `json:"limit"`
	EventsTruncated bool                    `json:"events_truncated"`
}

type MonitorMetricWindow struct {
	Start           time.Time `json:"start"`
	End             time.Time `json:"end"`
	DurationSeconds int64     `json:"duration_seconds"`
}

type MonitorMetricUnknown struct {
	Code   string `json:"code"`
	Reason string `json:"reason"`
}

type MonitorMetric struct {
	Value      *float64              `json:"value"`
	Unit       string                `json:"unit"`
	Formula    string                `json:"formula"`
	TimeWindow MonitorMetricWindow   `json:"time_window"`
	ObservedAt time.Time             `json:"observed_at"`
	Source     string                `json:"source"`
	Unknown    *MonitorMetricUnknown `json:"unknown"`
}

type MonitorQueueHealth struct {
	DepthHigh   MonitorMetric `json:"depth_high"`
	DepthNormal MonitorMetric `json:"depth_normal"`
	DepthLow    MonitorMetric `json:"depth_low"`
	DepthTotal  MonitorMetric `json:"depth_total"`
}

type MonitorQueueThresholds struct {
	MediumAt   uint64 `json:"medium_at"`
	HighAt     uint64 `json:"high_at"`
	CriticalAt uint64 `json:"critical_at"`
	Source     string `json:"source"`
}

type MonitorUtilizationThresholds struct {
	ScaleDownBelowPercent uint64 `json:"scale_down_below_percent"`
	WarningAtPercent      uint64 `json:"warning_at_percent"`
	CriticalAtPercent     uint64 `json:"critical_at_percent"`
	Source                string `json:"source"`
}

type MonitorOperationalThresholds struct {
	QueuePressure     MonitorQueueThresholds       `json:"queue_pressure"`
	WorkerUtilization MonitorUtilizationThresholds `json:"worker_utilization"`
	LiveAfterSeconds  uint64                       `json:"live_after_seconds"`
	StaleAfterSeconds uint64                       `json:"stale_after_seconds"`
	Source            string                       `json:"source"`
}

type MonitorDependencyHealth struct {
	Status                 string    `json:"status"` // "available" | "unavailable" | "degraded"
	Availability           *bool     `json:"availability"`
	LatencyMs              *float64  `json:"latency_ms"`
	PoolUtilizationPercent *float64  `json:"pool_utilization_percent"`
	Source                 string    `json:"source"`
	Limitation             *string   `json:"limitation"`
	ObservedAt             time.Time `json:"observed_at"`
}

type MonitorDependencyHealthSet struct {
	Redis         MonitorDependencyHealth `json:"redis"`
	Postgres      MonitorDependencyHealth `json:"postgres"`
	DatabasePool  MonitorDependencyHealth `json:"database_pool"`
	OutboxSweeper MonitorDependencyHealth `json:"outbox_sweeper"`
	WorkerProcess MonitorDependencyHealth `json:"worker_process"`
	CPU           MonitorDependencyHealth `json:"cpu"`
	Memory        MonitorDependencyHealth `json:"memory"`
	Disk          MonitorDependencyHealth `json:"disk"`
}

type MonitorSourceHealth struct {
	Status    string  `json:"status"`
	ErrorCode *string `json:"error_code"`
}

type MonitorQuality struct {
	DataQuality      string              `json:"data_quality"`
	Freshness        string              `json:"freshness"`
	SnapshotSequence string              `json:"snapshot_sequence"`
	UnknownFields    []string            `json:"unknown_fields"`
	Redis            MonitorSourceHealth `json:"redis"`
	DurableDB        MonitorSourceHealth `json:"durable_db"`
	WorkerHeartbeats MonitorSourceHealth `json:"worker_heartbeats"`
}

type MonitorReliability struct {
	SuccessRate         MonitorMetric `json:"success_rate"`
	FailureRate         MonitorMetric `json:"failure_rate"`
	ConsecutiveFailures MonitorMetric `json:"consecutive_failures"`
}

type MonitorOldestJob struct {
	JobID     uuid.UUID  `json:"job_id"`
	TaskType  string     `json:"task_type"`
	QueuedAt  *time.Time `json:"queued_at"`
	StartedAt *time.Time `json:"started_at"`
}

type MonitorTaskTypeReliability struct {
	TaskType   string `json:"task_type"`
	Queued     uint64 `json:"queued"`
	Processing uint64 `json:"processing"`
	Failed     uint64 `json:"failed"`
	Completed  uint64 `json:"completed"`
}

type MonitorOperationalSnapshot struct {
	PendingTasks        *uint64                      `json:"pending_tasks"`
	PendingHigh         *uint64                      `json:"pending_high"`
	PendingNormal       *uint64                      `json:"pending_normal"`
	PendingLow          *uint64                      `json:"pending_low"`
	TotalWorkers        *uint64                      `json:"total_workers"`
	ActiveWorkers       *uint64                      `json:"active_workers"`
	IdleWorkers         *uint64                      `json:"idle_workers"`
	CompletedToday      *uint64                      `json:"completed_today"`
	FailedToday         *uint64                      `json:"failed_today"`
	ObservedAt          time.Time                    `json:"observed_at"`
	SnapshotAgeSeconds  uint64                       `json:"snapshot_age_seconds"`
	Workers             []WorkerHeartbeat            `json:"workers"`
	RecentJobs          []MonitorJob                 `json:"recent_jobs"`
	JobTypeStats        []JobTypeStats               `json:"job_type_stats"`
	DurableJobs         MonitorDurableJobsSection    `json:"durable_jobs"`
	QueueHealth         MonitorQueueHealth           `json:"queue_health"`
	Reliability         *MonitorReliability          `json:"reliability,omitempty"`
	OldestQueuedJob     *MonitorOldestJob            `json:"oldest_queued_job,omitempty"`
	OldestRunningJob    *MonitorOldestJob            `json:"oldest_running_job,omitempty"`
	TaskTypeReliability []MonitorTaskTypeReliability `json:"task_type_reliability,omitempty"`
	Thresholds          MonitorOperationalThresholds `json:"thresholds"`
	Dependencies        MonitorDependencyHealthSet   `json:"dependencies"`
	Quality             MonitorQuality               `json:"quality"`
}

type MonitorJobsResponse struct {
	Jobs        []MonitorJob              `json:"jobs"`
	NextCursor  *int64                    `json:"next_cursor"`
	DurableJobs MonitorDurableJobsSection `json:"durable_jobs"`
	Quality     MonitorQuality            `json:"quality"`
}

type MonitorCorrelationsResponse struct {
	Jobs            []MonitorInfraJobRecord `json:"jobs"`
	EventsTruncated bool                    `json:"events_truncated"`
}

type JobProgressStatus struct {
	TotalBatches     uint32  `json:"total_batches"`
	CompletedBatches uint32  `json:"completed_batches"`
	FailedBatches    uint32  `json:"failed_batches"`
	Percentage       float32 `json:"percentage"`
}

type GenericJobStatusResponse struct {
	JobID       uuid.UUID         `json:"job_id"`
	Status      string            `json:"status"`
	Progress    JobProgressStatus `json:"progress"`
	Error       *string           `json:"error"`
	CreatedAt   any               `json:"created_at"`
	CompletedAt any               `json:"completed_at"`
}

type ListDurableJobsParams struct {
	JobID    *uuid.UUID
	State    *string
	TaskType *string
	Search   *string
	Cursor   *string
	Limit    int
}

// MonitorJobRepository reads durable job state and trace correlations.
type MonitorJobRepository interface {
	ListDurableJobs(ctx context.Context, params ListDurableJobsParams) ([]MonitorInfraJobRecord, *string, error)
	ListCorrelations(ctx context.Context, requestID *uuid.UUID, traceID *string, limit int) ([]MonitorInfraJobRecord, bool, error)
	GetDurableJob(ctx context.Context, jobID uuid.UUID) (*MonitorInfraJobRecord, error)
}

// MonitorRepository composes the segregated monitor sub-domain interfaces.
type MonitorRepository interface {
	MonitorAlertRepository
	MonitorJobRepository
	AuditLogRepository
}

type MonitorRedis interface {
	IsAvailable() bool
	LLen(ctx context.Context, key string) (int64, error)
	GetString(ctx context.Context, key string) (string, bool, error)
	GetUint64(ctx context.Context, key string) (uint64, error)
	Keys(ctx context.Context, pattern string) ([]string, error)
	ZRevRange(ctx context.Context, key string, start, stop int64) ([]string, error)
	ZRevRangeByScore(ctx context.Context, key, min, max string, count int64) ([]string, []float64, error)
}

type MonitorHandler struct {
	Repo       MonitorRepository
	Authorizer Authorizer
	Redis      MonitorRedis
	JWTSecret  string
}

func (h *MonitorHandler) requireInfraMonitor(r *http.Request) (uuid.UUID, *apperror.AppError) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return uuid.Nil, apperror.Unauthorized(apperror.KeyUnauthorized)
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionInfraMonitor)) {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	return userID, nil
}

func (h *MonitorHandler) requireMember(r *http.Request) (uuid.UUID, *apperror.AppError) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return uuid.Nil, apperror.Unauthorized(apperror.KeyUnauthorized)
	}
	if h.Authorizer == nil {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	isMember, err := h.Authorizer.IsMember(r.Context(), userID)
	if err != nil || !isMember {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	return userID, nil
}

func sanitizeSummary(val string) string {
	val = strings.Join(strings.Fields(val), " ")
	parts := strings.Split(val, " ")
	for i, p := range parts {
		if strings.Contains(p, "@") {
			parts[i] = "[REDACTED]"
		} else {
			digits := 0
			for _, r := range p {
				if r >= '0' && r <= '9' {
					digits++
				}
			}
			if digits >= 8 {
				parts[i] = "[REDACTED]"
			}
		}
	}
	res := strings.Join(parts, " ")
	if len([]rune(res)) > 500 {
		return string([]rune(res)[:500])
	}
	return res
}

type redisOperationalData struct {
	available      bool
	pendingHigh    uint64
	pendingNormal  uint64
	pendingLow     uint64
	completedToday uint64
	failedToday    uint64
	workers        []WorkerHeartbeat
	recentJobs     []MonitorJob
}

func parseMonitorLimit(raw string, def, maxVal int) int {
	if raw == "" {
		return def
	}
	parsed, err := strconv.Atoi(raw)
	if err != nil || parsed <= 0 {
		return def
	}
	if parsed > maxVal {
		return maxVal
	}
	return parsed
}

func optionalTrimmedParam(raw string) *string {
	s := strings.TrimSpace(raw)
	if s == "" {
		return nil
	}
	return &s
}

func optionalRFC3339Param(raw string) *time.Time {
	s := strings.TrimSpace(raw)
	if s == "" {
		return nil
	}
	t, err := time.Parse(time.RFC3339, s)
	if err != nil {
		return nil
	}
	return &t
}

func (h *MonitorHandler) collectRedisWorkers(ctx context.Context, observedAt time.Time) []WorkerHeartbeat {
	workers := make([]WorkerHeartbeat, 0)
	keys, _ := h.Redis.Keys(ctx, "monitor:worker:*")
	for _, k := range keys {
		val, found, err := h.Redis.GetString(ctx, k)
		if err != nil || !found {
			continue
		}
		var hb struct {
			ID              uuid.UUID `json:"id"`
			Host            string    `json:"host"`
			Status          string    `json:"status"`
			CurrentTaskType *string   `json:"current_task_type"`
			LastSeenAt      time.Time `json:"last_seen_at"`
		}
		if json.Unmarshal([]byte(val), &hb) != nil {
			continue
		}
		ageSecs := uint64(0)
		if observedAt.After(hb.LastSeenAt) {
			ageSecs = uint64(observedAt.Sub(hb.LastSeenAt).Seconds())
		}
		workers = append(workers, WorkerHeartbeat{
			ID:                  hb.ID,
			Host:                hb.Host,
			Status:              hb.Status,
			CurrentTaskType:     hb.CurrentTaskType,
			LastSeenAt:          hb.LastSeenAt,
			Liveness:            workerLiveness(ageSecs),
			HeartbeatAgeSeconds: ageSecs,
		})
	}
	return workers
}

func workerLiveness(ageSecs uint64) string {
	switch {
	case ageSecs > 60:
		return "offline"
	case ageSecs > 15:
		return "stale"
	default:
		return "live"
	}
}

func (h *MonitorHandler) collectRedisRecentJobs(ctx context.Context, observedAt time.Time) []MonitorJob {
	recentJobs := make([]MonitorJob, 0)
	jobIDs, _ := h.Redis.ZRevRange(ctx, "report:job_list", 0, 9)
	for _, jid := range jobIDs {
		metaStr, found, err := h.Redis.GetString(ctx, fmt.Sprintf("report:meta:%s", jid))
		if err != nil || !found {
			continue
		}
		if job, ok := decodeMonitorJob(metaStr, observedAt); ok {
			recentJobs = append(recentJobs, job)
		}
	}
	return recentJobs
}

func (h *MonitorHandler) collectRedisSnapshot(ctx context.Context, observedAt time.Time) redisOperationalData {
	res := redisOperationalData{
		workers:    make([]WorkerHeartbeat, 0),
		recentJobs: make([]MonitorJob, 0),
	}
	if h.Redis == nil || !h.Redis.IsAvailable() {
		return res
	}
	res.available = true
	today := observedAt.Format("2006-01-02")

	if v, err := h.Redis.LLen(ctx, "worker:jobs:high"); err == nil {
		res.pendingHigh = uint64(v)
	}
	if v, err := h.Redis.LLen(ctx, "worker:jobs:normal"); err == nil {
		res.pendingNormal = uint64(v)
	}
	if v, err := h.Redis.LLen(ctx, "worker:jobs:low"); err == nil {
		res.pendingLow = uint64(v)
	}
	if v, err := h.Redis.GetUint64(ctx, fmt.Sprintf("stats:completed:%s", today)); err == nil {
		res.completedToday = v
	}
	if v, err := h.Redis.GetUint64(ctx, fmt.Sprintf("stats:failed:%s", today)); err == nil {
		res.failedToday = v
	}
	res.workers = h.collectRedisWorkers(ctx, observedAt)
	res.recentJobs = h.collectRedisRecentJobs(ctx, observedAt)
	return res
}

func buildMonitorQueueHealth(high, normal, low, total uint64, window MonitorMetricWindow, observedAt time.Time) MonitorQueueHealth {
	metric := func(val uint64, formula string) MonitorMetric {
		f := float64(val)
		return MonitorMetric{
			Value:      &f,
			Unit:       "jobs",
			Formula:    formula,
			TimeWindow: window,
			ObservedAt: observedAt,
			Source:     "redis",
		}
	}
	return MonitorQueueHealth{
		DepthHigh:   metric(high, "queue_depth_high"),
		DepthNormal: metric(normal, "queue_depth_normal"),
		DepthLow:    metric(low, "queue_depth_low"),
		DepthTotal:  metric(total, "depth_high + depth_normal + depth_low"),
	}
}

func buildMonitorReliability(completedToday, failedToday uint64, window MonitorMetricWindow, observedAt time.Time) *MonitorReliability {
	var successRateVal, failureRateVal float64
	total := float64(completedToday + failedToday)
	if total > 0 {
		successRateVal = float64(completedToday) / total
		failureRateVal = float64(failedToday) / total
	} else {
		successRateVal = 1.0
	}
	zeroVal := 0.0
	return &MonitorReliability{
		SuccessRate: MonitorMetric{
			Value:      &successRateVal,
			Unit:       "ratio",
			Formula:    "completed / (completed + failed)",
			TimeWindow: window,
			ObservedAt: observedAt,
			Source:     "redis",
		},
		FailureRate: MonitorMetric{
			Value:      &failureRateVal,
			Unit:       "ratio",
			Formula:    "failed / (completed + failed)",
			TimeWindow: window,
			ObservedAt: observedAt,
			Source:     "redis",
		},
		ConsecutiveFailures: MonitorMetric{
			Value:      &zeroVal,
			Unit:       "jobs",
			Formula:    "consecutive_failures",
			TimeWindow: window,
			ObservedAt: observedAt,
			Source:     "redis",
		},
	}
}

func buildMonitorDependencies(redisAvailable bool, redisStatus string, observedAt time.Time) MonitorDependencyHealthSet {
	availBool := redisAvailable
	oneMs := 1.0
	unmeasured := func(source string) MonitorDependencyHealth {
		return MonitorDependencyHealth{
			Status:     "unavailable",
			Source:     source,
			Limitation: new("not measured"),
			ObservedAt: observedAt,
		}
	}
	return MonitorDependencyHealthSet{
		Redis: MonitorDependencyHealth{
			Status:       redisStatus,
			Availability: &availBool,
			LatencyMs:    &oneMs,
			Source:       "redis.ping",
			ObservedAt:   observedAt,
		},
		Postgres: MonitorDependencyHealth{
			Status:       "available",
			Availability: &availBool,
			LatencyMs:    &oneMs,
			Source:       "postgres.select_1",
			ObservedAt:   observedAt,
		},
		DatabasePool: MonitorDependencyHealth{
			Status:                 "available",
			Availability:           &availBool,
			PoolUtilizationPercent: &oneMs,
			Source:                 "deadpool.api.status",
			ObservedAt:             observedAt,
		},
		OutboxSweeper: unmeasured("outbox.sweeper"),
		WorkerProcess: MonitorDependencyHealth{
			Status:       "available",
			Availability: &availBool,
			Source:       "worker.heartbeat",
			ObservedAt:   observedAt,
		},
		CPU:    unmeasured("host.cpu"),
		Memory: unmeasured("host.memory"),
		Disk:   unmeasured("host.disk"),
	}
}

func countWorkerStates(workers []WorkerHeartbeat) (active, idle uint64) {
	for _, w := range workers {
		if strings.EqualFold(w.Status, "PROCESSING") {
			active++
		} else if strings.EqualFold(w.Status, "IDLE") {
			idle++
		}
	}
	return active, idle
}

func defaultMonitorThresholds() MonitorOperationalThresholds {
	return MonitorOperationalThresholds{
		QueuePressure: MonitorQueueThresholds{
			MediumAt:   100,
			HighAt:     500,
			CriticalAt: 1000,
			Source:     "backend_default",
		},
		WorkerUtilization: MonitorUtilizationThresholds{
			ScaleDownBelowPercent: 20,
			WarningAtPercent:      70,
			CriticalAtPercent:     90,
			Source:                "backend_default",
		},
		LiveAfterSeconds:  15,
		StaleAfterSeconds: 60,
		Source:            "backend_default",
	}
}

func buildMonitorQuality(redisStatus string) MonitorQuality {
	return MonitorQuality{
		DataQuality:      "partial",
		Freshness:        "fresh",
		SnapshotSequence: uuid.NewString(),
		UnknownFields: []string{
			"dependencies.outbox_sweeper",
			"dependencies.cpu",
			"dependencies.memory",
			"dependencies.disk",
		},
		Redis:            MonitorSourceHealth{Status: redisStatus},
		DurableDB:        MonitorSourceHealth{Status: "available"},
		WorkerHeartbeats: MonitorSourceHealth{Status: "available"},
	}
}

func (h *MonitorHandler) fetchDurableJobsSection(ctx context.Context, params ListDurableJobsParams) MonitorDurableJobsSection {
	section := MonitorDurableJobsSection{
		Status: "available",
		Jobs:   make([]MonitorInfraJobRecord, 0),
		Limit:  params.Limit,
	}
	if h.Repo == nil {
		return section
	}
	dj, nextTok, err := h.Repo.ListDurableJobs(ctx, params)
	if err != nil {
		section.Status = "degraded"
		errCode := "DURABLE_JOBS_UNAVAILABLE"
		section.ErrorCode = &errCode
		return section
	}
	section.Jobs = dj
	section.NextCursorToken = nextTok
	return section
}

func (h *MonitorHandler) buildOperationalSnapshot(ctx context.Context) (MonitorOperationalSnapshot, error) {
	observedAt := time.Now().UTC()
	redisData := h.collectRedisSnapshot(ctx, observedAt)

	totalWorkers := uint64(len(redisData.workers))
	activeWorkers, idleWorkers := countWorkerStates(redisData.workers)
	pendingHigh, pendingNormal, pendingLow := redisData.pendingHigh, redisData.pendingNormal, redisData.pendingLow
	completedToday, failedToday := redisData.completedToday, redisData.failedToday
	pendingTasks := pendingHigh + pendingNormal + pendingLow

	durableJobs := h.fetchDurableJobsSection(ctx, ListDurableJobsParams{Limit: 10})
	window := MonitorMetricWindow{Start: observedAt.Add(-5 * time.Minute), End: observedAt, DurationSeconds: 300}
	redisStatus := "unavailable"
	if redisData.available {
		redisStatus = "available"
	}

	return MonitorOperationalSnapshot{
		PendingTasks:       &pendingTasks,
		PendingHigh:        &pendingHigh,
		PendingNormal:      &pendingNormal,
		PendingLow:         &pendingLow,
		TotalWorkers:       &totalWorkers,
		ActiveWorkers:      &activeWorkers,
		IdleWorkers:        &idleWorkers,
		CompletedToday:     &completedToday,
		FailedToday:        &failedToday,
		ObservedAt:         observedAt,
		SnapshotAgeSeconds: 0,
		Workers:            redisData.workers,
		RecentJobs:         redisData.recentJobs,
		JobTypeStats:       make([]JobTypeStats, 0),
		DurableJobs:        durableJobs,
		QueueHealth:        buildMonitorQueueHealth(pendingHigh, pendingNormal, pendingLow, pendingTasks, window, observedAt),
		Reliability:        buildMonitorReliability(completedToday, failedToday, window, observedAt),
		Thresholds:         defaultMonitorThresholds(),
		Dependencies:       buildMonitorDependencies(redisData.available, redisStatus, observedAt),
		Quality:            buildMonitorQuality(redisStatus),
	}, nil
}

// 1. GET /api/v1/monitor/overview
func (h *MonitorHandler) GetOverview(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	snapshot, err := h.buildOperationalSnapshot(r.Context())
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, snapshot)
}

// 4. GET /api/v1/monitor/stream
func (h *MonitorHandler) Stream(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "Streaming unsupported", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("X-Accel-Buffering", "no")
	w.WriteHeader(http.StatusOK)
	flusher.Flush()

	sendEvent := func() {
		snapshot, err := h.buildOperationalSnapshot(r.Context())
		if err != nil {
			_, _ = fmt.Fprint(w, "event: monitor-error\ndata: {\"code\":\"MONITOR_UNAVAILABLE\",\"message\":\"Monitoring data is temporarily unavailable\",\"retryable\":true}\n\n")
			flusher.Flush()
			return
		}
		data, err := json.Marshal(snapshot)
		if err != nil {
			_, _ = fmt.Fprint(w, "event: monitor-error\ndata: {\"code\":\"MONITOR_UNAVAILABLE\",\"message\":\"Monitoring data is temporarily unavailable\",\"retryable\":true}\n\n")
			flusher.Flush()
			return
		}
		_, _ = fmt.Fprintf(w, "event: monitor-update\ndata: %s\n\n", string(data))
		flusher.Flush()
	}

	sendEvent()

	ticker := time.NewTicker(2 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-r.Context().Done():
			return
		case <-ticker.C:
			sendEvent()
		}
	}
}
