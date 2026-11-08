package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

type MonitorRepository struct {
	SystemDB DBTX
	DB       DBTX
}

func (r MonitorRepository) ListAlerts(ctx context.Context, status, severity *string, limit int, cursor *string) ([]api.MonitorAlertResponse, int64, *string, error) {
	if r.SystemDB == nil {
		return make([]api.MonitorAlertResponse, 0), 0, nil, nil
	}

	whereClauses := []string{"1=1"}
	args := make([]any, 0)
	argIdx := 1

	if status != nil && *status != "" {
		whereClauses = append(whereClauses, fmt.Sprintf("status = $%d", argIdx))
		args = append(args, *status)
		argIdx++
	}
	if severity != nil && *severity != "" {
		whereClauses = append(whereClauses, fmt.Sprintf("severity = $%d", argIdx))
		args = append(args, *severity)
		argIdx++
	}

	countSQL := fmt.Sprintf("SELECT COUNT(*) FROM monitor_alerts WHERE %s", strings.Join(whereClauses, " AND "))
	var total int64
	if err := r.SystemDB.QueryRow(ctx, countSQL, args...).Scan(&total); err != nil {
		return nil, 0, nil, err
	}

	if cursor != nil && *cursor != "" {
		parts := strings.SplitN(*cursor, ":", 2)
		if len(parts) == 2 {
			if micros, err := strconv.ParseInt(parts[0], 10, 64); err == nil {
				if curID, err := uuid.Parse(parts[1]); err == nil {
					t := time.UnixMicro(micros)
					whereClauses = append(whereClauses, fmt.Sprintf("(last_seen_at < $%d OR (last_seen_at = $%d AND id < $%d))", argIdx, argIdx, argIdx+1))
					args = append(args, t, curID)
					argIdx += 2
				}
			}
		}
	}

	querySQL := fmt.Sprintf(`
		SELECT id, alert_type, severity, status, title, summary, source,
		       first_seen_at, last_seen_at, occurrence_count, resolved_at
		FROM monitor_alerts
		WHERE %s
		ORDER BY last_seen_at DESC, id DESC
		LIMIT $%d
	`, strings.Join(whereClauses, " AND "), argIdx)
	args = append(args, limit)

	rows, err := r.SystemDB.Query(ctx, querySQL, args...)
	if err != nil {
		return nil, 0, nil, err
	}
	defer rows.Close()

	items := make([]api.MonitorAlertResponse, 0)
	for rows.Next() {
		var a api.MonitorAlertResponse
		if err := rows.Scan(
			&a.ID, &a.AlertType, &a.Severity, &a.Status, &a.Title,
			&a.Summary, &a.Source, &a.FirstSeenAt, &a.LastSeenAt,
			&a.OccurrenceCount, &a.ResolvedAt,
		); err != nil {
			return nil, 0, nil, err
		}
		items = append(items, a)
	}
	if err := rows.Err(); err != nil {
		return nil, 0, nil, err
	}

	var nextCursor *string
	if len(items) == limit {
		c := fmt.Sprintf("%d:%s", items[len(items)-1].LastSeenAt.UnixMicro(), items[len(items)-1].ID.String())
		nextCursor = &c
	}

	return items, total, nextCursor, nil
}

func (r MonitorRepository) GetAlert(ctx context.Context, alertID uuid.UUID) (*api.MonitorAlertResponse, []api.MonitorAlertEventResponse, error) {
	if r.SystemDB == nil {
		return nil, nil, apperror.NotFound(apperror.KeyNotFound)
	}

	var a api.MonitorAlertResponse
	err := r.SystemDB.QueryRow(ctx, `
		SELECT id, alert_type, severity, status, title, summary, source,
		       first_seen_at, last_seen_at, occurrence_count, resolved_at
		FROM monitor_alerts
		WHERE id = $1
	`, alertID).Scan(
		&a.ID, &a.AlertType, &a.Severity, &a.Status, &a.Title,
		&a.Summary, &a.Source, &a.FirstSeenAt, &a.LastSeenAt,
		&a.OccurrenceCount, &a.ResolvedAt,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil, apperror.NotFound(apperror.KeyNotFound)
		}
		return nil, nil, err
	}

	events := make([]api.MonitorAlertEventResponse, 0)
	rows, err := r.SystemDB.Query(ctx, `
		SELECT id, event_type, severity, summary, occurred_at
		FROM monitor_alert_events
		WHERE alert_id = $1
		ORDER BY occurred_at ASC, id ASC
		LIMIT 200
	`, alertID)
	if err == nil {
		defer rows.Close()
		for rows.Next() {
			var ev api.MonitorAlertEventResponse
			if err := rows.Scan(&ev.ID, &ev.EventType, &ev.Severity, &ev.Summary, &ev.OccurredAt); err == nil {
				events = append(events, ev)
			}
		}
	}

	return &a, events, nil
}

