package api

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockMonitorRepo struct {
	alerts       map[uuid.UUID]MonitorAlertResponse
	alertEvents  map[uuid.UUID][]MonitorAlertEventResponse
	durableJobs  []MonitorInfraJobRecord
	auditLogs    []AuditLogRecord
	acknowledged map[uuid.UUID]bool
}

func newMockMonitorRepo() *mockMonitorRepo {
	return &mockMonitorRepo{
		alerts:       make(map[uuid.UUID]MonitorAlertResponse),
		alertEvents:  make(map[uuid.UUID][]MonitorAlertEventResponse),
		durableJobs:  make([]MonitorInfraJobRecord, 0),
		auditLogs:    make([]AuditLogRecord, 0),
		acknowledged: make(map[uuid.UUID]bool),
	}
}

func (m *mockMonitorRepo) ListAlerts(_ context.Context, status, severity *string, limit int, _ *string) ([]MonitorAlertResponse, int64, *string, error) {
	result := make([]MonitorAlertResponse, 0)
	for _, a := range m.alerts {
		if status != nil && *status != "" && !strings.EqualFold(a.Status, *status) {
			continue
		}
		if severity != nil && *severity != "" && !strings.EqualFold(a.Severity, *severity) {
			continue
		}
		result = append(result, a)
		if len(result) >= limit {
			break
		}
	}
	return result, int64(len(result)), nil, nil
}

func (m *mockMonitorRepo) GetAlert(_ context.Context, alertID uuid.UUID) (*MonitorAlertResponse, []MonitorAlertEventResponse, error) {
	a, ok := m.alerts[alertID]
	if !ok {
		return nil, nil, apperror.NotFound(apperror.KeyNotFound)
	}
	events := m.alertEvents[alertID]
	return &a, events, nil
}

func (m *mockMonitorRepo) MutateAlert(_ context.Context, alertID uuid.UUID, action string, reason string, _ uuid.UUID) (*MonitorAlertResponse, error) {
	a, ok := m.alerts[alertID]
	if !ok {
		return nil, apperror.NotFound(apperror.KeyNotFound)
	}
	switch action {
	case "acknowledge":
		if a.Status == "active" && m.acknowledged[alertID] {
			return &a, nil
		}
		if a.Status != "active" {
			return nil, apperror.Conflict(apperror.KeyConflict)
		}
		m.acknowledged[alertID] = true
		ev := MonitorAlertEventResponse{
			ID:         uuid.New(),
			EventType:  "acknowledged",
			Severity:   a.Severity,
			Summary:    reason,
			OccurredAt: time.Now().UTC(),
		}
		m.alertEvents[alertID] = append(m.alertEvents[alertID], ev)
		return &a, nil
	case "resolve":
		if a.Status == "resolved" {
			return &a, nil
		}
		if a.Status != "active" {
			return nil, apperror.Conflict(apperror.KeyConflict)
		}
		now := time.Now().UTC()
		a.Status = "resolved"
		a.ResolvedAt = &now
		m.alerts[alertID] = a
		ev := MonitorAlertEventResponse{
			ID:         uuid.New(),
			EventType:  "resolved",
			Severity:   a.Severity,
			Summary:    reason,
			OccurredAt: now,
		}
		m.alertEvents[alertID] = append(m.alertEvents[alertID], ev)
		return &a, nil
	default:
		return nil, fmt.Errorf("unknown action: %s", action)
	}
}

func (m *mockMonitorRepo) ListDurableJobs(_ context.Context, params ListDurableJobsParams) ([]MonitorInfraJobRecord, *string, error) {
	result := make([]MonitorInfraJobRecord, 0)
	for _, j := range m.durableJobs {
		if params.WorkspaceID != nil && j.WorkspaceID != *params.WorkspaceID {
			continue
		}
		if params.JobID != nil && j.JobID != *params.JobID {
			continue
		}
		if params.State != nil && !strings.EqualFold(j.State, *params.State) {
			continue
		}
		if params.TaskType != nil && !strings.EqualFold(j.TaskType, *params.TaskType) {
			continue
		}
		if params.Search != nil && !(strings.Contains(j.TaskType, *params.Search) || (j.ErrorCode != nil && strings.Contains(*j.ErrorCode, *params.Search))) {
			continue
		}
		result = append(result, j)
		if params.Limit > 0 && len(result) >= params.Limit {
			break
		}
	}
	return result, nil, nil
}

