package api

import (
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
)

func decodeMonitorJob(metaStr string, fallbackTime time.Time) (MonitorJob, bool) {
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
	if json.Unmarshal([]byte(metaStr), &m) != nil {
		return MonitorJob{}, false
	}
	createdAt := fallbackTime
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
	return MonitorJob{
		JobID:            m.JobID,
		Status:           m.Status,
		TotalBatches:     m.TotalBatches,
		CompletedBatches: m.CompletedBatches,
		FailedBatches:    m.FailedBatches,
		Error:            m.ErrorMessage,
		CreatedAt:        createdAt,
		CompletedAt:      completedAt,
		TaskType:         m.TaskType,
	}, true
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
		writeValidation(w, r)
		return
	}

	search := strings.TrimSpace(q.Get("search"))
	if len([]rune(search)) > 64 {
		writeValidation(w, r)
		return
	}

	state := strings.TrimSpace(strings.ToLower(q.Get("state")))
	validStates := map[string]bool{
		"dispatching": true, "queued": true, "processing": true, "completed": true,
		"failed": true, "retryable": true, "dead_letter": true, "lost": true,
	}
	if state != "" && !validStates[state] {
		writeValidation(w, r)
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
				job, ok := decodeMonitorJob(metaStr, time.Now().UTC())
				if !ok {
					continue
				}
				if status != "" && !strings.EqualFold(job.Status, status) {
					continue
				}
				if taskTypePtr != nil && (job.TaskType == nil || !strings.EqualFold(*job.TaskType, *taskTypePtr)) {
					continue
				}
				jobs = append(jobs, job)
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
		writeValidation(w, r)
		return
	}

	var parsedReqID *uuid.UUID
	if reqIDStr != "" {
		id, err := uuid.Parse(reqIDStr)
		if err != nil {
			writeValidation(w, r)
			return
		}
		parsedReqID = &id
	}

	var parsedTraceID *string
	if traceIDStr != "" {
		if len(traceIDStr) > 128 {
			writeValidation(w, r)
			return
		}
		for _, c := range traceIDStr {
			if !((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '-' || c == '_' || c == '.') {
				writeValidation(w, r)
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

// 11. GET /api/v1/jobs/{job_id}
func (h *MonitorHandler) GetJobStatus(w http.ResponseWriter, r *http.Request) {
	jobID, err := uuid.Parse(r.PathValue("job_id"))
	if err != nil {
		writeValidation(w, r)
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
				writeForbidden(w, r)
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
		writeValidation(w, r)
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
