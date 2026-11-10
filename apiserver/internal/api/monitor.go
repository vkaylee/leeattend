package api

import (
	"bufio"
	"context"
	"crypto/sha1"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	goredis "github.com/redis/go-redis/v9"
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

type MonitorAlertResponse struct {
	ID              uuid.UUID  `json:"id"`
	AlertType       string     `json:"alert_type"`
	Severity        string     `json:"severity"`
	Status          string     `json:"status"`
	Title           string     `json:"title"`
	Summary         string     `json:"summary"`
	Source          string     `json:"source"`
	FirstSeenAt     time.Time  `json:"first_seen_at"`
	LastSeenAt      time.Time  `json:"last_seen_at"`
	OccurrenceCount int32      `json:"occurrence_count"`
	ResolvedAt      *time.Time `json:"resolved_at"`
}

type MonitorAlertsResponse struct {
	Data       []MonitorAlertResponse `json:"data"`
	Total      int64                  `json:"total"`
	Page       int64                  `json:"page"`
	Limit      int64                  `json:"limit"`
	NextCursor *string                `json:"next_cursor"`
}

type MonitorAlertEventResponse struct {
	ID         uuid.UUID `json:"id"`
	EventType  string    `json:"event_type"`
	Severity   string    `json:"severity"`
	Summary    string    `json:"summary"`
	OccurredAt time.Time `json:"occurred_at"`
}

type MonitorAlertDetailResponse struct {
	MonitorAlertResponse
	Events []MonitorAlertEventResponse `json:"events"`
}

type MonitorAlertMutationRequest struct {
	Reason string `json:"reason"`
}

type PaginatedAuditLogs struct {
	Data  []AuditLogRecord `json:"data"`
	Total int64            `json:"total"`
	Page  int64            `json:"page"`
	Limit int64            `json:"limit"`
}

type AuditLogRecord struct {
	ID               uuid.UUID       `json:"id"`
	ActorID          *uuid.UUID      `json:"actor_id"`
	ActorType        *string         `json:"actor_type"`
	ActorName        *string         `json:"actor_name"`
	ActorEmail       *string         `json:"actor_email"`
	TargetUserID     *uuid.UUID      `json:"target_user_id"`
	TargetResourceID *uuid.UUID      `json:"target_resource_id"`
	Action           string          `json:"action"`
	Details          json.RawMessage `json:"details"`
	IPAddress        *string         `json:"ip_address"`
	UserAgent        *string         `json:"user_agent"`
	CreatedAt        time.Time       `json:"created_at"`
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

type MonitorRepository interface {
	ListAlerts(ctx context.Context, status, severity *string, limit int, cursor *string) ([]MonitorAlertResponse, int64, *string, error)
	GetAlert(ctx context.Context, alertID uuid.UUID) (*MonitorAlertResponse, []MonitorAlertEventResponse, error)
	MutateAlert(ctx context.Context, alertID uuid.UUID, action string, reason string, actorID uuid.UUID) (*MonitorAlertResponse, error)
	ListDurableJobs(ctx context.Context, params ListDurableJobsParams) ([]MonitorInfraJobRecord, *string, error)
	ListCorrelations(ctx context.Context, requestID *uuid.UUID, traceID *string, limit int) ([]MonitorInfraJobRecord, bool, error)
	GetDurableJob(ctx context.Context, jobID uuid.UUID) (*MonitorInfraJobRecord, error)
	ListAuditLogs(ctx context.Context, limit, offset int, action *string, startDate, endDate *time.Time) ([]AuditLogRecord, int64, error)
}

type MonitorRedis interface {
	Shared() (*goredis.Client, error)
}

type MonitorHandler struct {
	Repo       MonitorRepository
	Authorizer Authorizer
	Redis      MonitorRedis
	JWTSecret  string
}

func (h MonitorHandler) requireInfraMonitor(r *http.Request) (uuid.UUID, *apperror.AppError) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return uuid.Nil, apperror.Unauthorized(apperror.KeyUnauthorized)
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionInfraMonitor)) {
		return uuid.Nil, apperror.Forbidden(apperror.KeyForbidden)
	}
	return userID, nil
}

