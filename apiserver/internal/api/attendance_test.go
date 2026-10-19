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
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAttendanceAuthorizer struct {
	allowed bool
}

func (m mockAttendanceAuthorizer) HasPermission(_ context.Context, _, _ uuid.UUID, _ string) bool {
	return m.allowed
}

type mockAttendanceRepo struct {
	lastWorkspaceID   uuid.UUID
	lastEmploymentID  uuid.UUID
	lastEntryType     string
	lastNote          *string
	lastBreakOvertime bool
	err               error
}

func (m *mockAttendanceRepo) PunchIn(_ context.Context, wsID, empID uuid.UUID, entryType string, note *string, isBreakOvertime bool, punchTime time.Time) (TimeEntryRecord, error) {
	m.lastWorkspaceID = wsID
	m.lastEmploymentID = empID
	m.lastEntryType = entryType
	m.lastBreakOvertime = isBreakOvertime
	m.lastNote = note
	if m.err != nil {
		return TimeEntryRecord{}, m.err
	}
	formattedTime := punchTime.Format(time.RFC3339)
	return TimeEntryRecord{
		ID:           uuid.New(),
		EmploymentID: empID,
		WorkspaceID:  wsID,
		PunchInTime:  formattedTime,
		EntryType:    entryType,
		Note:         note,
		CreatedAt:    formattedTime,
		UpdatedAt:    formattedTime,
	}, nil
}

func (m *mockAttendanceRepo) PunchOut(_ context.Context, wsID, empID uuid.UUID, note *string, punchTime time.Time) (TimeEntryRecord, error) {
	if m.err != nil {
		return TimeEntryRecord{}, m.err
	}
	formatted := punchTime.Format(time.RFC3339)
	return TimeEntryRecord{ID: uuid.New(), EmploymentID: empID, WorkspaceID: wsID, PunchInTime: formatted, PunchOutTime: &formatted, EntryType: "work", Note: note, CreatedAt: formatted, UpdatedAt: formatted}, nil
}

func (m *mockAttendanceRepo) ListTimeHistory(_ context.Context, _, _ uuid.UUID, _, _ time.Time) ([]TimeEntryRecord, error) {
	if m.err != nil {
		return nil, m.err
	}
	return []TimeEntryRecord{}, nil
}

func (m *mockAttendanceRepo) CreateAdjustment(_ context.Context, wsID uuid.UUID, req AdjustmentRequest, userID uuid.UUID) (AdjustmentRecord, error) {
	if m.err != nil {
		return AdjustmentRecord{}, m.err
	}
	record := AdjustmentRecord{ID: uuid.New(), EmploymentID: req.EmploymentID, RequestedPunchTime: req.RequestedPunchTime, Reason: req.Reason, Status: "pending", RequestedByUserID: &userID}
	return record, nil
}

func (m *mockAttendanceRepo) IsEmployeeOwnedByUser(_ context.Context, _, employeeID, userID uuid.UUID) (bool, error) {
	return employeeID == userID, nil
}

func (m *mockAttendanceRepo) ListAdjustments(_ context.Context, _ uuid.UUID, _ *string, _ *uuid.UUID, page, limit int) (PaginatedAdjustments, error) {
	if m.err != nil {
		return PaginatedAdjustments{}, m.err
	}
	return PaginatedAdjustments{Data: []AdjustmentRecord{}, Total: 0, Page: page, Limit: limit}, nil
}

func (m *mockAttendanceRepo) ListEmployeeAdjustments(_ context.Context, _, _ uuid.UUID) ([]AdjustmentRecord, error) {
	if m.err != nil {
		return nil, m.err
	}
	return []AdjustmentRecord{}, nil
}

func (m *mockAttendanceRepo) ApproveAdjustment(_ context.Context, _, _, _ uuid.UUID, _ string) error {
	return m.err
}

