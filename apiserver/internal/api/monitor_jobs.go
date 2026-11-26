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

type monitorJobsQuery struct {
	limit         int
	status        string
	taskType      *string
	state         *string
	search        *string
	legacyCursor  string
	durableCursor *string
	jobID         *uuid.UUID
}

var validMonitorJobStatuses = map[string]bool{
	"pending": true, "processing": true, "completed": true, "failed": true,
}

var validMonitorJobStates = map[string]bool{
	"dispatching": true, "queued": true, "processing": true, "completed": true,
	"failed": true, "retryable": true, "dead_letter": true, "lost": true,
}

func optionalRawParam(raw string) *string {
	if raw == "" {
		return nil
	}
	return &raw
}

func parseMonitorJobsQuery(r *http.Request) (monitorJobsQuery, bool) {
	q := r.URL.Query()
	status := strings.TrimSpace(strings.ToLower(q.Get("status")))
	if status != "" && !validMonitorJobStatuses[status] {
		return monitorJobsQuery{}, false
	}
	search := strings.TrimSpace(q.Get("search"))
	if len([]rune(search)) > 64 {
		return monitorJobsQuery{}, false
	}
	state := strings.TrimSpace(strings.ToLower(q.Get("state")))
	if state != "" && !validMonitorJobStates[state] {
		return monitorJobsQuery{}, false
	}
	var jobIDPtr *uuid.UUID
	if id, err := uuid.Parse(q.Get("job_id")); err == nil {
		jobIDPtr = &id
	}
	return monitorJobsQuery{
		limit:         parseMonitorLimit(q.Get("limit"), 20, 100),
		status:        status,
		taskType:      optionalTrimmedParam(q.Get("task_type")),
		state:         optionalTrimmedParam(state),
		search:        optionalTrimmedParam(search),
		legacyCursor:  q.Get("cursor"),
		durableCursor: optionalRawParam(q.Get("durable_cursor")),
		jobID:         jobIDPtr,
	}, true
}

func legacyRedisMaxScore(cursor string) string {
	if parsed, err := strconv.ParseInt(cursor, 10, 64); err == nil && parsed > 0 {
		return fmt.Sprintf("(%d", parsed)
	}
	return "+inf"
}

func (h *MonitorHandler) listLegacyRedisJobs(r *http.Request, query monitorJobsQuery) ([]MonitorJob, *int64) {
	jobs := make([]MonitorJob, 0)
	if h.Redis == nil || !h.Redis.IsAvailable() {
		return jobs, nil
	}
	members, scores, err := h.Redis.ZRevRangeByScore(r.Context(), "report:job_list", "-inf", legacyRedisMaxScore(query.legacyCursor), int64(query.limit))
	if err != nil {
		return jobs, nil
	}
	for _, jid := range members {
		metaStr, found, err := h.Redis.GetString(r.Context(), fmt.Sprintf("report:meta:%s", jid))
		if err != nil || !found {
			continue
		}
		if job, ok := decodeMonitorJob(metaStr, time.Now().UTC()); ok && matchesMonitorJobFilter(job, query.status, query.taskType) {
			jobs = append(jobs, job)
		}
	}
	if len(scores) == query.limit {
		lastScore := int64(scores[len(scores)-1])
		return jobs, &lastScore
	}
	return jobs, nil
}

func matchesMonitorJobFilter(job MonitorJob, status string, taskType *string) bool {
	if status != "" && !strings.EqualFold(job.Status, status) {
		return false
	}
	if taskType != nil && (job.TaskType == nil || !strings.EqualFold(*job.TaskType, *taskType)) {
		return false
	}
	return true
}

// 2. GET /api/v1/monitor/jobs
func (h *MonitorHandler) GetMonitorJobs(w http.ResponseWriter, r *http.Request) {
	if _, appErr := h.requireInfraMonitor(r); appErr != nil {
		apperror.WriteError(w, r, appErr)
		return
	}
	query, ok := parseMonitorJobsQuery(r)
	if !ok {
		writeValidation(w, r)
		return
	}

	jobs, nextCursor := h.listLegacyRedisJobs(r, query)
	durableJobs := h.fetchDurableJobsSection(r.Context(), ListDurableJobsParams{
		JobID:    query.jobID,
		State:    query.state,
		TaskType: query.taskType,
		Search:   query.search,
		Cursor:   query.durableCursor,
		Limit:    query.limit,
	})

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

func isTraceIDRune(c rune) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '-' || c == '_' || c == '.'
}

func isValidTraceID(traceID string) bool {
	if len(traceID) > 128 {
		return false
	}
	for _, c := range traceID {
		if !isTraceIDRune(c) {
			return false
		}
	}
	return true
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
	if (reqIDStr == "") == (traceIDStr == "") {
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
		if !isValidTraceID(traceIDStr) {
			writeValidation(w, r)
			return
		}
		parsedTraceID = &traceIDStr
	}

	if h.Repo == nil {
		apperror.WriteJSON(w, http.StatusOK, MonitorCorrelationsResponse{Jobs: make([]MonitorInfraJobRecord, 0)})
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

func (h *MonitorHandler) canViewJobStatus(r *http.Request, userID uuid.UUID, requesterID *string) bool {
	if requesterID == nil || *requesterID == "" {
		return true
	}
	reqID, err := uuid.Parse(*requesterID)
	if err != nil || reqID == userID {
		return true
	}
	return h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionInfraMonitor))
}

type legacyJobStatusMeta struct {
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

	var meta legacyJobStatusMeta
	if err := json.Unmarshal([]byte(val), &meta); err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if !h.canViewJobStatus(r, userID, meta.RequesterID) {
		writeForbidden(w, r)
		return
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
