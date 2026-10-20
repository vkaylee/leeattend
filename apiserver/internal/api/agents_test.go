package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAgentDeviceRepo struct {
	devices []Device
	agents  []AuthorizedAgent
	err     error
}

func (m *mockAgentDeviceRepo) ListAuthorizedAgents(_ context.Context, _ uuid.UUID, includeInactive bool) ([]AuthorizedAgent, error) {
	if m.err != nil {
		return nil, m.err
	}
	var res []AuthorizedAgent
	for _, a := range m.agents {
		if includeInactive || a.IsActive {
			res = append(res, a)
		}
	}
	return res, nil
}

func (m *mockAgentDeviceRepo) CreateAuthorizedAgent(_ context.Context, a AuthorizedAgent) error {
	if m.err != nil {
		return m.err
	}
	m.agents = append(m.agents, a)
	return nil
}

func (m *mockAgentDeviceRepo) GetAuthorizedAgent(_ context.Context, _ uuid.UUID, id string) (*AuthorizedAgent, error) {
	if m.err != nil {
		return nil, m.err
	}
	for i := range m.agents {
		if m.agents[i].ID == id {
			copy := m.agents[i]
			copy.Token = MaskedToken
			return &copy, nil
		}
	}
	return nil, pgx.ErrNoRows
}

func (m *mockAgentDeviceRepo) GetAuthorizedAgentByID(_ context.Context, id string) (*AuthorizedAgent, error) {
	if m.err != nil {
		return nil, m.err
	}
	for i := range m.agents {
		if m.agents[i].ID == id {
			copy := m.agents[i]
			return &copy, nil
		}
	}
	return nil, pgx.ErrNoRows
}

func (m *mockAgentDeviceRepo) UpdateAuthorizedAgent(_ context.Context, _ uuid.UUID, id string, req UpdateAuthorizedAgentRequest) (*AuthorizedAgent, error) {
	if m.err != nil {
		return nil, m.err
	}
	for i := range m.agents {
		if m.agents[i].ID == id {
			if req.Name != nil {
				m.agents[i].Name = *req.Name
			}
			if req.IsTrusted != nil {
				m.agents[i].IsTrusted = *req.IsTrusted
			}
			if req.IsActive != nil {
				m.agents[i].IsActive = *req.IsActive
			}
			copy := m.agents[i]
			copy.Token = MaskedToken
			return &copy, nil
		}
	}
	return nil, pgx.ErrNoRows
}

func (m *mockAgentDeviceRepo) DeleteAuthorizedAgent(_ context.Context, _ uuid.UUID, id string) error {
	if m.err != nil {
		return m.err
	}
	for i := range m.agents {
		if m.agents[i].ID == id {
			m.agents[i].IsActive = false
			return nil
		}
	}
	return pgx.ErrNoRows
}

func (m *mockAgentDeviceRepo) UpdateAgentTrust(_ context.Context, _ uuid.UUID, id string, isTrusted bool) error {
	if m.err != nil {
		return m.err
	}
	for i := range m.agents {
		if m.agents[i].ID == id {
			m.agents[i].IsTrusted = isTrusted
			return nil
		}
	}
	return pgx.ErrNoRows
}

func (m *mockAgentDeviceRepo) UpdateAgentLastIP(_ context.Context, _ string, _ string) error {
	return m.err
}

func (m *mockAgentDeviceRepo) AgentExistsInWorkspace(_ context.Context, _ uuid.UUID, id string) (bool, error) {
	if m.err != nil {
		return false, m.err
	}
	for i := range m.agents {
		if m.agents[i].ID == id && m.agents[i].IsActive {
			return true, nil
		}
	}
	return false, nil
}

func (m *mockAgentDeviceRepo) ListDevices(_ context.Context, _ uuid.UUID) ([]Device, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.devices, nil
}

func (m *mockAgentDeviceRepo) ListDevicesByAgentID(_ context.Context, _ uuid.UUID, agentID string) ([]Device, error) {
	if m.err != nil {
		return nil, m.err
	}
	var res []Device
	for _, d := range m.devices {
		if d.AgentID == agentID {
			res = append(res, d)
		}
	}
	return res, nil
}

func (m *mockAgentDeviceRepo) CreateDevice(_ context.Context, dev Device) (uuid.UUID, error) {
	if m.err != nil {
		return uuid.Nil, m.err
	}
	m.devices = append(m.devices, dev)
	return dev.ID, nil
}

