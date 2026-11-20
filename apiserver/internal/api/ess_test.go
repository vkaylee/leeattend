package api

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockESSRepository struct {
	profile          MyProfileResponse
	profileRecord    EmployeeProfileRecord
	contacts         []EmployeeEmergencyContactRecord
	empID            uuid.UUID
	resolveByUser    map[uuid.UUID]uuid.UUID
	resolvedUserID   uuid.UUID
	lastEmployeeID   uuid.UUID
	lastContactID    uuid.UUID
	lastDocumentID   uuid.UUID
	resolveErr       error
	getProfileErr    error
	upsertProfileErr error
	getContactsErr   error
	addContactErr    error
	deleteContactErr error
	addCalls         int
	deleteCalls      int
	upsertCalls      int
	pendingDocs      []AcknowledgementRecord
	pendingDocsErr   error
	ackDocErr        error
	empDocs          []EmployeeDocumentRecord
	empDocsErr       error
}

func (m *mockESSRepository) ResolveEmployeeForUser(_ context.Context, userID uuid.UUID) (uuid.UUID, error) {
	m.resolvedUserID = userID
	if m.resolveErr != nil {
		return uuid.Nil, m.resolveErr
	}
	if empID, ok := m.resolveByUser[userID]; ok {
		return empID, nil
	}
	return m.empID, nil
}

func (m *mockESSRepository) GetMyProfile(_ context.Context, employeeID uuid.UUID) (MyProfileResponse, error) {
	m.lastEmployeeID = employeeID
	return m.profile, m.getProfileErr
}
func (m *mockESSRepository) UpsertProfile(_ context.Context, employeeID uuid.UUID, _ UpdateEmployeeProfileRequest) (EmployeeProfileRecord, error) {
	m.lastEmployeeID = employeeID
	m.upsertCalls++
	return m.profileRecord, m.upsertProfileErr
}
func (m *mockESSRepository) GetEmergencyContacts(_ context.Context, employeeID uuid.UUID) ([]EmployeeEmergencyContactRecord, error) {
	m.lastEmployeeID = employeeID
	return m.contacts, m.getContactsErr
}
func (m *mockESSRepository) AddEmergencyContact(_ context.Context, employeeID uuid.UUID, _ CreateEmergencyContactRequest) (EmployeeEmergencyContactRecord, error) {
	m.lastEmployeeID = employeeID
	m.addCalls++
	return EmployeeEmergencyContactRecord{ID: uuid.New(), EmployeeID: employeeID, ContactName: "Family", Relationship: "Spouse", Phone: "123456789"}, m.addContactErr
}
func (m *mockESSRepository) DeleteEmergencyContact(_ context.Context, employeeID, contactID uuid.UUID) error {
	m.lastEmployeeID, m.lastContactID = employeeID, contactID
	m.deleteCalls++
	return m.deleteContactErr
}

func (m *mockESSRepository) GetPendingDocuments(_ context.Context, employeeID uuid.UUID) ([]AcknowledgementRecord, error) {
	m.lastEmployeeID = employeeID
	return m.pendingDocs, m.pendingDocsErr
}
func (m *mockESSRepository) AcknowledgeDocument(_ context.Context, employeeID, docID uuid.UUID) error {
	m.lastEmployeeID, m.lastDocumentID = employeeID, docID
	return m.ackDocErr
}
func (m *mockESSRepository) GetEmployeeDocuments(_ context.Context, employeeID uuid.UUID) ([]EmployeeDocumentRecord, error) {
	m.lastEmployeeID = employeeID
	return m.empDocs, m.empDocsErr
}

type mockESSAttendanceRepo struct {
	AttendanceRepository
	punchErr       error
	history        []TimeEntryRecord
	historyErr     error
	lastEmployeeID uuid.UUID
	actionResponse AttendanceActionResponse
}