func (m *mockMonitorRepo) ListCorrelations(_ context.Context, requestID *uuid.UUID, traceID *string, limit int) ([]MonitorInfraJobRecord, bool, error) {
	result := make([]MonitorInfraJobRecord, 0)
	for _, j := range m.durableJobs {
		if requestID != nil && j.RequestID != nil && *j.RequestID == *requestID {
			result = append(result, j)
		} else if traceID != nil && j.TraceID != nil && *j.TraceID == *traceID {
			result = append(result, j)
		}
		if limit > 0 && len(result) >= limit {
			break
		}
	}
	return result, false, nil
}

func (m *mockMonitorRepo) GetWorkspaceDurableJob(_ context.Context, workspaceID, jobID uuid.UUID) (*MonitorInfraJobRecord, error) {
	for _, j := range m.durableJobs {
		if j.WorkspaceID == workspaceID && j.JobID == jobID {
			return &j, nil
		}
	}
	return nil, ErrMonitorNotFound
}

func (m *mockMonitorRepo) ListAuditLogs(_ context.Context, workspaceID uuid.UUID, limit, offset int, action *string, _, _ *time.Time) ([]AuditLogRecord, int64, error) {
	filtered := make([]AuditLogRecord, 0)
	for _, l := range m.auditLogs {
		if l.WorkspaceID == workspaceID {
			if action != nil && !strings.EqualFold(l.Action, *action) {
				continue
			}
			filtered = append(filtered, l)
		}
	}
	total := int64(len(filtered))
	if offset >= len(filtered) {
		return make([]AuditLogRecord, 0), total, nil
	}
	end := offset + limit
	if end > len(filtered) {
		end = len(filtered)
	}
	return filtered[offset:end], total, nil
}

type mockMonitorAuthz struct {
	members     map[string]bool
	permissions map[string]bool
}

func (a *mockMonitorAuthz) IsMember(_ context.Context, userID, workspaceID uuid.UUID) (bool, error) {
	key := fmt.Sprintf("%s:%s", userID, workspaceID)
	return a.members[key], nil
}

func (a *mockMonitorAuthz) HasPermission(_ context.Context, userID, workspaceID uuid.UUID, permission string) bool {
	key := fmt.Sprintf("%s:%s:%s", userID, workspaceID, permission)
	return a.permissions[key]
}

type mockMonitorRoles struct {
	roles map[uuid.UUID][]string
}

func (r *mockMonitorRoles) GetSystemRolesForUser(_ context.Context, userID uuid.UUID) ([]string, error) {
	return r.roles[userID], nil
}

func setReqContext(r *http.Request, userID uuid.UUID) *http.Request {
	return r.WithContext(httpinfra.WithUserIDForTest(r.Context(), userID))
}

func TestMonitorOverview_Auth(t *testing.T) {
	roles := &mockMonitorRoles{roles: make(map[uuid.UUID][]string)}
	handler := MonitorHandler{
		RolesRepo: roles,
	}

	// 1. Missing user context -> 401
	req := httptest.NewRequest(http.MethodGet, "/api/v1/monitor/overview", nil)
	w := httptest.NewRecorder()
	handler.GetOverview(w, req)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", w.Code)
	}

	// 2. Regular user (no platform_admin or support) -> 403
	userNormal := uuid.New()
	roles.roles[userNormal] = []string{"billing_admin"}
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/overview", nil), userNormal)
	w = httptest.NewRecorder()
	handler.GetOverview(w, req)
	if w.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", w.Code)
	}

	// 3. PlatformAdmin -> 200
	userAdmin := uuid.New()
	roles.roles[userAdmin] = []string{"platform_admin"}
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/overview", nil), userAdmin)
	w = httptest.NewRecorder()
	handler.GetOverview(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}

	// 4. Support -> 200
	userSupport := uuid.New()
	roles.roles[userSupport] = []string{"support"}
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/overview", nil), userSupport)
	w = httptest.NewRecorder()
	handler.GetOverview(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}

	// 5. Unified Admin -> 200
	userUnifiedAdmin := uuid.New()
	roles.roles[userUnifiedAdmin] = []string{"admin"}
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/overview", nil), userUnifiedAdmin)
	w = httptest.NewRecorder()
	handler.GetOverview(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200 for unified admin, got %d", w.Code)
	}
}

