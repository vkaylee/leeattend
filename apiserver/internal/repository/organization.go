package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

var ErrUniqueViolation = errors.New("repository: unique constraint violation")

type OrganizationRepository struct {
	DB DBTX
}

func (r OrganizationRepository) db() (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("organization repository: database is not configured")
	}
	return r.DB, nil
}

func (r OrganizationRepository) GetOrganizationID(ctx context.Context) (uuid.UUID, error) {
	db, err := r.db()
	if err != nil {
		return uuid.Nil, err
	}
	var id uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM organization_settings LIMIT 1`).Scan(&id)
	return id, err
}

func (r OrganizationRepository) GetOrganization(ctx context.Context) (api.OrganizationRecord, error) {
	return r.getOrganization(ctx)
}

func (r OrganizationRepository) getOrganization(ctx context.Context) (api.OrganizationRecord, error) {
	db, err := r.db()
	if err != nil {
		return api.OrganizationRecord{}, err
	}
	var record api.OrganizationRecord
	var id uuid.UUID
	var createdAt time.Time
	err = db.QueryRow(ctx, `SELECT id, name, date_format, time_format, timezone, COALESCE(theme_primary_color, '#7c3aed'), default_language, logo_url, tagline, created_at FROM organization_settings LIMIT 1`).Scan(
		&id, &record.Name, &record.DateFormat, &record.TimeFormat, &record.Timezone,
		&record.ThemePrimaryColor, &record.DefaultLanguage, &record.LogoURL, &record.Tagline, &createdAt,
	)
	if err != nil {
		return api.OrganizationRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.HolidayOvernight = "split_at_midnight"
	record.DefaultHolidayOT = "3.0"
	return record, nil
}

func (r OrganizationRepository) UpdateOrganization(ctx context.Context, patch api.OrganizationPatch) (api.OrganizationRecord, error) {
	db, err := r.db()
	if err != nil {
		return api.OrganizationRecord{}, err
	}
	sets, args := make([]string, 0, 8), make([]any, 0, 8)
	add := func(column string, value any) {
		args = append(args, value)
		sets = append(sets, fmt.Sprintf("%s = $%d", column, len(args)))
	}
	if patch.Name != nil {
		add("name", *patch.Name)
	}
	if patch.DateFormat != nil {
		add("date_format", *patch.DateFormat)
	}
	if patch.TimeFormat != nil {
		add("time_format", *patch.TimeFormat)
	}
	if patch.Timezone != nil {
		add("timezone", *patch.Timezone)
	}
	if patch.ThemePrimaryColor != nil {
		add("theme_primary_color", *patch.ThemePrimaryColor)
	}
	if patch.DefaultLanguage != nil {
		add("default_language", *patch.DefaultLanguage)
	}
	if patch.LogoURL != nil {
		add("logo_url", *patch.LogoURL)
	}
	if patch.Tagline != nil {
		add("tagline", *patch.Tagline)
	}
	if len(sets) > 0 {
		query := fmt.Sprintf("UPDATE organization_settings SET %s", strings.Join(sets, ", "))
		if _, err := db.Exec(ctx, query, args...); err != nil {
			return api.OrganizationRecord{}, mapConstraintError(err)
		}
	}
	return r.getOrganization(ctx)
}

func (r OrganizationRepository) CreateRole(ctx context.Context, req api.CreateRoleRequest) error {
	db, err := r.db()
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO roles (id, name, description, color, is_system, scope_type) VALUES ($1, $2, $3, $4, false, $5)`, uuid.New(), req.Name, req.Description, req.Color, req.ScopeType)
	return mapConstraintError(err)
}

