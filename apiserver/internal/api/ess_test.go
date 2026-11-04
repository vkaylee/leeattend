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
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockESSRepository struct {
	profile          MyProfileResponse
	profileRecord    EmployeeProfileRecord
	contacts         []EmployeeEmergencyContactRecord
	empID            uuid.UUID
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

func (m *mockESSRepository) ResolveEmployeeForUser(context.Context, uuid.UUID, uuid.UUID) (uuid.UUID, error) {
	if m.resolveErr != nil {
		return uuid.Nil, m.resolveErr
	}
	return m.empID, nil
}
func (m *mockESSRepository) GetMyProfile(context.Context, uuid.UUID, uuid.UUID) (MyProfileResponse, error) {
	return m.profile, m.getProfileErr
}
func (m *mockESSRepository) UpsertProfile(context.Context, uuid.UUID, uuid.UUID, UpdateEmployeeProfileRequest) (EmployeeProfileRecord, error) {
	m.upsertCalls++
	return m.profileRecord, m.upsertProfileErr
}
func (m *mockESSRepository) GetEmergencyContacts(context.Context, uuid.UUID, uuid.UUID) ([]EmployeeEmergencyContactRecord, error) {
	return m.contacts, m.getContactsErr
}
func (m *mockESSRepository) AddEmergencyContact(context.Context, uuid.UUID, uuid.UUID, CreateEmergencyContactRequest) (EmployeeEmergencyContactRecord, error) {
	m.addCalls++
	return EmployeeEmergencyContactRecord{ID: uuid.New(), ContactName: "Family", Relationship: "Spouse", Phone: "123456789"}, m.addContactErr
}
func (m *mockESSRepository) DeleteEmergencyContact(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error {
	m.deleteCalls++
	return m.deleteContactErr
}

func (m *mockESSRepository) GetPendingDocuments(context.Context, uuid.UUID, uuid.UUID) ([]AcknowledgementRecord, error) {
	return m.pendingDocs, m.pendingDocsErr
}
func (m *mockESSRepository) AcknowledgeDocument(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error {
	return m.ackDocErr
}
func (m *mockESSRepository) GetEmployeeDocuments(context.Context, uuid.UUID, uuid.UUID) ([]EmployeeDocumentRecord, error) {
	return m.empDocs, m.empDocsErr
}

type mockESSAttendanceRepo struct {
	AttendanceRepository
	punchErr     error
	history      []TimeEntryRecord
	historyErr   error
	toggleStatus bool
}

func (m *mockESSAttendanceRepo) ESSPunch(_ context.Context, wsID, empID uuid.UUID, entryType string, note *string, punchTime time.Time) (TimeEntryRecord, error) {
	if m.punchErr != nil {
		return TimeEntryRecord{}, m.punchErr
	}
	if !m.toggleStatus {
		m.toggleStatus = true
		return TimeEntryRecord{
			ID:           uuid.New(),
			WorkspaceID:  wsID,
			EmploymentID: empID,
			PunchInTime:  punchTime.Format(time.RFC3339),
			EntryType:    entryType,
			Note:         note,
		}, nil
	}
	m.toggleStatus = false
	inTime := punchTime.Add(-8 * time.Hour).Format(time.RFC3339)
	outTime := punchTime.Format(time.RFC3339)
	dur := 480
	return TimeEntryRecord{
		ID:              uuid.New(),
		WorkspaceID:     wsID,
		EmploymentID:    empID,
		PunchInTime:     inTime,
		PunchOutTime:    &outTime,
		DurationMinutes: &dur,
		EntryType:       entryType,
		Note:            note,
	}, nil
}

func (m *mockESSAttendanceRepo) ListTimeHistory(_ context.Context, _, _ uuid.UUID, _, _ time.Time) ([]TimeEntryRecord, error) {
	return m.history, m.historyErr
}

type mockESSPayrollRepo struct {
	PayrollRepository
	payslips    []PayslipRecord
	payslipsErr error
}

func (m *mockESSPayrollRepo) ListEmployeePayslips(_ context.Context, _, _ uuid.UUID) ([]PayslipRecord, error) {
	return m.payslips, m.payslipsErr
}

type mockESSLeaveRepo struct {
	LeaveRepository
	balances    []EmployeeLeaveBalanceRecord
	balancesErr error
	requests    []LeaveRequestRecord
	requestsErr error
}

func (m *mockESSLeaveRepo) GetEmployeeLeaveBalances(_ context.Context, _, _ uuid.UUID, _ int) ([]EmployeeLeaveBalanceRecord, error) {
	return m.balances, m.balancesErr
}

func (m *mockESSLeaveRepo) ListEmployeeLeaveRequests(_ context.Context, _, _ uuid.UUID) ([]LeaveRequestRecord, error) {
	return m.requests, m.requestsErr
}

type mockESSShiftSwapRepo struct {
	ShiftSwapRepository
	swaps      []ShiftSwapRequestWithNames
	swapsErr   error
	createErr  error
	respondErr error
	cancelErr  error
}

func (m *mockESSShiftSwapRepo) ListEmployeeSwaps(_ context.Context, _, _ uuid.UUID) ([]ShiftSwapRequestWithNames, error) {
	return m.swaps, m.swapsErr
}

func (m *mockESSShiftSwapRepo) CreateESSSwap(_ context.Context, wsID, reqID, userID uuid.UUID, req EssCreateShiftSwapRequest) (ShiftSwapRequest, error) {
	if m.createErr != nil {
		return ShiftSwapRequest{}, m.createErr
	}
	return ShiftSwapRequest{
		ID:                uuid.New(),
		WorkspaceID:       wsID,
		RequesterID:       reqID,
		TargetID:          req.TargetID,
		SwapDate:          req.SwapDate,
		Status:            ShiftSwapPendingTarget,
		Reason:            req.Reason,
		RequestedByUserID: &userID,
	}, nil
}

func (m *mockESSShiftSwapRepo) RespondSwap(_ context.Context, _, _, _ uuid.UUID, _ bool) error {
	return m.respondErr
}

func (m *mockESSShiftSwapRepo) CancelSwap(_ context.Context, _, _, _ uuid.UUID) error {
	return m.cancelErr
}

func essRequest(t *testing.T, method, path, body string, wsID uuid.UUID) *http.Request {
	t.Helper()
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	ctx := httpinfra.WithUserIDForTest(req.Context(), uuid.New())
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, wsID)
	return req.WithContext(ctx)
}

func TestESSGetProfileNoLinkedEmployeeForbidden(t *testing.T) {
	wsID := uuid.New()
	repo := &mockESSRepository{resolveErr: ErrESSNoEmployeeRecord}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/profile", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestESSGetProfileSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{
		empID:   empID,
		profile: MyProfileResponse{Employee: EmployeeRecord{ID: empID, FullName: "John Doe"}},
	}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/profile", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "John Doe") {
		t.Fatalf("profile body missing name: %s", rec.Body.String())
	}
}