func TestMonitorJobs_Validation(t *testing.T) {
	roles := &mockMonitorRoles{roles: make(map[uuid.UUID][]string)}
	admin := uuid.New()
	roles.roles[admin] = []string{"platform_admin"}
	handler := MonitorHandler{RolesRepo: roles}

	// Invalid status
	req := setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/jobs?status=bogus", nil), admin)
	w := httptest.NewRecorder()
	handler.GetMonitorJobs(w, req)
	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad status, got %d", w.Code)
	}

	// Overlong search (>64)
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/jobs?search="+strings.Repeat("a", 65), nil), admin)
	w = httptest.NewRecorder()
	handler.GetMonitorJobs(w, req)
	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for overlong search, got %d", w.Code)
	}

	// Invalid state
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/jobs?state=nonexistent", nil), admin)
	w = httptest.NewRecorder()
	handler.GetMonitorJobs(w, req)
	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad state, got %d", w.Code)
	}

	// Valid request
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/jobs?status=pending&state=queued", nil), admin)
	w = httptest.NewRecorder()
	handler.GetMonitorJobs(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}
}

func TestMonitorCorrelations_Validation(t *testing.T) {
	roles := &mockMonitorRoles{roles: make(map[uuid.UUID][]string)}
	admin := uuid.New()
	roles.roles[admin] = []string{"platform_admin"}
	handler := MonitorHandler{RolesRepo: roles}

	// Both empty -> 400
	req := setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/correlations", nil), admin)
	w := httptest.NewRecorder()
	handler.GetMonitorCorrelations(w, req)
	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 when both empty, got %d", w.Code)
	}

	// Both provided -> 400
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/correlations?request_id="+uuid.NewString()+"&trace_id=trace-1", nil), admin)
	w = httptest.NewRecorder()
	handler.GetMonitorCorrelations(w, req)
	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 when both provided, got %d", w.Code)
	}

	// Invalid UUID request_id -> 400
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/correlations?request_id=not-a-uuid", nil), admin)
	w = httptest.NewRecorder()
	handler.GetMonitorCorrelations(w, req)
	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad uuid, got %d", w.Code)
	}

	// Invalid trace_id chars -> 400
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/correlations?trace_id=trace%20with%20spaces", nil), admin)
	w = httptest.NewRecorder()
	handler.GetMonitorCorrelations(w, req)
	if w.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad trace_id chars, got %d", w.Code)
	}

	// Valid trace_id -> 200
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/correlations?trace_id=valid-trace_123.abc", nil), admin)
	w = httptest.NewRecorder()
	handler.GetMonitorCorrelations(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200 for valid trace_id, got %d", w.Code)
	}
}

func TestMonitorStream_SSE(t *testing.T) {
	roles := &mockMonitorRoles{roles: make(map[uuid.UUID][]string)}
	admin := uuid.New()
	roles.roles[admin] = []string{"platform_admin"}
	handler := MonitorHandler{RolesRepo: roles}

	ctx, cancel := context.WithCancel(context.Background())
	req := setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/stream", nil).WithContext(ctx), admin)
	w := httptest.NewRecorder()

	go func() {
		time.Sleep(100 * time.Millisecond)
		cancel()
	}()

	handler.Stream(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}
	body := w.Body.String()
	if !strings.Contains(body, "event: monitor-update") {
		t.Fatalf("expected event: monitor-update in body, got: %s", body)
	}
}

