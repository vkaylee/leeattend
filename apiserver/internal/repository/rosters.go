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

type RosterRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r RosterRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "roster repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("roster repository: database does not support transactions")
	}
	return beginner, nil
}

const rosterSelect = `SELECT id, employment_id, template_id, shift_version_id, date::text, status, notes, created_at, updated_at FROM shift_rosters`

func (r RosterRepository) ResolveActiveEmployment(ctx context.Context, employeeID uuid.UUID) (uuid.UUID, error) {
	db, err := requireDB(r.DB, "roster repository")
	if err != nil {
		return uuid.Nil, err
	}
	var employmentID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC, id LIMIT 1`, employeeID).Scan(&employmentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, api.ErrEmploymentNotFound
	}
	return employmentID, err
}

func (r RosterRepository) ListRosters(ctx context.Context, employmentID uuid.UUID, startDate, endDate string, page, limit int) (api.PaginatedShiftRosters, error) {
	db, err := requireDB(r.DB, "roster repository")
	if err != nil {
		return api.PaginatedShiftRosters{}, err
	}
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM shift_rosters WHERE employment_id = $1 AND date >= $2::date AND date <= $3::date`, employmentID, startDate, endDate).Scan(&total); err != nil {
		return api.PaginatedShiftRosters{}, err
	}
	rows, err := db.Query(ctx, rosterSelect+` WHERE employment_id = $1 AND date >= $2::date AND date <= $3::date ORDER BY date ASC, id ASC LIMIT $4 OFFSET $5`, employmentID, startDate, endDate, limit, (page-1)*limit)
	if err != nil {
		return api.PaginatedShiftRosters{}, err
	}
	defer rows.Close()
	data := make([]api.ShiftRoster, 0)
	for rows.Next() {
		roster, err := scanRoster(rows)
		if err != nil {
			return api.PaginatedShiftRosters{}, err
		}
		data = append(data, roster)
	}
	return api.PaginatedShiftRosters{Data: data, Total: total, Page: page, Limit: limit}, rows.Err()
}

func (r RosterRepository) UpsertRoster(ctx context.Context, req api.CreateRosterRequest) (api.ShiftRoster, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.ShiftRoster{}, err
	}
	id := uuid.New()
	now := time.Now().UTC()
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{req.EmploymentID}, []string{req.Date}); err != nil {
			return err
		}
		if err := validateRosterReferences(ctx, db, req.EmploymentID, req.ShiftID); err != nil {
			return err
		}
		shiftVersionID, err := resolveRosterShiftVersion(ctx, db, req.ShiftID, req.ShiftVersionID, req.Date)
		if err != nil {
			return err
		}
		_, err = db.Exec(ctx, `INSERT INTO shift_rosters (id, employment_id, template_id, shift_version_id, date, status, notes, created_at, updated_at) VALUES ($1, $2, $3, $4, $5::date, 'confirmed', $6, $7, $7) ON CONFLICT (employment_id, date) DO UPDATE SET template_id = EXCLUDED.template_id, shift_version_id = EXCLUDED.shift_version_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = EXCLUDED.updated_at`, id, req.EmploymentID, req.ShiftID, shiftVersionID, req.Date, req.Notes, now)
		if err != nil {
			return mapRosterDBError(err)
		}
		if _, scanErr := scanRoster(db.QueryRow(ctx, rosterSelect+` WHERE employment_id = $1 AND date = $2::date`, req.EmploymentID, req.Date)); scanErr != nil {
			return scanErr
		}
		return EnqueueRosterCacheInvalidation(ctx, db)
	})
	err = finishLifecycleMutation(ctx, tx, err)
	if err != nil {
		return api.ShiftRoster{}, err
	}
	return r.getRoster(ctx, req.EmploymentID, req.Date)
}

