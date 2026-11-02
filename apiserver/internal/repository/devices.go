package repository

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// DeviceRepository provides database access to workspace devices and pull sources.
type DeviceRepository struct {
	SystemDB DBTX
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r DeviceRepository) tenant(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("tenant db not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

// ListDevices returns all devices in workspace tenant DB.
func (r DeviceRepository) ListDevices(ctx context.Context, workspaceID uuid.UUID) ([]api.Device, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	query := `SELECT id, agent_id, name, ip, port, brand, password, timezone, is_enabled, last_status, created_at, workspace_id, location_id FROM devices WHERE workspace_id = $1 ORDER BY created_at DESC`
	rows, err := db.Query(ctx, query, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var list []api.Device
	for rows.Next() {
		var d api.Device
		if err := rows.Scan(&d.ID, &d.AgentID, &d.Name, &d.IP, &d.Port, &d.Brand, &d.Password, &d.Timezone, &d.IsEnabled, &d.LastStatus, &d.CreatedAt, &d.WorkspaceID, &d.LocationID); err != nil {
			return nil, err
		}
		list = append(list, d)
	}
	return list, rows.Err()
}

// ListPullDevices returns enabled devices. Provider ID equals device UUID string by contract.
func (r DeviceRepository) ListPullDevices(ctx context.Context, workspaceID uuid.UUID) ([]service.PullDevice, error) {
	devices, err := r.ListDevices(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	result := make([]service.PullDevice, 0, len(devices))
	for _, device := range devices {
		if device.IsEnabled {
			result = append(result, service.PullDevice{ID: device.ID, ProviderID: device.ID.String()})
		}
	}
	return result, nil
}

// CreateDevice inserts a device into tenant DB.
func (r DeviceRepository) CreateDevice(ctx context.Context, dev api.Device) (uuid.UUID, error) {
	db, err := r.tenant(ctx, dev.WorkspaceID)
	if err != nil {
		return uuid.Nil, err
	}
	query := `INSERT INTO devices (id, agent_id, name, ip, port, brand, password, timezone, is_enabled, last_status, created_at, workspace_id, location_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, CURRENT_TIMESTAMP, $11, $12) RETURNING id`
	var id uuid.UUID
	err = db.QueryRow(ctx, query, dev.ID, dev.AgentID, dev.Name, dev.IP, dev.Port, dev.Brand, dev.Password, dev.Timezone, dev.IsEnabled, dev.LastStatus, dev.WorkspaceID, dev.LocationID).Scan(&id)
	return id, err
}

// GetDevice fetches a device by ID from tenant DB.
func (r DeviceRepository) GetDevice(ctx context.Context, workspaceID, id uuid.UUID) (*api.Device, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	query := `SELECT id, agent_id, name, ip, port, brand, password, timezone, is_enabled, last_status, created_at, workspace_id, location_id FROM devices WHERE id = $1 AND workspace_id = $2`
	var d api.Device
	err = db.QueryRow(ctx, query, id, workspaceID).Scan(&d.ID, &d.AgentID, &d.Name, &d.IP, &d.Port, &d.Brand, &d.Password, &d.Timezone, &d.IsEnabled, &d.LastStatus, &d.CreatedAt, &d.WorkspaceID, &d.LocationID)
	if err != nil {
		return nil, err
	}
	return &d, nil
}

// UpdateDevice updates device attributes in tenant DB.
func (r DeviceRepository) UpdateDevice(ctx context.Context, workspaceID, id uuid.UUID, req api.UpdateDeviceRequest) (*api.Device, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	existing, err := r.GetDevice(ctx, workspaceID, id)
	if err != nil {
		return nil, err
	}
	name := existing.Name
	if req.Name != nil && *req.Name != "" {
		name = *req.Name
	}
	ip := existing.IP
	if req.IP != nil && *req.IP != "" {
		ip = *req.IP
	}
	port := existing.Port
	if req.Port != nil {
		port = req.Port
	}
	brand := existing.Brand
	if req.Brand != nil && *req.Brand != "" {
		brand = *req.Brand
	}
	password := existing.Password
	if req.Password != nil {
		password = req.Password
	}
	timezone := existing.Timezone
	if req.Timezone != nil && *req.Timezone != "" {
		timezone = *req.Timezone
	}
	isEnabled := existing.IsEnabled
	if req.IsEnabled != nil {
		isEnabled = *req.IsEnabled
	}
	locationID := existing.LocationID
	if req.LocationID != nil {
		locationID = req.LocationID
	}

	query := `UPDATE devices SET name = $1, ip = $2, port = $3, brand = $4, password = $5, timezone = $6, is_enabled = $7, location_id = $8 WHERE id = $9 AND workspace_id = $10 RETURNING id, agent_id, name, ip, port, brand, password, timezone, is_enabled, last_status, created_at, workspace_id, location_id`
	var updated api.Device
	err = db.QueryRow(ctx, query, name, ip, port, brand, password, timezone, isEnabled, locationID, id, workspaceID).Scan(&updated.ID, &updated.AgentID, &updated.Name, &updated.IP, &updated.Port, &updated.Brand, &updated.Password, &updated.Timezone, &updated.IsEnabled, &updated.LastStatus, &updated.CreatedAt, &updated.WorkspaceID, &updated.LocationID)
	if err != nil {
		return nil, err
	}
	return &updated, nil
}

// DeleteDevice removes a device from tenant DB.
func (r DeviceRepository) DeleteDevice(ctx context.Context, workspaceID, id uuid.UUID) error {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return err
	}
	cmd, err := db.Exec(ctx, `DELETE FROM devices WHERE id = $1 AND workspace_id = $2`, id, workspaceID)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}