func (m *mockAgentDeviceRepo) GetDevice(_ context.Context, _ uuid.UUID, id uuid.UUID) (*Device, error) {
	if m.err != nil {
		return nil, m.err
	}
	for i := range m.devices {
		if m.devices[i].ID == id {
			return &m.devices[i], nil
		}
	}
	return nil, pgx.ErrNoRows
}

func (m *mockAgentDeviceRepo) UpdateDevice(_ context.Context, _ uuid.UUID, id uuid.UUID, req UpdateDeviceRequest) (*Device, error) {
	if m.err != nil {
		return nil, m.err
	}
	for i := range m.devices {
		if m.devices[i].ID == id {
			if req.Name != nil {
				m.devices[i].Name = *req.Name
			}
			if req.IsEnabled != nil {
				m.devices[i].IsEnabled = *req.IsEnabled
			}
			return &m.devices[i], nil
		}
	}
	return nil, pgx.ErrNoRows
}

func (m *mockAgentDeviceRepo) DeleteDevice(_ context.Context, _ uuid.UUID, id uuid.UUID) (string, error) {
	if m.err != nil {
		return "", m.err
	}
	for i := range m.devices {
		if m.devices[i].ID == id {
			agentID := m.devices[i].AgentID
			m.devices = append(m.devices[:i], m.devices[i+1:]...)
			return agentID, nil
		}
	}
	return "", pgx.ErrNoRows
}

func TestHashAgentToken(t *testing.T) {
	token := "secret_token_123"
	hash := HashAgentToken(token)
	if hash == "" || len(hash) != 64 {
		t.Fatalf("expected 64 char hex hash, got %s", hash)
	}
	if hash != HashAgentToken(token) {
		t.Fatalf("hash must be deterministic")
	}
}

func TestDevicesCRUD_Success(t *testing.T) {
	wsID := uuid.New()
	repo := &mockAgentDeviceRepo{}
	hub := NewAgentHub(repo, nil)
	handler := AgentDeviceHandler{Repository: repo, Hub: hub}

	// 1. Create Device
	createBody := `{"agent_id":"agent-01","name":"Front Door","ip":"192.168.1.100","port":4370,"brand":"zkteco"}`
	reqCreate := httptest.NewRequest(http.MethodPost, "/api/v1/devices", bytes.NewReader([]byte(createBody)))
	reqCreate.Header.Set("X-Workspace-Id", wsID.String())
	reqCreate = reqCreate.WithContext(httpinfra.WithWorkspaceIDForTest(reqCreate.Context(), wsID))

	recCreate := httptest.NewRecorder()
	handler.CreateDevice(recCreate, reqCreate)
	if recCreate.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", recCreate.Code, recCreate.Body.String())
	}
	var createdResp map[string]interface{}
	_ = json.Unmarshal(recCreate.Body.Bytes(), &createdResp)
	devIDStr, ok := createdResp["id"].(string)
	if !ok || devIDStr == "" {
		t.Fatalf("expected valid id in create response")
	}

	// 2. List Devices
	reqList := httptest.NewRequest(http.MethodGet, "/api/v1/devices", nil)
	reqList.Header.Set("X-Workspace-Id", wsID.String())
	reqList = reqList.WithContext(httpinfra.WithWorkspaceIDForTest(reqList.Context(), wsID))

	recList := httptest.NewRecorder()
	handler.ListDevices(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recList.Code)
	}
	var devs []Device
	_ = json.Unmarshal(recList.Body.Bytes(), &devs)
	if len(devs) != 1 || devs[0].Name != "Front Door" {
		t.Fatalf("expected 1 device named Front Door, got %+v", devs)
	}

	// 3. Update Device
	reqUpdate := httptest.NewRequest(http.MethodPut, "/api/v1/devices/"+devIDStr, bytes.NewReader([]byte(`{"name":"Back Door"}`)))
	reqUpdate.SetPathValue("id", devIDStr)
	reqUpdate.Header.Set("X-Workspace-Id", wsID.String())
	reqUpdate = reqUpdate.WithContext(httpinfra.WithWorkspaceIDForTest(reqUpdate.Context(), wsID))

	recUpdate := httptest.NewRecorder()
	handler.UpdateDevice(recUpdate, reqUpdate)
	if recUpdate.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recUpdate.Code, recUpdate.Body.String())
	}

	// 4. Delete Device
	reqDelete := httptest.NewRequest(http.MethodDelete, "/api/v1/devices/"+devIDStr, nil)
	reqDelete.SetPathValue("id", devIDStr)
	reqDelete.Header.Set("X-Workspace-Id", wsID.String())
	reqDelete = reqDelete.WithContext(httpinfra.WithWorkspaceIDForTest(reqDelete.Context(), wsID))

	recDelete := httptest.NewRecorder()
	handler.DeleteDevice(recDelete, reqDelete)
	if recDelete.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDelete.Code)
	}
}

