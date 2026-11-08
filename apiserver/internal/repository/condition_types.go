package repository

import (
	"context"
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type ConditionTypeRepository struct {
	DB DBTX
}

func (r ConditionTypeRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("condition type repository: database is not configured")
	}
	return r.DB, nil
}

func (r ConditionTypeRepository) List(ctx context.Context, activeOnly bool) ([]api.ConditionTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	query := `SELECT
		id, code, name_i18n, category, default_minute_allowance,
		default_is_paid, affects_rules, icon, color, is_system, sort_order,
		is_active, created_at, updated_at
	FROM condition_type_definitions
	WHERE ($1 = FALSE OR is_active = TRUE)
	ORDER BY sort_order ASC, code ASC`
	rows, err := db.Query(ctx, query, activeOnly)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.ConditionTypeRecord, 0)
	for rows.Next() {
		var rec api.ConditionTypeRecord
		var created, updated time.Time
		if err := rows.Scan(
			&rec.ID, &rec.Code, &rec.NameI18N, &rec.Category,
			&rec.DefaultMinuteAllowance, &rec.DefaultIsPaid, &rec.AffectsRules,
			&rec.Icon, &rec.Color, &rec.IsSystem, &rec.SortOrder, &rec.IsActive,
			&created, &updated,
		); err != nil {
			return nil, err
		}
		rec.CreatedAt = created.UTC().Format(time.RFC3339)
		rec.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, rec)
	}
	return result, rows.Err()
}

func (r ConditionTypeRepository) Get(ctx context.Context, id uuid.UUID) (api.ConditionTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.ConditionTypeRecord{}, err
	}
	var rec api.ConditionTypeRecord
	var created, updated time.Time
	err = db.QueryRow(ctx, `SELECT
		id, code, name_i18n, category, default_minute_allowance,
		default_is_paid, affects_rules, icon, color, is_system, sort_order,
		is_active, created_at, updated_at
	FROM condition_type_definitions
	WHERE id = $1`, id).Scan(
		&rec.ID, &rec.Code, &rec.NameI18N, &rec.Category,
		&rec.DefaultMinuteAllowance, &rec.DefaultIsPaid, &rec.AffectsRules,
		&rec.Icon, &rec.Color, &rec.IsSystem, &rec.SortOrder, &rec.IsActive,
		&created, &updated,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ConditionTypeRecord{}, api.ErrConditionTypeNotFound
		}
		return api.ConditionTypeRecord{}, err
	}
	rec.CreatedAt = created.UTC().Format(time.RFC3339)
	rec.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return rec, nil
}
func (r ConditionTypeRepository) Create(ctx context.Context, record api.ConditionTypeRecord) (api.ConditionTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.ConditionTypeRecord{}, err
	}
	_, err = db.Exec(ctx, `INSERT INTO condition_type_definitions (
		id, code, name_i18n, category, default_minute_allowance,
		default_is_paid, affects_rules, icon, color, is_system, sort_order,
		is_active, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6,
		$7, $8, $9, $10, $11, $12,
		NOW(), NOW()
	)`, record.ID, record.Code, record.NameI18N, record.Category,
		record.DefaultMinuteAllowance, record.DefaultIsPaid, record.AffectsRules,
		record.Icon, record.Color, record.IsSystem, record.SortOrder, record.IsActive)
	if err != nil {
		if strings.Contains(err.Error(), "unique") || strings.Contains(err.Error(), "duplicate key") {
			return api.ConditionTypeRecord{}, api.ErrConditionTypeConflict
		}
		return api.ConditionTypeRecord{}, err
	}
	return r.Get(ctx, record.ID)
}

func (r ConditionTypeRepository) Update(ctx context.Context, id uuid.UUID, req api.UpdateConditionTypeRequest) (api.ConditionTypeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.ConditionTypeRecord{}, err
	}
	existing, err := r.Get(ctx, id)
	if err != nil {
		return api.ConditionTypeRecord{}, err
	}
	if req.NameI18N != nil {
		existing.NameI18N = *req.NameI18N
	}
	if req.Category != nil {
		existing.Category = *req.Category
	}
	if req.DefaultMinuteAllowance != nil {
		existing.DefaultMinuteAllowance = *req.DefaultMinuteAllowance
	}
	if req.DefaultIsPaid != nil {
		existing.DefaultIsPaid = *req.DefaultIsPaid
	}
	if req.AffectsRules != nil {
		existing.AffectsRules = *req.AffectsRules
	}
	if req.Icon != nil {
		existing.Icon = req.Icon
	}
	if req.Color != nil {
		existing.Color = req.Color
	}
	if req.SortOrder != nil {
		existing.SortOrder = *req.SortOrder
	}
	if req.IsActive != nil {
		existing.IsActive = *req.IsActive
	}
	_, err = db.Exec(ctx, `UPDATE condition_type_definitions SET
		name_i18n = $1, category = $2, default_minute_allowance = $3,
		default_is_paid = $4, affects_rules = $5, icon = $6, color = $7,
		sort_order = $8, is_active = $9, updated_at = NOW()
	WHERE id = $10`,
		existing.NameI18N, existing.Category, existing.DefaultMinuteAllowance,
		existing.DefaultIsPaid, existing.AffectsRules, existing.Icon, existing.Color,
		existing.SortOrder, existing.IsActive, id)
	if err != nil {
		return api.ConditionTypeRecord{}, err
	}
	return r.Get(ctx, id)
}

func (r ConditionTypeRepository) Delete(ctx context.Context, id uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	var isSystem bool
	err = db.QueryRow(ctx, `SELECT is_system FROM condition_type_definitions WHERE id = $1`, id).Scan(&isSystem)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrConditionTypeNotFound
		}
		return err
	}
	if isSystem {
		return api.ErrConditionTypeSystem
	}
	_, err = db.Exec(ctx, `DELETE FROM condition_type_definitions WHERE id = $1`, id)
	return err
}
