package repository

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"time"
)

type LeaveRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

var (
	_ api.LeaveRepository       = LeaveRepository{}
	_ api.LeaveImportRepository = LeaveRepository{}
)

func (r LeaveRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("leave repository: database does not support transactions")
	}
	return beginner, nil
}

func (r LeaveRepository) CreateLeaveType(ctx context.Context, req api.CreateLeaveTypeRequest) (api.LeaveTypeRecord, error) {
	db, err := requireDB(r.DB, "leave repository")
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
	err = db.QueryRow(ctx, `INSERT INTO leave_types (id, code, name, is_paid, policy_config, created_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, code, name, is_paid, policy_config, created_at`, id, req.Code, req.Name, isPaid, policyConfigJSON, now).Scan(&record.ID, &record.Code, &record.Name, &record.IsPaid, &rawConfig, &createdAt)
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

func (r LeaveRepository) ListLeaveTypes(ctx context.Context) ([]api.LeaveTypeRecord, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, code, name, is_paid, policy_config, created_at FROM leave_types WHERE deleted_at IS NULL ORDER BY name ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	types := make([]api.LeaveTypeRecord, 0)
	for rows.Next() {
		var item api.LeaveTypeRecord
		var rawConfig []byte
		var createdAt time.Time
		if err := rows.Scan(&item.ID, &item.Code, &item.Name, &item.IsPaid, &rawConfig, &createdAt); err != nil {
			return nil, err
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		item.PolicyConfig = json.RawMessage(rawConfig)
		types = append(types, item)
	}
	return types, rows.Err()
}

func (r LeaveRepository) UpdateLeaveTypeConfig(ctx context.Context, leaveTypeID uuid.UUID, policyConfig json.RawMessage) (api.LeaveTypeRecord, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return api.LeaveTypeRecord{}, err
	}
	var record api.LeaveTypeRecord
	var rawConfig []byte
	var createdAt time.Time
	err = db.QueryRow(ctx, `UPDATE leave_types SET policy_config = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING id, code, name, is_paid, policy_config, created_at`, policyConfig, leaveTypeID).
		Scan(&record.ID, &record.Code, &record.Name, &record.IsPaid, &rawConfig, &createdAt)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.LeaveTypeRecord{}, api.ErrLeaveTypeNotFound
		}
		return api.LeaveTypeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.PolicyConfig = json.RawMessage(rawConfig)
	return record, nil
}

func (r LeaveRepository) DeleteLeaveType(ctx context.Context, leaveTypeID uuid.UUID) error {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return err
	}
	var inUse bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM leave_balances WHERE leave_type_id = $1)`, leaveTypeID).Scan(&inUse)
	if err != nil {
		return err
	}
	if inUse {
		return api.ErrLeaveConflict
	}
	tag, err := db.Exec(ctx, `UPDATE leave_types SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1`, leaveTypeID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrLeaveTypeNotFound
	}
	return nil
}

func (r LeaveRepository) RestoreLeaveType(ctx context.Context, leaveTypeID uuid.UUID) (api.LeaveTypeRecord, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return api.LeaveTypeRecord{}, err
	}
	var record api.LeaveTypeRecord
	var rawConfig []byte
	var createdAt time.Time
	err = db.QueryRow(ctx, `UPDATE leave_types SET deleted_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NOT NULL RETURNING id, code, name, is_paid, policy_config, created_at`, leaveTypeID).
		Scan(&record.ID, &record.Code, &record.Name, &record.IsPaid, &rawConfig, &createdAt)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.LeaveTypeRecord{}, api.ErrLeaveTypeNotFound
		}
		return api.LeaveTypeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.PolicyConfig = json.RawMessage(rawConfig)
	return record, nil
}

func (r LeaveRepository) SubmitLeaveRequest(ctx context.Context, requesterID uuid.UUID, req api.SubmitLeaveRequest, autoApprove bool) (api.LeaveRequestRecord, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.LeaveRequestRecord{}, err
	}
	now := time.Now().UTC()
	var record api.LeaveRequestRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employment_segments WHERE id = $1 AND deleted_at IS NULL)`, req.EmploymentID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrLeaveNoEmployment
		}
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM leave_types WHERE id = $1 AND deleted_at IS NULL)`, req.LeaveTypeID).Scan(&exists); err != nil {
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

func (r LeaveRepository) ListOrganizationLeaveRequests(ctx context.Context, status *string) ([]api.LeaveRequestRecord, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT lr.id, lr.employment_id, lr.leave_type_id, lr.start_date::text, lr.end_date::text, lr.reason, lr.status::text, lr.duration_type, lr.start_time::text, lr.end_time::text, lr.requested_by_user_id, lr.approved_by_user_id, lr.created_at, lr.updated_at, lr.attachment_file_ids FROM leave_requests lr JOIN employment_segments e ON e.id = lr.employment_id WHERE e.deleted_at IS NULL AND ($1::text IS NULL OR lr.status::text = $1) ORDER BY lr.created_at DESC`, status)
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