func TestAuthorizedAgentsCRUD_Success(t *testing.T) {
	wsID := uuid.New()
	repo := &mockAgentDeviceRepo{}
	hub := NewAgentHub(repo, nil)
	handler := AgentDeviceHandler{Repository: repo, Hub: hub}

	// 1. Create Agent
	agentBody := `{"id":"agent-hanoi","name":"Hanoi Office Agent","token":"raw_secret_token"}`
	reqCreate := httptest.NewRequest(http.MethodPost, "/api/v1/authorized-agents", bytes.NewReader([]byte(agentBody)))
	reqCreate.Header.Set("X-Workspace-Id", wsID.String())
	reqCreate = reqCreate.WithContext(httpinfra.WithWorkspaceIDForTest(reqCreate.Context(), wsID))

	recCreate := httptest.NewRecorder()
	handler.CreateAuthorizedAgent(recCreate, reqCreate)
	if recCreate.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", recCreate.Code, recCreate.Body.String())
	}

	// 2. Get Agent
	reqGet := httptest.NewRequest(http.MethodGet, "/api/v1/authorized-agents/agent-hanoi", nil)
	reqGet.SetPathValue("id", "agent-hanoi")
	reqGet.Header.Set("X-Workspace-Id", wsID.String())
	reqGet = reqGet.WithContext(httpinfra.WithWorkspaceIDForTest(reqGet.Context(), wsID))

	recGet := httptest.NewRecorder()
	handler.GetAuthorizedAgent(recGet, reqGet)
	if recGet.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recGet.Code, recGet.Body.String())
	}
	var agent AuthorizedAgent
	_ = json.Unmarshal(recGet.Body.Bytes(), &agent)
	if agent.Token != MaskedToken {
		t.Fatalf("expected masked token, got %s", agent.Token)
	}

	// 3. Trust Agent
	reqTrust := httptest.NewRequest(http.MethodPut, "/api/v1/authorized-agents/agent-hanoi/trust", bytes.NewReader([]byte(`{"is_trusted":true}`)))
	reqTrust.SetPathValue("id", "agent-hanoi")
	reqTrust.Header.Set("X-Workspace-Id", wsID.String())
	reqTrust = reqTrust.WithContext(httpinfra.WithWorkspaceIDForTest(reqTrust.Context(), wsID))

	recTrust := httptest.NewRecorder()
	handler.UpdateAgentTrust(recTrust, reqTrust)
	if recTrust.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recTrust.Code)
	}

	// 4. Patch Agent
	reqPatch := httptest.NewRequest(http.MethodPatch, "/api/v1/authorized-agents/agent-hanoi", bytes.NewReader([]byte(`{"name":"Updated Agent"}`)))
	reqPatch.SetPathValue("id", "agent-hanoi")
	reqPatch.Header.Set("X-Workspace-Id", wsID.String())
	reqPatch = reqPatch.WithContext(httpinfra.WithWorkspaceIDForTest(reqPatch.Context(), wsID))

	recPatch := httptest.NewRecorder()
	handler.UpdateAuthorizedAgent(recPatch, reqPatch)
	if recPatch.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recPatch.Code)
	}

	// 5. Delete Agent
	reqDelete := httptest.NewRequest(http.MethodDelete, "/api/v1/authorized-agents/agent-hanoi", nil)
	reqDelete.SetPathValue("id", "agent-hanoi")
	reqDelete.Header.Set("X-Workspace-Id", wsID.String())
	reqDelete = reqDelete.WithContext(httpinfra.WithWorkspaceIDForTest(reqDelete.Context(), wsID))

	recDelete := httptest.NewRecorder()
	handler.DeleteAuthorizedAgent(recDelete, reqDelete)
	if recDelete.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDelete.Code)
	}
}

