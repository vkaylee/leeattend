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
	CreatedJobs []ReportJob
}

func (m *mockReportRepo) CreateExportJob(ctx context.Context, job ReportJob, req ExportReportRequest) error {
	m.CreatedJobs = append(m.CreatedJobs, job)
	return nil
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

type mockReportAuthorizer struct{}

func (mockReportAuthorizer) IsSystemAdmin(context.Context, uuid.UUID) bool { return false }
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
