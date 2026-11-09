package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"sync"
	"time"

	"github.com/google/uuid"
	goredis "github.com/redis/go-redis/v9"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

type ReportRepository struct {
	DB DBTX
}

func (r ReportRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("report repository: database is not configured")
	}
	return r.DB, nil
}

func (r ReportRepository) CreateExportJob(ctx context.Context, job api.ReportJob, req api.ExportReportRequest) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	payload, err := json.Marshal(map[string]any{
		"job_id":       job.JobID,
		"request": map[string]any{
			"report_type":  req.ReportType,
			"start_date":   req.StartDate,
			"end_date":     req.EndDate,
			"employee_ids": req.EmployeeIDs,
		},
		"requester_id": job.RequesterID,
	})
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO outbox_jobs (id, job_type, payload, status, created_at, updated_at) VALUES ($1, 'AttendanceReport', $2, 'pending', NOW(), NOW())`, job.JobID, payload)
	return err
}
func (r ReportRepository) ResolveBehalfEmployeeIDs(ctx context.Context, userID uuid.UUID) ([]uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT DISTINCT e.id
FROM behalf_assignments ba
JOIN employments emp ON (
    (ba.target_employee_id IS NOT NULL AND emp.employee_id = ba.target_employee_id)
    OR (ba.target_department_id IS NOT NULL AND emp.department_id = ba.target_department_id)
)
JOIN employees e ON e.id = emp.employee_id
WHERE ba.assignee_user_id = $1
  AND ba.assignment_type = 'overtime_claim'`, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	ids := make([]uuid.UUID, 0)
	for rows.Next() {
		var id uuid.UUID
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		ids = append(ids, id)
	}
	return ids, rows.Err()
}