func TestMonitorAlerts_CRUD_And_Permissions(t *testing.T) {
	repo := newMockMonitorRepo()
	roles := &mockMonitorRoles{roles: make(map[uuid.UUID][]string)}
	admin := uuid.New()
	support := uuid.New()
	regular := uuid.New()
	roles.roles[admin] = []string{"platform_admin"}
	roles.roles[support] = []string{"support"}
	roles.roles[regular] = []string{"viewer"}

	handler := MonitorHandler{
		Repo:      repo,
		RolesRepo: roles,
	}

	alertID := uuid.New()
	now := time.Now().UTC()
	repo.alerts[alertID] = MonitorAlertResponse{
		ID:              alertID,
		AlertType:       "queue_pressure",
		Severity:        "critical",
		Status:          "active",
		Title:           "Queue pressure critical",
		Summary:         "Queue high depth exceeded 1000",
		Source:          "worker.queue",
		FirstSeenAt:     now,
		LastSeenAt:      now,
		OccurrenceCount: 1,
	}

	// 1. List alerts (Support -> 200)
	req := setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/alerts", nil), support)
	w := httptest.NewRecorder()
	handler.ListAlerts(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}

	// 2. Get alert (Support -> 200)
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/monitor/alerts/"+alertID.String(), nil), support)
	req.SetPathValue("alert_id", alertID.String())
	w = httptest.NewRecorder()
	handler.GetAlert(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}

	// 3. Acknowledge with Support -> MUST return 403 Forbidden!
	ackBody := `{"reason":"Investigating incident"}`
	req = setReqContext(httptest.NewRequest(http.MethodPost, "/api/v1/monitor/alerts/"+alertID.String()+"/acknowledge", strings.NewReader(ackBody)), support)
	req.SetPathValue("alert_id", alertID.String())
	w = httptest.NewRecorder()
	handler.AcknowledgeAlert(w, req)
	if w.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for Support acknowledge, got %d", w.Code)
	}

	// 4. Acknowledge with PlatformAdmin -> 200 OK
	req = setReqContext(httptest.NewRequest(http.MethodPost, "/api/v1/monitor/alerts/"+alertID.String()+"/acknowledge", strings.NewReader(ackBody)), admin)
	req.SetPathValue("alert_id", alertID.String())
	w = httptest.NewRecorder()
	handler.AcknowledgeAlert(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200 for Admin acknowledge, got %d", w.Code)
	}

	// 5. Idempotent acknowledge -> 200 OK
	req = setReqContext(httptest.NewRequest(http.MethodPost, "/api/v1/monitor/alerts/"+alertID.String()+"/acknowledge", strings.NewReader(ackBody)), admin)
	req.SetPathValue("alert_id", alertID.String())
	w = httptest.NewRecorder()
	handler.AcknowledgeAlert(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200 for idempotent acknowledge, got %d", w.Code)
	}

	// 6. Resolve with Support -> MUST return 403 Forbidden!
	resBody := `{"reason":"Issue resolved"}`
	req = setReqContext(httptest.NewRequest(http.MethodPost, "/api/v1/monitor/alerts/"+alertID.String()+"/resolve", strings.NewReader(resBody)), support)
	req.SetPathValue("alert_id", alertID.String())
	w = httptest.NewRecorder()
	handler.ResolveAlert(w, req)
	if w.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for Support resolve, got %d", w.Code)
	}

	// 7. Resolve with PlatformAdmin -> 200 OK
	req = setReqContext(httptest.NewRequest(http.MethodPost, "/api/v1/monitor/alerts/"+alertID.String()+"/resolve", strings.NewReader(resBody)), admin)
	req.SetPathValue("alert_id", alertID.String())
	w = httptest.NewRecorder()
	handler.ResolveAlert(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200 for Admin resolve, got %d", w.Code)
	}

	// 8. Subsequent acknowledge on resolved alert -> 409 Conflict!
	req = setReqContext(httptest.NewRequest(http.MethodPost, "/api/v1/monitor/alerts/"+alertID.String()+"/acknowledge", strings.NewReader(ackBody)), admin)
	req.SetPathValue("alert_id", alertID.String())
	w = httptest.NewRecorder()
	handler.AcknowledgeAlert(w, req)
	if w.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict for acknowledge on resolved alert, got %d", w.Code)
	}
}