func TestESSUpdateProfileValidatesDOB(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodPut, "/api/v1/ess/profile", `{"date_of_birth":"invalid-date"}`, wsID)
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
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodPost, "/api/v1/ess/emergency-contacts", `{"contact_name":""}`, wsID)
	rec := httptest.NewRecorder()
	handler.AddContact(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing fields, got %d", rec.Code)
	}
	if repo.addCalls != 0 {
		t.Fatal("add contact called on validation failure")
	}
}

func TestESSDeleteContactSuccess(t *testing.T) {
	wsID, empID, contactID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodDelete, "/api/v1/ess/emergency-contacts/"+contactID.String(), "", wsID)
	req.SetPathValue("contact_id", contactID.String())
	rec := httptest.NewRecorder()
	handler.DeleteContact(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	if repo.deleteCalls != 1 {
		t.Fatalf("expected 1 delete call, got %d", repo.deleteCalls)
	}
}

func TestESSMissingWorkspaceBadRequest(t *testing.T) {
	repo := &mockESSRepository{}
	handler := ESSHandler{Repository: repo}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/ess/profile", nil)
	rec := httptest.NewRecorder()
	handler.GetProfile(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing workspace, got %d", rec.Code)
	}
}

func TestESSGetPendingDocumentsSuccess(t *testing.T) {
	wsID, empID, docID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{
		empID: empID,
		pendingDocs: []AcknowledgementRecord{
			{ID: uuid.New(), DocumentID: docID, EmployeeID: empID, Status: "pending"},
		},
	}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/documents/pending", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetPendingDocuments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var items []AcknowledgementRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &items); err != nil || len(items) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSAcknowledgeDocumentSuccess(t *testing.T) {
	wsID, empID, ackID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodPost, "/api/v1/ess/documents/"+ackID.String()+"/acknowledge", "", wsID)
	req.SetPathValue("id", ackID.String())
	rec := httptest.NewRecorder()
	handler.AcknowledgeDocument(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "acknowledged") {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetDocumentsSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{
		empID: empID,
		empDocs: []EmployeeDocumentRecord{
			{ID: uuid.New(), EmployeeID: empID, WorkspaceID: wsID, DocumentType: "contract", FileName: "contract.pdf"},
		},
	}
	handler := ESSHandler{Repository: repo}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/documents", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetDocuments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var docs []EmployeeDocumentRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &docs); err != nil || len(docs) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetPayslipsSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	payroll := &mockESSPayrollRepo{
		payslips: []PayslipRecord{
			{ID: uuid.New(), EmploymentID: empID, NetPayAmount: decimal.RequireFromString("2500.00")},
		},
	}
	handler := ESSHandler{Repository: repo, Payroll: payroll}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/payslips", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetPayslips(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var payslips []PayslipRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &payslips); err != nil || len(payslips) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetAttendanceSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	attendance := &mockESSAttendanceRepo{
		history: []TimeEntryRecord{
			{ID: uuid.New(), WorkspaceID: wsID, EmploymentID: empID, PunchInTime: "2026-10-20T08:00:00Z"},
		},
	}
	handler := ESSHandler{Repository: repo, Attendance: attendance}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/attendance?start_date=2026-10-01&end_date=2026-10-20", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetAttendance(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var entries []TimeEntryRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &entries); err != nil || len(entries) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSPunchToggle(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	attendance := &mockESSAttendanceRepo{}
	handler := ESSHandler{Repository: repo, Attendance: attendance}

	// 1. Punch In
	req1 := essRequest(t, http.MethodPost, "/api/v1/ess/punch", `{"note":"starting shift"}`, wsID)
	rec1 := httptest.NewRecorder()
	handler.Punch(rec1, req1)
	if rec1.Code != http.StatusCreated {
		t.Fatalf("expected 201 for punch in, got %d: %s", rec1.Code, rec1.Body.String())
	}
	var entry1 TimeEntryRecord
	if err := json.Unmarshal(rec1.Body.Bytes(), &entry1); err != nil || entry1.PunchOutTime != nil {
		t.Fatalf("expected open entry, got %+v", entry1)
	}

	// 2. Punch Out
	req2 := essRequest(t, http.MethodPost, "/api/v1/ess/punch", `{"note":"ending shift"}`, wsID)
	rec2 := httptest.NewRecorder()
	handler.Punch(rec2, req2)
	if rec2.Code != http.StatusCreated {
		t.Fatalf("expected 201 for punch out, got %d: %s", rec2.Code, rec2.Body.String())
	}
	var entry2 TimeEntryRecord
	if err := json.Unmarshal(rec2.Body.Bytes(), &entry2); err != nil || entry2.PunchOutTime == nil {
		t.Fatalf("expected closed entry with punch_out_time, got %+v", entry2)
	}
}

