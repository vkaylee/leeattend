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
	CreatedJobs []ReportJob
}

func (m *mockReportRepo) CreateExportJob(ctx context.Context, job ReportJob, req ExportReportRequest) error {
	m.CreatedJobs = append(m.CreatedJobs, job)
	return nil
}

func (m *mockReportRepo) GetAttendanceReport(ctx context.Context, params AttendanceReportFilter) (PaginatedReportResponse, error) {
	if params.EmploymentID != nil && *params.EmploymentID == uuid.Nil {
		return PaginatedReportResponse{}, ErrReportNotFound
	}
	if params.ReportType == "daily" {
		return PaginatedReportResponse{
			Data:  json.RawMessage(`[{"date":"2026-01-01","employment_id":"` + uuid.New().String() + `","employee_id":"` + uuid.New().String() + `","employee_name":"Alice","employee_id_number":"EMP01","actual_work_minutes":480,"std_work_units":1,"ot_work_units":0,"status":"present"}]`),
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

func (m *mockReportRepo) ResolveBehalfEmployeeIDs(ctx context.Context, wsID, userID uuid.UUID) ([]uuid.UUID, error) {
	return []uuid.UUID{uuid.New()}, nil
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

type mockReportAuthorizer struct{}

func (mockReportAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return true
}

func TestReportCreateExportAndStatus(t *testing.T) {
	wsID := uuid.New()
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
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/reports/export", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
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

	// Now poll status
	pollReq := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/jobs/"+resp.JobID.String(), nil)
	pollReq.SetPathValue("ws_id", wsID.String())
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

func TestReportExportValidation(t *testing.T) {
	handler := ReportHandler{Authorizer: mockReportAuthorizer{}}
	wsID := uuid.New()
	userID := uuid.New()

	payload := ExportReportRequest{
		ReportType: "invalid_type",
		StartDate:  "2026-01-01",
		EndDate:    "2026-01-31",
	}
	body, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/reports/export", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.CreateExport(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 Bad Request, got %d", rec.Code)
	}
}

func TestReportJobNotFound(t *testing.T) {
	queue := &mockReportQueue{Jobs: make(map[uuid.UUID]ReportJob)}
	handler := ReportHandler{
		Queue:      queue,
		Authorizer: mockReportAuthorizer{},
	}
	wsID := uuid.New()
	missingID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/jobs/"+missingID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", missingID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Status(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 Not Found, got %d", rec.Code)
	}
}

func TestReportCancel(t *testing.T) {
	wsID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, WorkspaceID: wsID, Status: "pending"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/reports/jobs/"+jobID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
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
	wsID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, WorkspaceID: wsID, Status: "completed"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/reports/jobs/"+jobID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Cancel(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestReportCancelOtherWorkspaceForbidden(t *testing.T) {
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		uuid.New(): {WorkspaceID: uuid.New()},
	}}
	var jobID uuid.UUID
	for id := range queue.Jobs {
		jobID = id
	}
	wsID := uuid.New()
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/reports/jobs/"+jobID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Cancel(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestReportDownloadCompleted(t *testing.T) {
	wsID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, WorkspaceID: wsID, Status: "completed"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/jobs/"+jobID.String()+"/download", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
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
	wsID := uuid.New()
	jobID := uuid.New()
	queue := &mockReportQueue{Jobs: map[uuid.UUID]ReportJob{
		jobID: {JobID: jobID, WorkspaceID: wsID, Status: "processing"},
	}}
	handler := ReportHandler{Queue: queue, Authorizer: mockReportAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/jobs/"+jobID.String()+"/download", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("job_id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Download(rec, req)
	if rec.Code != http.StatusAccepted {
		t.Fatalf("expected 202 Accepted, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAttendanceReportSummarySuccess(t *testing.T) {
	wsID := uuid.New()
	repo := &mockReportRepo{}
	handler := ReportHandler{Repository: repo, Authorizer: mockReportAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/attendance?start_date=2026-01-01&end_date=2026-01-31&type=summary", nil)
	req.SetPathValue("ws_id", wsID.String())
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
	wsID := uuid.New()
	repo := &mockReportRepo{}
	handler := ReportHandler{Repository: repo, Authorizer: mockReportAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/attendance?start_date=2026-01-01&end_date=2026-01-31&type=daily", nil)
	req.SetPathValue("ws_id", wsID.String())
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
	if len(daily) != 1 || daily[0].Status != "present" {
		t.Fatalf("unexpected daily rows: %+v", daily)
	}
}

func TestAttendanceReportDateValidation(t *testing.T) {
	wsID := uuid.New()
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}

	// Missing start date
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/attendance?end_date=2026-01-31", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing start date, got %d", rec.Code)
	}

	// Inverted dates
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/attendance?start_date=2026-02-01&end_date=2026-01-01", nil)
	req2.SetPathValue("ws_id", wsID.String())
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), uuid.New()))
	rec2 := httptest.NewRecorder()
	handler.AttendanceReport(rec2, req2)
	if rec2.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for inverted date range, got %d", rec2.Code)
	}

	// Range > 365 days
	req3 := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/attendance?start_date=2025-01-01&end_date=2026-01-05", nil)
	req3.SetPathValue("ws_id", wsID.String())
	req3 = req3.WithContext(httpinfra.WithUserIDForTest(req3.Context(), uuid.New()))
	rec3 := httptest.NewRecorder()
	handler.AttendanceReport(rec3, req3)
	if rec3.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for range > 365 days, got %d", rec3.Code)
	}
}

type mockDenyAuthorizer struct{}

func (mockDenyAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return false
}

func TestAttendanceReportForbiddenWithoutPermissions(t *testing.T) {
	wsID := uuid.New()
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockDenyAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/attendance?start_date=2026-01-01&end_date=2026-01-31", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.AttendanceReport(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d", rec.Code)
	}
}

func TestNoShowReportSuccess(t *testing.T) {
	wsID := uuid.New()
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/no-show?date=2026-01-01", nil)
	req.SetPathValue("ws_id", wsID.String())
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
	wsID := uuid.New()
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockReportAuthorizer{}}

	// Missing date
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/no-show", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.NoShowReport(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing date, got %d", rec.Code)
	}

	// Invalid UUID in employee_ids must return 400
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/no-show?date=2026-01-01&employee_ids=not-a-uuid", nil)
	req2.SetPathValue("ws_id", wsID.String())
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), uuid.New()))
	rec2 := httptest.NewRecorder()
	handler.NoShowReport(rec2, req2)
	if rec2.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid employee_ids UUID, got %d", rec2.Code)
	}
}

func TestNoShowReportForbidden(t *testing.T) {
	wsID := uuid.New()
	handler := ReportHandler{Repository: &mockReportRepo{}, Authorizer: mockDenyAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/reports/no-show?date=2026-01-01", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.NoShowReport(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden, got %d", rec.Code)
	}
}