func (r LeaveRepository) ApproveLeaveRequest(ctx context.Context, approverID, requestID uuid.UUID, status string) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var currentStatus string
		err := db.QueryRow(ctx, `SELECT status::text FROM leave_requests WHERE id = $1 FOR UPDATE`, requestID).Scan(&currentStatus)
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

func (r LeaveRepository) GetEmployeeLeaveBalances(ctx context.Context, employeeID uuid.UUID, year int) ([]api.EmployeeLeaveBalanceRecord, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT lt.id, lt.code, lt.name, lt.is_paid, COALESCE(lb.entitlement, 12), COALESCE(lb.accrued_amount::text, '0.00'), COALESCE(lb.carried_over, 0), COALESCE((
		SELECT ROUND(COALESCE(SUM(
			(SELECT COALESCE(SUM(
				CASE
					WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type <> 'half_day') THEN 0::numeric
					WHEN lr.duration_type IN ('first_half', 'second_half') THEN 0.5::numeric
					WHEN EXISTS (SELECT 1 FROM holidays h WHERE h.date = d::date AND h.holiday_type = 'half_day') THEN 0.5::numeric
					ELSE 1::numeric
				END
			), 0)
			FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
			JOIN schedule_day_revisions sdr
			  ON sdr.employment_id = lr.employment_id
			 AND sdr.business_date = d::date
			 AND sdr.state = 'published')
		), 0))::bigint
		FROM leave_requests lr
		JOIN employment_segments e ON e.id = lr.employment_id
		WHERE e.employee_id = $1 AND e.deleted_at IS NULL AND lr.leave_type_id = lt.id AND lr.status IN ('approved', 'pre_approved') AND EXTRACT(YEAR FROM lr.start_date) = $2
	), 0) AS used_count FROM leave_types lt LEFT JOIN LATERAL (SELECT id FROM employment_segments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id DESC LIMIT 1) active_emp ON true LEFT JOIN leave_balances lb ON lb.leave_type_id = lt.id AND lb.employment_id = active_emp.id AND lb.year = $2 WHERE lt.deleted_at IS NULL ORDER BY lt.name ASC`, employeeID, year)
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

func (r LeaveRepository) ResolveEmployeeForUser(ctx context.Context, userID uuid.UUID) (uuid.UUID, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `SELECT t.employee_id FROM employee_tenures t JOIN employees e ON e.id = t.employee_id WHERE t.user_id = $1 AND t.deleted_at IS NULL AND t.start_date <= `+orgTodaySQL+` AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`) AND e.deleted_at IS NULL ORDER BY t.start_date DESC, t.id DESC LIMIT 1`, userID).Scan(&empID)
	return empID, err
}

func (r LeaveRepository) ListEmployeeLeaveRequests(ctx context.Context, employeeID uuid.UUID) ([]api.LeaveRequestRecord, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT lr.id, lr.employment_id, lr.leave_type_id, lr.start_date::text, lr.end_date::text, lr.reason, lr.status::text, lr.duration_type, lr.start_time::text, lr.end_time::text, lr.requested_by_user_id, lr.approved_by_user_id, lr.created_at, lr.updated_at, lr.attachment_file_ids
		FROM leave_requests lr
		JOIN employment_segments e ON e.id = lr.employment_id
		JOIN employment_relationships er ON er.id = e.relationship_id AND er.employee_id = e.employee_id
		JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = er.employee_id
		WHERE e.employee_id = $1 AND e.deleted_at IS NULL AND t.deleted_at IS NULL
		  AND t.start_date <= CURRENT_DATE AND (t.end_date IS NULL OR t.end_date >= CURRENT_DATE)
		ORDER BY lr.created_at DESC
	`, employeeID)
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
