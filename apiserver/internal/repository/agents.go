package repository

import (
	"context"
	"errors"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// AgentDeviceRepository provides database access for edge agents and devices.
type AgentDeviceRepository struct {
	SystemDB DBTX
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
}

func (r AgentDeviceRepository) tenant(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("tenant db not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

// ListAuthorizedAgents queries authorized agents for a workspace.
func (r AgentDeviceRepository) ListAuthorizedAgents(ctx context.Context, workspaceID uuid.UUID, includeInactive bool) ([]api.AuthorizedAgent, error) {
	if r.SystemDB == nil {
		return nil, errors.New("system db not configured")
	}
	query := `SELECT id, name, is_trusted, last_ip, created_at, is_active, workspace_id, last_seen_at, max_concurrent_workers, success_cooldown_secs, retention_days FROM authorized_agents WHERE workspace_id = $1 AND ($2 = TRUE OR is_active = TRUE) ORDER BY created_at DESC`
	rows, err := r.SystemDB.Query(ctx, query, workspaceID, includeInactive)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var list []api.AuthorizedAgent
	for rows.Next() {
		var a api.AuthorizedAgent
		a.Token = api.MaskedToken
		if err := rows.Scan(&a.ID, &a.Name, &a.IsTrusted, &a.LastIP, &a.CreatedAt, &a.IsActive, &a.WorkspaceID, &a.LastSeenAt, &a.MaxConcurrentWorkers, &a.SuccessCooldownSecs, &a.RetentionDays); err != nil {
			return nil, err
		}
		list = append(list, a)
	}
	return list, rows.Err()
}

// CreateAuthorizedAgent inserts an authorized agent row into System DB.
func (r AgentDeviceRepository) CreateAuthorizedAgent(ctx context.Context, agent api.AuthorizedAgent) error {
	if r.SystemDB == nil {
		return errors.New("system db not configured")
	}
	query := `INSERT INTO authorized_agents (id, name, token, is_trusted, is_active, workspace_id, max_concurrent_workers, success_cooldown_secs, retention_days, created_at, last_seen_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`
	_, err := r.SystemDB.Exec(ctx, query, agent.ID, agent.Name, agent.Token, agent.IsTrusted, agent.IsActive, agent.WorkspaceID, agent.MaxConcurrentWorkers, agent.SuccessCooldownSecs, agent.RetentionDays)
	return err
}

// GetAuthorizedAgent retrieves a single authorized agent with masked token.
func (r AgentDeviceRepository) GetAuthorizedAgent(ctx context.Context, workspaceID uuid.UUID, id string) (*api.AuthorizedAgent, error) {
	if r.SystemDB == nil {
		return nil, errors.New("system db not configured")
	}
	query := `SELECT id, name, is_trusted, last_ip, created_at, is_active, workspace_id, last_seen_at, max_concurrent_workers, success_cooldown_secs, retention_days FROM authorized_agents WHERE id = $1 AND workspace_id = $2`
	var a api.AuthorizedAgent
	a.Token = api.MaskedToken
	err := r.SystemDB.QueryRow(ctx, query, id, workspaceID).Scan(&a.ID, &a.Name, &a.IsTrusted, &a.LastIP, &a.CreatedAt, &a.IsActive, &a.WorkspaceID, &a.LastSeenAt, &a.MaxConcurrentWorkers, &a.SuccessCooldownSecs, &a.RetentionDays)
	if err != nil {
		return nil, err
	}
	return &a, nil
}

// GetAuthorizedAgentByID retrieves an authorized agent including hashed token for auth.
func (r AgentDeviceRepository) GetAuthorizedAgentByID(ctx context.Context, id string) (*api.AuthorizedAgent, error) {
	if r.SystemDB == nil {
		return nil, errors.New("system db not configured")
	}
	query := `SELECT id, name, token, is_trusted, last_ip, created_at, is_active, workspace_id, last_seen_at, max_concurrent_workers, success_cooldown_secs, retention_days FROM authorized_agents WHERE id = $1`
	var a api.AuthorizedAgent
	err := r.SystemDB.QueryRow(ctx, query, id).Scan(&a.ID, &a.Name, &a.Token, &a.IsTrusted, &a.LastIP, &a.CreatedAt, &a.IsActive, &a.WorkspaceID, &a.LastSeenAt, &a.MaxConcurrentWorkers, &a.SuccessCooldownSecs, &a.RetentionDays)
	if err != nil {
		return nil, err
	}
	return &a, nil
}

// UpdateAuthorizedAgent updates an authorized agent row and returns masked representation.
func (r AgentDeviceRepository) UpdateAuthorizedAgent(ctx context.Context, workspaceID uuid.UUID, id string, req api.UpdateAuthorizedAgentRequest) (*api.AuthorizedAgent, error) {
	if r.SystemDB == nil {
		return nil, errors.New("system db not configured")
	}
	agent, err := r.GetAuthorizedAgentByID(ctx, id)
	if err != nil {
		return nil, err
	}
	if agent.WorkspaceID != workspaceID {
		return nil, pgx.ErrNoRows
	}
	name := agent.Name
	if req.Name != nil && *req.Name != "" {
		name = *req.Name
	}
	token := agent.Token
	if req.Token != nil && *req.Token != "" && *req.Token != api.MaskedToken {
		token = api.HashAgentToken(strings.TrimSpace(*req.Token))
	}
	isTrusted := agent.IsTrusted
	if req.IsTrusted != nil {
		isTrusted = *req.IsTrusted
	}
	isActive := agent.IsActive
	if req.IsActive != nil {
		isActive = *req.IsActive
	}
	maxWorkers := agent.MaxConcurrentWorkers
	if req.MaxConcurrentWorkers != nil {
		maxWorkers = *req.MaxConcurrentWorkers
	}
	cooldown := agent.SuccessCooldownSecs
	if req.SuccessCooldownSecs != nil {
		cooldown = *req.SuccessCooldownSecs
	}
	retention := agent.RetentionDays
	if req.RetentionDays != nil {
		retention = *req.RetentionDays
	}

	query := `UPDATE authorized_agents SET name = $1, token = $2, is_trusted = $3, is_active = $4, max_concurrent_workers = $5, success_cooldown_secs = $6, retention_days = $7 WHERE id = $8 AND workspace_id = $9 RETURNING id, name, is_trusted, last_ip, created_at, is_active, workspace_id, last_seen_at, max_concurrent_workers, success_cooldown_secs, retention_days`
	var updated api.AuthorizedAgent
	updated.Token = api.MaskedToken
	err = r.SystemDB.QueryRow(ctx, query, name, token, isTrusted, isActive, maxWorkers, cooldown, retention, id, workspaceID).Scan(&updated.ID, &updated.Name, &updated.IsTrusted, &updated.LastIP, &updated.CreatedAt, &updated.IsActive, &updated.WorkspaceID, &updated.LastSeenAt, &updated.MaxConcurrentWorkers, &updated.SuccessCooldownSecs, &updated.RetentionDays)
	if err != nil {
		return nil, err
	}
	return &updated, nil
}

// DeleteAuthorizedAgent deactivates an authorized agent.
func (r AgentDeviceRepository) DeleteAuthorizedAgent(ctx context.Context, workspaceID uuid.UUID, id string) error {
	if r.SystemDB == nil {
		return errors.New("system db not configured")
	}
	cmd, err := r.SystemDB.Exec(ctx, `UPDATE authorized_agents SET is_active = FALSE WHERE id = $1 AND workspace_id = $2`, id, workspaceID)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

// UpdateAgentTrust updates agent trust state.
func (r AgentDeviceRepository) UpdateAgentTrust(ctx context.Context, workspaceID uuid.UUID, id string, isTrusted bool) error {
	if r.SystemDB == nil {
		return errors.New("system db not configured")
	}
	cmd, err := r.SystemDB.Exec(ctx, `UPDATE authorized_agents SET is_trusted = $1 WHERE id = $2 AND workspace_id = $3`, isTrusted, id, workspaceID)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

// UpdateAgentLastIP updates agent's last recorded IP and timestamp.
func (r AgentDeviceRepository) UpdateAgentLastIP(ctx context.Context, id string, ip string) error {
	if r.SystemDB == nil {
		return errors.New("system db not configured")
	}
	_, err := r.SystemDB.Exec(ctx, `UPDATE authorized_agents SET last_ip = $1, last_seen_at = CURRENT_TIMESTAMP WHERE id = $2`, ip, id)
	return err
}

// AgentExistsInWorkspace verifies workspace agent membership.
func (r AgentDeviceRepository) AgentExistsInWorkspace(ctx context.Context, workspaceID uuid.UUID, id string) (bool, error) {
	if r.SystemDB == nil {
		return false, errors.New("system db not configured")
	}
	var exists bool
	err := r.SystemDB.QueryRow(ctx, `SELECT EXISTS(SELECT 1 FROM authorized_agents WHERE id = $1 AND workspace_id = $2 AND is_active = TRUE)`, id, workspaceID).Scan(&exists)
	return exists, err
}

// ListDevices returns all devices in workspace tenant DB.
func (r AgentDeviceRepository) ListDevices(ctx context.Context, workspaceID uuid.UUID) ([]api.Device, error) {
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
func (r AgentDeviceRepository) ListPullDevices(ctx context.Context, workspaceID uuid.UUID) ([]service.PullDevice, error) {
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

// ListDevicesByAgentID returns devices belonging to an agent in tenant DB.
func (r AgentDeviceRepository) ListDevicesByAgentID(ctx context.Context, workspaceID uuid.UUID, agentID string) ([]api.Device, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	query := `SELECT id, agent_id, name, ip, port, brand, password, timezone, is_enabled, last_status, created_at, workspace_id, location_id FROM devices WHERE workspace_id = $1 AND agent_id = $2 ORDER BY created_at DESC`
	rows, err := db.Query(ctx, query, workspaceID, agentID)
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

// CreateDevice inserts a device into tenant DB.
func (r AgentDeviceRepository) CreateDevice(ctx context.Context, dev api.Device) (uuid.UUID, error) {
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
func (r AgentDeviceRepository) GetDevice(ctx context.Context, workspaceID, id uuid.UUID) (*api.Device, error) {
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
func (r AgentDeviceRepository) UpdateDevice(ctx context.Context, workspaceID, id uuid.UUID, req api.UpdateDeviceRequest) (*api.Device, error) {
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

// DeleteDevice removes a device from tenant DB and returns associated agent_id.
func (r AgentDeviceRepository) DeleteDevice(ctx context.Context, workspaceID, id uuid.UUID) (string, error) {
	db, err := r.tenant(ctx, workspaceID)
	if err != nil {
		return "", err
	}
	existing, err := r.GetDevice(ctx, workspaceID, id)
	if err != nil {
		return "", err
	}
	cmd, err := db.Exec(ctx, `DELETE FROM devices WHERE id = $1 AND workspace_id = $2`, id, workspaceID)
	if err != nil {
		return "", err
	}
	if cmd.RowsAffected() == 0 {
		return "", pgx.ErrNoRows
	}
	return existing.AgentID, nil
}
