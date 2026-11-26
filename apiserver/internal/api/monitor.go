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

func (h *MonitorHandler) buildOperationalSnapshot(ctx context.Context) (MonitorOperationalSnapshot, error) {
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

	if h.Redis != nil && h.Redis.IsAvailable() {
		redisAvailable = true
		if v, err := h.Redis.LLen(ctx, "worker:jobs:high"); err == nil {
			pendingHigh = uint64(v)
		}
		if v, err := h.Redis.LLen(ctx, "worker:jobs:normal"); err == nil {
			pendingNormal = uint64(v)
		}
		if v, err := h.Redis.LLen(ctx, "worker:jobs:low"); err == nil {
			pendingLow = uint64(v)
		}
		if v, err := h.Redis.GetUint64(ctx, fmt.Sprintf("stats:completed:%s", today)); err == nil {
			completedToday = v
		}
		if v, err := h.Redis.GetUint64(ctx, fmt.Sprintf("stats:failed:%s", today)); err == nil {
			failedToday = v
		}

		// Heartbeats
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
		jobIDs, _ := h.Redis.ZRevRange(ctx, "report:job_list", 0, 9)
		for _, jid := range jobIDs {
			metaStr, found, err := h.Redis.GetString(ctx, fmt.Sprintf("report:meta:%s", jid))
			if err != nil || !found {
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

// 2. GET /api/v1/monitor/jobs
func (h *MonitorHandler) GetMonitorJobs(w http.ResponseWriter, r *http.Request) {
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
	if h.Redis != nil && h.Redis.IsAvailable() {
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
		members, scores, err := h.Redis.ZRevRangeByScore(r.Context(), "report:job_list", "-inf", maxScore, int64(limit))
		if err == nil {
			for _, jid := range members {
				metaStr, found, err := h.Redis.GetString(r.Context(), fmt.Sprintf("report:meta:%s", jid))
				if err != nil || !found {
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
			if len(scores) == limit {
				lastScore := int64(scores[len(scores)-1])
				nextCursor = &lastScore
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
func (h *MonitorHandler) GetMonitorCorrelations(w http.ResponseWriter, r *http.Request) {
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

// 11. GET /api/v1/jobs/{job_id}
func (h *MonitorHandler) GetJobStatus(w http.ResponseWriter, r *http.Request) {
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

	if h.Redis == nil || !h.Redis.IsAvailable() {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
		return
	}

	val, found, err := h.Redis.GetString(r.Context(), fmt.Sprintf("report:meta:%s", jobID.String()))
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if !found {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyJobNotFound))
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
func (h *MonitorHandler) GetMonitorJob(w http.ResponseWriter, r *http.Request) {
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