func (r OrganizationRepository) PutConfig(ctx context.Context, req api.ConfigEntryRequest, actorID uuid.UUID) error {
	db, err := r.db()
	if err != nil {
		return err
	}
	scopeType := req.ScopeType
	if scopeType == "" {
		scopeType = "organization"
	}
	scopeID := req.ScopeID
	if scopeType == "organization" {
		scopeID = nil
	}
	if scopeType == "employer" {
		if scopeID == nil || *scopeID == uuid.Nil {
			return api.ErrEmployerNotFound
		}
		var active bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employers WHERE id = $1 AND deleted_at IS NULL)`, *scopeID).Scan(&active); err != nil {
			return err
		}
		if !active {
			return api.ErrEmployerNotFound
		}
	}
	var valJSON any
	if len(req.Value.JSON) > 0 {
		valJSON = []byte(req.Value.JSON)
	}
	var valInt *int32
	if req.Value.Int != nil {
		v := int32(*req.Value.Int)
		valInt = &v
	}
	_, err = db.Exec(ctx, `INSERT INTO config_entries (
		id, scope_type, scope_id, config_key,
		value_int, value_decimal, value_text, value_bool, value_json,
		effective_from, effective_to, created_by, created_at, updated_at
	) VALUES (
		gen_random_uuid(), $1, $2, $3,
		$4, $5, $6, $7, $8,
		$9::date, $10::date, $11, NOW(), NOW()
	) ON CONFLICT (scope_type, scope_id, config_key, effective_from)
	DO UPDATE SET
		value_int = EXCLUDED.value_int,
		value_decimal = EXCLUDED.value_decimal,
		value_text = EXCLUDED.value_text,
		value_bool = EXCLUDED.value_bool,
		value_json = EXCLUDED.value_json,
		effective_to = EXCLUDED.effective_to,
		updated_at = NOW()`,
		scopeType, scopeID, req.ConfigKey,
		valInt, req.Value.Decimal, req.Value.Text, req.Value.Bool, valJSON,
		req.EffectiveFrom, req.EffectiveTo, actorID,
	)
	return mapConstraintError(err)
}

func (r OrganizationRepository) ListConfigs(ctx context.Context) ([]api.OrganizationConfigEntry, error) {
	db, err := r.db()
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, scope_type, scope_id, config_key,
		value_int, value_decimal, value_text, value_bool, value_json,
		effective_from, effective_to, created_by, created_at, updated_at
FROM config_entries
WHERE scope_type = 'organization' AND scope_id IS NULL
ORDER BY created_at DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var entries []api.OrganizationConfigEntry
	for rows.Next() {
		var entry api.OrganizationConfigEntry
		var effectiveFrom time.Time
		var effectiveTo *time.Time
		var createdAt, updatedAt time.Time
		var valJSON []byte
		if err := rows.Scan(
			&entry.ID, &entry.ScopeType, &entry.ScopeID, &entry.ConfigKey,
			&entry.ValueInt, &entry.ValueDecimal, &entry.ValueText, &entry.ValueBool, &valJSON,
			&effectiveFrom, &effectiveTo, &entry.CreatedBy, &createdAt, &updatedAt,
		); err != nil {
			return nil, err
		}
		entry.EffectiveFrom = effectiveFrom.Format("2006-01-02")
		if effectiveTo != nil {
			s := effectiveTo.Format("2006-01-02")
			entry.EffectiveTo = &s
		}
		if len(valJSON) > 0 {
			entry.ValueJSON = json.RawMessage(valJSON)
		}
		entry.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		entry.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		entries = append(entries, entry)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if entries == nil {
		entries = []api.OrganizationConfigEntry{}
	}
	return entries, nil
}

func (r OrganizationRepository) GetConfig(ctx context.Context, key string, request api.ConfigReadRequest) (api.ResolvedConfig, error) {
	db, err := r.db()
	if err != nil {
		return api.ResolvedConfig{}, err
	}
	var entry api.OrganizationConfigEntry
	var effectiveTo *time.Time
	var valJSON []byte
	scopeID := any(nil)
	if request.ScopeID != nil {
		scopeID = *request.ScopeID
	}
	row := db.QueryRow(ctx, `SELECT id, scope_type, scope_id, config_key,
		value_int, value_decimal, value_text, value_bool, value_json, effective_to
FROM config_entries
WHERE config_key = $1 AND scope_type = $2 AND scope_id IS NOT DISTINCT FROM $3::uuid
  AND effective_from <= $4::date
  AND (effective_to IS NULL OR effective_to >= $4::date)
ORDER BY effective_from DESC, id DESC LIMIT 1`, key, request.ScopeType, scopeID, request.AsOf.Format("2006-01-02"))
	err = row.Scan(&entry.ID, &entry.ScopeType, &entry.ScopeID, &entry.ConfigKey,
		&entry.ValueInt, &entry.ValueDecimal, &entry.ValueText, &entry.ValueBool, &valJSON, &effectiveTo)
	if err != nil {
		return api.ResolvedConfig{}, err
	}
	res := api.ResolvedConfig{
		Key:       key,
		ScopeType: entry.ScopeType,
		ScopeID:   entry.ScopeID,
	}
	if entry.ValueInt != nil {
		v := int64(*entry.ValueInt)
		res.Value.Int = &v
	} else if entry.ValueDecimal != nil {
		res.Value.Decimal = entry.ValueDecimal
	} else if entry.ValueText != nil {
		res.Value.Text = entry.ValueText
	} else if entry.ValueBool != nil {
		res.Value.Bool = entry.ValueBool
	} else if len(valJSON) > 0 {
		res.Value.JSON = json.RawMessage(valJSON)
	}
	return res, nil
}

func (r OrganizationRepository) DeleteConfig(ctx context.Context, entryID uuid.UUID) error {
	db, err := r.db()
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `DELETE FROM config_entries WHERE id = $1`, entryID)
	return mapConstraintError(err)
}

func scanFeatureFlag(row pgx.Row) (api.FeatureFlagRecord, error) {
	var record api.FeatureFlagRecord
	var metadata []byte
	var createdAt, updatedAt time.Time
	err := row.Scan(
		&record.ID, &record.FlagKey, &record.Enabled,
		&record.Description, &metadata, &createdAt, &updatedAt,
	)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if len(metadata) == 0 {
		metadata = []byte(`{}`)
	}
	record.Metadata = append(json.RawMessage(nil), metadata...)
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

const featureFlagColumns = `id, flag_key, enabled, description, metadata, created_at, updated_at`

func (r OrganizationRepository) ListFeatureFlags(ctx context.Context) ([]api.FeatureFlagRecord, error) {
	db, err := r.db()
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+featureFlagColumns+` FROM feature_flags ORDER BY flag_key ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	flags := make([]api.FeatureFlagRecord, 0)
	for rows.Next() {
		var record api.FeatureFlagRecord
		var metadata []byte
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&record.ID, &record.FlagKey, &record.Enabled, &record.Description, &metadata, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		if len(metadata) == 0 {
			metadata = []byte(`{}`)
		}
		record.Metadata = append(json.RawMessage(nil), metadata...)
		record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		flags = append(flags, record)
	}
	return flags, rows.Err()
}