func (r MonitorRepository) MutateAlert(ctx context.Context, alertID uuid.UUID, action string, reason string, actorID uuid.UUID) (*api.MonitorAlertResponse, error) {
	if r.SystemDB == nil {
		return nil, apperror.NotFound(apperror.KeyNotFound)
	}

	var current api.MonitorAlertResponse
	err := r.SystemDB.QueryRow(ctx, `
		SELECT id, alert_type, severity, status, title, summary, source,
		       first_seen_at, last_seen_at, occurrence_count, resolved_at
		FROM monitor_alerts
		WHERE id = $1
	`, alertID).Scan(
		&current.ID, &current.AlertType, &current.Severity, &current.Status, &current.Title,
		&current.Summary, &current.Source, &current.FirstSeenAt, &current.LastSeenAt,
		&current.OccurrenceCount, &current.ResolvedAt,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, apperror.NotFound(apperror.KeyNotFound)
		}
		return nil, err
	}

	now := time.Now().UTC()

	switch action {
	case "acknowledge":
		var alreadyRecorded bool
		_ = r.SystemDB.QueryRow(ctx, `
			SELECT EXISTS (
				SELECT 1 FROM monitor_alert_events
				WHERE alert_id = $1 AND event_type = 'acknowledged'
			)
		`, alertID).Scan(&alreadyRecorded)

		if current.Status == "active" && alreadyRecorded {
			return &current, nil
		}
		if current.Status != "active" {
			return nil, apperror.Conflict(apperror.KeyConflict)
		}

		eventID := uuid.New()
		_, err = r.SystemDB.Exec(ctx, `
			INSERT INTO monitor_alert_events (id, alert_id, event_type, severity, summary, occurred_at, created_at)
			VALUES ($1, $2, 'acknowledged', $3, $4, $5, $5)
		`, eventID, alertID, current.Severity, reason, now)
		if err != nil {
			return nil, err
		}
		return &current, nil

	case "resolve":
		if current.Status == "resolved" {
			return &current, nil
		}
		if current.Status != "active" {
			return nil, apperror.Conflict(apperror.KeyConflict)
		}

		var updated api.MonitorAlertResponse
		err = r.SystemDB.QueryRow(ctx, `
			UPDATE monitor_alerts
			SET status = 'resolved', resolved_at = $2, updated_at = $2
			WHERE id = $1
			RETURNING id, alert_type, severity, status, title, summary, source,
			          first_seen_at, last_seen_at, occurrence_count, resolved_at
		`, alertID, now).Scan(
			&updated.ID, &updated.AlertType, &updated.Severity, &updated.Status, &updated.Title,
			&updated.Summary, &updated.Source, &updated.FirstSeenAt, &updated.LastSeenAt,
			&updated.OccurrenceCount, &updated.ResolvedAt,
		)
		if err != nil {
			return nil, err
		}

		eventID := uuid.New()
		_, err = r.SystemDB.Exec(ctx, `
			INSERT INTO monitor_alert_events (id, alert_id, event_type, severity, summary, occurred_at, created_at)
			VALUES ($1, $2, 'resolved', $3, $4, $5, $5)
		`, eventID, alertID, current.Severity, reason, now)
		if err != nil {
			return nil, err
		}
		return &updated, nil

	default:
		return nil, errors.New("unknown alert action")
	}
}