func (m *mockESSAttendanceRepo) ESSAttendanceAction(_ context.Context, empID uuid.UUID, action string, note *string, _ string, actionTime time.Time) (AttendanceActionResponse, error) {
	m.lastEmployeeID = empID
	if m.punchErr != nil {
		return AttendanceActionResponse{}, m.punchErr
	}
	if m.actionResponse.State != "" {
		return m.actionResponse, nil
	}
	return AttendanceActionResponse{State: action, NextAllowedActions: []string{AttendanceActionWorkIn}, Entry: &TimeEntryRecord{ID: uuid.New(), EmploymentID: empID, PunchInTime: actionTime.Format(time.RFC3339), EntryType: "work", Note: note}}, nil
}

func (m *mockESSAttendanceRepo) GetESSAttendanceState(_ context.Context, employeeID uuid.UUID) (AttendanceActionResponse, error) {
	m.lastEmployeeID = employeeID
	if m.punchErr != nil {
		return AttendanceActionResponse{}, m.punchErr
	}
	return m.actionResponse, nil
}

func (m *mockESSAttendanceRepo) ListTimeHistory(_ context.Context, employeeID uuid.UUID, _, _ time.Time) ([]TimeEntryRecord, error) {
	m.lastEmployeeID = employeeID
	return m.history, m.historyErr
}

type mockESSPayrollRepo struct {
	PayrollRepository
	payslips       []PayslipRecord
	payslipsErr    error
	lastEmployeeID uuid.UUID
}

func (m *mockESSPayrollRepo) ListEmployeePayslips(_ context.Context, employeeID ids.EmployeeID) ([]PayslipRecord, error) {
	m.lastEmployeeID = uuid.UUID(employeeID)
	return m.payslips, m.payslipsErr
}

type mockESSLeaveRepo struct {
	LeaveRepository
	balances       []EmployeeLeaveBalanceRecord
	balancesErr    error
	requests       []LeaveRequestRecord
	requestsErr    error
	lastEmployeeID uuid.UUID
}

func (m *mockESSLeaveRepo) GetEmployeeLeaveBalances(_ context.Context, employeeID uuid.UUID, _ int) ([]EmployeeLeaveBalanceRecord, error) {
	m.lastEmployeeID = employeeID
	return m.balances, m.balancesErr
}

func (m *mockESSLeaveRepo) ListEmployeeLeaveRequests(_ context.Context, employeeID uuid.UUID) ([]LeaveRequestRecord, error) {
	m.lastEmployeeID = employeeID
	return m.requests, m.requestsErr
}

type mockESSShiftSwapRepo struct {
	ShiftSwapRepository
	swaps           []ShiftSwapRequestWithNames
	swapsErr        error
	createErr       error
	respondErr      error
	cancelErr       error
	lastEmployeeID  uuid.UUID
	lastRequesterID uuid.UUID
	lastUserID      uuid.UUID
}

func (m *mockESSShiftSwapRepo) ListEmployeeSwaps(_ context.Context, employeeID uuid.UUID) ([]ShiftSwapRequestWithNames, error) {
	m.lastEmployeeID = employeeID
	return m.swaps, m.swapsErr
}

func (m *mockESSShiftSwapRepo) CreateESSSwap(_ context.Context, reqID, userID uuid.UUID, req EssCreateShiftSwapRequest) (ShiftSwapRequest, error) {
	m.lastRequesterID, m.lastUserID = reqID, userID
	if m.createErr != nil {
		return ShiftSwapRequest{}, m.createErr
	}
	return ShiftSwapRequest{
		ID:                uuid.New(),
		RequesterID:       reqID,
		TargetID:          req.TargetID,
		SwapDate:          req.SwapDate,
		Status:            ShiftSwapPendingTarget,
		Reason:            req.Reason,
		RequestedByUserID: &userID,
	}, nil
}

func (m *mockESSShiftSwapRepo) RespondSwap(_ context.Context, employeeID, _ uuid.UUID, _ bool) error {
	m.lastEmployeeID = employeeID
	return m.respondErr
}

func (m *mockESSShiftSwapRepo) CancelSwap(_ context.Context, employeeID, _ uuid.UUID) error {
	m.lastEmployeeID = employeeID
	return m.cancelErr
}

