package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockReportRepo struct {
	BehalfEmployeeIDs    []uuid.UUID
	LastAttendanceFilter AttendanceReportFilter
	AttendanceCalls      int
	exportEmployeeIDs    []uuid.UUID
}

func (m *mockReportRepo) ExportAttendanceReport(_ context.Context, request ExportReportRequest) ([]byte, string, error) {
	m.exportEmployeeIDs = request.EmployeeIDs
	return []byte("PK"), "attendance-summary.xlsx", nil
}

func (m *mockReportRepo) GetAttendanceReport(ctx context.Context, params AttendanceReportFilter) (PaginatedReportResponse, error) {
	m.AttendanceCalls++
	m.LastAttendanceFilter = params
	if params.EmploymentID != nil && *params.EmploymentID == uuid.Nil {
		return PaginatedReportResponse{}, ErrReportNotFound
	}
	if params.ReportType == "daily" {
		return PaginatedReportResponse{
			Data:  json.RawMessage(`[{"date":"2026-01-01","employment_id":"` + uuid.New().String() + `","employee_id":"` + uuid.New().String() + `","employee_name":"Alice","employee_id_number":"EMP01","actual_work_minutes":480,"std_work_units":1,"ot_work_units":0,"ot_claim_status":"pending_employee","status":"present"}]`),
			Total: 1,
			Page:  params.Page,
			Limit: params.Limit,
		}, nil
	}
	return PaginatedReportResponse{
		Data:  json.RawMessage(`[{"employment_id":"` + uuid.New().String() + `","employee_id":"` + uuid.New().String() + `","employee_name":"Alice","employee_id_number":"EMP01","total_std_units":20,"total_ot_units":2,"total_leave_days":0,"unit_type":"DAY","unit_coefficient_hours":8,"computed_std_hours":160,"computed_ot_hours":16}]`),
		Total: 1,
		Page:  params.Page,
		Limit: params.Limit,
	}, nil
}

func (m *mockReportRepo) GetNoShowReport(ctx context.Context, params NoShowReportFilter) (PaginatedReportResponse, error) {
	return PaginatedReportResponse{
		Data:  json.RawMessage(`[{"date":"2026-01-01","employment_id":"` + uuid.New().String() + `","employee_id":"` + uuid.New().String() + `","employee_name":"Bob","employee_id_number":"EMP02","status":"absent","recording_issue":"missing_punch_in"}]`),
		Total: 1,
		Page:  params.Page,
		Limit: params.Limit,
	}, nil
}

func (m *mockReportRepo) ResolveBehalfEmployeeIDs(ctx context.Context, userID uuid.UUID) ([]uuid.UUID, error) {
	return m.BehalfEmployeeIDs, nil
}

// mockReportAuthorizer grants every permission while remaining a member of the singleton organization.
type mockReportAuthorizer struct{}

