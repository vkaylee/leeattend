package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAttendanceAuthorizer struct {
	allowed bool
}

func (m mockAttendanceAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, _ string) bool {
	return m.allowed
}
func (m mockAttendanceAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return m.allowed, nil
}
func (mockAttendanceAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockAttendanceAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (m mockAttendanceAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool {
	return m.allowed
}
func (m mockAttendanceAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, _ string, _ *uuid.UUID) bool {
	return m.allowed
}
func (mockAttendanceAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

type mockAttendanceRepo struct {
	lastEmploymentID  uuid.UUID
	lastEntryType     string
	lastNote          *string
	lastBreakOvertime bool
	err               error
}

func (m *mockAttendanceRepo) PunchIn(_ context.Context, empID uuid.UUID, _ *uuid.UUID, entryType string, note *string, isBreakOvertime bool, punchTime time.Time) (TimeEntryRecord, error) {
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
		PunchInTime:  formattedTime,
		EntryType:    entryType,
		Note:         note,
		CreatedAt:    formattedTime,
		UpdatedAt:    formattedTime,
	}, nil
}

func (m *mockAttendanceRepo) PunchOut(_ context.Context, empID uuid.UUID, note *string, punchTime time.Time) (TimeEntryRecord, error) {
	if m.err != nil {
		return TimeEntryRecord{}, m.err
	}
	formatted := punchTime.Format(time.RFC3339)
	return TimeEntryRecord{ID: uuid.New(), EmploymentID: empID, PunchInTime: formatted, PunchOutTime: &formatted, EntryType: "work", Note: note, CreatedAt: formatted, UpdatedAt: formatted}, nil
}

func (m *mockAttendanceRepo) ListTimeHistory(_ context.Context, _ uuid.UUID, _, _ time.Time) ([]TimeEntryRecord, error) {
	if m.err != nil {
		return nil, m.err
	}
	return []TimeEntryRecord{}, nil
}

func (m *mockAttendanceRepo) CreateAdjustment(_ context.Context, req AdjustmentRequest, userID ids.UserID) (AdjustmentRecord, error) {
	if m.err != nil {
		return AdjustmentRecord{}, m.err
	}
	record := AdjustmentRecord{ID: uuid.New(), EmploymentID: req.EmploymentID, RequestedPunchTime: req.RequestedPunchTime, Reason: req.Reason, Status: "pending", RequestedByUserID: func() *uuid.UUID { value := userID.UUID(); return &value }()}
	return record, nil
}

func (m *mockAttendanceRepo) IsEmployeeOwnedByUser(_ context.Context, employeeID, userID uuid.UUID) (bool, error) {
	return employeeID == userID, nil
}

func (m *mockAttendanceRepo) ListAdjustments(_ context.Context, _ *string, _ *uuid.UUID, page, limit int) (PaginatedAdjustments, error) {
	if m.err != nil {
		return PaginatedAdjustments{}, m.err
	}
	return PaginatedAdjustments{Data: []AdjustmentRecord{}, Total: 0, Page: page, Limit: limit}, nil
}

func (m *mockAttendanceRepo) ListEmployeeAdjustments(_ context.Context, _ uuid.UUID) ([]AdjustmentRecord, error) {
	if m.err != nil {
		return nil, m.err
	}
	return []AdjustmentRecord{}, nil
}

func (m *mockAttendanceRepo) ApproveAdjustment(_ context.Context, _, _ uuid.UUID, _ string) error {
	return m.err
}

func (m *mockAttendanceRepo) ESSAttendanceAction(_ context.Context, empID uuid.UUID, action string, note *string, _ string, actionTime time.Time) (AttendanceActionResponse, error) {
	if m.err != nil {
		return AttendanceActionResponse{}, m.err
	}
	return AttendanceActionResponse{State: action, Entry: &TimeEntryRecord{ID: uuid.New(), EmploymentID: empID, PunchInTime: actionTime.Format(time.RFC3339), EntryType: "work", Note: note}}, nil
}

func (m *mockAttendanceRepo) GetESSAttendanceState(_ context.Context, _ uuid.UUID) (AttendanceActionResponse, error) {
	if m.err != nil {
		return AttendanceActionResponse{}, m.err
	}
	return AttendanceActionResponse{State: "off", NextAllowedActions: []string{AttendanceActionWorkIn}}, nil
}

func (m *mockAttendanceRepo) GetAttendanceSummaries(_ context.Context, _, _ string, _ *uuid.UUID, _ *string, page, limit int) (PaginatedSummaries, error) {
	if m.err != nil {
		return PaginatedSummaries{}, m.err
	}
	return PaginatedSummaries{Data: []AttendanceDailySummaryRecord{}, Total: 0, Page: page, Limit: limit}, nil
}

func (m *mockAttendanceRepo) GetEmployeeIDByUserID(_ context.Context, userID uuid.UUID) (uuid.UUID, error) {
	if m.err != nil {
		return uuid.Nil, m.err
	}
	return userID, nil
}

func (m *mockAttendanceRepo) UpdateTimeEntry(_ context.Context, entryID uuid.UUID, req UpdateTimeEntryRequest) (TimeEntryRecord, error) {
	if m.err != nil {
		return TimeEntryRecord{}, m.err
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return TimeEntryRecord{
		ID:           entryID,
		EmploymentID: uuid.New(),
		PunchInTime:  now,
		EntryType:    "work",
		CreatedAt:    now,
		UpdatedAt:    now,
	}, nil
}

func (m *mockAttendanceRepo) DeleteTimeEntry(_ context.Context, _ uuid.UUID) error {
	return m.err
}

func (m *mockAttendanceRepo) ReprocessAttendance(_ context.Context, startDate, endDate string, employeeID *uuid.UUID) (ReprocessResponse, error) {
	if m.err != nil {
		return ReprocessResponse{}, m.err
	}
	return ReprocessResponse{EmploymentID: employeeID, StartDate: startDate, EndDate: endDate, Days: 1, Recalculated: 1}, nil
}

func TestPunchIn_PermissionDenied(t *testing.T) {
	userID := uuid.New()
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{},
		Authorizer: mockAttendanceAuthorizer{allowed: false},
	}

	body := []byte(`{"employment_id":"` + uuid.New().String() + `"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/time/punch-in", bytes.NewReader(body))

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

	rec := httptest.NewRecorder()
	handler.PunchIn(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestPunchIn_ValidationFailed(t *testing.T) {
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
			req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/time/punch-in", bytes.NewReader([]byte(tc.body)))

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
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/time/punch-in", bytes.NewReader(body))

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
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/time/punch-out/"+empID.String(), bytes.NewReader([]byte(`{}`)))

	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.PunchOut(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestPunchOut_Success(t *testing.T) {
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/time/punch-out/"+empID.String(), bytes.NewReader([]byte(`{}`)))

	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.PunchOut(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestTimeHistory_AdminSuccess(t *testing.T) {
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/time/history/"+empID.String()+"?start=2026-01-01T00:00:00Z&end=2026-12-31T23:59:59Z", nil)

	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.TimeHistory(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestTimeHistory_MissingDateRange(t *testing.T) {
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/time/history/"+empID.String(), nil)

	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.TimeHistory(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400 or 422, got %d", rec.Code)
	}
}

func TestParseTimeHistoryQuery(t *testing.T) {
	tests := []struct {
		name  string
		query string
		valid bool
	}{
		{name: "valid range", query: "?start=2026-01-01T00:00:00Z&end=2026-12-31T23:59:59Z", valid: true},
		{name: "missing end", query: "?start=2026-01-01T00:00:00Z", valid: false},
		{name: "invalid start", query: "?start=not-a-time&end=2026-12-31T23:59:59Z", valid: false},
		{name: "invalid end", query: "?start=2026-01-01T00:00:00Z&end=not-a-time", valid: false},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, "/api/v1/time/history/"+uuid.New().String()+test.query, nil)
			_, valid := parseTimeHistoryQuery(req)
			if valid != test.valid {
				t.Fatalf("valid = %t, want %t", valid, test.valid)
			}
		})
	}
}

func TestCreateAdjustment_Success(t *testing.T) {
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body, _ := json.Marshal(AdjustmentRequest{EmploymentID: empID, RequestedPunchTime: "2026-10-19T08:00:00Z"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/adjustments", bytes.NewReader(body))

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

func TestCreateAdjustment_RejectsIncompleteTypedBreakWorked(t *testing.T) {
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body, _ := json.Marshal(AdjustmentRequest{EmploymentID: uuid.New(), Type: AttendanceAdjustmentBreakWorked, Reason: func() *string { value := "worked through break"; return &value }()})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/attendance/adjustments", bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.CreateAdjustment(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}
}

func TestCreateAdjustment_AcceptsTypedMissingPunch(t *testing.T) {
	repo := &mockAttendanceRepo{}
	handler := AttendanceHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	reason := "forgot clock-in"
	body, _ := json.Marshal(AdjustmentRequest{EmploymentID: uuid.New(), Type: AttendanceAdjustmentMissingPunch, RequestedPunchTime: "2026-10-19T08:00:00Z", Direction: "in", Reason: &reason})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/attendance/adjustments", bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.CreateAdjustment(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestCreateAdjustment_PermissionDenied(t *testing.T) {
	userID := uuid.New()
	empID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	body, _ := json.Marshal(AdjustmentRequest{EmploymentID: empID, RequestedPunchTime: "2026-10-19T08:00:00Z"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/adjustments", bytes.NewReader(body))

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.CreateAdjustment(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestListAdjustments_ForbiddenWithoutApprovePermission(t *testing.T) {
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/adjustments", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListAdjustments(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestListAdjustments_RejectsInvalidStatus(t *testing.T) {
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/adjustments?status=unknown", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListAdjustments(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}
}

func TestListAdjustments_Success(t *testing.T) {
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/adjustments?page=2&limit=10&status=pending", nil)

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
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1"+"/attendance/adjustments/"+uuid.New().String()+"/approve", bytes.NewReader([]byte(`{"status":"pending"}`)))

	req.SetPathValue("id", uuid.New().String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ApproveAdjustment(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}
}

func TestApproveAdjustment_Success(t *testing.T) {
	adjID := uuid.New()
	handler := AttendanceHandler{Repository: &mockAttendanceRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1"+"/attendance/adjustments/"+adjID.String()+"/approve", bytes.NewReader([]byte(`{"status":"approved"}`)))

	req.SetPathValue("id", adjID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ApproveAdjustment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAttendanceSummaries_Success(t *testing.T) {
	userID := uuid.New()
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{},
		Authorizer: mockAttendanceAuthorizer{allowed: true},
	}
	req := httptest.NewRequest(http.MethodGet, "/api/v1"+"/attendance/summaries?start_date=2026-10-01&end_date=2026-10-20", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

	rec := httptest.NewRecorder()
	handler.Summaries(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestAttendanceReprocessAllReturnsResult(t *testing.T) {
	userID := uuid.New()
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{},
		Authorizer: mockAttendanceAuthorizer{allowed: true},
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/reprocess-all?start_date=2026-10-01&end_date=2026-10-20", nil)

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

	rec := httptest.NewRecorder()
	handler.ReprocessAll(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "recalculated") {
		t.Fatalf("expected reprocess result: %s", rec.Body.String())
	}
}

func TestAttendanceReprocessAll_ReturnsConflictOnLockedPeriod(t *testing.T) {
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{err: ErrAttendanceLockedPeriod},
		Authorizer: mockAttendanceAuthorizer{allowed: true},
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/reprocess-all?start_date=2026-10-01&end_date=2026-10-20", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ReprocessAll(rec, req)
	if rec.Code != http.StatusConflict || !strings.Contains(rec.Body.String(), "error.locked_period") {
		t.Fatalf("expected 409 error.locked_period, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAttendanceReprocessEmployeeUnavailable(t *testing.T) {
	empID := uuid.New()
	userID := uuid.New()
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{},
		Authorizer: mockAttendanceAuthorizer{allowed: true},
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/attendance/reprocess/"+empID.String()+"?start_date=2026-10-01&end_date=2026-10-20", nil)

	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

	rec := httptest.NewRecorder()
	handler.ReprocessEmployee(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "recalculated") {
		t.Fatalf("expected reprocess result: %s", rec.Body.String())
	}
}

func TestTimeEntry_UpdateAndDelete(t *testing.T) {
	entryID := uuid.New()
	userID := uuid.New()
	handler := AttendanceHandler{
		Repository: &mockAttendanceRepo{},
		Authorizer: mockAttendanceAuthorizer{allowed: true},
	}
	body := []byte(`{"note":"Updated note"}`)
	reqUpdate := httptest.NewRequest(http.MethodPut, "/api/v1"+"/time/entries/"+entryID.String(), bytes.NewReader(body))

	reqUpdate.SetPathValue("id", entryID.String())
	reqUpdate = reqUpdate.WithContext(httpinfra.WithUserIDForTest(reqUpdate.Context(), userID))

	recUpdate := httptest.NewRecorder()
	handler.UpdateTimeEntry(recUpdate, reqUpdate)
	if recUpdate.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recUpdate.Code)
	}

	reqDelete := httptest.NewRequest(http.MethodDelete, "/api/v1"+"/time/entries/"+entryID.String(), nil)

	reqDelete.SetPathValue("id", entryID.String())
	reqDelete = reqDelete.WithContext(httpinfra.WithUserIDForTest(reqDelete.Context(), userID))

	recDelete := httptest.NewRecorder()
	handler.DeleteTimeEntry(recDelete, reqDelete)
	if recDelete.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDelete.Code)
	}
}