func TestPunchIn_PermissionDenied(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{},
		Authorizer: mockAttendanceAuthorizer{allowed: false},
	}

	body := []byte(`{"employment_id":"` + uuid.New().String() + `"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/time/punch-in", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

	rec := httptest.NewRecorder()
	handler.PunchIn(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestPunchIn_ValidationFailed(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{},
		Authorizer: mockAttendanceAuthorizer{allowed: true},
	}

	tests := []struct {
		name string
		body string
	}{
		{"nil employment id", `{"employment_id":"00000000-0000-0000-0000-000000000000"}`},
		{"malformed json", `{"employment_id":`},
		{"empty body", ``},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/time/punch-in", bytes.NewReader([]byte(tc.body)))
			req.SetPathValue("ws_id", wsID.String())
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

			rec := httptest.NewRecorder()
			handler.PunchIn(rec, req)

			if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
				t.Fatalf("expected 400 or 422, got %d: %s", rec.Code, rec.Body.String())
			}
		})
	}
}

func TestPunchIn_Success(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	note := "Morning punch-in"
	repo := &mockAttendanceRepo{}
	handler := AttendanceHandler{
		Repository: repo,
		Authorizer: mockAttendanceAuthorizer{allowed: true},
	}

	payload := PunchInRequest{
		EmploymentID: empID,
		EntryType:    "work",
		Note:         &note,
	}
	body, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/time/punch-in", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

	rec := httptest.NewRecorder()
	handler.PunchIn(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp TimeEntryRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("invalid json response: %v", err)
	}

	if resp.WorkspaceID != wsID {
		t.Errorf("expected workspace ID %s, got %s", wsID, resp.WorkspaceID)
	}
	if resp.EmploymentID != empID {
		t.Errorf("expected employment ID %s, got %s", empID, resp.EmploymentID)
	}
	if resp.EntryType != "work" {
		t.Errorf("expected entry_type work, got %s", resp.EntryType)
	}
	if resp.Note == nil || *resp.Note != note {
		t.Errorf("expected note %q, got %v", note, resp.Note)
	}
}

func TestPunchOut_PermissionDenied(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/time/punch-out/"+empID.String(), bytes.NewReader([]byte(`{}`)))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.PunchOut(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestPunchOut_Success(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/time/punch-out/"+empID.String(), bytes.NewReader([]byte(`{}`)))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.PunchOut(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestTimeHistory_AdminSuccess(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/time/history/"+empID.String()+"?start=2026-01-01T00:00:00Z&end=2026-12-31T23:59:59Z", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.TimeHistory(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestTimeHistory_MissingDateRange(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/time/history/"+empID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.TimeHistory(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400 or 422, got %d", rec.Code)
	}
}

func TestCreateAdjustment_Success(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body, _ := json.Marshal(AdjustmentRequest{EmploymentID: empID, RequestedPunchTime: "2026-10-19T08:00:00Z"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/attendance/adjustments", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.CreateAdjustment(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp AdjustmentRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("invalid json: %v", err)
	}
	if resp.Status != "pending" {
		t.Errorf("expected status pending, got %s", resp.Status)
	}
}

func TestCreateAdjustment_PermissionDenied(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	body, _ := json.Marshal(AdjustmentRequest{EmploymentID: empID, RequestedPunchTime: "2026-10-19T08:00:00Z"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/attendance/adjustments", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.CreateAdjustment(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestListAdjustments_ForbiddenWithoutApprovePermission(t *testing.T) {
	wsID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/attendance/adjustments", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListAdjustments(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestListAdjustments_RejectsInvalidStatus(t *testing.T) {
	wsID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/attendance/adjustments?status=unknown", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListAdjustments(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}
}

func TestListAdjustments_Success(t *testing.T) {
	wsID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/attendance/adjustments?page=2&limit=10&status=pending", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListAdjustments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp PaginatedAdjustments
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("invalid json: %v", err)
	}
	if resp.Page != 2 || resp.Limit != 10 {
		t.Fatalf("expected page=2 limit=10, got %d/%d", resp.Page, resp.Limit)
	}
}

func TestApproveAdjustment_RejectsPendingStatus(t *testing.T) {
	wsID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/attendance/adjustments/"+uuid.New().String()+"/approve", bytes.NewReader([]byte(`{"status":"pending"}`)))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", uuid.New().String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ApproveAdjustment(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}
}

func TestApproveAdjustment_Success(t *testing.T) {
	wsID := uuid.New()
	adjID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/attendance/adjustments/"+adjID.String()+"/approve", bytes.NewReader([]byte(`{"status":"approved"}`)))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", adjID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ApproveAdjustment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}