func essRequest(t *testing.T, method, path, body string) *http.Request {
	t.Helper()
	return essRequestAs(t, uuid.New(), method, path, body)
}

func essRequestAs(t *testing.T, userID uuid.UUID, method, path, body string) *http.Request {
	t.Helper()
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	ctx := httpinfra.WithUserIDForTest(req.Context(), userID)
	return req.WithContext(ctx)
}

func TestESSGetProfileNoLinkedEmployeeForbidden(t *testing.T) {
	repo := &mockESSRepository{resolveErr: ErrESSNoEmployeeRecord}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/profile", "")
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestESSGetProfileUsesAuthenticatedUserEmployee(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{
		empID:         uuid.New(),
		resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID},
		profile:       MyProfileResponse{Employee: EmployeeRecord{ID: empID, FullName: "John Doe"}},
	}
	handler := ESSHandler{Repository: repo}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/profile", "")
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.resolvedUserID != userID {
		t.Fatalf("expected resolution for user %s, got %s", userID, repo.resolvedUserID)
	}
	if repo.lastEmployeeID != empID {
		t.Fatalf("expected employee %s, got %s", empID, repo.lastEmployeeID)
	}
	if !strings.Contains(rec.Body.String(), "John Doe") {
		t.Fatalf("profile body missing name: %s", rec.Body.String())
	}
}

func TestESSUpdateProfileValidatesDOB(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	handler := ESSHandler{Repository: repo}
	req := essRequestAs(t, userID, http.MethodPut, "/api/v1/ess/profile", `{"date_of_birth":"invalid-date"}`)
	rec := httptest.NewRecorder()
	handler.UpdateProfile(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad DOB, got %d", rec.Code)
	}
	if repo.upsertCalls != 0 {
		t.Fatal("upsert called on validation failure")
	}
}

func TestESSAddContactValidatesFields(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	handler := ESSHandler{Repository: repo}
	req := essRequestAs(t, userID, http.MethodPost, "/api/v1/ess/emergency-contacts", `{"contact_name":""}`)
	rec := httptest.NewRecorder()
	handler.AddContact(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing fields, got %d", rec.Code)
	}
	if repo.addCalls != 0 {
		t.Fatal("add contact called on validation failure")
	}
}

func TestESSDeleteContactScopesToAuthenticatedEmployee(t *testing.T) {
	userID, empID, contactID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	handler := ESSHandler{Repository: repo}
	req := essRequestAs(t, userID, http.MethodDelete, "/api/v1/ess/emergency-contacts/"+contactID.String(), "")
	req.SetPathValue("contact_id", contactID.String())
	rec := httptest.NewRecorder()
	handler.DeleteContact(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	if repo.deleteCalls != 1 {
		t.Fatalf("expected 1 delete call, got %d", repo.deleteCalls)
	}
	if repo.lastEmployeeID != empID || repo.lastContactID != contactID {
		t.Fatalf("expected employee %s contact %s, got employee %s contact %s", empID, contactID, repo.lastEmployeeID, repo.lastContactID)
	}
}

func TestESSRequiresAuthenticatedPrincipal(t *testing.T) {
	repo := &mockESSRepository{}
	handler := ESSHandler{Repository: repo}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/ess/profile", nil)
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without an authenticated principal, got %d", rec.Code)
	}
	if repo.resolvedUserID != uuid.Nil {
		t.Fatalf("employee resolution attempted without a principal: %s", repo.resolvedUserID)
	}
}