func (r ReportRepository) GetAttendanceReport(ctx context.Context, params api.AttendanceReportFilter) (api.PaginatedReportResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	if params.EmploymentID != nil {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND deleted_at IS NULL)`, *params.EmploymentID).Scan(&exists); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		if !exists {
			return api.PaginatedReportResponse{}, api.ErrReportNotFound
		}
	}

	offset := (params.Page - 1) * params.Limit

	if params.ReportType == "daily" {
		whereClause := `emp.deleted_at IS NULL AND e.deleted_at IS NULL AND ads.date >= $1::date AND ads.date <= $2::date`
		filterClause := ""
		args := []any{params.StartDate, params.EndDate}
		argIndex := 3

		if len(params.EmployeeIDs) > 0 {
			filterClause += fmt.Sprintf(" AND e.id = ANY($%d)", argIndex)
			args = append(args, params.EmployeeIDs)
			argIndex++
		}
		if params.DepartmentID != nil {
			filterClause += fmt.Sprintf(" AND emp.department_id = $%d", argIndex)
			args = append(args, *params.DepartmentID)
			argIndex++
		}
		if params.EmployerID != nil {
			filterClause += fmt.Sprintf(" AND emp.employer_id = $%d", argIndex)
			args = append(args, *params.EmployerID)
			argIndex++
		}
		if params.EmploymentID != nil {
			filterClause += fmt.Sprintf(" AND emp.id = $%d", argIndex)
			args = append(args, *params.EmploymentID)
			argIndex++
		}

		var total int64
		countQuery := fmt.Sprintf(`SELECT count(*) FROM employments emp JOIN employees e ON e.id = emp.employee_id JOIN attendance_daily_summaries ads ON ads.employment_id = emp.id WHERE %s%s`, whereClause, filterClause)
		if err := db.QueryRow(ctx, countQuery, args...).Scan(&total); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		dataQuery := fmt.Sprintf(`SELECT ads.date::text, emp.id AS employment_id, e.id AS employee_id, e.full_name, e.employee_id_number,
       st.name AS shift_name, ws.scheduled_start::text, ws.scheduled_end::text,
       te.punch_in_time::text, te.punch_out_time::text,
       COALESCE(ads.actual_work_minutes, 0),
       COALESCE(ads.standard_minutes, 0),
       COALESCE(ads.approved_ot_minutes, 0),
       COALESCE(ads.ot_status::text, 'none')
FROM employments emp
JOIN employees e ON e.id = emp.employee_id
JOIN attendance_daily_summaries ads ON ads.employment_id = emp.id
LEFT JOIN work_segments ws ON ws.employment_id = emp.id AND ws.date = ads.date
LEFT JOIN shift_templates st ON st.id = ws.template_id
LEFT JOIN time_entries te ON te.employment_id = emp.id AND te.punch_in_time::date = ads.date
WHERE %s%s
ORDER BY ads.date ASC, e.full_name ASC
LIMIT $%d OFFSET $%d`, whereClause, filterClause, argIndex, argIndex+1)

		args = append(args, params.Limit, offset)
		rows, err := db.Query(ctx, dataQuery, args...)
		if err != nil {
			return api.PaginatedReportResponse{}, err
		}
		defer rows.Close()

		results := make([]api.DailyAttendanceRow, 0)
		for rows.Next() {
			var row api.DailyAttendanceRow
			var actMins, stdMins, otMins int
			var shiftName, schedStart, schedEnd, punchIn, punchOut *string
			var otStatus string
			if err := rows.Scan(&row.Date, &row.EmploymentID, &row.EmployeeID, &row.EmployeeName, &row.EmployeeIDNumber,
				&shiftName, &schedStart, &schedEnd, &punchIn, &punchOut,
				&actMins, &stdMins, &otMins, &otStatus); err != nil {
				return api.PaginatedReportResponse{}, err
			}
			row.ShiftName = shiftName
			row.ScheduledStart = schedStart
			row.ScheduledEnd = schedEnd
			row.ActualPunchIn = punchIn
			row.ActualPunchOut = punchOut
			row.ActualWorkMinutes = actMins
			row.ComputedStdHours = math.Round((float64(stdMins)/60.0)*100) / 100
			row.ComputedOTHours = math.Round((float64(otMins)/60.0)*100) / 100
			row.StdWorkUnits = math.Round((float64(stdMins)/480.0)*100) / 100
			row.OTWorkUnits = math.Round((float64(otMins)/480.0)*100) / 100
			row.OTClaimStatus = otStatus
			if actMins > 0 || stdMins > 0 || (punchIn != nil && punchOut != nil) {
				row.Status = "present"
			} else {
				row.Status = "absent"
			}
			results = append(results, row)
		}
		if err := rows.Err(); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		raw, err := json.Marshal(results)
		if err != nil {
			return api.PaginatedReportResponse{}, err
		}
		return api.PaginatedReportResponse{
			Data:  raw,
			Total: total,
			Page:  params.Page,
			Limit: params.Limit,
		}, nil
	}

	baseClause := `emp.deleted_at IS NULL AND e.deleted_at IS NULL`
	countFilterClause := ""
	countArgs := []any{}
	countArgIndex := 1
	dataFilterClause := ""
	dataArgs := []any{params.StartDate, params.EndDate}
	dataArgIndex := 3

	if len(params.EmployeeIDs) > 0 {
		countFilterClause += fmt.Sprintf(" AND e.id = ANY($%d)", countArgIndex)
		countArgs = append(countArgs, params.EmployeeIDs)
		countArgIndex++
		dataFilterClause += fmt.Sprintf(" AND e.id = ANY($%d)", dataArgIndex)
		dataArgs = append(dataArgs, params.EmployeeIDs)
		dataArgIndex++
	}
	if params.DepartmentID != nil {
		countFilterClause += fmt.Sprintf(" AND emp.department_id = $%d", countArgIndex)
		countArgs = append(countArgs, *params.DepartmentID)
		countArgIndex++
		dataFilterClause += fmt.Sprintf(" AND emp.department_id = $%d", dataArgIndex)
		dataArgs = append(dataArgs, *params.DepartmentID)
		dataArgIndex++
	}
	if params.EmployerID != nil {
		countFilterClause += fmt.Sprintf(" AND emp.employer_id = $%d", countArgIndex)
		countArgs = append(countArgs, *params.EmployerID)
		countArgIndex++
		dataFilterClause += fmt.Sprintf(" AND emp.employer_id = $%d", dataArgIndex)
		dataArgs = append(dataArgs, *params.EmployerID)
		dataArgIndex++
	}
	if params.EmploymentID != nil {
		countFilterClause += fmt.Sprintf(" AND emp.id = $%d", countArgIndex)
		countArgs = append(countArgs, *params.EmploymentID)
		dataFilterClause += fmt.Sprintf(" AND emp.id = $%d", dataArgIndex)
		dataArgs = append(dataArgs, *params.EmploymentID)
		dataArgIndex++
	}

	var total int64
	countQuery := fmt.Sprintf(`SELECT count(DISTINCT emp.id) FROM employments emp JOIN employees e ON e.id = emp.employee_id WHERE %s%s`, baseClause, countFilterClause)
	if err := db.QueryRow(ctx, countQuery, countArgs...).Scan(&total); err != nil {
		return api.PaginatedReportResponse{}, err
	}

	dataQuery := fmt.Sprintf(`SELECT emp.id AS employment_id, e.id AS employee_id, e.full_name, e.employee_id_number,
       COALESCE(SUM(ads.standard_minutes), 0) / 480 AS total_std_units,
       COALESCE(SUM(ads.approved_ot_minutes), 0) / 480 AS total_ot_units,
       ROUND((COALESCE(SUM(ads.standard_minutes), 0)::numeric / 60.0), 2) AS computed_std_hours,
       ROUND((COALESCE(SUM(ads.approved_ot_minutes), 0)::numeric / 60.0), 2) AS computed_ot_hours,
       emp.employer_id, em.name AS employer_name
FROM employments emp
JOIN employees e ON e.id = emp.employee_id
LEFT JOIN employers em ON em.id = emp.employer_id
LEFT JOIN attendance_daily_summaries ads ON ads.employment_id = emp.id AND ads.date >= $1::date AND ads.date <= $2::date
		WHERE %s%s
		GROUP BY emp.id, e.id, e.full_name, e.employee_id_number, emp.employer_id, em.name
		ORDER BY e.full_name ASC
		LIMIT $%d OFFSET $%d`, baseClause, dataFilterClause, dataArgIndex, dataArgIndex+1)

	dataArgs = append(dataArgs, params.Limit, offset)
	rows, err := db.Query(ctx, dataQuery, dataArgs...)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	defer rows.Close()

	summaries := make([]api.AttendanceSummaryRow, 0)
	for rows.Next() {
		var row api.AttendanceSummaryRow
		row.UnitType = "DAY"
		row.UnitCoefficientHours = 8.0
		if err := rows.Scan(&row.EmploymentID, &row.EmployeeID, &row.EmployeeName, &row.EmployeeIDNumber,
			&row.TotalStdUnits, &row.TotalOTUnits,
			&row.ComputedStdHours, &row.ComputedOTHours,
			&row.EmployerID, &row.EmployerName); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		summaries = append(summaries, row)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedReportResponse{}, err
	}
	raw, err := json.Marshal(summaries)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	return api.PaginatedReportResponse{
		Data:  raw,
		Total: total,
		Page:  params.Page,
		Limit: params.Limit,
	}, nil
}

func (r ReportRepository) GetNoShowReport(ctx context.Context, params api.NoShowReportFilter) (api.PaginatedReportResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	offset := (params.Page - 1) * params.Limit

	subquery := `(
    SELECT id, employment_id, template_id, date, scheduled_start, scheduled_end FROM work_segments
    UNION ALL
    SELECT sr.id, sr.employment_id, sr.template_id, sr.date, '09:00:00'::time, '17:00:00'::time
    FROM shift_rosters sr
    JOIN employments emp ON emp.id = sr.employment_id
    WHERE NOT EXISTS (SELECT 1 FROM work_segments ws2 WHERE ws2.employment_id = sr.employment_id AND ws2.date = sr.date)
) ws`

	whereClause := `ws.date = $1::date AND te.id IS NULL AND emp.deleted_at IS NULL AND e.deleted_at IS NULL`
	args := []any{params.Date}
	idx := 2

	if len(params.EmployeeIDs) > 0 {
		whereClause += fmt.Sprintf(" AND e.id = ANY($%d)", idx)
		args = append(args, params.EmployeeIDs)
		idx++
	}
	if params.DepartmentID != nil {
		whereClause += fmt.Sprintf(" AND emp.department_id = $%d", idx)
		args = append(args, *params.DepartmentID)
		idx++
	}

	var total int64
	countQuery := fmt.Sprintf(`SELECT count(*) FROM %s
JOIN employments emp ON emp.id = ws.employment_id
JOIN employees e ON e.id = emp.employee_id
LEFT JOIN time_entries te ON te.employment_id = ws.employment_id AND te.punch_in_time::date = ws.date
WHERE %s`, subquery, whereClause)

	if err := db.QueryRow(ctx, countQuery, args...).Scan(&total); err != nil {
		return api.PaginatedReportResponse{}, err
	}

	dataQuery := fmt.Sprintf(`SELECT ws.id, ws.employment_id, e.id AS employee_id, e.full_name, e.employee_id_number,
       st.name AS shift_name, ws.scheduled_start::text, ws.scheduled_end::text,
       ws.date::text
FROM %s
JOIN employments emp ON emp.id = ws.employment_id
JOIN employees e ON e.id = emp.employee_id
LEFT JOIN shift_templates st ON st.id = ws.template_id
LEFT JOIN time_entries te ON te.employment_id = ws.employment_id AND te.punch_in_time::date = ws.date
WHERE %s
ORDER BY e.full_name ASC
LIMIT $%d OFFSET $%d`, subquery, whereClause, idx, idx+1)

	args = append(args, params.Limit, offset)
	rows, err := db.Query(ctx, dataQuery, args...)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	defer rows.Close()

	results := make([]api.NoShowReportRow, 0)
	for rows.Next() {
		var dummyID uuid.UUID
		var row api.NoShowReportRow
		var shiftName, schedStart, schedEnd string
		if err := rows.Scan(&dummyID, &row.EmploymentID, &row.EmployeeID, &row.EmployeeName, &row.EmployeeIDNumber,
			&shiftName, &schedStart, &schedEnd, &row.Date); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		row.ShiftName = &shiftName
		row.ScheduledStart = &schedStart
		row.ScheduledEnd = &schedEnd
		row.Status = "absent"
		row.RecordingIssue = "missing_punch_in"
		results = append(results, row)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedReportResponse{}, err
	}
	raw, err := json.Marshal(results)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	return api.PaginatedReportResponse{
		Data:  raw,
		Total: total,
		Page:  params.Page,
		Limit: params.Limit,
	}, nil
}

type RedisReportQueue struct {
	Client          *redisinfra.Client
	Fallback        sync.Map
	ResultsFallback sync.Map
}

func (q *RedisReportQueue) EnqueueReport(ctx context.Context, job api.ReportJob, req api.ExportReportRequest) error {
	if q == nil {
		return errors.New("report queue is nil")
	}
	meta := map[string]any{
		"job_id":            job.JobID.String(),
		"requester_id":      job.RequesterID.String(),
		"status":            job.Status,
		"total_batches":     job.TotalBatches,
		"completed_batches": 0,
		"failed_batches":    0,
		"created_at":        job.CreatedAt.Format(time.RFC3339),
	}
	encoded, err := json.Marshal(meta)
	if err != nil {
		return err
	}
	q.Fallback.Store(job.JobID.String(), job)
	if q.Client != nil {
		shared, err := q.Client.Shared()
		if err == nil && shared != nil {
			pipe := shared.TxPipeline()
			pipe.Set(ctx, redisinfra.KeyReportMeta(job.JobID), encoded, 24*time.Hour)
			pipe.Set(ctx, fmt.Sprintf("job:metadata:%s", job.JobID.String()), encoded, 24*time.Hour)
			workerPayload, _ := json.Marshal(map[string]any{
				"task_type":     "attendance_report",
				"request_id":    job.JobID.String(),
				"parent_job_id": job.JobID.String(),
				"priority":      "high",
			})
			pipe.RPush(ctx, "worker:jobs:high", workerPayload)
			_, _ = pipe.Exec(ctx)
		}
	}
	return nil
}

func (q *RedisReportQueue) GetReport(ctx context.Context, jobID uuid.UUID) (api.ReportJob, error) {
	if q == nil {
		return api.ReportJob{}, errors.New("report queue is nil")
	}
	if q.Client != nil {
		shared, err := q.Client.Shared()
		if err == nil && shared != nil {
			val, err := shared.Get(ctx, redisinfra.KeyReportMeta(jobID)).Result()
			if errors.Is(err, goredis.Nil) {
				val, err = shared.Get(ctx, fmt.Sprintf("job:metadata:%s", jobID.String())).Result()
			}
			if err == nil {
				var meta struct {
					JobID            string  `json:"job_id"`
					RequesterID      string  `json:"requester_id"`
					Status           string  `json:"status"`
					TotalBatches     int     `json:"total_batches"`
					CompletedBatches int     `json:"completed_batches"`
					FailedBatches    int     `json:"failed_batches"`
					ErrorMessage     *string `json:"error_message"`
					CreatedAt        *string `json:"created_at"`
					CompletedAt      *string `json:"completed_at"`
				}
				if json.Unmarshal([]byte(val), &meta) == nil {
					parsedJobID, _ := uuid.Parse(meta.JobID)
					parsedReqID, _ := uuid.Parse(meta.RequesterID)
					createdAt := time.Now().UTC()
					if meta.CreatedAt != nil {
						if t, err := time.Parse(time.RFC3339, *meta.CreatedAt); err == nil {
							createdAt = t
						}
					}
					var completedAt *time.Time
					if meta.CompletedAt != nil {
						if t, err := time.Parse(time.RFC3339, *meta.CompletedAt); err == nil {
							completedAt = &t
						}
					}
					return api.ReportJob{
						JobID:            parsedJobID,
						RequesterID:      parsedReqID,
						Status:           meta.Status,
						TotalBatches:     meta.TotalBatches,
						CompletedBatches: meta.CompletedBatches,
						FailedBatches:    meta.FailedBatches,
						Error:            meta.ErrorMessage,
						CreatedAt:        createdAt,
						CompletedAt:      completedAt,
					}, nil
				}
			} else if !errors.Is(err, goredis.Nil) {
				return api.ReportJob{}, err
			}
		}
	}
	if val, ok := q.Fallback.Load(jobID.String()); ok {
		if job, ok := val.(api.ReportJob); ok {
			return job, nil
		}
	}

	return api.ReportJob{}, api.ErrReportNotFound
}
func (q *RedisReportQueue) CancelReport(ctx context.Context, jobID uuid.UUID) error {
	if q == nil {
		return errors.New("report queue is nil")
	}
	job, err := q.GetReport(ctx, jobID)
	if err != nil {
		return err
	}
	if job.Status == "completed" || job.Status == "failed" {
		return api.ErrReportCannotCancel
	}
	now := time.Now().UTC()
	cancelledErrMsg := "Cancelled by user"
	job.Status = "failed"
	job.Error = &cancelledErrMsg
	job.CompletedAt = &now

	q.Fallback.Store(jobID.String(), job)

	if q.Client != nil {
		shared, err := q.Client.Shared()
		if err == nil && shared != nil {
			meta := map[string]any{
				"job_id":            job.JobID.String(),
				"requester_id":      job.RequesterID.String(),
				"status":            "failed",
				"total_batches":     job.TotalBatches,
				"completed_batches": job.CompletedBatches,
				"failed_batches":    job.FailedBatches,
				"error_message":     cancelledErrMsg,
				"created_at":        job.CreatedAt.Format(time.RFC3339),
				"completed_at":      now.Format(time.RFC3339),
			}
			if encoded, err := json.Marshal(meta); err == nil {
				pipe := shared.TxPipeline()
				pipe.Set(ctx, redisinfra.KeyReportMeta(jobID), encoded, 24*time.Hour)
				pipe.Set(ctx, fmt.Sprintf("job:metadata:%s", jobID.String()), encoded, 24*time.Hour)
				_, _ = pipe.Exec(ctx)
			}
		}
	}
	return nil
}

func (q *RedisReportQueue) GetResultData(ctx context.Context, jobID uuid.UUID) ([]byte, string, error) {
	if q == nil {
		return nil, "", errors.New("report queue is nil")
	}
	if q.Client != nil {
		shared, err := q.Client.Shared()
		if err == nil && shared != nil {
			dataKey := redisinfra.KeyReportResultData(jobID)
			fnameKey := redisinfra.KeyReportResultFilename(jobID)

			pipe := shared.TxPipeline()
			dataCmd := pipe.Get(ctx, dataKey)
			fnameCmd := pipe.Get(ctx, fnameKey)
			_, err := pipe.Exec(ctx)
			if err == nil {
				dataBytes, dataErr := dataCmd.Bytes()
				fname, fnameErr := fnameCmd.Result()
				if dataErr == nil && fnameErr == nil && len(dataBytes) > 0 {
					return dataBytes, fname, nil
				}
			}
		}
	}
	if val, ok := q.ResultsFallback.Load(jobID.String()); ok {
		if res, ok := val.(ReportResultPayload); ok {
			return res.Data, res.Filename, nil
		}
	}
	return nil, "", api.ErrReportResultNotFound
}

type ReportResultPayload struct {
	Data     []byte
	Filename string
}

func (q *RedisReportQueue) StoreResultForTest(jobID uuid.UUID, data []byte, filename string) {
	q.ResultsFallback.Store(jobID.String(), ReportResultPayload{Data: data, Filename: filename})
}