func (r MonitorRepository) ListDurableJobs(ctx context.Context, params api.ListDurableJobsParams) ([]api.MonitorInfraJobRecord, *string, error) {
	if r.SystemDB == nil {
		return make([]api.MonitorInfraJobRecord, 0), nil, nil
	}

	whereClauses := []string{"1=1"}
	args := make([]any, 0)
	argIdx := 1

	if params.JobID != nil {
		whereClauses = append(whereClauses, fmt.Sprintf("id = $%d", argIdx))
		args = append(args, *params.JobID)
		argIdx++
	}
	if params.State != nil && *params.State != "" {
		whereClauses = append(whereClauses, fmt.Sprintf("state = $%d", argIdx))
		args = append(args, *params.State)
		argIdx++
	}
	if params.TaskType != nil && *params.TaskType != "" {
		whereClauses = append(whereClauses, fmt.Sprintf("task_type = $%d", argIdx))
		args = append(args, *params.TaskType)
		argIdx++
	}
	if params.Search != nil && *params.Search != "" {
		whereClauses = append(whereClauses, fmt.Sprintf("(task_type = $%d OR error_code = $%d)", argIdx, argIdx))
		args = append(args, *params.Search)
		argIdx++
	}

	if params.Cursor != nil && *params.Cursor != "" {
		parts := strings.SplitN(*params.Cursor, ":", 2)
		if len(parts) == 2 {
			if micros, err := strconv.ParseInt(parts[0], 10, 64); err == nil {
				if curID, err := uuid.Parse(parts[1]); err == nil {
					t := time.UnixMicro(micros)
					whereClauses = append(whereClauses, fmt.Sprintf("(created_at < $%d OR (created_at = $%d AND id < $%d))", argIdx, argIdx, argIdx+1))
					args = append(args, t, curID)
					argIdx += 2
				}
			}
		}
	}

	limit := params.Limit
	if limit <= 0 {
		limit = 10
	}
	querySQL := fmt.Sprintf(`
		SELECT id, parent_job_id, task_type, priority, state,
		       attempt_count, lease_owner, request_id, trace_id,
		       created_at, queued_at, started_at, finished_at, updated_at,
		       error_code, error_summary
		FROM infra_job_records
		WHERE %s
		ORDER BY created_at DESC, id DESC
		LIMIT $%d
	`, strings.Join(whereClauses, " AND "), argIdx)
	args = append(args, limit)

	rows, err := r.SystemDB.Query(ctx, querySQL, args...)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()

	jobs := make([]api.MonitorInfraJobRecord, 0)
	jobIDs := make([]uuid.UUID, 0)
	for rows.Next() {
		var j api.MonitorInfraJobRecord
		if err := rows.Scan(
			&j.JobID, &j.ParentJobID, &j.TaskType, &j.Priority, &j.State,
			&j.AttemptCount, &j.Worker, &j.RequestID, &j.TraceID,
			&j.CreatedAt, &j.QueuedAt, &j.StartedAt, &j.FinishedAt, &j.UpdatedAt,
			&j.ErrorCode, &j.ErrorSummary,
		); err != nil {
			return nil, nil, err
		}
		j.Events = make([]api.MonitorInfraJobEvent, 0)
		jobs = append(jobs, j)
		jobIDs = append(jobIDs, j.JobID)
	}
	if err := rows.Err(); err != nil {
		return nil, nil, err
	}

	// Fetch events
	if len(jobIDs) > 0 {
		evRows, err := r.SystemDB.Query(ctx, `
			SELECT id, job_id, event_type, from_state, to_state, attempt_count,
			       worker_id, request_id, trace_id, reason_code, reason_summary, created_at
			FROM infra_job_events
			WHERE job_id = ANY($1)
			ORDER BY created_at ASC, id ASC
			LIMIT 200
		`, jobIDs)
		if err == nil {
			defer evRows.Close()
			eventsByJob := make(map[uuid.UUID][]api.MonitorInfraJobEvent)
			for evRows.Next() {
				var ev api.MonitorInfraJobEvent
				if err := evRows.Scan(
					&ev.EventID, &ev.JobID, &ev.EventType, &ev.FromState, &ev.ToState, &ev.AttemptCount,
					&ev.WorkerID, &ev.RequestID, &ev.TraceID, &ev.ErrorCode, &ev.ErrorSummary, &ev.CreatedAt,
				); err == nil {
					eventsByJob[ev.JobID] = append(eventsByJob[ev.JobID], ev)
				}
			}
			for i := range jobs {
				if evs, ok := eventsByJob[jobs[i].JobID]; ok {
					jobs[i].Events = evs
				}
			}
		}
	}

	var nextTok *string
	if len(jobs) == limit {
		t := fmt.Sprintf("%d:%s", jobs[len(jobs)-1].CreatedAt.UnixMicro(), jobs[len(jobs)-1].JobID.String())
		nextTok = &t
	}

	return jobs, nextTok, nil
}

