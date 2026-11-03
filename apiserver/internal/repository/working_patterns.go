package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type WorkingPatternRepository struct {
	DB      DBTX
	BeginTx func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r WorkingPatternRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("working pattern repository: tenant database is not configured")
	}
	return r.DB, nil
}

func (r WorkingPatternRepository) begin(ctx context.Context, workspaceID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, workspaceID)
	}
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("working pattern repository: tenant database does not support transactions")
	}
	return beginner, nil
}

func (r WorkingPatternRepository) CreatePattern(ctx context.Context, workspaceID uuid.UUID, req api.CreateWorkingPatternRequest) (api.WorkingPattern, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.WorkingPattern{}, err
	}
	patternID := uuid.New()
	now := time.Now().UTC()
	patternType := req.PatternType
	if patternType == "" {
		patternType = "weekly"
	}
	var pattern api.WorkingPattern
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		_, err := db.Exec(ctx, `INSERT INTO working_patterns (id, workspace_id, name, description, is_active, pattern_type, cycle_length_days, created_at, updated_at) VALUES ($1, $2, $3, $4, true, $5, $6, $7, $7)`, patternID, workspaceID, req.Name, req.Description, patternType, req.CycleLengthDays, now)
		if err != nil {
			return err
		}
		for _, d := range req.Details {
			_, err := db.Exec(ctx, `INSERT INTO working_pattern_details (id, pattern_id, schedule_day, template_id) VALUES ($1, $2, $3, $4)`, uuid.New(), patternID, d.ScheduleDay, d.ShiftID)
			if err != nil {
				return err
			}
		}
		pattern = api.WorkingPattern{
			ID:              patternID,
			WorkspaceID:     workspaceID,
			Name:            req.Name,
			Description:     req.Description,
			IsActive:        true,
			PatternType:     patternType,
			CycleLengthDays: req.CycleLengthDays,
			CreatedAt:       now.Format(time.RFC3339),
			UpdatedAt:       now.Format(time.RFC3339),
		}
		return nil
	})
	if err != nil {
		return api.WorkingPattern{}, err
	}
	return pattern, nil
}

