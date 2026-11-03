package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"strings"
	"time"
)

var ErrUniqueViolation = errors.New("repository: unique constraint violation")

type WorkspaceRepository struct {
	SystemDB DBTX
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r WorkspaceRepository) tenant(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return r.SystemDB, nil
	}
	return r.TenantDB(ctx, workspaceID)
}

func (r WorkspaceRepository) GetWorkspace(ctx context.Context, id uuid.UUID) (api.WorkspaceRecord, error) {
	return r.getWorkspace(ctx, id)
}

func (r WorkspaceRepository) getWorkspace(ctx context.Context, id uuid.UUID) (api.WorkspaceRecord, error) {
	if r.SystemDB == nil {
		return api.WorkspaceRecord{}, errors.New("workspace repository: nil system database")
	}
	var record api.WorkspaceRecord
	var createdAt time.Time
	err := r.SystemDB.QueryRow(ctx, `SELECT id, name, date_format, time_format, timezone, COALESCE(theme_primary_color, '#7c3aed'), default_language, logo_url, tagline, created_at FROM workspaces WHERE id = $1`, id).Scan(&record.ID, &record.Name, &record.DateFormat, &record.TimeFormat, &record.Timezone, &record.ThemePrimaryColor, &record.DefaultLanguage, &record.LogoURL, &record.Tagline, &createdAt)
	if err != nil {
		return api.WorkspaceRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.HolidayOvernight = "split_at_midnight"
	record.DefaultHolidayOT = "3.0"
	return record, nil
}

func (r WorkspaceRepository) UpdateWorkspace(ctx context.Context, id uuid.UUID, patch api.WorkspacePatch) (api.WorkspaceRecord, error) {
	if r.SystemDB == nil {
		return api.WorkspaceRecord{}, errors.New("workspace repository: nil system database")
	}
	sets, args := make([]string, 0, 8), make([]any, 0, 9)
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
	args = append(args, id)
	query := fmt.Sprintf("UPDATE workspaces SET %s WHERE id = $%d", strings.Join(sets, ", "), len(args))
	if _, err := r.SystemDB.Exec(ctx, query, args...); err != nil {
		return api.WorkspaceRecord{}, mapConstraintError(err)
	}
	return r.getWorkspace(ctx, id)
}

func (r WorkspaceRepository) CreateRole(ctx context.Context, workspaceID uuid.UUID, req api.CreateRoleRequest) error {
	if r.SystemDB == nil {
		return errors.New("workspace repository: nil system database")
	}
	_, err := r.SystemDB.Exec(ctx, `INSERT INTO workspace_roles (id, workspace_id, name, description, color, is_system, scope_type) VALUES ($1, $2, $3, $4, $5, false, $6)`, uuid.New(), workspaceID, req.Name, req.Description, req.Color, req.ScopeType)
	return mapConstraintError(err)
}

func (r WorkspaceRepository) PutConfig(ctx context.Context, workspaceID uuid.UUID, req api.ConfigEntryRequest, actorID uuid.UUID) error {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return err
	}
	if db == nil {
		return errors.New("workspace repository: nil tenant database")
	}
	scopeType := req.ScopeType
	if scopeType == "" {
		scopeType = "workspace"
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
	_, err = db.Exec(ctx, `INSERT INTO workspace_config_entries (
		id, workspace_id, scope_type, scope_id, config_key,
		value_int, value_decimal, value_text, value_bool, value_json,
		effective_from, effective_to, created_by, created_at, updated_at
	) VALUES (
		gen_random_uuid(), $1, $2, $3, $4,
		$5, $6, $7, $8, $9,
		$10::date, $11::date, $12, NOW(), NOW()
	) ON CONFLICT (workspace_id, scope_type, scope_id, config_key, effective_from)
	DO UPDATE SET
		value_int = EXCLUDED.value_int,
		value_decimal = EXCLUDED.value_decimal,
		value_text = EXCLUDED.value_text,
		value_bool = EXCLUDED.value_bool,
		value_json = EXCLUDED.value_json,
		effective_to = EXCLUDED.effective_to,
		updated_at = NOW()`,
		workspaceID, scopeType, req.ScopeID, req.ConfigKey,
		valInt, req.Value.Decimal, req.Value.Text, req.Value.Bool, valJSON,
		req.EffectiveFrom, req.EffectiveTo, actorID,
	)
	return mapConstraintError(err)
}

func (r WorkspaceRepository) ListConfigs(ctx context.Context, workspaceID uuid.UUID) ([]api.WorkspaceConfigEntry, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	if db == nil {
		return nil, errors.New("workspace repository: nil tenant database")
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, scope_type, scope_id, config_key,
		value_int, value_decimal, value_text, value_bool, value_json,
		effective_from, effective_to, created_by, created_at, updated_at
FROM workspace_config_entries
WHERE workspace_id = $1
ORDER BY created_at DESC`, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var entries []api.WorkspaceConfigEntry
	for rows.Next() {
		var entry api.WorkspaceConfigEntry
		var effectiveFrom time.Time
		var effectiveTo *time.Time
		var createdAt, updatedAt time.Time
		var valJSON []byte
		if err := rows.Scan(
			&entry.ID, &entry.WorkspaceID, &entry.ScopeType, &entry.ScopeID, &entry.ConfigKey,
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
		entries = []api.WorkspaceConfigEntry{}
	}
	return entries, nil
}

func (r WorkspaceRepository) GetConfig(ctx context.Context, workspaceID uuid.UUID, key string) (api.ResolvedConfig, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.ResolvedConfig{}, err
	}
	if db == nil {
		return api.ResolvedConfig{}, errors.New("workspace repository: nil tenant database")
	}
	var entry api.WorkspaceConfigEntry
	var effectiveTo *time.Time
	var valJSON []byte
	row := db.QueryRow(ctx, `SELECT id, workspace_id, scope_type, scope_id, config_key,
		value_int, value_decimal, value_text, value_bool, value_json, effective_to
FROM workspace_config_entries
WHERE workspace_id = $1 AND config_key = $2 AND scope_type = 'workspace'
  AND effective_from <= CURRENT_DATE
ORDER BY effective_from DESC LIMIT 1`, workspaceID, key)
	err = row.Scan(&entry.ID, &entry.WorkspaceID, &entry.ScopeType, &entry.ScopeID, &entry.ConfigKey,
		&entry.ValueInt, &entry.ValueDecimal, &entry.ValueText, &entry.ValueBool, &valJSON, &effectiveTo)
	if err == nil {
		now := time.Now().UTC().Truncate(24 * time.Hour)
		if effectiveTo == nil || !now.After(*effectiveTo) {
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
	}
	mockText := "mock"
	return api.ResolvedConfig{
		Key:       key,
		Value:     api.ConfigValue{Text: &mockText},
		ScopeType: "workspace",
		ScopeID:   nil,
	}, nil
}

func (r WorkspaceRepository) DeleteConfig(ctx context.Context, workspaceID, entryID uuid.UUID) error {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return err
	}
	if db == nil {
		return errors.New("workspace repository: nil tenant database")
	}
	_, err = db.Exec(ctx, `DELETE FROM workspace_config_entries WHERE id = $1 AND workspace_id = $2`, entryID, workspaceID)
	return mapConstraintError(err)
}

func scanFeatureFlag(row pgx.Row) (api.FeatureFlagRecord, error) {
	var record api.FeatureFlagRecord
	var metadata []byte
	var createdAt, updatedAt time.Time
	err := row.Scan(
		&record.ID, &record.WorkspaceID, &record.FlagKey, &record.Enabled,
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

const featureFlagColumns = `id, workspace_id, flag_key, enabled, description, metadata, created_at, updated_at`

func (r WorkspaceRepository) ListFeatureFlags(ctx context.Context, workspaceID uuid.UUID) ([]api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	if db == nil {
		return nil, errors.New("workspace repository: nil tenant database")
	}
	rows, err := db.Query(ctx, `SELECT `+featureFlagColumns+` FROM feature_flags WHERE workspace_id = $1 ORDER BY flag_key ASC`, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	flags := make([]api.FeatureFlagRecord, 0)
	for rows.Next() {
		var record api.FeatureFlagRecord
		var metadata []byte
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&record.ID, &record.WorkspaceID, &record.FlagKey, &record.Enabled, &record.Description, &metadata, &createdAt, &updatedAt); err != nil {
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

func (r WorkspaceRepository) CreateFeatureFlag(ctx context.Context, workspaceID uuid.UUID, req api.FeatureFlagRequest) (api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if db == nil {
		return api.FeatureFlagRecord{}, errors.New("workspace repository: nil tenant database")
	}
	metadata := req.Metadata
	if len(metadata) == 0 {
		metadata = json.RawMessage(`{}`)
	}
	return scanFeatureFlag(db.QueryRow(ctx, `INSERT INTO feature_flags (id, workspace_id, flag_key, enabled, description, metadata) VALUES ($1, $2, $3, $4, $5, $6) RETURNING `+featureFlagColumns, uuid.New(), workspaceID, req.FlagKey, req.Enabled, req.Description, metadata))
}

func (r WorkspaceRepository) UpdateFeatureFlag(ctx context.Context, workspaceID, flagID uuid.UUID, patch api.FeatureFlagPatch) (api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if db == nil {
		return api.FeatureFlagRecord{}, errors.New("workspace repository: nil tenant database")
	}
	sets := []string{"updated_at = NOW()"}
	args := make([]any, 0, 5)
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
	args = append(args, flagID, workspaceID)
	query := fmt.Sprintf(`UPDATE feature_flags SET %s WHERE id = $%d AND workspace_id = $%d RETURNING %s`, strings.Join(sets, ", "), len(args)-1, len(args), featureFlagColumns)
	return scanFeatureFlag(db.QueryRow(ctx, query, args...))
}

func (r WorkspaceRepository) GetFeatureFlag(ctx context.Context, workspaceID uuid.UUID, key string) (api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if db == nil {
		return api.FeatureFlagRecord{}, errors.New("workspace repository: nil tenant database")
	}
	return scanFeatureFlag(db.QueryRow(ctx, `SELECT `+featureFlagColumns+` FROM feature_flags WHERE workspace_id = $1 AND flag_key = $2`, workspaceID, key))
}

func (r WorkspaceRepository) GetFeatureFlagByID(ctx context.Context, workspaceID, flagID uuid.UUID) (api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if db == nil {
		return api.FeatureFlagRecord{}, errors.New("workspace repository: nil tenant database")
	}
	return scanFeatureFlag(db.QueryRow(ctx, `SELECT `+featureFlagColumns+` FROM feature_flags WHERE workspace_id = $1 AND id = $2`, workspaceID, flagID))
}

func (r WorkspaceRepository) DeleteFeatureFlag(ctx context.Context, workspaceID, flagID uuid.UUID) (bool, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return false, err
	}
	if db == nil {
		return false, errors.New("workspace repository: nil tenant database")
	}
	tag, err := db.Exec(ctx, `DELETE FROM feature_flags WHERE workspace_id = $1 AND id = $2`, workspaceID, flagID)
	return tag.RowsAffected() > 0, err
}

func (r WorkspaceRepository) UpsertFeatureFlag(ctx context.Context, workspaceID uuid.UUID, req api.FeatureFlagRequest) (api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if db == nil {
		return api.FeatureFlagRecord{}, errors.New("workspace repository: nil tenant database")
	}
	metadata := req.Metadata
	if len(metadata) == 0 {
		metadata = json.RawMessage(`{}`)
	}
	return scanFeatureFlag(db.QueryRow(ctx, `INSERT INTO feature_flags (id, workspace_id, flag_key, enabled, description, metadata) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (workspace_id, flag_key) DO UPDATE SET enabled = EXCLUDED.enabled, description = EXCLUDED.description, metadata = EXCLUDED.metadata, updated_at = NOW() RETURNING `+featureFlagColumns, uuid.New(), workspaceID, req.FlagKey, req.Enabled, req.Description, metadata))
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
