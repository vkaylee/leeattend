package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"os"
	"path/filepath"
	"strings"
	"time"
)

func (r LeaveRepository) CreateAccrualJob(ctx context.Context, userID uuid.UUID, dateStr string) (uuid.UUID, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return uuid.Nil, err
	}
	jobID := uuid.New()
	payload, _ := json.Marshal(map[string]any{"date": dateStr, "user_id": userID.String(), "force": true})
	_, err = db.Exec(ctx, `INSERT INTO outbox_jobs (id, job_type, payload, status, error_message, created_at, updated_at) VALUES ($1, 'LeaveAccrual', $2, 'pending', NULL, NOW(), NOW())`, jobID, payload)
	if err != nil {
		return uuid.Nil, err
	}
	return jobID, nil
}

func (r LeaveRepository) GetJobStatus(ctx context.Context, jobID uuid.UUID) (api.LeaveJobStatusResponse, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return api.LeaveJobStatusResponse{}, err
	}
	var status string
	var errorMsg *string
	var updatedAt time.Time
	if err = db.QueryRow(ctx, `SELECT status, error_message, updated_at FROM outbox_jobs WHERE id = $1`, jobID).Scan(&status, &errorMsg, &updatedAt); err == nil {
		var completedAt *string
		if status == "completed" || status == "failed" {
			value := updatedAt.UTC().Format(time.RFC3339)
			completedAt = &value
		}
		return api.LeaveJobStatusResponse{JobID: jobID, Status: status, CompletedAt: completedAt, ErrorMessage: errorMsg}, nil
	}
	if err = db.QueryRow(ctx, `SELECT status::text, error_log, updated_at FROM attendance_import_jobs WHERE id = $1`, jobID).Scan(&status, &errorMsg, &updatedAt); err == nil {
		var completedAt *string
		if status == "completed" || status == "failed" || status == "partially_failed" {
			value := updatedAt.UTC().Format(time.RFC3339)
			completedAt = &value
		}
		return api.LeaveJobStatusResponse{JobID: jobID, Status: status, CompletedAt: completedAt, ErrorMessage: errorMsg}, nil
	}
	return api.LeaveJobStatusResponse{}, api.ErrLeaveNotFound
}

func (r LeaveRepository) CreateLeaveImportJob(ctx context.Context, userID uuid.UUID, filePath string) (api.LeaveImportJob, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return api.LeaveImportJob{}, err
	}
	var job api.LeaveImportJob
	err = db.QueryRow(ctx, `INSERT INTO attendance_import_jobs (id, user_id, status, total_rows, processed_rows, file_path, target_type, created_at, updated_at) VALUES ($1, $2, 'processing', 0, 0, $3, 'leave_import', NOW(), NOW()) RETURNING id, user_id, status::text, total_rows, processed_rows, file_path, error_log, created_at, updated_at`, uuid.New(), userID, filePath).Scan(
		&job.ID, &job.UserID, &job.Status, &job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.CreatedAt, &job.UpdatedAt,
	)
	return job, err
}

func (r LeaveRepository) ProcessLeaveImport(ctx context.Context, userID uuid.UUID, job api.LeaveImportJob, rows []api.LeaveImportRow, autoApprove bool) error {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return err
	}
	processed := 0
	failed := 0
	var errorLines []string
	for _, row := range rows {
		startDate, startErr := time.Parse("2006-01-02", row.StartDate)
		if startErr != nil {
			failed++
			errorLines = append(errorLines, fmt.Sprintf("line %d: invalid start date", row.LineNumber))
			continue
		}
		employmentID, resolveErr := resolveImportedEmployment(ctx, db, row.EmployeeCode, startDate)
		if resolveErr != nil {
			failed++
			message := "employee not found"
			if errors.Is(resolveErr, errAmbiguousImportedEmployment) {
				message = "employee number is ambiguous for date"
			}
			errorLines = append(errorLines, fmt.Sprintf("line %d: %s", row.LineNumber, message))
			continue
		}
		var leaveTypeID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT id FROM leave_types WHERE UPPER(code) = UPPER($1) AND deleted_at IS NULL LIMIT 1`, row.LeaveTypeCode).Scan(&leaveTypeID); err != nil {
			failed++
			errorLines = append(errorLines, fmt.Sprintf("line %d: leave type %s not found", row.LineNumber, row.LeaveTypeCode))
			continue
		}
		status := "pending"
		var approvedBy *uuid.UUID
		if autoApprove {
			status = "pre_approved"
			approvedBy = &userID
		}
		if _, err := db.Exec(ctx, `INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, reason, status, duration_type, requested_by_user_id, approved_by_user_id, created_at, updated_at) VALUES (gen_random_uuid(), $1, $2, $3::date, $4::date, $5, $6::leave_status_type, $7, $8, $9, NOW(), NOW())`, employmentID, leaveTypeID, row.StartDate, row.EndDate, row.Reason, status, row.DurationType, userID, approvedBy); err != nil {
			failed++
			errorLines = append(errorLines, fmt.Sprintf("line %d: %v", row.LineNumber, err))
			continue
		}
		processed++
	}
	finalStatus := "completed"
	var errorLog *string
	if failed > 0 {
		finalStatus = "partially_failed"
		message := strings.Join(errorLines, "; ")
		errorLog = &message
		if job.FilePath != nil {
			errorPath := filepath.Join(filepath.Dir(*job.FilePath), "error_"+job.ID.String()+".xlsx")
			if data, genErr := api.BuildErrorWorkbook(job.ID, errorLog); genErr == nil {
				_ = os.WriteFile(errorPath, data, 0o640)
			}
		}
	}
	_, err = db.Exec(ctx, `UPDATE attendance_import_jobs SET status = $1, total_rows = $2, processed_rows = $3, error_log = $4, updated_at = NOW() WHERE id = $5`, finalStatus, len(rows), processed, errorLog, job.ID)
	return err
}

func (r LeaveRepository) GetLatestLeaveImport(ctx context.Context) (*api.LeaveImportJob, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	var job api.LeaveImportJob
	err = db.QueryRow(ctx, `SELECT id, user_id, status::text, total_rows, processed_rows, file_path, error_log, created_at, updated_at FROM attendance_import_jobs WHERE target_type = 'leave_import' ORDER BY created_at DESC LIMIT 1`).Scan(
		&job.ID, &job.UserID, &job.Status, &job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.CreatedAt, &job.UpdatedAt,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &job, nil
}

func (r LeaveRepository) GetLeaveImport(ctx context.Context, jobID uuid.UUID) (*api.LeaveImportJob, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	var job api.LeaveImportJob
	err = db.QueryRow(ctx, `SELECT id, user_id, status::text, total_rows, processed_rows, file_path, error_log, created_at, updated_at FROM attendance_import_jobs WHERE id = $1 AND target_type = 'leave_import'`, jobID).Scan(
		&job.ID, &job.UserID, &job.Status, &job.TotalRows, &job.ProcessedRows, &job.FilePath, &job.ErrorLog, &job.CreatedAt, &job.UpdatedAt,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &job, nil
}

func (r LeaveRepository) GetLeaveImportErrorFile(ctx context.Context, jobID uuid.UUID) ([]byte, error) {
	job, err := r.GetLeaveImport(ctx, jobID)
	if err != nil {
		return nil, err
	}
	if job == nil || job.FilePath == nil {
		return nil, errNoRows
	}
	errorPath := filepath.Join(filepath.Dir(*job.FilePath), "error_"+jobID.String()+".xlsx")
	return os.ReadFile(errorPath)
}