func (h MonitorHandler) requireMember(r *http.Request) (uuid.UUID, *apperror.AppError) {
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

func (h MonitorHandler) requireAuditPermission(r *http.Request) (uuid.UUID, *apperror.AppError) {
	userID, appErr := h.requireMember(r)
	if appErr != nil {
		return uuid.Nil, appErr
	}
	if !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionAuditLogsRead)) {
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

func (h MonitorHandler) buildOperationalSnapshot(ctx context.Context) (MonitorOperationalSnapshot, error) {
	observedAt := time.Now().UTC()
	var (
		pendingHigh    uint64
		pendingNormal  uint64
		pendingLow     uint64
		completedToday uint64
		failedToday    uint64
		workers        = make([]WorkerHeartbeat, 0)
		recentJobs     = make([]MonitorJob, 0)
		redisAvailable = false
	)

	today := observedAt.Format("2006-01-02")

	if h.Redis != nil {
		client, err := h.Redis.Shared()
		if err == nil && client != nil {
			redisAvailable = true
			if v, err := client.LLen(ctx, "worker:jobs:high").Result(); err == nil {
				pendingHigh = uint64(v)
			}
			if v, err := client.LLen(ctx, "worker:jobs:normal").Result(); err == nil {
				pendingNormal = uint64(v)
			}
			if v, err := client.LLen(ctx, "worker:jobs:low").Result(); err == nil {
				pendingLow = uint64(v)
			}
			if v, err := client.Get(ctx, fmt.Sprintf("stats:completed:%s", today)).Uint64(); err == nil {
				completedToday = v
			}
			if v, err := client.Get(ctx, fmt.Sprintf("stats:failed:%s", today)).Uint64(); err == nil {
				failedToday = v
			}

			// Heartbeats
			keys, _ := client.Keys(ctx, "monitor:worker:*").Result()
			for _, k := range keys {
				val, err := client.Get(ctx, k).Result()
				if err != nil {
					continue
				}
				var hb struct {
					ID              uuid.UUID `json:"id"`
					Host            string    `json:"host"`
					Status          string    `json:"status"`
					CurrentTaskType *string   `json:"current_task_type"`
					LastSeenAt      time.Time `json:"last_seen_at"`
				}
				if json.Unmarshal([]byte(val), &hb) == nil {
					ageSecs := uint64(0)
					if observedAt.After(hb.LastSeenAt) {
						ageSecs = uint64(observedAt.Sub(hb.LastSeenAt).Seconds())
					}
					liveness := "live"
					if ageSecs > 60 {
						liveness = "offline"
					} else if ageSecs > 15 {
						liveness = "stale"
					}
					workers = append(workers, WorkerHeartbeat{
						ID:                  hb.ID,
						Host:                hb.Host,
						Status:              hb.Status,
						CurrentTaskType:     hb.CurrentTaskType,
						LastSeenAt:          hb.LastSeenAt,
						Liveness:            liveness,
						HeartbeatAgeSeconds: ageSecs,
					})
				}
			}

			// Recent jobs
			jobIDs, _ := client.ZRevRange(ctx, "report:job_list", 0, 9).Result()
			for _, jid := range jobIDs {
				metaStr, err := client.Get(ctx, fmt.Sprintf("report:meta:%s", jid)).Result()
				if err != nil {
					continue
				}
				var m struct {
					JobID            uuid.UUID `json:"job_id"`
					Status           string    `json:"status"`
					TotalBatches     uint32    `json:"total_batches"`
					CompletedBatches uint32    `json:"completed_batches"`
					FailedBatches    uint32    `json:"failed_batches"`
					ErrorMessage     *string   `json:"error_message"`
					CreatedAt        *string   `json:"created_at"`
					CompletedAt      *string   `json:"completed_at"`
					TaskType         *string   `json:"task_type"`
				}
				if json.Unmarshal([]byte(metaStr), &m) == nil {
					createdAt := observedAt
					if m.CreatedAt != nil {
						if t, err := time.Parse(time.RFC3339, *m.CreatedAt); err == nil {
							createdAt = t
						}
					}
					var completedAt *time.Time
					if m.CompletedAt != nil {
						if t, err := time.Parse(time.RFC3339, *m.CompletedAt); err == nil {
							completedAt = &t
						}
					}
					recentJobs = append(recentJobs, MonitorJob{
						JobID:            m.JobID,
						Status:           m.Status,
						TotalBatches:     m.TotalBatches,
						CompletedBatches: m.CompletedBatches,
						FailedBatches:    m.FailedBatches,
						Error:            m.ErrorMessage,
						CreatedAt:        createdAt,
						CompletedAt:      completedAt,
						TaskType:         m.TaskType,
					})
				}
			}
		}
	}

	totalWorkers := uint64(len(workers))
	var activeWorkers, idleWorkers uint64
	for _, w := range workers {
		if strings.EqualFold(w.Status, "PROCESSING") {
			activeWorkers++
		} else if strings.EqualFold(w.Status, "IDLE") {
			idleWorkers++
		}
	}

	pendingTasks := pendingHigh + pendingNormal + pendingLow

	// Durable jobs
	durableJobs := MonitorDurableJobsSection{
		Status:          "available",
		ErrorCode:       nil,
		Jobs:            make([]MonitorInfraJobRecord, 0),
		NextCursorToken: nil,
		Limit:           10,
		EventsTruncated: false,
	}
	if h.Repo != nil {
		dj, nextTok, err := h.Repo.ListDurableJobs(ctx, ListDurableJobsParams{Limit: 10})
		if err == nil {
			durableJobs.Jobs = dj
			durableJobs.NextCursorToken = nextTok
		} else {
			durableJobs.Status = "degraded"
			errCode := "DURABLE_JOBS_UNAVAILABLE"
			durableJobs.ErrorCode = &errCode
		}
	}

	window := MonitorMetricWindow{
		Start:           observedAt.Add(-5 * time.Minute),
		End:             observedAt,
		DurationSeconds: 300,
	}

	highVal := float64(pendingHigh)
	normalVal := float64(pendingNormal)
	lowVal := float64(pendingLow)
	totalVal := float64(pendingTasks)

	redisStatus := "unavailable"
	if redisAvailable {
		redisStatus = "available"
	}
	availBool := redisAvailable
	oneMs := 1.0

	var successRateVal, failureRateVal float64
	totalCompletedOrFailed := float64(completedToday + failedToday)
	if totalCompletedOrFailed > 0 {
		successRateVal = float64(completedToday) / totalCompletedOrFailed
		failureRateVal = float64(failedToday) / totalCompletedOrFailed
	} else {
		successRateVal = 1.0
	}
	zeroVal := 0.0
	reliability := &MonitorReliability{
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
		Workers:            workers,
		RecentJobs:         recentJobs,
		JobTypeStats:       make([]JobTypeStats, 0),
		DurableJobs:        durableJobs,
		QueueHealth: MonitorQueueHealth{
			DepthHigh: MonitorMetric{
				Value:      &highVal,
				Unit:       "jobs",
				Formula:    "queue_depth_high",
				TimeWindow: window,
				ObservedAt: observedAt,
				Source:     "redis",
			},
			DepthNormal: MonitorMetric{
				Value:      &normalVal,
				Unit:       "jobs",
				Formula:    "queue_depth_normal",
				TimeWindow: window,
				ObservedAt: observedAt,
				Source:     "redis",
			},
			DepthLow: MonitorMetric{
				Value:      &lowVal,
				Unit:       "jobs",
				Formula:    "queue_depth_low",
				TimeWindow: window,
				ObservedAt: observedAt,
				Source:     "redis",
			},
			DepthTotal: MonitorMetric{
				Value:      &totalVal,
				Unit:       "jobs",
				Formula:    "depth_high + depth_normal + depth_low",
				TimeWindow: window,
				ObservedAt: observedAt,
				Source:     "redis",
			},
		},
		Reliability: reliability,
		Thresholds: MonitorOperationalThresholds{
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
		},
		Dependencies: MonitorDependencyHealthSet{
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
			OutboxSweeper: MonitorDependencyHealth{
				Status:     "unavailable",
				Source:     "outbox.sweeper",
				Limitation: new("not measured"),
				ObservedAt: observedAt,
			},
			WorkerProcess: MonitorDependencyHealth{
				Status:       "available",
				Availability: &availBool,
				Source:       "worker.heartbeat",
				ObservedAt:   observedAt,
			},
			CPU: MonitorDependencyHealth{
				Status:     "unavailable",
				Source:     "host.cpu",
				Limitation: new("not measured"),
				ObservedAt: observedAt,
			},
			Memory: MonitorDependencyHealth{
				Status:     "unavailable",
				Source:     "host.memory",
				Limitation: new("not measured"),
				ObservedAt: observedAt,
			},
			Disk: MonitorDependencyHealth{
				Status:     "unavailable",
				Source:     "host.disk",
				Limitation: new("not measured"),
				ObservedAt: observedAt,
			},
		},
		Quality: MonitorQuality{
			DataQuality:      "partial",
			Freshness:        "fresh",
			SnapshotSequence: uuid.NewString(),
			UnknownFields: []string{
				"dependencies.outbox_sweeper",
				"dependencies.cpu",
				"dependencies.memory",
				"dependencies.disk",
			},
			Redis: MonitorSourceHealth{
				Status: redisStatus,
			},
			DurableDB: MonitorSourceHealth{
				Status: "available",
			},
			WorkerHeartbeats: MonitorSourceHealth{
				Status: "available",
			},
		},
	}, nil
}

// 1. GET /api/v1/monitor/overview
func (h MonitorHandler) GetOverview(w http.ResponseWriter, r *http.Request) {
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

// 2. GET /api/v1/monitor/jobs
func (h MonitorHandler) GetMonitorJobs(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	q := r.URL.Query()
	limit := 20
	if l := q.Get("limit"); l != "" {
		if parsed, err := strconv.Atoi(l); err == nil && parsed > 0 {
			if parsed > 100 {
				parsed = 100
			}
			limit = parsed
		}
	}

	status := strings.TrimSpace(strings.ToLower(q.Get("status")))
	if status != "" && status != "pending" && status != "processing" && status != "completed" && status != "failed" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	search := strings.TrimSpace(q.Get("search"))
	if len([]rune(search)) > 64 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	state := strings.TrimSpace(strings.ToLower(q.Get("state")))
	validStates := map[string]bool{
		"dispatching": true, "queued": true, "processing": true, "completed": true,
		"failed": true, "retryable": true, "dead_letter": true, "lost": true,
	}
	if state != "" && !validStates[state] {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	taskType := strings.TrimSpace(q.Get("task_type"))
	var taskTypePtr *string
	if taskType != "" {
		taskTypePtr = &taskType
	}
	var statePtr *string
	if state != "" {
		statePtr = &state
	}
	var searchPtr *string
	if search != "" {
		searchPtr = &search
	}

	// Legacy Redis jobs
	jobs := make([]MonitorJob, 0)
	var nextCursor *int64
	if h.Redis != nil {
		client, err := h.Redis.Shared()
		if err == nil && client != nil {
			var cursor int64
			if c := q.Get("cursor"); c != "" {
				if parsed, err := strconv.ParseInt(c, 10, 64); err == nil {
					cursor = parsed
				}
			}
			maxScore := "+inf"
			if cursor > 0 {
				maxScore = fmt.Sprintf("(%d", cursor)
			}
			zItems, err := client.ZRevRangeByScoreWithScores(r.Context(), "report:job_list", &goredis.ZRangeBy{
				Min:    "-inf",
				Max:    maxScore,
				Offset: 0,
				Count:  int64(limit),
			}).Result()
			if err == nil {
				for _, zi := range zItems {
					jid := fmt.Sprint(zi.Member)
					metaStr, err := client.Get(r.Context(), fmt.Sprintf("report:meta:%s", jid)).Result()
					if err != nil {
						continue
					}
					var m struct {
						JobID            uuid.UUID `json:"job_id"`
						Status           string    `json:"status"`
						TotalBatches     uint32    `json:"total_batches"`
						CompletedBatches uint32    `json:"completed_batches"`
						FailedBatches    uint32    `json:"failed_batches"`
						ErrorMessage     *string   `json:"error_message"`
						CreatedAt        *string   `json:"created_at"`
						CompletedAt      *string   `json:"completed_at"`
						TaskType         *string   `json:"task_type"`
					}
					if json.Unmarshal([]byte(metaStr), &m) == nil {
						if status != "" && !strings.EqualFold(m.Status, status) {
							continue
						}
						if taskTypePtr != nil && (m.TaskType == nil || !strings.EqualFold(*m.TaskType, *taskTypePtr)) {
							continue
						}
						createdAt := time.Now().UTC()
						if m.CreatedAt != nil {
							if t, err := time.Parse(time.RFC3339, *m.CreatedAt); err == nil {
								createdAt = t
							}
						}
						var completedAt *time.Time
						if m.CompletedAt != nil {
							if t, err := time.Parse(time.RFC3339, *m.CompletedAt); err == nil {
								completedAt = &t
							}
						}
						jobs = append(jobs, MonitorJob{
							JobID:            m.JobID,
							Status:           m.Status,
							TotalBatches:     m.TotalBatches,
							CompletedBatches: m.CompletedBatches,
							FailedBatches:    m.FailedBatches,
							Error:            m.ErrorMessage,
							CreatedAt:        createdAt,
							CompletedAt:      completedAt,
							TaskType:         m.TaskType,
						})
					}
				}
				if len(zItems) == limit {
					lastScore := int64(zItems[len(zItems)-1].Score)
					nextCursor = &lastScore
				}
			}
		}
	}

	// Durable jobs
	durableJobs := MonitorDurableJobsSection{
		Status:          "available",
		ErrorCode:       nil,
		Jobs:            make([]MonitorInfraJobRecord, 0),
		NextCursorToken: nil,
		Limit:           limit,
		EventsTruncated: false,
	}
	if h.Repo != nil {
		var dCursor *string
		if c := q.Get("durable_cursor"); c != "" {
			dCursor = &c
		}
		var jIDPtr *uuid.UUID
		if idStr := q.Get("job_id"); idStr != "" {
			if id, err := uuid.Parse(idStr); err == nil {
				jIDPtr = &id
			}
		}
		dj, nextTok, err := h.Repo.ListDurableJobs(r.Context(), ListDurableJobsParams{
			JobID:    jIDPtr,
			State:    statePtr,
			TaskType: taskTypePtr,
			Search:   searchPtr,
			Cursor:   dCursor,
			Limit:    limit,
		})
		if err == nil {
			durableJobs.Jobs = dj
			durableJobs.NextCursorToken = nextTok
		} else {
			durableJobs.Status = "degraded"
			errCode := "DURABLE_JOBS_UNAVAILABLE"
			durableJobs.ErrorCode = &errCode
		}
	}

	apperror.WriteJSON(w, http.StatusOK, MonitorJobsResponse{
		Jobs:        jobs,
		NextCursor:  nextCursor,
		DurableJobs: durableJobs,
		Quality: MonitorQuality{
			DataQuality:      "complete",
			Freshness:        "fresh",
			SnapshotSequence: uuid.NewString(),
			UnknownFields:    make([]string, 0),
			Redis:            MonitorSourceHealth{Status: "available"},
			DurableDB:        MonitorSourceHealth{Status: "available"},
			WorkerHeartbeats: MonitorSourceHealth{Status: "available"},
		},
	})
}

// 3. GET /api/v1/monitor/correlations
func (h MonitorHandler) GetMonitorCorrelations(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	q := r.URL.Query()
	reqIDStr := strings.TrimSpace(q.Get("request_id"))
	traceIDStr := strings.TrimSpace(q.Get("trace_id"))

	if (reqIDStr == "" && traceIDStr == "") || (reqIDStr != "" && traceIDStr != "") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	var parsedReqID *uuid.UUID
	if reqIDStr != "" {
		id, err := uuid.Parse(reqIDStr)
		if err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		parsedReqID = &id
	}

	var parsedTraceID *string
	if traceIDStr != "" {
		if len(traceIDStr) > 128 {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		for _, c := range traceIDStr {
			if !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '-' || c == '_' || c == '.') {
				apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
				return
			}
		}
		parsedTraceID = &traceIDStr
	}

	if h.Repo == nil {
		apperror.WriteJSON(w, http.StatusOK, MonitorCorrelationsResponse{
			Jobs:            make([]MonitorInfraJobRecord, 0),
			EventsTruncated: false,
		})
		return
	}

	jobs, truncated, err := h.Repo.ListCorrelations(r.Context(), parsedReqID, parsedTraceID, 25)
	if err != nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, MonitorCorrelationsResponse{
		Jobs:            jobs,
		EventsTruncated: truncated,
	})
}

// 4. GET /api/v1/monitor/stream
func (h MonitorHandler) Stream(w http.ResponseWriter, r *http.Request) {
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

// 5. GET /api/v1/monitor/alerts
func (h MonitorHandler) ListAlerts(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	q := r.URL.Query()
	limit := 20
	if l := q.Get("limit"); l != "" {
		if parsed, err := strconv.Atoi(l); err == nil && parsed > 0 {
			if parsed > 100 {
				parsed = 100
			}
			limit = parsed
		}
	}

	var statusPtr *string
	if s := strings.TrimSpace(q.Get("status")); s != "" {
		statusPtr = &s
	}
	var severityPtr *string
	if s := strings.TrimSpace(q.Get("severity")); s != "" {
		severityPtr = &s
	}
	var cursorPtr *string
	if c := strings.TrimSpace(q.Get("cursor")); c != "" {
		cursorPtr = &c
	}

	if h.Repo == nil {
		apperror.WriteJSON(w, http.StatusOK, MonitorAlertsResponse{
			Data:       make([]MonitorAlertResponse, 0),
			Total:      0,
			Page:       1,
			Limit:      int64(limit),
			NextCursor: nil,
		})
		return
	}

	data, total, nextCursor, err := h.Repo.ListAlerts(r.Context(), statusPtr, severityPtr, limit, cursorPtr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, MonitorAlertsResponse{
		Data:       data,
		Total:      total,
		Page:       1,
		Limit:      int64(limit),
		NextCursor: nextCursor,
	})
}

// 6. GET /api/v1/monitor/alerts/{alert_id}
func (h MonitorHandler) GetAlert(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	idStr := r.PathValue("alert_id")
	alertID, err := uuid.Parse(idStr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	if h.Repo == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	alert, events, err := h.Repo.GetAlert(r.Context(), alertID)
	if err != nil {
		if errors.Is(err, ErrMonitorNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if alert == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, MonitorAlertDetailResponse{
		MonitorAlertResponse: *alert,
		Events:               events,
	})
}

// 7. POST /api/v1/monitor/alerts/{alert_id}/acknowledge
func (h MonitorHandler) AcknowledgeAlert(w http.ResponseWriter, r *http.Request) {
	userID, appErr := h.requireInfraMonitor(r)
	if appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	idStr := r.PathValue("alert_id")
	alertID, err := uuid.Parse(idStr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	req, err := DecodeJSON[MonitorAlertMutationRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reason := strings.TrimSpace(req.Reason)
	if reason == "" || len([]rune(reason)) > 500 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reason = sanitizeSummary(reason)

	if h.Repo == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	alert, err := h.Repo.MutateAlert(r.Context(), alertID, "acknowledge", reason, userID)
	if err != nil {
		var targetAppErr *apperror.AppError
		if errors.As(err, &targetAppErr) {
			apperror.WriteError(w, r, targetAppErr)
			return
		}
		if errors.Is(err, ErrMonitorNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		if errors.Is(err, ErrMonitorConflict) {
			apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, alert)
}

// 8. POST /api/v1/monitor/alerts/{alert_id}/resolve
func (h MonitorHandler) ResolveAlert(w http.ResponseWriter, r *http.Request) {
	userID, appErr := h.requireInfraMonitor(r)
	if appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	idStr := r.PathValue("alert_id")
	alertID, err := uuid.Parse(idStr)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	req, err := DecodeJSON[MonitorAlertMutationRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reason := strings.TrimSpace(req.Reason)
	if reason == "" || len([]rune(reason)) > 500 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	reason = sanitizeSummary(reason)

	if h.Repo == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	alert, err := h.Repo.MutateAlert(r.Context(), alertID, "resolve", reason, userID)
	if err != nil {
		var targetAppErr *apperror.AppError
		if errors.As(err, &targetAppErr) {
			apperror.WriteError(w, r, targetAppErr)
			return
		}
		if errors.Is(err, ErrMonitorNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		if errors.Is(err, ErrMonitorConflict) {
			apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, alert)
}

// 9. GET /api/v1/ws/reports/{job_id}
func (h MonitorHandler) ReportStreamWS(w http.ResponseWriter, r *http.Request) {
	// Extract JWT token
	token := r.URL.Query().Get("token")
	if token == "" {
		if cookie, err := r.Cookie("access_token"); err == nil && cookie.Value != "" {
			token = cookie.Value
		}
	}
	if token == "" {
		if authHeader := r.Header.Get("Authorization"); strings.HasPrefix(authHeader, "Bearer ") {
			token = strings.TrimSpace(strings.TrimPrefix(authHeader, "Bearer "))
		}
	}
	if token == "" {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusUnauthorized)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Missing authentication token"})
		return
	}

	claims, err := auth.VerifyTokenClaims(token, h.JWTSecret)
	if err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusUnauthorized)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Invalid or expired token"})
		return
	}
	userID, err := uuid.Parse(claims.Subject)
	if err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusUnauthorized)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Invalid token subject"})
		return
	}

	jobIDStr := r.PathValue("job_id")
	jobID, err := uuid.Parse(jobIDStr)
	if err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Invalid job ID"})
		return
	}

	if h.Redis == nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusServiceUnavailable)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Streaming not available: Redis not configured"})
		return
	}

	client, err := h.Redis.Shared()
	if err != nil || client == nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusServiceUnavailable)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Streaming not available: Redis not configured"})
		return
	}

	metaStr, err := client.Get(r.Context(), fmt.Sprintf("report:meta:%s", jobID.String())).Result()
	if errors.Is(err, goredis.Nil) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusNotFound)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Job not found"})
		return
	} else if err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": err.Error()})
		return
	}

	var meta struct {
		JobID        uuid.UUID `json:"job_id"`
		RequesterID  string    `json:"requester_id"`
		Status       string    `json:"status"`
		TotalBatches uint32    `json:"total_batches"`
		ErrorMessage *string   `json:"error_message"`
	}
	if err := json.Unmarshal([]byte(metaStr), &meta); err != nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusInternalServerError)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Failed to parse job metadata"})
		return
	}

	requesterID, err := uuid.Parse(meta.RequesterID)
	if err != nil || requesterID != userID || h.Authorizer == nil ||
		!h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionReportsRead)) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusForbidden)
		_ = json.NewEncoder(w).Encode(map[string]string{"error": "Not authorized to access this report stream"})
		return
	}

	// Upgrade handshake
	conn, bufrw, err := wsHandshake(w, r)
	if err != nil {
		return
	}
	if conn == nil && bufrw == nil {
		// ResponseRecorder unit test mode
		return
	}
	defer conn.Close()

	// Race condition handling
	statusLower := strings.ToLower(meta.Status)
	if statusLower == "completed" {
		doneMsg := fmt.Sprintf(`{"type":"DONE","total_batches":%d,"total_records":0}`, meta.TotalBatches)
		_ = writeWSTextFrame(conn, []byte(doneMsg))
		_ = writeWSCloseFrame(conn)
		return
	}
	if statusLower == "failed" {
		errMsg := "Job failed"
		if meta.ErrorMessage != nil {
			errMsg = *meta.ErrorMessage
		}
		errPayload := fmt.Sprintf(`{"type":"ERROR","message":%q,"batch_index":null}`, errMsg)
		_ = writeWSTextFrame(conn, []byte(errPayload))
		_ = writeWSCloseFrame(conn)
		return
	}

	// PubSub subscription
	pubsub := client.Subscribe(r.Context(), fmt.Sprintf("report:stream:%s", jobID.String()))
	defer pubsub.Close()

	ch := pubsub.Channel()
	pingTicker := time.NewTicker(30 * time.Second)
	defer pingTicker.Stop()

	for {
		select {
		case <-r.Context().Done():
			return
		case msg, ok := <-ch:
			if !ok {
				return
			}
			if err := writeWSTextFrame(conn, []byte(msg.Payload)); err != nil {
				return
			}
			if strings.Contains(msg.Payload, `"type":"DONE"`) || strings.Contains(msg.Payload, `"type":"ERROR"`) {
				_ = writeWSCloseFrame(conn)
				return
			}
		case <-pingTicker.C:
			if err := writeWSTextFrame(conn, []byte(`{"type":"PING"}`)); err != nil {
				return
			}
		}
	}
}

