package api

import (
	"bufio"
	"context"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"errors"
	"io"
	"net"
	"net/http"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

// MaskedToken represents a sanitized agent token placeholder.
const MaskedToken = "••••••••"

// Device models the schema for physical attendance devices.
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

// CreateDeviceRequest models request to register a device.
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

// UpdateDeviceRequest models partial update for a device.
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

// AuthorizedAgent models an edge agent authorization entry.
type AuthorizedAgent struct {
	ID                   string     `json:"id"`
	Name                 string     `json:"name"`
	Token                string     `json:"token"`
	IsTrusted            bool       `json:"is_trusted"`
	LastIP               *string    `json:"last_ip"`
	CreatedAt            *time.Time `json:"created_at"`
	LastSeenAt           *time.Time `json:"last_seen_at"`
	IsActive             bool       `json:"is_active"`
	WorkspaceID          uuid.UUID  `json:"workspace_id"`
	MaxConcurrentWorkers int        `json:"max_concurrent_workers"`
	SuccessCooldownSecs  int        `json:"success_cooldown_secs"`
	RetentionDays        int        `json:"retention_days"`
}

// CreateAuthorizedAgentRequest models request to authorize an agent.
type CreateAuthorizedAgentRequest struct {
	ID                   string     `json:"id"`
	Name                 string     `json:"name"`
	Token                string     `json:"token"`
	WorkspaceID          *uuid.UUID `json:"workspace_id,omitempty"`
	MaxConcurrentWorkers *int       `json:"max_concurrent_workers,omitempty"`
	SuccessCooldownSecs  *int       `json:"success_cooldown_secs,omitempty"`
	RetentionDays        *int       `json:"retention_days,omitempty"`
}

// UpdateAuthorizedAgentRequest models update to an authorized agent.
type UpdateAuthorizedAgentRequest struct {
	Name                 *string `json:"name,omitempty"`
	Token                *string `json:"token,omitempty"`
	IsTrusted            *bool   `json:"is_trusted,omitempty"`
	IsActive             *bool   `json:"is_active,omitempty"`
	MaxConcurrentWorkers *int    `json:"max_concurrent_workers,omitempty"`
	SuccessCooldownSecs  *int    `json:"success_cooldown_secs,omitempty"`
	RetentionDays        *int    `json:"retention_days,omitempty"`
}

// TrustAgentRequest models trust toggle.
type TrustAgentRequest struct {
	IsTrusted bool `json:"is_trusted"`
}

// DeviceStatus models status of a single device reported by agent.
type DeviceStatus struct {
	DeviceID string          `json:"device_id"`
	Status   string          `json:"status"`
	LastSync *time.Time      `json:"last_sync"`
	Error    *string         `json:"error"`
	Info     json.RawMessage `json:"info"`
}

// AgentStatus models real-time status of an agent and its devices.
type AgentStatus struct {
	AgentID  string         `json:"agent_id"`
	LastSeen time.Time      `json:"last_seen"`
	IsOnline bool           `json:"is_online"`
	Devices  []DeviceStatus `json:"devices"`
}

// AgentCommandRequest models user command submission.
type AgentCommandRequest struct {
	Method   string          `json:"method"`
	DeviceID *string         `json:"device_id,omitempty"`
	Data     json.RawMessage `json:"data,omitempty"`
}

// AgentCommand models command frame pushed to agent over WS.
type AgentCommand struct {
	ID       string          `json:"id"`
	Type     string          `json:"type"`
	Method   string          `json:"method"`
	DeviceID *string         `json:"device_id,omitempty"`
	Data     json.RawMessage `json:"data,omitempty"`
}

// DeviceConfigPush models device payload in config push.
type DeviceConfigPush struct {
	DeviceID  string  `json:"device_id"`
	Name      string  `json:"name"`
	IP        string  `json:"ip"`
	Port      *int    `json:"port"`
	Brand     string  `json:"brand"`
	Password  *string `json:"password"`
	Timezone  *string `json:"timezone"`
	IsEnabled bool    `json:"is_enabled"`
}

// ConfigData models settings and devices pushed to an agent.
type ConfigData struct {
	Devices              []DeviceConfigPush `json:"devices"`
	MaxConcurrentWorkers *int               `json:"max_concurrent_workers"`
	SuccessCooldownSecs  *int               `json:"success_cooldown_secs"`
	RetentionDays        *int               `json:"retention_days"`
}

// CentralConfigPush models top-level config push frame.
type CentralConfigPush struct {
	Type string     `json:"type"`
	Data ConfigData `json:"data"`
}

// HashAgentToken computes SHA-256 hex encoding of token matching Rust crypto contract.
func HashAgentToken(token string) string {
	sum := sha256.Sum256([]byte(token))
	return hex.EncodeToString(sum[:])
}

// AgentDeviceRepo defines data access interface for agents and devices.
type AgentDeviceRepo interface {
	ListAuthorizedAgents(ctx context.Context, workspaceID uuid.UUID, includeInactive bool) ([]AuthorizedAgent, error)
	CreateAuthorizedAgent(ctx context.Context, agent AuthorizedAgent) error
	GetAuthorizedAgent(ctx context.Context, workspaceID uuid.UUID, id string) (*AuthorizedAgent, error)
	GetAuthorizedAgentByID(ctx context.Context, id string) (*AuthorizedAgent, error)
	UpdateAuthorizedAgent(ctx context.Context, workspaceID uuid.UUID, id string, req UpdateAuthorizedAgentRequest) (*AuthorizedAgent, error)
	DeleteAuthorizedAgent(ctx context.Context, workspaceID uuid.UUID, id string) error
	UpdateAgentTrust(ctx context.Context, workspaceID uuid.UUID, id string, isTrusted bool) error
	UpdateAgentLastIP(ctx context.Context, id string, ip string) error
	AgentExistsInWorkspace(ctx context.Context, workspaceID uuid.UUID, id string) (bool, error)

	ListDevices(ctx context.Context, workspaceID uuid.UUID) ([]Device, error)
	ListDevicesByAgentID(ctx context.Context, workspaceID uuid.UUID, agentID string) ([]Device, error)
	CreateDevice(ctx context.Context, device Device) (uuid.UUID, error)
	GetDevice(ctx context.Context, workspaceID, id uuid.UUID) (*Device, error)
	UpdateDevice(ctx context.Context, workspaceID, id uuid.UUID, req UpdateDeviceRequest) (*Device, error)
	DeleteDevice(ctx context.Context, workspaceID, id uuid.UUID) (string, error)
}

type agentConn struct {
	agentID     string
	workspaceID uuid.UUID
	conn        net.Conn
	bufrw       *bufio.ReadWriter
	sendCh      chan []byte
	closed      chan struct{}
	closeOnce   sync.Once
}

func (ac *agentConn) Close() {
	ac.closeOnce.Do(func() {
		close(ac.closed)
		if ac.conn != nil {
			_ = writeWSCloseFrame(ac.conn)
			_ = ac.conn.Close()
		}
	})
}

type pendingCommand struct {
	responseCh chan json.RawMessage
	errCh      chan error
}

// AgentHub manages live agent WebSocket connections and message routing.
type AgentHub struct {
	mu         sync.RWMutex
	conns      map[string]*agentConn
	pendingMu  sync.Mutex
	pendingCmd map[string]pendingCommand
	repo       AgentDeviceRepo
	redis      *redisinfra.Client
}

// NewAgentHub initializes a WebSocket agent hub.
func NewAgentHub(repo AgentDeviceRepo, redis *redisinfra.Client) *AgentHub {
	return &AgentHub{
		conns:      make(map[string]*agentConn),
		pendingCmd: make(map[string]pendingCommand),
		repo:       repo,
		redis:      redis,
	}
}

// IsOnline reports whether agent is actively connected to this hub instance.
func (h *AgentHub) IsOnline(agentID string) bool {
	h.mu.RLock()
	defer h.mu.RUnlock()
	_, ok := h.conns[agentID]
	return ok
}

// SendCommand dispatches an agent command asynchronously.
func (h *AgentHub) SendCommand(agentID string, cmd AgentCommand) error {
	h.mu.RLock()
	ac, ok := h.conns[agentID]
	h.mu.RUnlock()
	if !ok {
		return errors.New("agent not found or offline")
	}
	payload, err := json.Marshal(cmd)
	if err != nil {
		return err
	}
	select {
	case ac.sendCh <- payload:
		return nil
	case <-ac.closed:
		return errors.New("agent disconnected")
	case <-time.After(2 * time.Second):
		return errors.New("send timed out")
	}
}

// GetDeviceInfo queries device information on demand with 10s timeout.
func (h *AgentHub) GetDeviceInfo(ctx context.Context, agentID, deviceID string) (json.RawMessage, error) {
	h.mu.RLock()
	ac, ok := h.conns[agentID]
	h.mu.RUnlock()
	if !ok {
		return nil, errors.New("agent not found or offline")
	}

	cmdID := uuid.New().String()
	respCh := make(chan json.RawMessage, 1)
	errCh := make(chan error, 1)

	h.pendingMu.Lock()
	h.pendingCmd[cmdID] = pendingCommand{responseCh: respCh, errCh: errCh}
	h.pendingMu.Unlock()

	defer func() {
		h.pendingMu.Lock()
		delete(h.pendingCmd, cmdID)
		h.pendingMu.Unlock()
	}()

	cmd := AgentCommand{
		ID:       cmdID,
		Type:     "command",
		Method:   "get_device_info",
		DeviceID: &deviceID,
	}
	payload, _ := json.Marshal(cmd)

	select {
	case ac.sendCh <- payload:
	case <-time.After(2 * time.Second):
		return nil, errors.New("command send timed out")
	case <-ac.closed:
		return nil, errors.New("agent disconnected")
	case <-ctx.Done():
		return nil, ctx.Err()
	}

	select {
	case resp := <-respCh:
		return resp, nil
	case err := <-errCh:
		return nil, err
	case <-time.After(10 * time.Second):
		return nil, errors.New("command timed out")
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

// NotifyConfigUpdate pushes latest device settings to connected agent.
func (h *AgentHub) NotifyConfigUpdate(ctx context.Context, workspaceID uuid.UUID, agentID string) {
	h.mu.RLock()
	ac, ok := h.conns[agentID]
	h.mu.RUnlock()
	if !ok || h.repo == nil {
		return
	}

	devices, _ := h.repo.ListDevicesByAgentID(ctx, workspaceID, agentID)
	agent, _ := h.repo.GetAuthorizedAgent(ctx, workspaceID, agentID)

	var pushes []DeviceConfigPush
	for _, d := range devices {
		tz := d.Timezone
		pushes = append(pushes, DeviceConfigPush{
			DeviceID:  d.ID.String(),
			Name:      d.Name,
			IP:        d.IP,
			Port:      d.Port,
			Brand:     d.Brand,
			Password:  d.Password,
			Timezone:  &tz,
			IsEnabled: d.IsEnabled,
		})
	}
	var maxWorkers, cooldown, retention *int
	if agent != nil {
		maxWorkers = &agent.MaxConcurrentWorkers
		cooldown = &agent.SuccessCooldownSecs
		retention = &agent.RetentionDays
	}

	msg := CentralConfigPush{
		Type: "config_push",
		Data: ConfigData{
			Devices:              pushes,
			MaxConcurrentWorkers: maxWorkers,
			SuccessCooldownSecs:  cooldown,
			RetentionDays:        retention,
		},
	}
	if b, err := json.Marshal(msg); err == nil {
		select {
		case ac.sendCh <- b:
		default:
		}
	}
}

// RemoveAgent closes active agent connection and evicts it from hub.
func (h *AgentHub) RemoveAgent(agentID string) {
	h.mu.Lock()
	ac, ok := h.conns[agentID]
	if ok {
		delete(h.conns, agentID)
	}
	h.mu.Unlock()
	if ok && ac != nil {
		ac.Close()
	}
}

// HandleWS manages upgraded agent WebSocket session loop.
func (h *AgentHub) HandleWS(agent *AuthorizedAgent, conn net.Conn, bufrw *bufio.ReadWriter) {
	ac := &agentConn{
		agentID:     agent.ID,
		workspaceID: agent.WorkspaceID,
		conn:        conn,
		bufrw:       bufrw,
		sendCh:      make(chan []byte, 128),
		closed:      make(chan struct{}),
	}

	h.mu.Lock()
	if old, exists := h.conns[agent.ID]; exists {
		old.Close()
	}
	h.conns[agent.ID] = ac
	h.mu.Unlock()

	defer func() {
		h.mu.Lock()
		if cur, ok := h.conns[agent.ID]; ok && cur == ac {
			delete(h.conns, agent.ID)
		}
		h.mu.Unlock()
		ac.Close()
	}()

	// Push initial config update
	go h.NotifyConfigUpdate(context.Background(), agent.WorkspaceID, agent.ID)

	// Sender goroutine
	go func() {
		for {
			select {
			case <-ac.closed:
				return
			case payload, ok := <-ac.sendCh:
				if !ok {
					return
				}
				if err := writeWSTextFrame(conn, payload); err != nil {
					ac.Close()
					return
				}
			}
		}
	}()

	// Reader loop
	for {
		opcode, payload, err := readWSFrame(bufrw.Reader)
		if err != nil {
			return
		}
		switch opcode {
		case 0x8: // Close
			_ = writeWSCloseFrame(conn)
			return
		case 0x9: // Ping
			_ = writeWSPongFrame(conn, payload)
		case 0x1: // Text
			h.processMessage(ac, payload)
		}
	}
}

func (h *AgentHub) processMessage(ac *agentConn, payload []byte) {
	var meta struct {
		Type      string          `json:"type"`
		AgentID   string          `json:"agent_id"`
		MsgID     string          `json:"msg_id"`
		CommandID string          `json:"command_id"`
		Status    string          `json:"status"`
		Data      json.RawMessage `json:"data"`
		Error     *string         `json:"error"`
	}
	if err := json.Unmarshal(payload, &meta); err != nil {
		return
	}

	switch meta.Type {
	case "heartbeat":
		if meta.AgentID == "" || meta.AgentID == ac.agentID {
			if h.redis != nil {
				if shared, err := h.redis.Shared(); err == nil && shared != nil {
					_ = shared.Set(context.Background(), "agent:presence:"+ac.agentID, time.Now().UTC().Format(time.RFC3339), 60*time.Second).Err()
				}
			}
			ack := map[string]string{
				"type":        "heartbeat_ack",
				"server_time": time.Now().UTC().Format(time.RFC3339),
			}
			if b, err := json.Marshal(ack); err == nil {
				select {
				case ac.sendCh <- b:
				default:
				}
			}
		}
	case "attendance":
		ack := map[string]interface{}{
			"type":      "ack",
			"msg_id":    meta.MsgID,
			"status":    "accepted",
			"timestamp": time.Now().UTC().Format(time.RFC3339),
		}
		if b, err := json.Marshal(ack); err == nil {
			select {
			case ac.sendCh <- b:
			default:
			}
		}
	case "command_response":
		if meta.CommandID != "" {
			h.pendingMu.Lock()
			pending, ok := h.pendingCmd[meta.CommandID]
			h.pendingMu.Unlock()
			if ok {
				if meta.Status == "success" {
					pending.responseCh <- meta.Data
				} else {
					errMsg := "command failed"
					if meta.Error != nil {
						errMsg = *meta.Error
					}
					pending.errCh <- errors.New(errMsg)
				}
			}
		}
	case "handshake":
		// Already authorized at HTTP level; noop
	}
}

func readWSFrame(r io.Reader) (opcode byte, payload []byte, err error) {
	var header [2]byte
	if _, err := io.ReadFull(r, header[:]); err != nil {
		return 0, nil, err
	}
	opcode = header[0] & 0x0F
	masked := header[1]&0x80 != 0
	payloadLen := int64(header[1] & 0x7F)

	if payloadLen == 126 {
		var extended [2]byte
		if _, err := io.ReadFull(r, extended[:]); err != nil {
			return 0, nil, err
		}
		payloadLen = int64(binary.BigEndian.Uint16(extended[:]))
	} else if payloadLen == 127 {
		var extended [8]byte
		if _, err := io.ReadFull(r, extended[:]); err != nil {
			return 0, nil, err
		}
		payloadLen = int64(binary.BigEndian.Uint64(extended[:]))
	}

	var maskKey [4]byte
	if masked {
		if _, err := io.ReadFull(r, maskKey[:]); err != nil {
			return 0, nil, err
		}
	}

	payload = make([]byte, payloadLen)
	if _, err := io.ReadFull(r, payload); err != nil {
		return 0, nil, err
	}

	if masked {
		for i := range payload {
			payload[i] ^= maskKey[i%4]
		}
	}

	return opcode, payload, nil
}

func writeWSPongFrame(w io.Writer, payload []byte) error {
	length := len(payload)
	header := make([]byte, 0, 10)
	header = append(header, 0x8A)
	if length <= 125 {
		header = append(header, byte(length))
	} else {
		header = append(header, 126, byte(length>>8), byte(length&0xFF))
	}
	if _, err := w.Write(header); err != nil {
		return err
	}
	_, err := w.Write(payload)
	return err
}

// AgentDeviceHandler serves device and agent management endpoints.
type AgentDeviceHandler struct {
	Repository AgentDeviceRepo
	Hub        *AgentHub
	Authorizer WorkspaceAuthorizer
	Redis      *redisinfra.Client
}

func resolveWorkspaceID(r *http.Request) (uuid.UUID, bool) {
	if wsID, ok := httpinfra.WorkspaceIDFromContext(r.Context()); ok && wsID != uuid.Nil {
		return wsID, true
	}
	if header := r.Header.Get("X-Workspace-Id"); header != "" {
		if id, err := uuid.Parse(header); err == nil && id != uuid.Nil {
			return id, true
		}
	}
	if query := r.URL.Query().Get("workspace_id"); query != "" {
		if id, err := uuid.Parse(query); err == nil && id != uuid.Nil {
			return id, true
		}
	}
	return uuid.Nil, false
}

// 1. GET /api/v1/devices
func (h AgentDeviceHandler) ListDevices(w http.ResponseWriter, r *http.Request) {
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

// 2. POST /api/v1/devices
func (h AgentDeviceHandler) CreateDevice(w http.ResponseWriter, r *http.Request) {
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
	port := 4370
	if req.Port != nil {
		port = *req.Port
	}
	brand := "zkteco"
	if req.Brand != nil && *req.Brand != "" {
		brand = *req.Brand
	}
	tz := "UTC"
	if req.Timezone != nil && *req.Timezone != "" {
		tz = *req.Timezone
	}
	maxWorkers := 2
	if req.MaxConcurrentWorkers != nil {
		maxWorkers = *req.MaxConcurrentWorkers
	}

	dev := Device{
		ID:                   uuid.New(),
		AgentID:              req.AgentID,
		Name:                 req.Name,
		IP:                   req.IP,
		Port:                 &port,
		Brand:                brand,
		Password:             req.Password,
		Timezone:             tz,
		IsEnabled:            true,
		LastStatus:           "offline",
		WorkspaceID:          wsID,
		LocationID:           req.LocationID,
		MaxConcurrentWorkers: maxWorkers,
	}

	devID, err := h.Repository.CreateDevice(r.Context(), dev)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if h.Hub != nil {
		h.Hub.NotifyConfigUpdate(r.Context(), wsID, dev.AgentID)
	}

	apperror.WriteJSON(w, http.StatusCreated, map[string]interface{}{"id": devID})
}

// 3. PUT /api/v1/devices/{id}
func (h AgentDeviceHandler) UpdateDevice(w http.ResponseWriter, r *http.Request) {
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

	dev, err := h.Repository.UpdateDevice(r.Context(), wsID, id, req)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if h.Hub != nil {
		h.Hub.NotifyConfigUpdate(r.Context(), wsID, dev.AgentID)
	}

	apperror.WriteJSON(w, http.StatusOK, dev)
}

// 4. DELETE /api/v1/devices/{id}
func (h AgentDeviceHandler) DeleteDevice(w http.ResponseWriter, r *http.Request) {
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

	agentID, err := h.Repository.DeleteDevice(r.Context(), wsID, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if h.Hub != nil && agentID != "" {
		h.Hub.NotifyConfigUpdate(r.Context(), wsID, agentID)
	}

	w.WriteHeader(http.StatusNoContent)
}

// 5. GET /api/v1/devices/{id}/info
func (h AgentDeviceHandler) GetDeviceInfo(w http.ResponseWriter, r *http.Request) {
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

	dev, err := h.Repository.GetDevice(r.Context(), wsID, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if h.Hub == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("hub offline")))
		return
	}

	info, err := h.Hub.GetDeviceInfo(r.Context(), dev.AgentID, dev.ID.String())
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(info)
}

// 6. GET /api/v1/authorized-agents
func (h AgentDeviceHandler) ListAuthorizedAgents(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	includeInactive := r.URL.Query().Get("include_inactive") == "true"
	agents, err := h.Repository.ListAuthorizedAgents(r.Context(), wsID, includeInactive)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if agents == nil {
		agents = []AuthorizedAgent{}
	}
	apperror.WriteJSON(w, http.StatusOK, agents)
}

// 7. POST /api/v1/authorized-agents
func (h AgentDeviceHandler) CreateAuthorizedAgent(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	req, err := DecodeJSON[CreateAuthorizedAgentRequest](r, 1<<20)
	if err != nil || req.ID == "" || req.Name == "" || req.Token == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	maxWorkers := 2
	if req.MaxConcurrentWorkers != nil {
		maxWorkers = *req.MaxConcurrentWorkers
	}
	cooldown := 300
	if req.SuccessCooldownSecs != nil {
		cooldown = *req.SuccessCooldownSecs
	}
	retention := 365
	if req.RetentionDays != nil {
		retention = *req.RetentionDays
	}

	agent := AuthorizedAgent{
		ID:                   req.ID,
		Name:                 req.Name,
		Token:                HashAgentToken(req.Token),
		IsTrusted:            false,
		IsActive:             true,
		WorkspaceID:          wsID,
		MaxConcurrentWorkers: maxWorkers,
		SuccessCooldownSecs:  cooldown,
		RetentionDays:        retention,
	}

	if err := h.Repository.CreateAuthorizedAgent(r.Context(), agent); err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusCreated, map[string]interface{}{"id": req.ID})
}

// 8. GET /api/v1/authorized-agents/{id}
func (h AgentDeviceHandler) GetAuthorizedAgent(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	id := r.PathValue("id")
	if id == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	agent, err := h.Repository.GetAuthorizedAgent(r.Context(), wsID, id)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, agent)
}

// 9. PATCH /api/v1/authorized-agents/{id}
func (h AgentDeviceHandler) UpdateAuthorizedAgent(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	id := r.PathValue("id")
	if id == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[UpdateAuthorizedAgentRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Name == nil && req.Token == nil && req.IsTrusted == nil && req.IsActive == nil &&
		req.MaxConcurrentWorkers == nil && req.SuccessCooldownSecs == nil && req.RetentionDays == nil {
		apperror.WriteError(w, r, apperror.Validation("error.no_fields_to_update"))
		return
	}

	agent, err := h.Repository.UpdateAuthorizedAgent(r.Context(), wsID, id, req)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if (!agent.IsTrusted || !agent.IsActive) && h.Hub != nil {
		h.Hub.RemoveAgent(id)
	} else if h.Hub != nil {
		h.Hub.NotifyConfigUpdate(r.Context(), wsID, id)
	}

	apperror.WriteJSON(w, http.StatusOK, agent)
}

// 10. DELETE /api/v1/authorized-agents/{id}
func (h AgentDeviceHandler) DeleteAuthorizedAgent(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	id := r.PathValue("id")
	if id == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	if err := h.Repository.DeleteAuthorizedAgent(r.Context(), wsID, id); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if h.Hub != nil {
		h.Hub.RemoveAgent(id)
	}

	w.WriteHeader(http.StatusNoContent)
}

// 11. PUT /api/v1/authorized-agents/{id}/trust
func (h AgentDeviceHandler) UpdateAgentTrust(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	id := r.PathValue("id")
	if id == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[TrustAgentRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	if err := h.Repository.UpdateAgentTrust(r.Context(), wsID, id, req.IsTrusted); err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	if !req.IsTrusted && h.Hub != nil {
		h.Hub.RemoveAgent(id)
	}

	w.WriteHeader(http.StatusOK)
}

// 12. GET /api/v1/agents
func (h AgentDeviceHandler) ListAgents(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	agents, err := h.Repository.ListAuthorizedAgents(r.Context(), wsID, false)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	result := make([]AgentStatus, 0, len(agents))
	for _, a := range agents {
		isOnline := false
		if h.Hub != nil && h.Hub.IsOnline(a.ID) {
			isOnline = true
		} else if h.Redis != nil {
			if shared, rErr := h.Redis.Shared(); rErr == nil && shared != nil {
				if exists, _ := shared.Exists(r.Context(), "agent:presence:"+a.ID).Result(); exists > 0 {
					isOnline = true
				}
			}
		}

		lastSeen := time.Now().UTC()
		if a.LastSeenAt != nil {
			lastSeen = *a.LastSeenAt
		}

		devs, _ := h.Repository.ListDevicesByAgentID(r.Context(), wsID, a.ID)
		var devStatuses []DeviceStatus
		for _, d := range devs {
			status := "offline"
			if isOnline && d.IsEnabled {
				status = "online"
			}
			devStatuses = append(devStatuses, DeviceStatus{
				DeviceID: d.ID.String(),
				Status:   status,
				LastSync: nil,
				Error:    nil,
				Info:     nil,
			})
		}
		if devStatuses == nil {
			devStatuses = []DeviceStatus{}
		}

		result = append(result, AgentStatus{
			AgentID:  a.ID,
			LastSeen: lastSeen,
			IsOnline: isOnline,
			Devices:  devStatuses,
		})
	}

	apperror.WriteJSON(w, http.StatusOK, result)
}

// 13. POST /api/v1/agent/{agent_id}/command
func (h AgentDeviceHandler) SendCommand(w http.ResponseWriter, r *http.Request) {
	wsID, ok := resolveWorkspaceID(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	agentID := r.PathValue("agent_id")
	if agentID == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	exists, err := h.Repository.AgentExistsInWorkspace(r.Context(), wsID, agentID)
	if err != nil || !exists {
		apperror.WriteError(w, r, apperror.NotFound("error.agent_not_found"))
		return
	}

	req, err := DecodeJSON[AgentCommandRequest](r, 1<<20)
	if err != nil || req.Method == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}

	cmd := AgentCommand{
		ID:       uuid.New().String(),
		Type:     "command",
		Method:   req.Method,
		DeviceID: req.DeviceID,
		Data:     req.Data,
	}

	if h.Hub == nil {
		apperror.WriteError(w, r, apperror.NotFound("error.agent_not_found"))
		return
	}

	if err := h.Hub.SendCommand(agentID, cmd); err != nil {
		apperror.WriteError(w, r, apperror.NotFound("error.agent_not_found"))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, map[string]interface{}{"message": "Command sent"})
}

// 14. GET /api/v1/ws and GET /ws
func (h AgentDeviceHandler) WebSocket(w http.ResponseWriter, r *http.Request) {
	agentID := r.Header.Get("X-Agent-Id")
	if agentID == "" {
		agentID = r.Header.Get("x-agent-id")
	}
	if agentID == "" {
		agentID = r.URL.Query().Get("agent_id")
	}

	token := r.Header.Get("X-Token")
	if token == "" {
		token = r.Header.Get("x-token")
	}
	if token == "" {
		token = r.URL.Query().Get("token")
	}

	if agentID == "" || token == "" {
		w.WriteHeader(http.StatusUnauthorized)
		return
	}

	agent, err := h.Repository.GetAuthorizedAgentByID(r.Context(), agentID)
	if err != nil || agent == nil || !agent.IsActive {
		w.WriteHeader(http.StatusUnauthorized)
		return
	}

	if HashAgentToken(token) != agent.Token {
		w.WriteHeader(http.StatusUnauthorized)
		return
	}

	if !agent.IsTrusted {
		w.WriteHeader(http.StatusUnauthorized)
		return
	}

	conn, bufrw, err := wsHandshake(w, r)
	if err != nil {
		return
	}
	if conn == nil {
		// Non-hijackable recorder (e.g. unit test): handshake headers set
		return
	}

	_ = h.Repository.UpdateAgentLastIP(r.Context(), agentID, r.RemoteAddr)

	if h.Hub != nil {
		h.Hub.HandleWS(agent, conn, bufrw)
	}
}