func TestESSGetPendingDocumentsSuccess(t *testing.T) {
	userID, empID, docID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{
		empID:         empID,
		resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID},
		pendingDocs: []AcknowledgementRecord{
			{ID: uuid.New(), DocumentID: docID, EmployeeID: empID, Status: "pending"},
		},
	}
	handler := ESSHandler{Repository: repo}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/documents/pending", "")
	rec := httptest.NewRecorder()
	handler.GetPendingDocuments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if repo.lastEmployeeID != empID {
		t.Fatalf("expected employee %s, got %s", empID, repo.lastEmployeeID)
	}
	var items []AcknowledgementRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &items); err != nil || len(items) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSAcknowledgeDocumentSuccess(t *testing.T) {
	userID, empID, ackID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	handler := ESSHandler{Repository: repo}
	req := essRequestAs(t, userID, http.MethodPost, "/api/v1/ess/documents/"+ackID.String()+"/acknowledge", "")
	req.SetPathValue("id", ackID.String())
	rec := httptest.NewRecorder()
	handler.AcknowledgeDocument(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.lastEmployeeID != empID || repo.lastDocumentID != ackID {
		t.Fatalf("expected employee %s document %s, got employee %s document %s", empID, ackID, repo.lastEmployeeID, repo.lastDocumentID)
	}
	if !strings.Contains(rec.Body.String(), "acknowledged") {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetDocumentsSuccess(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{
		empID:         empID,
		resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID},
		empDocs: []EmployeeDocumentRecord{
			{ID: uuid.New(), EmployeeID: empID, DocumentType: "contract", FileName: "contract.pdf"},
		},
	}
	handler := ESSHandler{Repository: repo}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/documents", "")
	rec := httptest.NewRecorder()
	handler.GetDocuments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if repo.lastEmployeeID != empID {
		t.Fatalf("expected employee %s, got %s", empID, repo.lastEmployeeID)
	}
	if strings.Contains(rec.Body.String(), "workspace_id") {
		t.Fatalf("document body still exposes workspace scope: %s", rec.Body.String())
	}
	var docs []EmployeeDocumentRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &docs); err != nil || len(docs) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetPayslipsSuccess(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	payroll := &mockESSPayrollRepo{
		payslips: []PayslipRecord{
			{ID: uuid.New(), EmploymentID: empID, NetPayAmount: decimal.RequireFromString("2500.00")},
		},
	}
	handler := ESSHandler{Repository: repo, Payroll: payroll}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/payslips", "")
	rec := httptest.NewRecorder()
	handler.GetPayslips(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if payroll.lastEmployeeID != empID {
		t.Fatalf("expected payslips for employee %s, got %s", empID, payroll.lastEmployeeID)
	}
	var payslips []PayslipRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &payslips); err != nil || len(payslips) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetAttendanceSuccess(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	attendance := &mockESSAttendanceRepo{
		history: []TimeEntryRecord{
			{ID: uuid.New(), EmploymentID: empID, PunchInTime: "2026-10-20T08:00:00Z"},
		},
	}
	handler := ESSHandler{Repository: repo, Attendance: attendance}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/attendance?start_date=2026-10-01&end_date=2026-10-20", "")
	rec := httptest.NewRecorder()
	handler.GetAttendance(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if attendance.lastEmployeeID != empID {
		t.Fatalf("expected attendance for employee %s, got %s", empID, attendance.lastEmployeeID)
	}
	var entries []TimeEntryRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &entries); err != nil || len(entries) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSAttendanceAction(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	attendance := &mockESSAttendanceRepo{}
	handler := ESSHandler{Repository: repo, Attendance: attendance, Authorizer: mockAttendanceAuthorizer{allowed: true}}

	req := essRequestAs(t, userID, http.MethodPost, "/api/v1/ess/attendance/actions", `{"action":"work_in","note":"starting shift"}`)
	req.Header.Set("Idempotency-Key", uuid.NewString())
	rec := httptest.NewRecorder()
	handler.AttendanceAction(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if attendance.lastEmployeeID != empID {
		t.Fatalf("expected action for employee %s, got %s", empID, attendance.lastEmployeeID)
	}
}

func TestESSAttendanceActionNoActiveEmployment(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	attendance := &mockESSAttendanceRepo{punchErr: ErrNoActiveEmployment}
	handler := ESSHandler{Repository: repo, Attendance: attendance, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := essRequestAs(t, userID, http.MethodPost, "/api/v1/ess/attendance/actions", `{"action":"work_in"}`)
	req.Header.Set("Idempotency-Key", uuid.NewString())
	rec := httptest.NewRecorder()
	handler.AttendanceAction(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for no active employment, got %d", rec.Code)
	}
}

func TestESSGetTimeEntriesSuccess(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	attendance := &mockESSAttendanceRepo{
		history: []TimeEntryRecord{
			{ID: uuid.New(), EmploymentID: empID, PunchInTime: "2026-10-20T09:00:00Z"},
		},
	}
	handler := ESSHandler{Repository: repo, Attendance: attendance}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/time-entries?start_date=2026-10-01T00:00:00Z&end_date=2026-10-20T23:59:59Z", "")
	rec := httptest.NewRecorder()
	handler.GetTimeEntries(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if attendance.lastEmployeeID != empID {
		t.Fatalf("expected time entries for employee %s, got %s", empID, attendance.lastEmployeeID)
	}
	var entries []TimeEntryRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &entries); err != nil || len(entries) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetLeaveBalancesSuccess(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	leaves := &mockESSLeaveRepo{
		balances: []EmployeeLeaveBalanceRecord{
			{
				LeaveTypeCode: "ANNUAL",
				Entitlement:   12,
				AccruedAmount: decimal.RequireFromString("10.0"),
				CarriedOver:   2,
				Used:          3,
				Remaining:     decimal.RequireFromString("9.0"),
			},
		},
	}
	handler := ESSHandler{Repository: repo, Leaves: leaves}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/leave/balances", "")
	rec := httptest.NewRecorder()
	handler.GetLeaveBalances(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if leaves.lastEmployeeID != empID {
		t.Fatalf("expected balances for employee %s, got %s", empID, leaves.lastEmployeeID)
	}
	var balances []EssLeaveBalance
	if err := json.Unmarshal(rec.Body.Bytes(), &balances); err != nil || len(balances) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
	if balances[0].LeaveType != "ANNUAL" || balances[0].Remaining != 9.0 {
		t.Fatalf("unexpected balance: %+v", balances[0])
	}
}

func TestESSGetLeaveRequestsSuccess(t *testing.T) {
	userID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	leaves := &mockESSLeaveRepo{
		requests: []LeaveRequestRecord{
			{ID: uuid.New(), EmploymentID: empID, Status: "pending"},
		},
	}
	handler := ESSHandler{Repository: repo, Leaves: leaves}
	req := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/leave/requests", "")
	rec := httptest.NewRecorder()
	handler.GetLeaveRequests(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if leaves.lastEmployeeID != empID {
		t.Fatalf("expected leave requests for employee %s, got %s", empID, leaves.lastEmployeeID)
	}
	var requests []LeaveRequestRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &requests); err != nil || len(requests) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSShiftSwapsCRUD(t *testing.T) {
	userID, empID, targetID, swapID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID, resolveByUser: map[uuid.UUID]uuid.UUID{userID: empID}}
	shiftSwaps := &mockESSShiftSwapRepo{
		swaps: []ShiftSwapRequestWithNames{
			{ShiftSwapRequest: ShiftSwapRequest{ID: swapID, RequesterID: empID, TargetID: targetID, Status: ShiftSwapPendingTarget}},
		},
	}
	handler := ESSHandler{Repository: repo, ShiftSwaps: shiftSwaps}

	// 1. List swaps
	reqList := essRequestAs(t, userID, http.MethodGet, "/api/v1/ess/shift-swaps", "")
	recList := httptest.NewRecorder()
	handler.GetShiftSwaps(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200 for list swaps, got %d", recList.Code)
	}
	if shiftSwaps.lastEmployeeID != empID {
		t.Fatalf("expected swaps for employee %s, got %s", empID, shiftSwaps.lastEmployeeID)
	}

	// 2. Create swap
	createBody := `{"target_id":"` + targetID.String() + `","swap_date":"2026-10-25","reason":"vacation"}`
	reqCreate := essRequestAs(t, userID, http.MethodPost, "/api/v1/ess/shift-swaps", createBody)
	recCreate := httptest.NewRecorder()
	handler.CreateShiftSwap(recCreate, reqCreate)
	if recCreate.Code != http.StatusCreated {
		t.Fatalf("expected 201 for create swap, got %d: %s", recCreate.Code, recCreate.Body.String())
	}
	if shiftSwaps.lastRequesterID != empID || shiftSwaps.lastUserID != userID {
		t.Fatalf("expected requester %s user %s, got requester %s user %s", empID, userID, shiftSwaps.lastRequesterID, shiftSwaps.lastUserID)
	}
	var created ShiftSwapRequest
	if err := json.Unmarshal(recCreate.Body.Bytes(), &created); err != nil || created.Status != ShiftSwapPendingTarget {
		t.Fatalf("unexpected created swap: %+v", created)
	}

	// 3. Create swap targeting self -> 400
	selfBody := `{"target_id":"` + empID.String() + `","swap_date":"2026-10-25"}`
	reqSelf := essRequestAs(t, userID, http.MethodPost, "/api/v1/ess/shift-swaps", selfBody)
	recSelf := httptest.NewRecorder()
	handler.CreateShiftSwap(recSelf, reqSelf)
	if recSelf.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for self target, got %d", recSelf.Code)
	}

	// 4. Respond to swap (accept) -> 200
	reqRespond := essRequestAs(t, userID, http.MethodPut, "/api/v1/ess/shift-swaps/"+swapID.String()+"/respond", `{"accept":true}`)
	reqRespond.SetPathValue("id", swapID.String())
	recRespond := httptest.NewRecorder()
	handler.RespondShiftSwap(recRespond, reqRespond)
	if recRespond.Code != http.StatusOK {
		t.Fatalf("expected 200 for accept swap, got %d", recRespond.Code)
	}
	if shiftSwaps.lastEmployeeID != empID {
		t.Fatalf("expected response from employee %s, got %s", empID, shiftSwaps.lastEmployeeID)
	}

	// 5. Respond to swap (reject) -> 200
	reqReject := essRequestAs(t, userID, http.MethodPut, "/api/v1/ess/shift-swaps/"+swapID.String()+"/respond", `{"accept":false}`)
	reqReject.SetPathValue("id", swapID.String())
	recReject := httptest.NewRecorder()
	handler.RespondShiftSwap(recReject, reqReject)
	if recReject.Code != http.StatusOK {
		t.Fatalf("expected 200 for reject swap, got %d", recReject.Code)
	}

	// 6. Respond forbidden -> 403
	shiftSwaps.respondErr = ErrShiftSwapForbidden
	reqForbidden := essRequestAs(t, userID, http.MethodPut, "/api/v1/ess/shift-swaps/"+swapID.String()+"/respond", `{"accept":true}`)
	reqForbidden.SetPathValue("id", swapID.String())
	recForbidden := httptest.NewRecorder()
	handler.RespondShiftSwap(recForbidden, reqForbidden)
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for forbidden respond, got %d", recForbidden.Code)
	}
	shiftSwaps.respondErr = nil

	// 7. Cancel swap -> 204
	reqCancel := essRequestAs(t, userID, http.MethodDelete, "/api/v1/ess/shift-swaps/"+swapID.String(), "")
	reqCancel.SetPathValue("id", swapID.String())
	recCancel := httptest.NewRecorder()
	handler.CancelShiftSwap(recCancel, reqCancel)
	if recCancel.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for cancel swap, got %d", recCancel.Code)
	}
	if shiftSwaps.lastEmployeeID != empID {
		t.Fatalf("expected cancel from employee %s, got %s", empID, shiftSwaps.lastEmployeeID)
	}

	// 8. Cancel forbidden -> 403
	shiftSwaps.cancelErr = ErrShiftSwapForbidden
	reqCancelForbid := essRequestAs(t, userID, http.MethodDelete, "/api/v1/ess/shift-swaps/"+swapID.String(), "")
	reqCancelForbid.SetPathValue("id", swapID.String())
	recCancelForbid := httptest.NewRecorder()
	handler.CancelShiftSwap(recCancelForbid, reqCancelForbid)
	if recCancelForbid.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for forbidden cancel, got %d", recCancelForbid.Code)
	}
}