func (r MonitorRepository) ListCorrelations(ctx context.Context, requestID *uuid.UUID, traceID *string, limit int) ([]api.MonitorInfraJobRecord, bool, error) {
	if r.SystemDB == nil {
		return make([]api.MonitorInfraJobRecord, 0), false, nil
	}

	var querySQL string
	var args []any
	if requestID != nil {
		querySQL = `
			SELECT id, parent_job_id, task_type, priority, state,
			       attempt_count, lease_owner, request_id, trace_id,
			       created_at, queued_at, started_at, finished_at, updated_at,
			       error_code, error_summary
			FROM infra_job_records
			WHERE request_id = $1
			ORDER BY created_at DESC, id DESC
			LIMIT $2
		`
		args = []any{*requestID, limit}
	} else if traceID != nil {
		querySQL = `
			SELECT id, parent_job_id, task_type, priority, state,
			       attempt_count, lease_owner, request_id, trace_id,
			       created_at, queued_at, started_at, finished_at, updated_at,
			       error_code, error_summary
			FROM infra_job_records
			WHERE trace_id = $1
			ORDER BY created_at DESC, id DESC
			LIMIT $2
		`
		args = []any{*traceID, limit}
	} else {
		return make([]api.MonitorInfraJobRecord, 0), false, nil
	}

	rows, err := r.SystemDB.Query(ctx, querySQL, args...)
	if err != nil {
		return nil, false, err
	}
	defer rows.Close()

	jobs := make([]api.MonitorInfraJobRecord, 0)
	jobIDs := make([]uuid.UUID, 0)
	for rows.Next() {
		var j api.MonitorInfraJobRecord
		if err := rows.Scan(
			&j.JobID, &j.ParentJobID, &j.TaskType, &j.Priority, &j.State,
			&j.AttemptCount, &j.Worker, &j.RequestID, &j.TraceID,
			&j.CreatedAt, &j.QueuedAt, &j.StartedAt, &j.FinishedAt, &j.UpdatedAt,
			&j.ErrorCode, &j.ErrorSummary,
		); err != nil {
			return nil, false, err
		}
		j.Events = make([]api.MonitorInfraJobEvent, 0)
		jobs = append(jobs, j)
		jobIDs = append(jobIDs, j.JobID)
	}
	if err := rows.Err(); err != nil {
		return nil, false, err
	}

	truncated := false
	if len(jobIDs) > 0 {
		evRows, err := r.SystemDB.Query(ctx, `
			SELECT id, job_id, event_type, from_state, to_state, attempt_count,
			       worker_id, request_id, trace_id, reason_code, reason_summary, created_at
			FROM infra_job_events
			WHERE job_id = ANY($1)
			ORDER BY created_at ASC, id ASC
			LIMIT 200
		`, jobIDs)
		if err == nil {
			defer evRows.Close()
			eventsByJob := make(map[uuid.UUID][]api.MonitorInfraJobEvent)
			totalEvents := 0
			for evRows.Next() {
				totalEvents++
				var ev api.MonitorInfraJobEvent
				if err := evRows.Scan(
					&ev.EventID, &ev.JobID, &ev.EventType, &ev.FromState, &ev.ToState, &ev.AttemptCount,
					&ev.WorkerID, &ev.RequestID, &ev.TraceID, &ev.ErrorCode, &ev.ErrorSummary, &ev.CreatedAt,
				); err == nil {
					eventsByJob[ev.JobID] = append(eventsByJob[ev.JobID], ev)
				}
			}
			truncated = totalEvents >= 200
			for i := range jobs {
				if evs, ok := eventsByJob[jobs[i].JobID]; ok {
					jobs[i].Events = evs
				}
			}
		}
	}

	return jobs, truncated, nil
}

