package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type LocationRepository struct {
	DB DBTX
}

func (r LocationRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("location repository: database is not configured")
	}
	return r.DB, nil
}

func scanLocation(row interface{ Scan(...any) error }) (api.WorkLocation, error) {
	var item api.WorkLocation
	var createdAt, updatedAt time.Time
	err := row.Scan(&item.ID, &item.WorkspaceID, &item.Name, &item.Address, &item.Timezone, &item.IsHeadquarters, &item.Status, &createdAt, &updatedAt)
	if err != nil {
		return api.WorkLocation{}, err
	}
	item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	item.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return item, nil
}

func (r LocationRepository) ListLocations(ctx context.Context, wsID uuid.UUID, page, limit int) (api.LocationListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LocationListResponse{}, err
	}
	res := api.LocationListResponse{Data: make([]api.WorkLocation, 0), Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM work_locations WHERE workspace_id = $1 AND status = 'active'`, wsID).Scan(&res.Total); err != nil {
		return api.LocationListResponse{}, err
	}
	offset := (page - 1) * limit
	rows, err := db.Query(ctx, `SELECT id, workspace_id, name, address, timezone, is_headquarters, status, created_at, updated_at FROM work_locations WHERE workspace_id = $1 AND status = 'active' ORDER BY is_headquarters DESC, name ASC LIMIT $2 OFFSET $3`, wsID, limit, offset)
	if err != nil {
		return api.LocationListResponse{}, err
	}
	defer rows.Close()
	for rows.Next() {
		item, err := scanLocation(rows)
		if err != nil {
			return api.LocationListResponse{}, err
		}
		res.Data = append(res.Data, item)
	}
	return res, rows.Err()
}

func (r LocationRepository) CreateLocation(ctx context.Context, wsID uuid.UUID, req api.CreateLocationRequest) (api.WorkLocation, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkLocation{}, err
	}
	locID := uuid.New()
	tz := "UTC"
	if req.Timezone != nil && strings.TrimSpace(*req.Timezone) != "" {
		tz = strings.TrimSpace(*req.Timezone)
	}
	status := "active"
	if req.Status != nil && strings.TrimSpace(*req.Status) != "" {
		status = strings.TrimSpace(*req.Status)
	}
	isHQ := false
	if req.IsHeadquarters != nil {
		isHQ = *req.IsHeadquarters
	}
	row := db.QueryRow(ctx, `INSERT INTO work_locations (id, workspace_id, name, address, timezone, is_headquarters, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) RETURNING id, workspace_id, name, address, timezone, is_headquarters, status, created_at, updated_at`, locID, wsID, req.Name, req.Address, tz, isHQ, status)
	return scanLocation(row)
}

func (r LocationRepository) GetLocation(ctx context.Context, wsID, id uuid.UUID) (api.WorkLocation, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkLocation{}, err
	}
	row := db.QueryRow(ctx, `SELECT id, workspace_id, name, address, timezone, is_headquarters, status, created_at, updated_at FROM work_locations WHERE workspace_id = $1 AND id = $2 AND status = 'active'`, wsID, id)
	loc, err := scanLocation(row)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.WorkLocation{}, api.ErrLocationNotFound
	}
	return loc, err
}

func (r LocationRepository) UpdateLocation(ctx context.Context, wsID, id uuid.UUID, req api.UpdateLocationRequest) (api.WorkLocation, error) {
	existing, err := r.GetLocation(ctx, wsID, id)
	if err != nil {
		return api.WorkLocation{}, err
	}
	if req.IsHeadquarters != nil {
		if existing.IsHeadquarters != *req.IsHeadquarters {
			return api.WorkLocation{}, api.ErrLocationHQCannotChange
		}
	}
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkLocation{}, err
	}
	sets := []string{"updated_at = CURRENT_TIMESTAMP"}
	args := []any{}
	add := func(col string, val any) {
		args = append(args, val)
		sets = append(sets, fmt.Sprintf("%s = $%d", col, len(args)))
	}
	if req.Name != nil {
		add("name", *req.Name)
	}
	if req.Address != nil {
		add("address", *req.Address)
	}
	if req.Timezone != nil {
		add("timezone", *req.Timezone)
	}
	if req.Status != nil {
		add("status", *req.Status)
	}
	args = append(args, id, wsID)
	query := fmt.Sprintf(`UPDATE work_locations SET %s WHERE id = $%d AND workspace_id = $%d AND status = 'active' RETURNING id, workspace_id, name, address, timezone, is_headquarters, status, created_at, updated_at`, strings.Join(sets, ", "), len(args)-1, len(args))
	loc, err := scanLocation(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.WorkLocation{}, api.ErrLocationNotFound
	}
	return loc, err
}

func (r LocationRepository) DeleteLocation(ctx context.Context, wsID, id uuid.UUID) error {
	existing, err := r.GetLocation(ctx, wsID, id)
	if err != nil {
		return err
	}
	if existing.IsHeadquarters {
		return api.ErrLocationHQCannotDelete
	}
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	var inUseCount int64
	err = db.QueryRow(ctx, `SELECT count(*) FROM employees e JOIN employments em ON em.employee_id = e.id WHERE e.workspace_id = $1 AND em.primary_location_id = $2 AND em.status = 'active' AND em.deleted_at IS NULL`, wsID, id).Scan(&inUseCount)
	if err != nil {
		return err
	}
	if inUseCount > 0 {
		return api.ErrLocationInUse
	}
	tag, err := db.Exec(ctx, `UPDATE work_locations SET status = 'deleted', updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND workspace_id = $2 AND status = 'active'`, id, wsID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrLocationNotFound
	}
	return nil
}