func TestWorkspaceAuditLogs(t *testing.T) {
	wsID := uuid.New()
	userNoPerm := uuid.New()
	userWithPerm := uuid.New()

	authz := &mockMonitorAuthz{
		members: map[string]bool{
			fmt.Sprintf("%s:%s", userNoPerm, wsID):   true,
			fmt.Sprintf("%s:%s", userWithPerm, wsID): true,
		},
		permissions: map[string]bool{
			fmt.Sprintf("%s:%s:%s", userWithPerm, wsID, auth.PermissionAuditLogsRead): true,
		},
	}
	repo := newMockMonitorRepo()
	repo.auditLogs = append(repo.auditLogs, AuditLogRecord{
		ID:          uuid.New(),
		WorkspaceID: wsID,
		Action:      "member.invite",
		CreatedAt:   time.Now().UTC(),
	})

	handler := MonitorHandler{
		Repo:       repo,
		Authorizer: authz,
	}

	// 1. User without permission -> 403
	req := setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/audit-logs", nil), userNoPerm)
	req.SetPathValue("ws_id", wsID.String())
	w := httptest.NewRecorder()
	handler.GetAuditLogs(w, req)
	if w.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", w.Code)
	}

	// 2. User with permission -> 200
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/audit-logs", nil), userWithPerm)
	req.SetPathValue("ws_id", wsID.String())
	w = httptest.NewRecorder()
	handler.GetAuditLogs(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}
	var resp PaginatedAuditLogs
	if err := json.NewDecoder(w.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if resp.Total != 1 || len(resp.Data) != 1 {
		t.Fatalf("expected 1 audit log record, got %d", resp.Total)
	}
}

func TestWorkspaceJobStatus(t *testing.T) {
	wsID := uuid.New()
	otherWsID := uuid.New()
	user := uuid.New()

	authz := &mockMonitorAuthz{
		members: map[string]bool{
			fmt.Sprintf("%s:%s", user, wsID): true,
		},
	}

	handler := MonitorHandler{
		Authorizer: authz,
	}

	// Non-member -> 403
	req := setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+otherWsID.String()+"/jobs/"+uuid.NewString(), nil), user)
	req.SetPathValue("ws_id", otherWsID.String())
	req.SetPathValue("job_id", uuid.NewString())
	w := httptest.NewRecorder()
	handler.GetWorkspaceJobStatus(w, req)
	if w.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for non-member, got %d", w.Code)
	}
}

func TestWorkspaceDurableMonitorJob(t *testing.T) {
	wsID := uuid.New()
	user := uuid.New()
	jobID := uuid.New()

	authz := &mockMonitorAuthz{
		members: map[string]bool{
			fmt.Sprintf("%s:%s", user, wsID): true,
		},
	}
	repo := newMockMonitorRepo()
	repo.durableJobs = append(repo.durableJobs, MonitorInfraJobRecord{
		JobID:       jobID,
		WorkspaceID: wsID,
		TaskType:    "payroll_calc",
		Priority:    "high",
		State:       "completed",
		CreatedAt:   time.Now().UTC(),
	})

	handler := MonitorHandler{
		Repo:       repo,
		Authorizer: authz,
	}

	// 1. Not found -> 404
	req := setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/monitor/jobs/"+uuid.NewString(), nil), user)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", uuid.NewString())
	w := httptest.NewRecorder()
	handler.GetWorkspaceMonitorJob(w, req)
	if w.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", w.Code)
	}

	// 2. Found -> 200
	req = setReqContext(httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/monitor/jobs/"+jobID.String(), nil), user)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", jobID.String())
	w = httptest.NewRecorder()
	handler.GetWorkspaceMonitorJob(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", w.Code)
	}
}

func TestReportStreamWS_PreUpgradeAuth(t *testing.T) {
	jwtSecret := "supersecretjwtkey123456789012345"
	jobID := uuid.New()

	handler := MonitorHandler{
		JWTSecret: jwtSecret,
	}

	// 1. Missing token -> 401
	req := httptest.NewRequest(http.MethodGet, "/api/v1/ws/reports/"+jobID.String(), nil)
	req.SetPathValue("job_id", jobID.String())
	w := httptest.NewRecorder()
	handler.ReportStreamWS(w, req)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for missing token, got %d", w.Code)
	}

	// 2. Invalid token -> 401
	req = httptest.NewRequest(http.MethodGet, "/api/v1/ws/reports/"+jobID.String()+"?token=invalid.token.here", nil)
	req.SetPathValue("job_id", jobID.String())
	w = httptest.NewRecorder()
	handler.ReportStreamWS(w, req)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for invalid token, got %d", w.Code)
	}
}
