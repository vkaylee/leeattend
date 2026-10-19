package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
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

func (r WorkspaceRepository) CreateWorkspace(ctx context.Context, req api.CreateWorkspaceRequest, ownerID uuid.UUID) (api.WorkspaceRecord, error) {
	if r.SystemDB == nil {
		return api.WorkspaceRecord{}, errors.New("workspace repository: nil system database")
	}
	workspaceID := uuid.New()
	ownerRoleID := uuid.New()
	if _, err := r.SystemDB.Exec(ctx, `INSERT INTO workspaces (id, name) VALUES ($1, $2)`, workspaceID, strings.TrimSpace(req.Name)); err != nil {
		return api.WorkspaceRecord{}, mapConstraintError(err)
	}
	if _, err := r.SystemDB.Exec(ctx, `INSERT INTO workspace_roles (id, workspace_id, name, description, color, is_system, scope_type) VALUES ($1, $2, 'owner', 'Workspace Owner', 'indigo', true, 'scoped')`, ownerRoleID, workspaceID); err != nil {
		return api.WorkspaceRecord{}, mapConstraintError(err)
	}
	if _, err := r.SystemDB.Exec(ctx, `INSERT INTO workspace_members (workspace_id, user_id, role_id) VALUES ($1, $2, $3)`, workspaceID, ownerID, ownerRoleID); err != nil {
		return api.WorkspaceRecord{}, mapConstraintError(err)
	}
	return r.getWorkspace(ctx, workspaceID)
}

func (r WorkspaceRepository) getWorkspace(ctx context.Context, id uuid.UUID) (api.WorkspaceRecord, error) {
	if r.SystemDB == nil {
		return api.WorkspaceRecord{}, errors.New("workspace repository: nil system database")
	}
	var record api.WorkspaceRecord
	err := r.SystemDB.QueryRow(ctx, `SELECT id, name, date_format, time_format, timezone, COALESCE(theme_primary_color, '#7c3aed'), default_language, logo_url, tagline, created_at FROM workspaces WHERE id = $1`, id).Scan(&record.ID, &record.Name, &record.DateFormat, &record.TimeFormat, &record.Timezone, &record.ThemePrimaryColor, &record.DefaultLanguage, &record.LogoURL, &record.Tagline, &record.CreatedAt)
	if err != nil {
		return api.WorkspaceRecord{}, err
	}
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
	_, err = db.Exec(ctx, `INSERT INTO workspace_config_entries (workspace_id, scope_type, scope_id, config_key, effective_from, effective_to, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7)`, workspaceID, req.ScopeType, req.ScopeID, req.ConfigKey, req.EffectiveFrom, req.EffectiveTo, actorID)
	return mapConstraintError(err)
}

func (r WorkspaceRepository) UpdateFeatureFlag(ctx context.Context, workspaceID, flagID uuid.UUID, patch api.FeatureFlagPatch) (api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if db == nil {
		return api.FeatureFlagRecord{}, errors.New("workspace repository: nil tenant database")
	}
	var record api.FeatureFlagRecord
	err = db.QueryRow(ctx, `UPDATE feature_flags SET enabled = $1 WHERE id = $2 AND workspace_id = $3 RETURNING id, workspace_id, flag_key, enabled, description`, *patch.Enabled, flagID, workspaceID).Scan(&record.ID, &record.WorkspaceID, &record.Key, &record.Enabled, &record.Description)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	return record, nil
}

func (r WorkspaceRepository) GetFeatureFlag(ctx context.Context, workspaceID uuid.UUID, key string) (api.FeatureFlagRecord, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return api.FeatureFlagRecord{}, err
	}
	if db == nil {
		return api.FeatureFlagRecord{}, errors.New("workspace repository: nil tenant database")
	}
	var record api.FeatureFlagRecord
	err = db.QueryRow(ctx, `SELECT id, workspace_id, flag_key, enabled, description FROM feature_flags WHERE workspace_id = $1 AND flag_key = $2`, workspaceID, key).Scan(&record.ID, &record.WorkspaceID, &record.Key, &record.Enabled, &record.Description)
	return record, err
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
