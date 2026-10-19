package repository

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type LeaveRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r LeaveRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("leave repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

func (r LeaveRepository) begin(ctx context.Context, workspaceID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, workspaceID)
	}
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("leave repository: tenant database does not support transactions")
	}
	return beginner, nil
}

func (r LeaveRepository) CreateLeaveType(ctx context.Context, workspaceID uuid.UUID, req api.CreateLeaveTypeRequest) (api.LeaveTypeRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.LeaveTypeRecord{}, err
	}
	id := uuid.New()
	now := time.Now().UTC()
	isPaid := true
	if req.IsPaid != nil {
		isPaid = *req.IsPaid
	}
	policyConfigJSON := req.PolicyConfig
	if len(policyConfigJSON) == 0 {
		policyConfigJSON = json.RawMessage(`{}`)
	}
	var record api.LeaveTypeRecord
	var rawConfig []byte
	var createdAt time.Time
	err = db.QueryRow(ctx, `INSERT INTO leave_types (id, workspace_id, code, name, is_paid, policy_config, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, workspace_id, code, name, is_paid, policy_config, created_at`, id, workspaceID, req.Code, req.Name, isPaid, policyConfigJSON, now).Scan(&record.ID, &record.WorkspaceID, &record.Code, &record.Name, &record.IsPaid, &rawConfig, &createdAt)
	if err != nil {
		if pgErr, ok := err.(*pgconn.PgError); ok && (pgErr.Code == "23505" || pgErr.Code == "23P01") {
			return api.LeaveTypeRecord{}, api.ErrLeaveConflict
		}
		return api.LeaveTypeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.PolicyConfig = json.RawMessage(rawConfig)
	return record, nil
}

func (r LeaveRepository) ListLeaveTypes(ctx context.Context, workspaceID uuid.UUID) ([]api.LeaveTypeRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, code, name, is_paid, policy_config, created_at FROM leave_types WHERE workspace_id = $1 AND deleted_at IS NULL ORDER BY name ASC`, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	types := make([]api.LeaveTypeRecord, 0)
	for rows.Next() {
		var item api.LeaveTypeRecord
		var rawConfig []byte
		var createdAt time.Time
		if err := rows.Scan(&item.ID, &item.WorkspaceID, &item.Code, &item.Name, &item.IsPaid, &rawConfig, &createdAt); err != nil {
			return nil, err
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		item.PolicyConfig = json.RawMessage(rawConfig)
		types = append(types, item)
	}
	return types, rows.Err()
}

func (r LeaveRepository) SubmitLeaveRequest(ctx context.Context, workspaceID, requesterID uuid.UUID, req api.SubmitLeaveRequest, autoApprove bool) (api.LeaveRequestRecord, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.LeaveRequestRecord{}, err
	}
	now := time.Now().UTC()
	var record api.LeaveRequestRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, req.EmploymentID, workspaceID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrLeaveNoEmployment
		}
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM leave_types WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, req.LeaveTypeID, workspaceID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrLeaveTypeNotFound
		}
		status := "pending"
		var approvedBy *uuid.UUID
		if autoApprove {
			status = "pre_approved"
			approvedBy = &requesterID
		}
		duration := req.DurationType
		if duration == "" {
			duration = "full_day"
		}
		reqID := uuid.New()
		var createdAt, updatedAt time.Time
		err := db.QueryRow(ctx, `INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, reason, status, duration_type, start_time, end_time, requested_by_user_id, approved_by_user_id, created_at, updated_at, attachment_file_ids) VALUES ($1, $2, $3, $4::date, $5::date, $6, $7::leave_status_type, $8, $9, $10, $11, $12, $13, $13, $14) RETURNING id, employment_id, leave_type_id, start_date::text, end_date::text, reason, status::text, duration_type, start_time::text, end_time::text, requested_by_user_id, approved_by_user_id, created_at, updated_at, attachment_file_ids`, reqID, req.EmploymentID, req.LeaveTypeID, req.StartDate, req.EndDate, req.Reason, status, duration, req.StartTime, req.EndTime, requesterID, approvedBy, now, req.AttachmentFileIDs).Scan(
			&record.ID, &record.EmploymentID, &record.LeaveTypeID, &record.StartDate, &record.EndDate, &record.Reason, &record.Status, &record.DurationType, &record.StartTime, &record.EndTime, &record.RequestedByUserID, &record.ApprovedByUserID, &createdAt, &updatedAt, &record.AttachmentFileIDs,
		)
		if err != nil {
			if pgErr, ok := err.(*pgconn.PgError); ok && (pgErr.Code == "23P01" || pgErr.Code == "23505") {
				return api.ErrLeaveConflict
			}
			return err
		}
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		return nil
	})
	if err != nil {
		return api.LeaveRequestRecord{}, err
	}
	return record, nil
}

func (r LeaveRepository) ListWorkspaceLeaveRequests(ctx context.Context, workspaceID uuid.UUID, status *string) ([]api.LeaveRequestRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT lr.id, lr.employment_id, lr.leave_type_id, lr.start_date::text, lr.end_date::text, lr.reason, lr.status::text, lr.duration_type, lr.start_time::text, lr.end_time::text, lr.requested_by_user_id, lr.approved_by_user_id, lr.created_at, lr.updated_at, lr.attachment_file_ids FROM leave_requests lr JOIN employments e ON e.id = lr.employment_id WHERE e.workspace_id = $1 AND ($2::text IS NULL OR lr.status::text = $2) ORDER BY lr.created_at DESC`, workspaceID, status)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	records := make([]api.LeaveRequestRecord, 0)
	for rows.Next() {
		var rec api.LeaveRequestRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&rec.ID, &rec.EmploymentID, &rec.LeaveTypeID, &rec.StartDate, &rec.EndDate, &rec.Reason, &rec.Status, &rec.DurationType, &rec.StartTime, &rec.EndTime, &rec.RequestedByUserID, &rec.ApprovedByUserID, &createdAt, &updatedAt, &rec.AttachmentFileIDs); err != nil {
			return nil, err
		}
		rec.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		rec.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		records = append(records, rec)
	}
	return records, rows.Err()
}