func (r WorkingPatternRepository) ListPatterns(ctx context.Context, workspaceID uuid.UUID) ([]api.WorkingPattern, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, name, description, is_active, pattern_type, cycle_length_days, created_at, updated_at FROM working_patterns WHERE workspace_id = $1 AND is_active = true AND deleted_at IS NULL ORDER BY name ASC`, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	patterns := make([]api.WorkingPattern, 0)
	for rows.Next() {
		var p api.WorkingPattern
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&p.ID, &p.WorkspaceID, &p.Name, &p.Description, &p.IsActive, &p.PatternType, &p.CycleLengthDays, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		p.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		p.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		patterns = append(patterns, p)
	}
	return patterns, rows.Err()
}

func (r WorkingPatternRepository) GetPatternDetails(ctx context.Context, workspaceID, patternID uuid.UUID) ([]api.WorkingPatternDetail, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM working_patterns WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, patternID, workspaceID).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, api.ErrPatternNotFound
	}
	rows, err := db.Query(ctx, `SELECT id, pattern_id, schedule_day, template_id FROM working_pattern_details WHERE pattern_id = $1 ORDER BY schedule_day ASC`, patternID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	details := make([]api.WorkingPatternDetail, 0)
	for rows.Next() {
		var d api.WorkingPatternDetail
		if err := rows.Scan(&d.ID, &d.PatternID, &d.ScheduleDay, &d.ShiftID); err != nil {
			return nil, err
		}
		details = append(details, d)
	}
	return details, rows.Err()
}

func (r WorkingPatternRepository) UpdatePattern(ctx context.Context, workspaceID, patternID uuid.UUID, req api.CreateWorkingPatternRequest) (api.WorkingPattern, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.WorkingPattern{}, err
	}
	now := time.Now().UTC()
	patternType := req.PatternType
	if patternType == "" {
		patternType = "weekly"
	}
	var pattern api.WorkingPattern
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		tag, err := db.Exec(ctx, `UPDATE working_patterns SET name = $1, description = $2, pattern_type = $3, cycle_length_days = $4, updated_at = $5 WHERE id = $6 AND workspace_id = $7 AND deleted_at IS NULL`, req.Name, req.Description, patternType, req.CycleLengthDays, now, patternID, workspaceID)
		if err != nil {
			return err
		}
		if tag.RowsAffected() == 0 {
			return api.ErrPatternNotFound
		}
		if _, err := db.Exec(ctx, `DELETE FROM working_pattern_details WHERE pattern_id = $1`, patternID); err != nil {
			return err
		}
		for _, d := range req.Details {
			if _, err := db.Exec(ctx, `INSERT INTO working_pattern_details (id, pattern_id, schedule_day, template_id) VALUES ($1, $2, $3, $4)`, uuid.New(), patternID, d.ScheduleDay, d.ShiftID); err != nil {
				return err
			}
		}
		pattern = api.WorkingPattern{
			ID:              patternID,
			WorkspaceID:     workspaceID,
			Name:            req.Name,
			Description:     req.Description,
			IsActive:        true,
			PatternType:     patternType,
			CycleLengthDays: req.CycleLengthDays,
			UpdatedAt:       now.Format(time.RFC3339),
		}
		return nil
	})
	if err != nil {
		return api.WorkingPattern{}, err
	}
	return pattern, nil
}

func (r WorkingPatternRepository) DeletePattern(ctx context.Context, workspaceID, patternID uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE working_patterns SET deleted_at = now() WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, patternID, workspaceID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrPatternNotFound
	}
	return nil
}

func (r WorkingPatternRepository) AssignPattern(ctx context.Context, workspaceID, employmentID uuid.UUID, req api.AssignPatternRequest) (api.EmployeeWorkingPattern, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.EmployeeWorkingPattern{}, err
	}
	now := time.Now().UTC()
	var assignment api.EmployeeWorkingPattern
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employmentID, workspaceID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrEmploymentNotFound
		}
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM working_patterns WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, req.PatternID, workspaceID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrPatternNotFound
		}
		id := uuid.New()
		_, err := db.Exec(ctx, `INSERT INTO employee_working_patterns (id, workspace_id, employment_id, pattern_id, start_date, end_date, anchor_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5::date, $6::date, $7::date, $8, $8)`, id, workspaceID, employmentID, req.PatternID, req.StartDate, req.EndDate, req.AnchorDate, now)
		if err != nil {
			return err
		}
		assignment = api.EmployeeWorkingPattern{
			ID:           id,
			WorkspaceID:  workspaceID,
			EmploymentID: employmentID,
			PatternID:    req.PatternID,
			StartDate:    req.StartDate,
			EndDate:      req.EndDate,
			AnchorDate:   req.AnchorDate,
			CreatedAt:    now.Format(time.RFC3339),
			UpdatedAt:    now.Format(time.RFC3339),
		}
		return nil
	})
	if err != nil {
		return api.EmployeeWorkingPattern{}, err
	}
	return assignment, nil
}

func (r WorkingPatternRepository) BulkAssignPattern(ctx context.Context, workspaceID uuid.UUID, req api.BulkAssignPatternRequest) (api.BulkAssignPatternResponse, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.BulkAssignPatternResponse{}, err
	}
	now := time.Now().UTC()
	var response api.BulkAssignPatternResponse
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var validCount int64
		if err := db.QueryRow(ctx, `SELECT count(*) FROM employments WHERE id = ANY($1) AND workspace_id = $2 AND deleted_at IS NULL`, req.EmploymentIDs, workspaceID).Scan(&validCount); err != nil {
			return err
		}
		if validCount != int64(len(req.EmploymentIDs)) {
			return api.ErrEmploymentNotFound
		}
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM working_patterns WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, req.PatternID, workspaceID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrPatternNotFound
		}
		rows, err := db.Query(ctx, `SELECT employment_id FROM employee_working_patterns WHERE workspace_id = $1 AND start_date = $2::date AND employment_id = ANY($3)`, workspaceID, req.StartDate, req.EmploymentIDs)
		if err != nil {
			return err
		}
		defer rows.Close()
		conflicts := make([]uuid.UUID, 0)
		for rows.Next() {
			var empID uuid.UUID
			if err := rows.Scan(&empID); err != nil {
				return err
			}
			conflicts = append(conflicts, empID)
		}
		if err := rows.Err(); err != nil {
			return err
		}
		if len(conflicts) > 0 {
			response = api.BulkAssignPatternResponse{
				SuccessCount:    0,
				FailedEmployees: conflicts,
			}
			return nil
		}
		for _, empID := range req.EmploymentIDs {
			_, err := db.Exec(ctx, `INSERT INTO employee_working_patterns (id, workspace_id, employment_id, pattern_id, start_date, end_date, anchor_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5::date, $6::date, $7::date, $8, $8)`, uuid.New(), workspaceID, empID, req.PatternID, req.StartDate, req.EndDate, req.AnchorDate, now)
			if err != nil {
				return err
			}
		}
		response = api.BulkAssignPatternResponse{
			SuccessCount:    len(req.EmploymentIDs),
			FailedEmployees: []uuid.UUID{},
		}
		return nil
	})
	if err != nil {
		return api.BulkAssignPatternResponse{}, err
	}
	return response, nil
}

func (r WorkingPatternRepository) EmploymentPatternHistory(ctx context.Context, workspaceID, employmentID uuid.UUID) ([]api.EmployeeWorkingPattern, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, employment_id, pattern_id, start_date::text, end_date::text, anchor_date::text, created_at, updated_at FROM employee_working_patterns WHERE workspace_id = $1 AND employment_id = $2 ORDER BY start_date DESC`, workspaceID, employmentID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	history := make([]api.EmployeeWorkingPattern, 0)
	for rows.Next() {
		var h api.EmployeeWorkingPattern
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&h.ID, &h.WorkspaceID, &h.EmploymentID, &h.PatternID, &h.StartDate, &h.EndDate, &h.AnchorDate, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		h.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		h.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		history = append(history, h)
	}
	return history, rows.Err()
}