func (r RosterRepository) BulkUpsertRoster(ctx context.Context, employmentID uuid.UUID, items []api.RosterItem) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	now := time.Now().UTC()
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if err := validateRosterEmployment(ctx, db, employmentID); err != nil {
			return err
		}
		dates := make([]string, 0, len(items))
		for _, item := range items {
			dates = append(dates, item.Date)
		}
		if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, dates); err != nil {
			return err
		}
		for _, item := range items {
			shiftVersionID, err := resolveRosterShiftVersion(ctx, db, item.ShiftID, item.ShiftVersionID, item.Date)
			if err != nil {
				return err
			}
			if err := validateRosterReferences(ctx, db, employmentID, item.ShiftID); err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `INSERT INTO shift_rosters (id, employment_id, template_id, shift_version_id, date, status, notes, created_at, updated_at) VALUES ($1, $2, $3, $4, $5::date, 'confirmed', $6, $7, $7) ON CONFLICT (employment_id, date) DO UPDATE SET template_id = EXCLUDED.template_id, shift_version_id = EXCLUDED.shift_version_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = EXCLUDED.updated_at`, uuid.New(), employmentID, item.ShiftID, shiftVersionID, item.Date, item.Notes, now); err != nil {
				return mapRosterDBError(err)
			}
		}
		return EnqueueRosterCacheInvalidation(ctx, db)
	})
	return finishLifecycleMutation(ctx, tx, err)
}

func (r RosterRepository) DeleteRoster(ctx context.Context, employmentID uuid.UUID, date string) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if err := guardLifecycleDateMutation(ctx, db, []uuid.UUID{employmentID}, []string{date}); err != nil {
			return err
		}
		result, err := db.Exec(ctx, `DELETE FROM shift_rosters WHERE employment_id = $1 AND date = $2::date`, employmentID, date)
		if err != nil {
			return err
		}
		if result.RowsAffected() == 0 {
			return api.ErrRosterNotFound
		}
		return EnqueueRosterCacheInvalidation(ctx, db)
	})
	return finishLifecycleMutation(ctx, tx, err)
}

func (r RosterRepository) getRoster(ctx context.Context, employmentID uuid.UUID, date string) (api.ShiftRoster, error) {
	db, err := requireDB(r.DB, "roster repository")
	if err != nil {
		return api.ShiftRoster{}, err
	}
	return scanRoster(db.QueryRow(ctx, rosterSelect+` WHERE employment_id = $1 AND date = $2::date`, employmentID, date))
}

func scanRoster(row pgx.Row) (api.ShiftRoster, error) {
	var roster api.ShiftRoster
	var createdAt, updatedAt *time.Time
	if err := row.Scan(&roster.ID, &roster.EmploymentID, &roster.ShiftID, &roster.ShiftVersionID, &roster.Date, &roster.Status, &roster.Notes, &createdAt, &updatedAt); err != nil {
		return api.ShiftRoster{}, err
	}
	if createdAt != nil {
		roster.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	}
	if updatedAt != nil {
		roster.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	}
	return roster, nil
}

func validateRosterReferences(ctx context.Context, db DBTX, employmentID uuid.UUID, shiftID *uuid.UUID) error {
	if err := validateRosterEmployment(ctx, db, employmentID); err != nil {
		return err
	}
	if shiftID == nil {
		return nil
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_templates WHERE id = $1)`, *shiftID).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return api.ErrShiftNotFound
	}
	return nil
}

func resolveRosterShiftVersion(ctx context.Context, db DBTX, templateID, requestedVersionID *uuid.UUID, date string) (*uuid.UUID, error) {
	if templateID == nil {
		return nil, nil
	}
	if requestedVersionID != nil {
		var valid bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_versions WHERE id = $1 AND template_id = $2 AND effective_from <= $3::date AND (effective_to IS NULL OR effective_to >= $3::date))`, *requestedVersionID, *templateID, date).Scan(&valid); err != nil {
			return nil, err
		}
		if !valid {
			return nil, api.ErrRosterVersionConflict
		}
		return requestedVersionID, nil
	}
	rows, err := db.Query(ctx, `SELECT id FROM shift_versions WHERE template_id = $1 AND effective_from <= $2::date AND (effective_to IS NULL OR effective_to >= $2::date) ORDER BY effective_from DESC, id LIMIT 2`, *templateID, date)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var versions []uuid.UUID
	for rows.Next() {
		var versionID uuid.UUID
		if err := rows.Scan(&versionID); err != nil {
			return nil, err
		}
		versions = append(versions, versionID)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if len(versions) != 1 {
		return nil, api.ErrRosterVersionConflict
	}
	return &versions[0], nil
}

func validateRosterEmployment(ctx context.Context, db DBTX, employmentID uuid.UUID) error {
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND deleted_at IS NULL)`, employmentID).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return api.ErrEmploymentNotFound
	}
	return nil
}

func mapRosterDBError(err error) error {
	if isUniqueViolation(err) {
		return api.ErrRosterConflict
	}
	return err
}