func (mockReportAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (mockReportAuthorizer) HasPermission(context.Context, uuid.UUID, string) bool {
	return true
}
func (mockReportAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockReportAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (mockReportAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return true }
func (mockReportAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return true
}
func (mockReportAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func TestReportCreateExportReturnsXLSX(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}
	body := bytes.NewBufferString(`{"report_type":"summary","start_date":"2026-01-01","end_date":"2026-01-31","format":"xlsx"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", body)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	if got := rec.Header().Get("Content-Type"); got != "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" {
		t.Fatalf("unexpected content type: %q", got)
	}
	if got := rec.Header().Get("Content-Disposition"); got != `attachment; filename="attendance-summary.xlsx"` {
		t.Fatalf("unexpected content disposition: %q", got)
	}
	if rec.Body.String() != "PK" {
		t.Fatalf("unexpected response body: %q", rec.Body.String())
	}
}

func TestReportCreateExportRejectsUnauthorized(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockDenyAuthorizer{}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", bytes.NewBufferString(`{"report_type":"summary","start_date":"2026-01-01","end_date":"2026-01-31"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d", rec.Code)
	}
}

func TestReportCreateExportRejectsInvalidRequest(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", bytes.NewBufferString(`{"report_type":"invalid","start_date":"2026-01-01","end_date":"2026-01-31"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 Bad Request, got %d", rec.Code)
	}
}

func TestReportCreateExportAppliesBehalfScope(t *testing.T) {
	assigned := uuid.New()
	repo := &mockReportRepo{BehalfEmployeeIDs: []uuid.UUID{assigned}}
	handler := ReportHandler{Repository: repo, Authorizer: mockBehalfAuthorizer{}}
	body := bytes.NewBufferString(`{"report_type":"summary","start_date":"2026-01-01","end_date":"2026-01-31"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", body)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.exportEmployeeIDs == nil || len(repo.exportEmployeeIDs) != 1 || repo.exportEmployeeIDs[0] != assigned {
		t.Fatalf("expected behalf employee scope, got %v", repo.exportEmployeeIDs)
	}
}

func TestReportCreateExportRejectsOversizedRequest(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}
	payload := bytes.Repeat([]byte("x"), maxExportRequestBytes+1)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", bytes.NewReader(payload))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	req.ContentLength = int64(len(payload))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusRequestEntityTooLarge {
		t.Fatalf("expected 413 Payload Too Large, got %d", rec.Code)
	}
}

func TestAttendanceReportSummarySuccess(t *testing.T) {
	repo := &mockReportRepo{}
	handler := ReportHandler{Repository: repo, Authorizer: mockReportAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31&type=summary", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp PaginatedReportResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if resp.Total != 1 || resp.Page != 1 || resp.Limit != 20 {
		t.Fatalf("unexpected pagination: %+v", resp)
	}
	var summaries []AttendanceSummaryRow
	if err := json.Unmarshal(resp.Data, &summaries); err != nil {
		t.Fatalf("failed to decode summary rows: %v", err)
	}
	if len(summaries) != 1 || summaries[0].EmployeeName != "Alice" {
		t.Fatalf("unexpected summaries: %+v", summaries)
	}
}

func TestAttendanceReportDailySuccess(t *testing.T) {
	repo := &mockReportRepo{}
	handler := ReportHandler{Repository: repo, Authorizer: mockReportAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31&type=daily", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp PaginatedReportResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	var daily []DailyAttendanceRow
	if err := json.Unmarshal(resp.Data, &daily); err != nil {
		t.Fatalf("failed to decode daily rows: %v", err)
	}
	if len(daily) != 1 || daily[0].Status != "present" || daily[0].OTClaimStatus != "pending_employee" {
		t.Fatalf("unexpected daily rows: %+v", daily)
	}
}

func TestAttendanceReportFiltersAreForwarded(t *testing.T) {
	repo := &mockReportRepo{}
	handler := ReportHandler{Repository: repo, Authorizer: mockReportAuthorizer{}}
	deptID := uuid.New()
	empID := uuid.New()

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31&type=summary&department_id="+deptID.String()+"&employee_ids="+empID.String()+"&page=2&limit=50", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	filter := repo.LastAttendanceFilter
	if filter.Page != 2 || filter.Limit != 50 {
		t.Fatalf("pagination not forwarded: %+v", filter)
	}
	if len(filter.EmployeeIDs) != 1 || filter.EmployeeIDs[0] != empID {
		t.Fatalf("employee filter not forwarded: %+v", filter.EmployeeIDs)
	}
	if filter.DepartmentID == nil || *filter.DepartmentID != deptID {
		t.Fatalf("department filter not forwarded: %+v", filter.DepartmentID)
	}
	if filter.StartDate != "2026-01-01" || filter.EndDate != "2026-01-31" {
		t.Fatalf("date range not forwarded: %+v", filter)
	}
}

func TestAttendanceReportBehalfScopeRestrictsEmployees(t *testing.T) {
	behalfID := uuid.New()
	repo := &mockReportRepo{BehalfEmployeeIDs: []uuid.UUID{behalfID}}
	handler := ReportHandler{Repository: repo, Authorizer: mockBehalfAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	filter := repo.LastAttendanceFilter
	if len(filter.EmployeeIDs) != 1 || filter.EmployeeIDs[0] != behalfID {
		t.Fatalf("behalf scope not applied, got employees %+v", filter.EmployeeIDs)
	}
}

func TestAttendanceReportBehalfWithoutAssignmentsReturnsEmpty(t *testing.T) {
	repo := &mockReportRepo{}
	handler := ReportHandler{Repository: repo, Authorizer: mockBehalfAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp PaginatedReportResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if resp.Total != 0 || string(resp.Data) != "[]" {
		t.Fatalf("expected empty result, got %+v", resp)
	}
	if repo.AttendanceCalls != 0 {
		t.Fatalf("repository must not be queried without behalf assignments")
	}
}

func TestAttendanceReportDateValidation(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}

	// Missing start date
	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?end_date=2026-01-31", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing start date, got %d", rec.Code)
	}

	// Inverted dates
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-02-01&end_date=2026-01-01", nil)
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), uuid.New()))
	rec2 := httptest.NewRecorder()
	handler.AttendanceReport(rec2, req2)
	if rec2.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for inverted date range, got %d", rec2.Code)
	}

	// Range > 365 days
	req3 := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2025-01-01&end_date=2026-01-05", nil)
	req3 = req3.WithContext(httpinfra.WithUserIDForTest(req3.Context(), uuid.New()))
	rec3 := httptest.NewRecorder()
	handler.AttendanceReport(rec3, req3)
	if rec3.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for range > 365 days, got %d", rec3.Code)
	}
}

type mockScopedReportAuthorizer struct{}

func (mockScopedReportAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return true, nil
}
func (mockScopedReportAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return permission == "attendance:summaries:read"
}
func (mockScopedReportAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockScopedReportAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return []uuid.UUID{uuid.New()}, nil
}
func (mockScopedReportAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (mockScopedReportAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, _ string, _ *uuid.UUID) bool {
	return false
}
func (mockScopedReportAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

// mockDenyAuthorizer denies every permission.
type mockDenyAuthorizer struct{}

func (mockDenyAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (mockDenyAuthorizer) HasPermission(context.Context, uuid.UUID, string) bool {
	return false
}
func (mockDenyAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockDenyAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (mockDenyAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (mockDenyAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (mockDenyAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

// mockBehalfAuthorizer holds only the overtime claim-on-behalf permission.
type mockBehalfAuthorizer struct{}

func (mockBehalfAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (mockBehalfAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return permission == "overtime:claim_on_behalf" || permission == "reports:export"
}
func (mockBehalfAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockBehalfAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (mockBehalfAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (mockBehalfAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, permission string, _ *uuid.UUID) bool {
	return permission == "overtime:claim_on_behalf"
}
func (mockBehalfAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func TestAttendanceReportScopedPermissionCannotBecomeGlobal(t *testing.T) {
	repo := &mockReportRepo{}
	handler := ReportHandler{Repository: repo, Authorizer: mockScopedReportAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected scoped report permission without behalf assignment to be forbidden, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.AttendanceCalls != 0 {
		t.Fatal("repository must not be queried when scoped permission is not global")
	}
}

func TestAttendanceReportForbiddenWithoutPermissions(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockDenyAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d", rec.Code)
	}
}

func TestNoShowReportSuccess(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/no-show?date=2026-01-01", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.NoShowReport(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp PaginatedReportResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	var noShow []NoShowReportRow
	if err := json.Unmarshal(resp.Data, &noShow); err != nil {
		t.Fatalf("failed to decode no-show rows: %v", err)
	}
	if len(noShow) != 1 || noShow[0].RecordingIssue != "missing_punch_in" {
		t.Fatalf("unexpected no-show data: %+v", noShow)
	}
}

func TestNoShowReportValidationErrors(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}

	// Missing date
	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/no-show", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.NoShowReport(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing date, got %d", rec.Code)
	}

	// Invalid UUID in employee_ids must return 400
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1/reports/no-show?date=2026-01-01&employee_ids=not-a-uuid", nil)
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), uuid.New()))
	rec2 := httptest.NewRecorder()
	handler.NoShowReport(rec2, req2)
	if rec2.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid employee_ids UUID, got %d", rec2.Code)
	}
}

func TestNoShowReportForbidden(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockDenyAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/no-show?date=2026-01-01", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.NoShowReport(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d", rec.Code)
	}
}
