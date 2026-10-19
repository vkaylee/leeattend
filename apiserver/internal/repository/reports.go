package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sync"
	"time"

	"github.com/google/uuid"
	goredis "github.com/redis/go-redis/v9"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

type ReportRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r ReportRepository) db(ctx context.Context, wsID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("report repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, wsID)
}

func (r ReportRepository) CreateExportJob(ctx context.Context, job api.ReportJob, req api.ExportReportRequest) error {
	db, err := r.db(ctx, job.WorkspaceID)
	if err != nil {
		return err
	}
	payload, err := json.Marshal(map[string]any{
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
	_, err = db.Exec(ctx, `INSERT INTO outbox_jobs (id, workspace_id, job_type, payload, status, created_at, updated_at) VALUES ($1, $2, 'AttendanceReport', $3, 'pending', NOW(), NOW())`, job.JobID, job.WorkspaceID, payload)
	return err
}

type RedisReportQueue struct {
	Client          *redisinfra.Client
	Fallback        sync.Map
	ResultsFallback sync.Map
	TenantDB        func(context.Context, uuid.UUID) (DBTX, error)
}

func (q *RedisReportQueue) EnqueueReport(ctx context.Context, job api.ReportJob, req api.ExportReportRequest) error {
	if q == nil {
		return errors.New("report queue is nil")
	}
	meta := map[string]any{
		"job_id":            job.JobID.String(),
		"workspace_id":      job.WorkspaceID.String(),
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
				"workspace_id":  job.WorkspaceID.String(),
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
					WorkspaceID      string  `json:"workspace_id"`
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
					parsedWsID, _ := uuid.Parse(meta.WorkspaceID)
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
						WorkspaceID:      parsedWsID,
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
	if q.TenantDB != nil {
		// Try resolving from outbox_jobs
		// Search across DB if possible
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
				"workspace_id":      job.WorkspaceID.String(),
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
