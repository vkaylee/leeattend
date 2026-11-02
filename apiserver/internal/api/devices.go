package api

import (
	"context"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type Device struct {
	ID                   uuid.UUID  `json:"id"`
	AgentID              string     `json:"agent_id"`
	Name                 string     `json:"name"`
	IP                   string     `json:"ip"`
	Port                 *int       `json:"port"`
	Brand                string     `json:"brand"`
	Password             *string    `json:"password,omitempty"`
	Timezone             string     `json:"timezone"`
	IsEnabled            bool       `json:"is_enabled"`
	LastStatus           string     `json:"last_status"`
	CreatedAt            *time.Time `json:"created_at,omitempty"`
	WorkspaceID          uuid.UUID  `json:"workspace_id"`
	LocationID           *uuid.UUID `json:"location_id,omitempty"`
	MaxConcurrentWorkers int        `json:"max_concurrent_workers,omitempty"`
}

type CreateDeviceRequest struct {
	AgentID              string     `json:"agent_id"`
	Name                 string     `json:"name"`
	IP                   string     `json:"ip"`
	Port                 *int       `json:"port,omitempty"`
	Brand                *string    `json:"brand,omitempty"`
	Password             *string    `json:"password,omitempty"`
	Timezone             *string    `json:"timezone,omitempty"`
	WorkspaceID          *uuid.UUID `json:"workspace_id,omitempty"`
	LocationID           *uuid.UUID `json:"location_id,omitempty"`
	MaxConcurrentWorkers *int       `json:"max_concurrent_workers,omitempty"`
}

type UpdateDeviceRequest struct {
	Name       *string    `json:"name,omitempty"`
	IP         *string    `json:"ip,omitempty"`
	Port       *int       `json:"port,omitempty"`
	Brand      *string    `json:"brand,omitempty"`
	Password   *string    `json:"password,omitempty"`
	Timezone   *string    `json:"timezone,omitempty"`
	IsEnabled  *bool      `json:"is_enabled,omitempty"`
	LocationID *uuid.UUID `json:"location_id,omitempty"`
}

type DeviceRepository interface {
	ListDevices(context.Context, uuid.UUID) ([]Device, error)
	CreateDevice(context.Context, Device) (uuid.UUID, error)
	GetDevice(context.Context, uuid.UUID, uuid.UUID) (*Device, error)
	UpdateDevice(context.Context, uuid.UUID, uuid.UUID, UpdateDeviceRequest) (*Device, error)
	DeleteDevice(context.Context, uuid.UUID, uuid.UUID) error
}

type DeviceHandler struct {
	Repository DeviceRepository
	Authorizer WorkspaceAuthorizer
}

func resolveWorkspaceID(r *http.Request) (uuid.UUID, bool) {
	if wsID, ok := httpinfra.WorkspaceIDFromContext(r.Context()); ok && wsID != uuid.Nil {
		return wsID, true
	}
	if id, err := uuid.Parse(r.Header.Get("X-Workspace-Id")); err == nil && id != uuid.Nil {
		return id, true
	}
	if id, err := uuid.Parse(r.URL.Query().Get("workspace_id")); err == nil && id != uuid.Nil {
		return id, true
	}
	return uuid.Nil, false
}

func (h DeviceHandler) ListDevices(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	devices, err := h.Repository.ListDevices(r.Context(), wsID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if devices == nil {
		devices = []Device{}
	}
	apperror.WriteJSON(w, http.StatusOK, devices)
}

func (h DeviceHandler) CreateDevice(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	req, err := DecodeJSON[CreateDeviceRequest](r, 1<<20)
	if err != nil || req.AgentID == "" || req.Name == "" || req.IP == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	port, brand, tz, maxWorkers := 4370, "zkteco", "UTC", 2
	if req.Port != nil {
		port = *req.Port
	}
	if req.Brand != nil && *req.Brand != "" {
		brand = *req.Brand
	}
	if req.Timezone != nil && *req.Timezone != "" {
		tz = *req.Timezone
	}
	if req.MaxConcurrentWorkers != nil {
		maxWorkers = *req.MaxConcurrentWorkers
	}
	deviceID, err := h.Repository.CreateDevice(r.Context(), Device{ID: uuid.New(), AgentID: req.AgentID, Name: req.Name, IP: req.IP, Port: &port, Brand: brand, Password: req.Password, Timezone: tz, IsEnabled: true, LastStatus: "offline", WorkspaceID: wsID, LocationID: req.LocationID, MaxConcurrentWorkers: maxWorkers})
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, map[string]interface{}{"id": deviceID})
}

func (h DeviceHandler) UpdateDevice(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[UpdateDeviceRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	device, err := h.Repository.UpdateDevice(r.Context(), wsID, id, req)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		} else {
			apperror.WriteError(w, r, apperror.Internal(err))
		}
		return
	}
	apperror.WriteJSON(w, http.StatusOK, device)
}

func (h DeviceHandler) DeleteDevice(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	id, err := uuid.Parse(r.PathValue("id"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err = h.Repository.DeleteDevice(r.Context(), wsID, id); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		} else {
			apperror.WriteError(w, r, apperror.Internal(err))
		}
		return
	}
	w.WriteHeader(http.StatusNoContent)
}