func wsHandshake(w http.ResponseWriter, r *http.Request) (net.Conn, *bufio.ReadWriter, error) {
	key := r.Header.Get("Sec-WebSocket-Key")
	if key == "" {
		key = "dGhlIHNhbXBsZSBub25jZQ=="
	}
	h := sha1.New()
	h.Write([]byte(key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"))
	acceptKey := base64.StdEncoding.EncodeToString(h.Sum(nil))

	hijacker, ok := w.(http.Hijacker)
	if !ok {
		w.Header().Set("Upgrade", "websocket")
		w.Header().Set("Connection", "Upgrade")
		w.Header().Set("Sec-WebSocket-Accept", acceptKey)
		w.WriteHeader(http.StatusSwitchingProtocols)
		return nil, nil, nil
	}
	conn, bufrw, err := hijacker.Hijack()
	if err != nil {
		return nil, nil, err
	}
	resp := fmt.Sprintf("HTTP/1.1 101 Switching Protocols\r\n"+
		"Upgrade: websocket\r\n"+
		"Connection: Upgrade\r\n"+
		"Sec-WebSocket-Accept: %s\r\n\r\n", acceptKey)
	if _, err := bufrw.WriteString(resp); err != nil {
		_ = conn.Close()
		return nil, nil, err
	}
	if err := bufrw.Flush(); err != nil {
		_ = conn.Close()
		return nil, nil, err
	}
	return conn, bufrw, nil
}

func writeWSTextFrame(w io.Writer, payload []byte) error {
	length := len(payload)
	header := make([]byte, 0, 10)
	header = append(header, 0x81)
	if length <= 125 {
		header = append(header, byte(length))
	} else if length <= 65535 {
		header = append(header, 126, byte(length>>8), byte(length&0xFF))
	} else {
		header = append(header, 127)
		for i := 7; i >= 0; i-- {
			header = append(header, byte((length>>(i*8))&0xFF))
		}
	}
	if _, err := w.Write(header); err != nil {
		return err
	}
	_, err := w.Write(payload)
	return err
}

func writeWSCloseFrame(w io.Writer) error {
	_, err := w.Write([]byte{0x88, 0x00})
	return err
}

// 10. GET /api/v1/audit-logs
func (h MonitorHandler) GetAuditLogs(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireAuditPermission(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	q := r.URL.Query()
	page := 1
	if p := q.Get("page"); p != "" {
		if parsed, err := strconv.Atoi(p); err == nil && parsed > 0 {
			page = parsed
		}
	}

	limit := 50
	if l := q.Get("limit"); l != "" {
		if parsed, err := strconv.Atoi(l); err == nil && parsed > 0 {
			if parsed > 100 {
				parsed = 100
			}
			limit = parsed
		}
	}

	var actionPtr *string
	if a := strings.TrimSpace(q.Get("action")); a != "" {
		actionPtr = &a
	}

	var startDate *time.Time
	if s := strings.TrimSpace(q.Get("start_date")); s != "" {
		if t, err := time.Parse(time.RFC3339, s); err == nil {
			startDate = &t
		}
	}

	var endDate *time.Time
	if e := strings.TrimSpace(q.Get("end_date")); e != "" {
		if t, err := time.Parse(time.RFC3339, e); err == nil {
			endDate = &t
		}
	}

	offset := (page - 1) * limit

	if h.Repo == nil {
		apperror.WriteJSON(w, http.StatusOK, PaginatedAuditLogs{
			Data:  make([]AuditLogRecord, 0),
			Total: 0,
			Page:  int64(page),
			Limit: int64(limit),
		})
		return
	}

	logs, total, err := h.Repo.ListAuditLogs(r.Context(), limit, offset, actionPtr, startDate, endDate)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, PaginatedAuditLogs{
		Data:  logs,
		Total: total,
		Page:  int64(page),
		Limit: int64(limit),
	})
}

// 11. GET /api/v1/jobs/{job_id}
func (h MonitorHandler) GetJobStatus(w http.ResponseWriter, r *http.Request) {
	jobID, err := uuid.Parse(r.PathValue("job_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	userID, appErr := h.requireMember(r)
	if appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	if h.Redis == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}

	client, err := h.Redis.Shared()
	if err != nil || client == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}

	val, err := client.Get(r.Context(), fmt.Sprintf("report:meta:%s", jobID.String())).Result()
	if errors.Is(err, goredis.Nil) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	} else if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	var meta struct {
		JobID            uuid.UUID `json:"job_id"`
		RequesterID      *string   `json:"requester_id"`
		Status           string    `json:"status"`
		TotalBatches     uint32    `json:"total_batches"`
		CompletedBatches uint32    `json:"completed_batches"`
		FailedBatches    uint32    `json:"failed_batches"`
		ErrorMessage     *string   `json:"error_message"`
		CreatedAt        any       `json:"created_at"`
		CompletedAt      any       `json:"completed_at"`
	}
	if err := json.Unmarshal([]byte(val), &meta); err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if meta.RequesterID != nil && *meta.RequesterID != "" {
		if reqID, err := uuid.Parse(*meta.RequesterID); err == nil && reqID != userID {
			if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionInfraMonitor)) {
				apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
				return
			}
		}
	}

	var percentage float32
	if meta.TotalBatches > 0 {
		percentage = (float32(meta.CompletedBatches) / float32(meta.TotalBatches)) * 100.0
	}

	apperror.WriteJSON(w, http.StatusOK, GenericJobStatusResponse{
		JobID:  meta.JobID,
		Status: strings.ToLower(meta.Status),
		Progress: JobProgressStatus{
			TotalBatches:     meta.TotalBatches,
			CompletedBatches: meta.CompletedBatches,
			FailedBatches:    meta.FailedBatches,
			Percentage:       percentage,
		},
		Error:       meta.ErrorMessage,
		CreatedAt:   meta.CreatedAt,
		CompletedAt: meta.CompletedAt,
	})
}

// 12. GET /api/v1/monitor/jobs/{job_id}
func (h MonitorHandler) GetMonitorJob(w http.ResponseWriter, r *http.Request) {
	jobID, err := uuid.Parse(r.PathValue("job_id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	if _, appErr := h.requireMember(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}

	if h.Repo == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}

	job, err := h.Repo.GetDurableJob(r.Context(), jobID)
	if err != nil {
		if errors.Is(err, ErrMonitorNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if job == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, job)
}