func (r LeaveRepository) ApproveLeaveRequest(ctx context.Context, workspaceID, approverID, requestID uuid.UUID, status string) error {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var currentStatus string
		err := db.QueryRow(ctx, `SELECT lr.status::text FROM leave_requests lr JOIN employments e ON e.id = lr.employment_id WHERE lr.id = $1 AND e.workspace_id = $2 FOR UPDATE`, requestID, workspaceID).Scan(&currentStatus)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNotFound
		}
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `UPDATE leave_requests SET status = $1::leave_status_type, approved_by_user_id = $2, updated_at = now() WHERE id = $3`, status, approverID, requestID)
		return err
	})
}

func (r LeaveRepository) GetEmployeeLeaveBalances(ctx context.Context, workspaceID, employeeID uuid.UUID, year int) ([]api.EmployeeLeaveBalanceRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT lt.id, lt.code, lt.name, lt.is_paid, COALESCE(lb.entitlement, 12), COALESCE(lb.accrued_amount::text, '0.00'), COALESCE(lb.carried_over, 0), (SELECT count(*) FROM leave_requests lr JOIN employments e ON e.id = lr.employment_id WHERE e.employee_id = $1 AND e.workspace_id = $2 AND lr.leave_type_id = lt.id AND lr.status IN ('approved', 'pre_approved') AND EXTRACT(YEAR FROM lr.start_date) = $3) AS used_count FROM leave_types lt LEFT JOIN employments active_emp ON active_emp.employee_id = $1 AND active_emp.workspace_id = $2 AND active_emp.deleted_at IS NULL LEFT JOIN leave_balances lb ON lb.leave_type_id = lt.id AND lb.employment_id = active_emp.id AND lb.year = $3 WHERE lt.workspace_id = $2 AND lt.deleted_at IS NULL ORDER BY lt.name ASC`, employeeID, workspaceID, year)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	balances := make([]api.EmployeeLeaveBalanceRecord, 0)
	for rows.Next() {
		var b api.EmployeeLeaveBalanceRecord
		var accruedStr string
		var usedCount int64
		if err := rows.Scan(&b.LeaveTypeID, &b.LeaveTypeCode, &b.LeaveTypeName, &b.IsPaid, &b.Entitlement, &accruedStr, &b.CarriedOver, &usedCount); err != nil {
			return nil, err
		}
		accrued, _ := decimal.NewFromString(accruedStr)
		b.AccruedAmount = accrued
		b.Used = int(usedCount)
		total := decimal.NewFromInt(int64(b.Entitlement + b.CarriedOver)).Add(b.AccruedAmount)
		b.Remaining = total.Sub(decimal.NewFromInt(int64(b.Used)))
		balances = append(balances, b)
	}
	return balances, rows.Err()
}

func (r LeaveRepository) ResolveEmployeeForUser(ctx context.Context, workspaceID, userID uuid.UUID) (uuid.UUID, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employees WHERE user_id = $1 AND workspace_id = $2 AND deleted_at IS NULL LIMIT 1`, userID, workspaceID).Scan(&empID)
	return empID, err
}