func (r OrganizationRepository) CreateFeatureFlag(ctx context.Context, req api.FeatureFlagRequest) (api.FeatureFlagRecord, error) {
	db, err := r.db()
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	metadata := req.Metadata
	if len(metadata) == 0 {
		metadata = json.RawMessage(`{}`)
	}
	return scanFeatureFlag(db.QueryRow(ctx, `INSERT INTO feature_flags (id, flag_key, enabled, description, metadata) VALUES ($1, $2, $3, $4, $5) RETURNING `+featureFlagColumns, uuid.New(), req.FlagKey, req.Enabled, req.Description, metadata))
}

func (r OrganizationRepository) UpdateFeatureFlag(ctx context.Context, flagID uuid.UUID, patch api.FeatureFlagPatch) (api.FeatureFlagRecord, error) {
	db, err := r.db()
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	sets := []string{"updated_at = NOW()"}
	args := make([]any, 0, 4)
	if patch.Enabled != nil {
		args = append(args, *patch.Enabled)
		sets = append([]string{fmt.Sprintf("enabled = $%d", len(args))}, sets...)
	}
	if patch.Description != nil {
		args = append(args, *patch.Description)
		sets = append([]string{fmt.Sprintf("description = $%d", len(args))}, sets...)
	}
	if patch.Metadata != nil {
		args = append(args, *patch.Metadata)
		sets = append([]string{fmt.Sprintf("metadata = $%d", len(args))}, sets...)
	}
	args = append(args, flagID)
	query := fmt.Sprintf(`UPDATE feature_flags SET %s WHERE id = $%d RETURNING %s`, strings.Join(sets, ", "), len(args), featureFlagColumns)
	return scanFeatureFlag(db.QueryRow(ctx, query, args...))
}

func (r OrganizationRepository) GetFeatureFlag(ctx context.Context, key string) (api.FeatureFlagRecord, error) {
	db, err := r.db()
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	return scanFeatureFlag(db.QueryRow(ctx, `SELECT `+featureFlagColumns+` FROM feature_flags WHERE flag_key = $1`, key))
}

func (r OrganizationRepository) GetFeatureFlagByID(ctx context.Context, flagID uuid.UUID) (api.FeatureFlagRecord, error) {
	db, err := r.db()
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	return scanFeatureFlag(db.QueryRow(ctx, `SELECT `+featureFlagColumns+` FROM feature_flags WHERE id = $1`, flagID))
}

func (r OrganizationRepository) DeleteFeatureFlag(ctx context.Context, flagID uuid.UUID) (bool, error) {
	db, err := r.db()
	if err != nil {
		return false, err
	}
	tag, err := db.Exec(ctx, `DELETE FROM feature_flags WHERE id = $1`, flagID)
	return tag.RowsAffected() > 0, err
}

func (r OrganizationRepository) UpsertFeatureFlag(ctx context.Context, req api.FeatureFlagRequest) (api.FeatureFlagRecord, error) {
	db, err := r.db()
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	metadata := req.Metadata
	if len(metadata) == 0 {
		metadata = json.RawMessage(`{}`)
	}
	return scanFeatureFlag(db.QueryRow(ctx, `INSERT INTO feature_flags (id, flag_key, enabled, description, metadata) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (flag_key) DO UPDATE SET enabled = EXCLUDED.enabled, description = EXCLUDED.description, metadata = EXCLUDED.metadata, updated_at = NOW() RETURNING `+featureFlagColumns, uuid.New(), req.FlagKey, req.Enabled, req.Description, metadata))
}

func mapConstraintError(err error) error {
	if err == nil {
		return nil
	}
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) && pgErr.Code == "23505" {
		return fmt.Errorf("%w: %s", api.ErrConflict, pgErr.ConstraintName)
	}
	if errors.Is(err, pgx.ErrNoRows) {
		return err
	}
	return err
}
