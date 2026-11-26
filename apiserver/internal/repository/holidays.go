package repository

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type HolidayRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r HolidayRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "holiday repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("holiday repository: database does not support transactions")
	}
	return beginner, nil
}

const holidayColumns = `id, name, date::text, holiday_type, is_paid, leave_type_id, ot_multiplier, description, work_threshold_percent, is_credit_on_work, created_at, updated_at, location_id`

func scanHoliday(row pgx.Row) (api.HolidayRecord, error) {
	var record api.HolidayRecord
	var createdAt, updatedAt time.Time
	if err := row.Scan(&record.ID, &record.Name, &record.Date, &record.HolidayType, &record.IsPaid, &record.LeaveTypeID, &record.OTMultiplier, &record.Description, &record.WorkThresholdPercent, &record.IsCreditOnWork, &createdAt, &updatedAt, &record.LocationID); err != nil {
		return api.HolidayRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

func (r HolidayRepository) ListHolidays(ctx context.Context, year *int32, locationID *uuid.UUID, page, limit int64) (api.PaginatedHolidays, error) {
	db, err := requireDB(r.DB, "holiday repository")
	if err != nil {
		return api.PaginatedHolidays{}, err
	}
	where := `deleted_at IS NULL`
	args := []any{}
	if year != nil {
		where += fmt.Sprintf(" AND EXTRACT(YEAR FROM date) = $%d", len(args)+1)
		args = append(args, *year)
	}
	if locationID != nil {
		where += fmt.Sprintf(" AND (location_id IS NULL OR location_id = $%d)", len(args)+1)
		args = append(args, *locationID)
	}
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM holidays WHERE `+where, args...).Scan(&total); err != nil {
		return api.PaginatedHolidays{}, err
	}
	dataArgs := append(append([]any{}, args...), limit, (page-1)*limit)
	rows, err := db.Query(ctx, `SELECT `+holidayColumns+` FROM holidays WHERE `+where+` ORDER BY date ASC, id ASC LIMIT $`+fmt.Sprint(len(args)+1)+` OFFSET $`+fmt.Sprint(len(args)+2), dataArgs...)
	if err != nil {
		return api.PaginatedHolidays{}, err
	}
	defer rows.Close()
	data := make([]api.HolidayRecord, 0)
	for rows.Next() {
		var record api.HolidayRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&record.ID, &record.Name, &record.Date, &record.HolidayType, &record.IsPaid, &record.LeaveTypeID, &record.OTMultiplier, &record.Description, &record.WorkThresholdPercent, &record.IsCreditOnWork, &createdAt, &updatedAt, &record.LocationID); err != nil {
			return api.PaginatedHolidays{}, err
		}
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		data = append(data, record)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedHolidays{}, err
	}
	return api.PaginatedHolidays{Data: data, Total: total, Page: page, Limit: limit}, nil
}

func (r HolidayRepository) GetHolidayByID(ctx context.Context, holidayID uuid.UUID) (api.HolidayRecord, error) {
	db, err := requireDB(r.DB, "holiday repository")
	if err != nil {
		return api.HolidayRecord{}, err
	}
	record, err := scanHoliday(db.QueryRow(ctx, `SELECT `+holidayColumns+` FROM holidays WHERE id = $1 AND deleted_at IS NULL`, holidayID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.HolidayRecord{}, api.ErrHolidayNotFound
	}
	return record, err
}

func (r HolidayRepository) CreateHoliday(ctx context.Context, req api.CreateHolidayRequest) (api.HolidayRecord, error) {
	db, err := requireDB(r.DB, "holiday repository")
	if err != nil {
		return api.HolidayRecord{}, err
	}
	holidayType := "national"
	if req.HolidayType != "" {
		holidayType = req.HolidayType
	}
	isPaid := true
	if req.IsPaid != nil {
		isPaid = *req.IsPaid
	}
	creditOnWork := true
	if req.IsCreditOnWork != nil {
		creditOnWork = *req.IsCreditOnWork
	}
	id := uuid.New()
	row := db.QueryRow(ctx, `INSERT INTO holidays (id, name, date, holiday_type, is_paid, leave_type_id, ot_multiplier, description, work_threshold_percent, is_credit_on_work, location_id) VALUES ($1, $2, $3::date, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING `+holidayColumns, id, req.Name, req.Date, holidayType, isPaid, req.LeaveTypeID, req.OTMultiplier, req.Description, req.WorkThresholdPercent, creditOnWork, req.LocationID)
	record, err := scanHoliday(row)
	if err != nil {
		if isUniqueError(err) {
			return api.HolidayRecord{}, api.ErrHolidayAlreadyExists
		}
		return api.HolidayRecord{}, err
	}
	return record, nil
}

func (r HolidayRepository) UpdateHoliday(ctx context.Context, holidayID uuid.UUID, req api.UpdateHolidayRequest) (api.HolidayRecord, error) {
	db, err := requireDB(r.DB, "holiday repository")
	if err != nil {
		return api.HolidayRecord{}, err
	}
	set := make([]string, 0, 10)
	args := make([]any, 0, 12)
	add := func(column string, value any) {
		set = append(set, fmt.Sprintf("%s = $%d", column, len(args)+1))
		args = append(args, value)
	}
	if req.Name != nil {
		add("name", *req.Name)
	}
	if req.Date != nil {
		add("date", *req.Date)
	}
	if req.HolidayType != nil {
		add("holiday_type", *req.HolidayType)
	}
	if req.IsPaid != nil {
		add("is_paid", *req.IsPaid)
	}
	if req.LeaveTypeID != nil {
		add("leave_type_id", *req.LeaveTypeID)
	}
	if req.OTMultiplier != nil {
		add("ot_multiplier", *req.OTMultiplier)
	}
	if req.Description != nil {
		add("description", *req.Description)
	}
	if req.WorkThresholdPercent != nil {
		add("work_threshold_percent", *req.WorkThresholdPercent)
	}
	if req.IsCreditOnWork != nil {
		add("is_credit_on_work", *req.IsCreditOnWork)
	}
	if req.LocationID != nil {
		add("location_id", *req.LocationID)
	}
	set = append(set, "updated_at = NOW()")
	args = append(args, holidayID)
	row := db.QueryRow(ctx, `UPDATE holidays SET `+joinHolidayStrings(set, ", ")+` WHERE id = $`+fmt.Sprint(len(args))+` AND deleted_at IS NULL RETURNING `+holidayColumns, args...)
	record, err := scanHoliday(row)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.HolidayRecord{}, api.ErrHolidayNotFound
	}
	if isUniqueError(err) {
		return api.HolidayRecord{}, api.ErrHolidayAlreadyExists
	}
	return record, err
}

func (r HolidayRepository) DeleteHoliday(ctx context.Context, holidayID uuid.UUID) error {
	db, err := requireDB(r.DB, "holiday repository")
	if err != nil {
		return err
	}
	result, err := db.Exec(ctx, `UPDATE holidays SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL`, holidayID)
	if err != nil {
		return err
	}
	if result.RowsAffected() == 0 {
		return api.ErrHolidayNotFound
	}
	return EnqueueRosterCacheInvalidation(ctx, db)
}

func isUniqueError(err error) bool {
	var pgErr *pgconn.PgError
	return errors.As(err, &pgErr) && (pgErr.Code == "23505" || pgErr.Code == "23P01")
}

func joinHolidayStrings(items []string, separator string) string {
	result := ""
	for i, item := range items {
		if i > 0 {
			result += separator
		}
		result += item
	}
	return result
}

func (r HolidayRepository) ProcessHolidayLeave(ctx context.Context, holidayID uuid.UUID) ([]api.HolidayLeaveResult, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return nil, err
	}
	var results []api.HolidayLeaveResult
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var holidayDate, holidayName string
		var isPaid bool
		var leaveTypeID *uuid.UUID
		if err := db.QueryRow(ctx, `SELECT date::text, name, is_paid, leave_type_id FROM holidays WHERE id = $1 AND deleted_at IS NULL FOR UPDATE`, holidayID).Scan(&holidayDate, &holidayName, &isPaid, &leaveTypeID); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrHolidayNotFound
			}
			return err
		}
		if isPaid {
			results = []api.HolidayLeaveResult{{Type: "Paid"}}
			return nil
		}
		if leaveTypeID == nil {
			var fallback uuid.UUID
			err := db.QueryRow(ctx, `SELECT id FROM leave_types WHERE is_paid = TRUE AND deleted_at IS NULL ORDER BY created_at, id LIMIT 1`).Scan(&fallback)
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrLeaveTypeNotFound
			}
			if err != nil {
				return err
			}
			leaveTypeID = &fallback
		}
		var leaveTypeName string
		if err := db.QueryRow(ctx, `SELECT name FROM leave_types WHERE id = $1 AND is_paid = TRUE AND deleted_at IS NULL`, *leaveTypeID).Scan(&leaveTypeName); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrLeaveTypeNotFound
			}
			return err
		}
		rows, err := db.Query(ctx, `SELECT id FROM employees WHERE deleted_at IS NULL`)
		if err != nil {
			return err
		}
		defer rows.Close()
		for rows.Next() {
			var employeeID uuid.UUID
			if err := rows.Scan(&employeeID); err != nil {
				return err
			}
			var employmentID uuid.UUID
			err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND start_date <= $2::date AND (end_date IS NULL OR end_date >= $2::date) AND deleted_at IS NULL ORDER BY start_date DESC, id LIMIT 1`, employeeID, holidayDate).Scan(&employmentID)
			if errors.Is(err, pgx.ErrNoRows) {
				results = append(results, api.HolidayLeaveResult{Type: "Unpaid", EmployeeID: &employeeID})
				continue
			}
			if err != nil {
				return err
			}
			var alreadyOnLeave bool
			if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM leave_requests WHERE employment_id = $1 AND status = 'approved' AND start_date <= $2::date AND end_date >= $2::date)`, employmentID, holidayDate).Scan(&alreadyOnLeave); err != nil {
				return err
			}
			if alreadyOnLeave {
				continue
			}
			var entitlement, carriedOver int
			var accruedText string
			if err := db.QueryRow(ctx, `SELECT COALESCE(entitlement, 0), COALESCE(carried_over, 0), COALESCE(accrued_amount::text, '0') FROM leave_balances WHERE employment_id = $1 AND leave_type_id = $2 AND year = EXTRACT(YEAR FROM $3::date)::int`, employmentID, *leaveTypeID, holidayDate).Scan(&entitlement, &carriedOver, &accruedText); err != nil && !errors.Is(err, pgx.ErrNoRows) {
				return err
			}
			accrued := decimal.Zero
			if accruedText != "" {
				accrued, _ = decimal.NewFromString(accruedText)
			}
			var used int64
			if err := db.QueryRow(ctx, `SELECT count(*) FROM leave_requests WHERE employment_id = $1 AND leave_type_id = $2 AND status = 'approved' AND EXTRACT(YEAR FROM start_date) = EXTRACT(YEAR FROM $3::date)`, employmentID, *leaveTypeID, holidayDate).Scan(&used); err != nil {
				return err
			}
			remaining := decimal.NewFromInt(int64(entitlement + carriedOver)).Add(accrued).Sub(decimal.NewFromInt(used))
			if remaining.LessThan(decimal.NewFromInt(1)) {
				results = append(results, api.HolidayLeaveResult{Type: "Unpaid", EmployeeID: &employeeID})
				continue
			}
			reason := "Nghỉ lễ (trừ phép): " + holidayName
			requestID := uuid.New()
			if _, err := db.Exec(ctx, `INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, reason, status, duration_type, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $4::date, $5, 'approved', 'full_day', NOW(), NOW())`, requestID, employmentID, *leaveTypeID, holidayDate, reason); err != nil {
				return err
			}
			if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{holidayDate}); err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `INSERT INTO shift_rosters (id, employment_id, date, status, notes, created_at, updated_at) VALUES ($1, $2, $3::date, 'on_leave', $4, NOW(), NOW()) ON CONFLICT (employment_id, date) DO UPDATE SET status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = NOW()`, uuid.New(), employmentID, holidayDate, reason); err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `INSERT INTO leave_accrual_logs (id, employment_id, leave_type_id, accrual_date, amount, notes, created_at) VALUES ($1, $2, $3, $4::date, 0.0, $5, NOW()) ON CONFLICT (employment_id, leave_type_id, accrual_date) DO NOTHING`, uuid.New(), employmentID, *leaveTypeID, holidayDate, "Auto-deducted: Holiday "+holidayName); err != nil {
				return err
			}
			results = append(results, api.HolidayLeaveResult{Type: "LeaveDeducted", EmployeeID: &employeeID, LeaveRequestID: &requestID, LeaveTypeName: &leaveTypeName})
		}
		if err := rows.Err(); err != nil {
			return err
		}
		return EnqueueRosterCacheInvalidation(ctx, db)
	})
	return results, err
}
