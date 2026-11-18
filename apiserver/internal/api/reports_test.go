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

type mockReportRepo struct {
	CreatedJobs          []ReportJob
	BehalfEmployeeIDs    []uuid.UUID
	LastAttendanceFilter AttendanceReportFilter
	AttendanceCalls      int
}

func (m *mockReportRepo) CreateExportJob(ctx context.Context, job ReportJob, req ExportReportRequest) error {
	m.CreatedJobs = append(m.CreatedJobs, job)
	return nil
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

type mockReportQueue struct {
	Jobs map[uuid.UUID]ReportJob
}

func (m *mockReportQueue) EnqueueReport(ctx context.Context, job ReportJob, req ExportReportRequest) error {
	if m.Jobs == nil {
		m.Jobs = make(map[uuid.UUID]ReportJob)
	}
	m.Jobs[job.JobID] = job
	return nil
}

func (m *mockReportQueue) GetReport(ctx context.Context, jobID uuid.UUID) (ReportJob, error) {
	if job, ok := m.Jobs[jobID]; ok {
		return job, nil
	}
	return ReportJob{}, ErrReportNotFound
}

func (m *mockReportQueue) CancelReport(ctx context.Context, jobID uuid.UUID) error {
	job, ok := m.Jobs[jobID]
	if !ok {
		return ErrReportNotFound
	}
	if job.Status == "completed" || job.Status == "failed" {
		return ErrReportCannotCancel
	}
	now := time.Now().UTC()
	message := "Cancelled by user"
	job.Status = "failed"
	job.Error = &message
	job.CompletedAt = &now
	m.Jobs[jobID] = job
	return nil
}

func (m *mockReportQueue) GetResultData(ctx context.Context, jobID uuid.UUID) ([]byte, string, error) {
	if job, ok := m.Jobs[jobID]; ok && job.Status == "completed" {
		return []byte("report-bytes"), "attendance.xlsx", nil
	}
	return nil, "", ErrReportResultNotFound
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

func TestReportCreateExportAndStatus(t *testing.T) {
	userID := uuid.New()
	repo := &mockReportRepo{}
	queue := &mockReportQueue{Jobs: make(map[uuid.UUID]ReportJob)}
	handler := ReportHandler{
		Repository: repo,
		Queue:      queue,
		Authorizer: mockReportAuthorizer{},
	}

	payload := ExportReportRequest{
		ReportType: "summary",
		StartDate:  "2026-01-01",
		EndDate:    "2026-01-31",
		Format:     "xlsx",
	}
	body, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusAccepted {
		t.Fatalf("expected 202 Accepted, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp ExportJobResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if resp.Status != "pending" || resp.JobID == uuid.Nil {
		t.Fatalf("invalid job response: %+v", resp)
	}
	if want := "/api/v1/reports/jobs/" + resp.JobID.String(); resp.StatusURL != want {
		t.Fatalf("expected status url %q, got %q", want, resp.StatusURL)
	}
	if len(repo.CreatedJobs) != 1 || repo.CreatedJobs[0].RequesterID != userID {
		t.Fatalf("export job not recorded for requester: %+v", repo.CreatedJobs)
	}

	// The requester can poll their own job.
	pollReq := httptest.NewRequest(http.MethodGet, "/api/v1/reports/jobs/"+resp.JobID.String(), nil)
	pollReq.SetPathValue("job_id", resp.JobID.String())
	pollReq = pollReq.WithContext(httpinfra.WithUserIDForTest(pollReq.Context(), userID))
	pollRec := httptest.NewRecorder()

	handler.Status(pollRec, pollReq)
	if pollRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", pollRec.Code, pollRec.Body.String())
	}
	var statusResp JobStatusResponse
	if err := json.Unmarshal(pollRec.Body.Bytes(), &statusResp); err != nil {
		t.Fatalf("failed to decode status response: %v", err)
	}
	if statusResp.Status != "pending" || statusResp.JobID != resp.JobID {
		t.Fatalf("invalid status response: %+v", statusResp)
	}
}

func TestReportStatusOtherRequesterForbidden(t *testing.T) {
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, RequesterID: uuid.New(), Status: "pending"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/jobs/"+jobID.String(), nil)
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Status(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestReportExportRequiresExportPermission(t *testing.T) {
	handler := ReportHandler{Repository: &mockReportRepo{}, Queue: &mockReportQueue{}, Authorizer: mockDenyAuthorizer{}}
	body, _ := json.Marshal(ExportReportRequest{ReportType: "summary", StartDate: "2026-01-01", EndDate: "2026-01-31"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestReportExportValidation(t *testing.T) {
	handler := ReportHandler{Authorizer: mockReportAuthorizer{}}
	userID := uuid.New()

	payload := ExportReportRequest{
		ReportType: "invalid_type",
		StartDate:  "2026-01-01",
		EndDate:    "2026-01-31",
	}
	body, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/reports/export", bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 Bad Request, got %d", rec.Code)
	}
}

func TestReportExportAcceptsOnlyXLSX(t *testing.T) {
	for name, testCase := range map[string]struct {
		format    string
		wantValid bool
	}{
		"default format": {format: "", wantValid: true},
		"xlsx format":    {format: "xlsx", wantValid: true},
		"csv format":     {format: "csv", wantValid: false},
	} {
		t.Run(name, func(t *testing.T) {
			request := ExportReportRequest{ReportType: "summary", StartDate: "2026-01-01", EndDate: "2026-01-31", Format: testCase.format}
			err := validateExportRequest(&request)
			if (err == nil) != testCase.wantValid {
				t.Fatalf("validation error = %v, want valid=%t", err, testCase.wantValid)
			}
			if testCase.wantValid && request.Format != "xlsx" {
				t.Fatalf("format = %q, want xlsx", request.Format)
			}
		})
	}
}

func TestReportJobNotFound(t *testing.T) {
	queue := &mockReportQueue{Jobs: make(map[uuid.UUID]ReportJob)}
	handler := ReportHandler{
		Queue:      queue,
		Authorizer: mockReportAuthorizer{},
	}
	missingID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/jobs/"+missingID.String(), nil)
	req.SetPathValue("job_id", missingID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Status(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 Not Found, got %d", rec.Code)
	}
}

func TestReportCancel(t *testing.T) {
	userID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, RequesterID: userID, Status: "pending"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/reports/jobs/"+jobID.String(), nil)
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Cancel(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 No Content, got %d: %s", rec.Code, rec.Body.String())
	}
	if job := queue.Jobs[jobID]; job.Status != "failed" || job.Error == nil || *job.Error != "Cancelled by user" {
		t.Fatalf("cancel did not mark job failed: %+v", job)
	}
}

func TestReportCancelRejectsCompletedJob(t *testing.T) {
	userID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, RequesterID: userID, Status: "completed"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/reports/jobs/"+jobID.String(), nil)
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Cancel(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestReportCancelOtherRequesterForbidden(t *testing.T) {
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, RequesterID: uuid.New(), Status: "pending"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/reports/jobs/"+jobID.String(), nil)
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Cancel(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d: %s", rec.Code, rec.Body.String())
	}
	if job := queue.Jobs[jobID]; job.Status != "pending" {
		t.Fatalf("foreign cancel must not mutate job: %+v", job)
	}
}

func TestReportDownloadCompleted(t *testing.T) {
	userID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, RequesterID: userID, Status: "completed"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/jobs/"+jobID.String()+"/download", nil)
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Download(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	if got := rec.Header().Get("Content-Type"); got != "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" {
		t.Fatalf("unexpected content type: %s", got)
	}
	if got := rec.Header().Get("Content-Disposition"); got != `attachment; filename="attendance.xlsx"` {
		t.Fatalf("unexpected content disposition: %s", got)
	}
	if rec.Body.String() != "report-bytes" {
		t.Fatalf("unexpected body: %s", rec.Body.String())
	}
}

func TestReportDownloadPendingReturnsAccepted(t *testing.T) {
	userID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, RequesterID: userID, Status: "processing"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/jobs/"+jobID.String()+"/download", nil)
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Download(rec, req)
	if rec.Code != http.StatusAccepted {
		t.Fatalf("expected 202 Accepted, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestReportDownloadOtherRequesterForbidden(t *testing.T) {
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, RequesterID: uuid.New(), Status: "completed"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/reports/jobs/"+jobID.String()+"/download", nil)
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Download(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d: %s", rec.Code, rec.Body.String())
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
	return permission == "overtime:claim_on_behalf"
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