func (r MonitorRepository) GetDurableJob(ctx context.Context, jobID uuid.UUID) (*api.MonitorInfraJobRecord, error) {
	if r.SystemDB == nil {
		return nil, apperror.NotFound(apperror.KeyJobNotFound)
	}

	var j api.MonitorInfraJobRecord
	err := r.SystemDB.QueryRow(ctx, `
		SELECT id, parent_job_id, task_type, priority, state,
		       attempt_count, lease_owner, request_id, trace_id,
		       created_at, queued_at, started_at, finished_at, updated_at,
		       error_code, error_summary
		FROM infra_job_records
		WHERE id = $1
	`, jobID).Scan(
		&j.JobID, &j.ParentJobID, &j.TaskType, &j.Priority, &j.State,
		&j.AttemptCount, &j.Worker, &j.RequestID, &j.TraceID,
		&j.CreatedAt, &j.QueuedAt, &j.StartedAt, &j.FinishedAt, &j.UpdatedAt,
		&j.ErrorCode, &j.ErrorSummary,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, apperror.NotFound(apperror.KeyJobNotFound)
		}
		return nil, err
	}

	j.Events = make([]api.MonitorInfraJobEvent, 0)
	rows, err := r.SystemDB.Query(ctx, `
		SELECT id, job_id, event_type, from_state, to_state, attempt_count,
		       worker_id, request_id, trace_id, reason_code, reason_summary, created_at
		FROM infra_job_events
		WHERE job_id = $1
		ORDER BY created_at ASC, id ASC
		LIMIT 200
	`, jobID)
	if err == nil {
		defer rows.Close()
		for rows.Next() {
			var ev api.MonitorInfraJobEvent
			if err := rows.Scan(
				&ev.EventID, &ev.JobID, &ev.EventType, &ev.FromState, &ev.ToState, &ev.AttemptCount,
				&ev.WorkerID, &ev.RequestID, &ev.TraceID, &ev.ErrorCode, &ev.ErrorSummary, &ev.CreatedAt,
			); err == nil {
				j.Events = append(j.Events, ev)
			}
		}
	}

	return &j, nil
}

func (r MonitorRepository) ListAuditLogs(ctx context.Context, limit, offset int, action *string, startDate, endDate *time.Time) ([]api.AuditLogRecord, int64, error) {
	if r.DB == nil {
		return make([]api.AuditLogRecord, 0), 0, nil
	}
	db := r.DB

	whereClauses := []string{"1=1"}
	args := make([]any, 0)
	argIdx := 1

	if action != nil && *action != "" {
		whereClauses = append(whereClauses, fmt.Sprintf("action = $%d", argIdx))
		args = append(args, *action)
		argIdx++
	}
	if startDate != nil {
		whereClauses = append(whereClauses, fmt.Sprintf("created_at >= $%d", argIdx))
		args = append(args, *startDate)
		argIdx++
	}
	if endDate != nil {
		whereClauses = append(whereClauses, fmt.Sprintf("created_at <= $%d", argIdx))
		args = append(args, *endDate)
		argIdx++
	}

	countSQL := fmt.Sprintf("SELECT COUNT(*) FROM audit_logs WHERE %s", strings.Join(whereClauses, " AND "))
	var total int64
	if err := db.QueryRow(ctx, countSQL, args...).Scan(&total); err != nil {
		return nil, 0, err
	}

	querySQL := fmt.Sprintf(`
		SELECT id, actor_id, actor_type, actor_name, actor_email,
		       target_user_id, target_resource_id, action, details,
		       ip_address, user_agent, created_at
		FROM audit_logs
		WHERE %s
		ORDER BY created_at DESC
		LIMIT $%d OFFSET $%d
	`, strings.Join(whereClauses, " AND "), argIdx, argIdx+1)
	args = append(args, limit, offset)

	rows, err := db.Query(ctx, querySQL, args...)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	logs := make([]api.AuditLogRecord, 0)
	for rows.Next() {
		var l api.AuditLogRecord
		var rawDetails []byte
		if err := rows.Scan(
			&l.ID, &l.ActorID, &l.ActorType, &l.ActorName, &l.ActorEmail,
			&l.TargetUserID, &l.TargetResourceID, &l.Action, &rawDetails,
			&l.IPAddress, &l.UserAgent, &l.CreatedAt,
		); err != nil {
			return nil, 0, err
		}
		if len(rawDetails) > 0 {
			l.Details = json.RawMessage(rawDetails)
		} else {
			l.Details = json.RawMessage(`{}`)
		}
		logs = append(logs, l)
	}
	if err := rows.Err(); err != nil {
		return nil, 0, err
	}

	return logs, total, nil
}