func TestAgentsListAndCommand(t *testing.T) {
	wsID := uuid.New()
	now := time.Now().UTC()
	repo := &mockAgentDeviceRepo{
		agents: []AuthorizedAgent{
			{
				ID:          "agent-01",
				Name:        "Test Agent",
				Token:       HashAgentToken("secret"),
				IsTrusted:   true,
				IsActive:    true,
				WorkspaceID: wsID,
				LastSeenAt:  &now,
			},
		},
		devices: []Device{
			{
				ID:          uuid.New(),
				AgentID:     "agent-01",
				Name:        "Device 1",
				IP:          "192.168.1.10",
				IsEnabled:   true,
				WorkspaceID: wsID,
			},
		},
	}
	hub := NewAgentHub(repo, nil)
	handler := AgentDeviceHandler{Repository: repo, Hub: hub}

	// List agents
	reqList := httptest.NewRequest(http.MethodGet, "/api/v1/agents", nil)
	reqList.Header.Set("X-Workspace-Id", wsID.String())
	reqList = reqList.WithContext(httpinfra.WithWorkspaceIDForTest(reqList.Context(), wsID))

	recList := httptest.NewRecorder()
	handler.ListAgents(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recList.Code)
	}
	var agents []AgentStatus
	_ = json.Unmarshal(recList.Body.Bytes(), &agents)
	if len(agents) != 1 || agents[0].AgentID != "agent-01" {
		t.Fatalf("expected agent-01, got %+v", agents)
	}

	// Send command to offline agent returns error
	reqCmd := httptest.NewRequest(http.MethodPost, "/api/v1/agent/agent-01/command", bytes.NewReader([]byte(`{"method":"sync_users"}`)))
	reqCmd.SetPathValue("agent_id", "agent-01")
	reqCmd.Header.Set("X-Workspace-Id", wsID.String())
	reqCmd = reqCmd.WithContext(httpinfra.WithWorkspaceIDForTest(reqCmd.Context(), wsID))

	recCmd := httptest.NewRecorder()
	handler.SendCommand(recCmd, reqCmd)
	// Offline agent returns 404
	if recCmd.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for offline agent command, got %d", recCmd.Code)
	}
}

func TestWebSocket_PreAuth(t *testing.T) {
	wsID := uuid.New()
	rawToken := "top_secret_token"
	repo := &mockAgentDeviceRepo{
		agents: []AuthorizedAgent{
			{
				ID:          "agent-trusted",
				Name:        "Trusted Agent",
				Token:       HashAgentToken(rawToken),
				IsTrusted:   true,
				IsActive:    true,
				WorkspaceID: wsID,
			},
			{
				ID:          "agent-untrusted",
				Name:        "Untrusted Agent",
				Token:       HashAgentToken(rawToken),
				IsTrusted:   false,
				IsActive:    true,
				WorkspaceID: wsID,
			},
		},
	}
	hub := NewAgentHub(repo, nil)
	handler := AgentDeviceHandler{Repository: repo, Hub: hub}

	// Missing headers -> 401
	req1 := httptest.NewRequest(http.MethodGet, "/api/v1/ws", nil)
	rec1 := httptest.NewRecorder()
	handler.WebSocket(rec1, req1)
	if rec1.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for missing headers, got %d", rec1.Code)
	}

	// Invalid token -> 401
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1/ws", nil)
	req2.Header.Set("X-Agent-Id", "agent-trusted")
	req2.Header.Set("X-Token", "wrong_token")
	rec2 := httptest.NewRecorder()
	handler.WebSocket(rec2, req2)
	if rec2.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for wrong token, got %d", rec2.Code)
	}

	// Untrusted agent -> 401
	req3 := httptest.NewRequest(http.MethodGet, "/api/v1/ws", nil)
	req3.Header.Set("X-Agent-Id", "agent-untrusted")
	req3.Header.Set("X-Token", rawToken)
	rec3 := httptest.NewRecorder()
	handler.WebSocket(rec3, req3)
	if rec3.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for untrusted agent, got %d", rec3.Code)
	}

	// Valid agent -> 101 Switching Protocols header written
	req4 := httptest.NewRequest(http.MethodGet, "/api/v1/ws", nil)
	req4.Header.Set("X-Agent-Id", "agent-trusted")
	req4.Header.Set("X-Token", rawToken)
	req4.Header.Set("Connection", "Upgrade")
	req4.Header.Set("Upgrade", "websocket")
	req4.Header.Set("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
	rec4 := httptest.NewRecorder()
	handler.WebSocket(rec4, req4)
	if rec4.Code != http.StatusSwitchingProtocols {
		t.Fatalf("expected 101 Switching Protocols, got %d", rec4.Code)
	}
}