func TestESSPunchNoActiveEmployment(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	attendance := &mockESSAttendanceRepo{punchErr: ErrNoActiveEmployment}
	handler := ESSHandler{Repository: repo, Attendance: attendance}
	req := essRequest(t, http.MethodPost, "/api/v1/ess/punch", `{}`, wsID)
	rec := httptest.NewRecorder()
	handler.Punch(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for no active employment, got %d", rec.Code)
	}
}

func TestESSGetTimeEntriesSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	attendance := &mockESSAttendanceRepo{
		history: []TimeEntryRecord{
			{ID: uuid.New(), WorkspaceID: wsID, EmploymentID: empID, PunchInTime: "2026-10-20T09:00:00Z"},
		},
	}
	handler := ESSHandler{Repository: repo, Attendance: attendance}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/time-entries?start_date=2026-10-01T00:00:00Z&end_date=2026-10-20T23:59:59Z", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetTimeEntries(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var entries []TimeEntryRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &entries); err != nil || len(entries) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSGetLeaveBalancesSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
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
	req := essRequest(t, http.MethodGet, "/api/v1/ess/leave/balances", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetLeaveBalances(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
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
	wsID, empID := uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	leaves := &mockESSLeaveRepo{
		requests: []LeaveRequestRecord{
			{ID: uuid.New(), EmploymentID: empID, Status: "pending"},
		},
	}
	handler := ESSHandler{Repository: repo, Leaves: leaves}
	req := essRequest(t, http.MethodGet, "/api/v1/ess/leave/requests", "", wsID)
	rec := httptest.NewRecorder()
	handler.GetLeaveRequests(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var requests []LeaveRequestRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &requests); err != nil || len(requests) != 1 {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestESSShiftSwapsCRUD(t *testing.T) {
	wsID, empID, targetID, swapID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	repo := &mockESSRepository{empID: empID}
	shiftSwaps := &mockESSShiftSwapRepo{
		swaps: []ShiftSwapRequestWithNames{
			{ShiftSwapRequest: ShiftSwapRequest{ID: swapID, RequesterID: empID, TargetID: targetID, Status: ShiftSwapPendingTarget}},
		},
	}
	handler := ESSHandler{Repository: repo, ShiftSwaps: shiftSwaps}

	// 1. List swaps
	reqList := essRequest(t, http.MethodGet, "/api/v1/ess/shift-swaps", "", wsID)
	recList := httptest.NewRecorder()
	handler.GetShiftSwaps(recList, reqList)
	if recList.Code != http.StatusOK {
		t.Fatalf("expected 200 for list swaps, got %d", recList.Code)
	}

	// 2. Create swap
	createBody := `{"target_id":"` + targetID.String() + `","swap_date":"2026-10-25","reason":"vacation"}`
	reqCreate := essRequest(t, http.MethodPost, "/api/v1/ess/shift-swaps", createBody, wsID)
	recCreate := httptest.NewRecorder()
	handler.CreateShiftSwap(recCreate, reqCreate)
	if recCreate.Code != http.StatusCreated {
		t.Fatalf("expected 201 for create swap, got %d: %s", recCreate.Code, recCreate.Body.String())
	}
	var created ShiftSwapRequest
	if err := json.Unmarshal(recCreate.Body.Bytes(), &created); err != nil || created.Status != ShiftSwapPendingTarget {
		t.Fatalf("unexpected created swap: %+v", created)
	}

	// 3. Create swap targeting self -> 400
	selfBody := `{"target_id":"` + empID.String() + `","swap_date":"2026-10-25"}`
	reqSelf := essRequest(t, http.MethodPost, "/api/v1/ess/shift-swaps", selfBody, wsID)
	recSelf := httptest.NewRecorder()
	handler.CreateShiftSwap(recSelf, reqSelf)
	if recSelf.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for self target, got %d", recSelf.Code)
	}

	// 4. Respond to swap (accept) -> 200
	reqRespond := essRequest(t, http.MethodPut, "/api/v1/ess/shift-swaps/"+swapID.String()+"/respond", `{"accept":true}`, wsID)
	reqRespond.SetPathValue("id", swapID.String())
	recRespond := httptest.NewRecorder()
	handler.RespondShiftSwap(recRespond, reqRespond)
	if recRespond.Code != http.StatusOK {
		t.Fatalf("expected 200 for accept swap, got %d", recRespond.Code)
	}

	// 5. Respond to swap (reject) -> 200
	reqReject := essRequest(t, http.MethodPut, "/api/v1/ess/shift-swaps/"+swapID.String()+"/respond", `{"accept":false}`, wsID)
	reqReject.SetPathValue("id", swapID.String())
	recReject := httptest.NewRecorder()
	handler.RespondShiftSwap(recReject, reqReject)
	if recReject.Code != http.StatusOK {
		t.Fatalf("expected 200 for reject swap, got %d", recReject.Code)
	}

	// 6. Respond forbidden -> 403
	shiftSwaps.respondErr = ErrShiftSwapForbidden
	reqForbidden := essRequest(t, http.MethodPut, "/api/v1/ess/shift-swaps/"+swapID.String()+"/respond", `{"accept":true}`, wsID)
	reqForbidden.SetPathValue("id", swapID.String())
	recForbidden := httptest.NewRecorder()
	handler.RespondShiftSwap(recForbidden, reqForbidden)
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for forbidden respond, got %d", recForbidden.Code)
	}
	shiftSwaps.respondErr = nil

	// 7. Cancel swap -> 204
	reqCancel := essRequest(t, http.MethodDelete, "/api/v1/ess/shift-swaps/"+swapID.String(), "", wsID)
	reqCancel.SetPathValue("id", swapID.String())
	recCancel := httptest.NewRecorder()
	handler.CancelShiftSwap(recCancel, reqCancel)
	if recCancel.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for cancel swap, got %d", recCancel.Code)
	}

	// 8. Cancel forbidden -> 403
	shiftSwaps.cancelErr = ErrShiftSwapForbidden
	reqCancelForbid := essRequest(t, http.MethodDelete, "/api/v1/ess/shift-swaps/"+swapID.String(), "", wsID)
	reqCancelForbid.SetPathValue("id", swapID.String())
	recCancelForbid := httptest.NewRecorder()
	handler.CancelShiftSwap(recCancelForbid, reqCancelForbid)
	if recCancelForbid.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for forbidden cancel, got %d", recCancelForbid.Code)
	}
}
