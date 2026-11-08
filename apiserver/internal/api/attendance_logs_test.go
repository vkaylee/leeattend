package api

import (
	"bytes"
	"context"
	"encoding/json"
	"mime/multipart"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAttendanceLogsRepo struct {
	latestJob  *AttendanceImportJob
	job        *AttendanceImportJob
	errorData  []byte
	logs       []AttendanceLogRecord
	rawLogs    []RawAttendanceLogRecord
	staging    []AttendanceStagingLog
	dismissed  bool
	summary    AttendanceDailySummary
	bulkResult BulkPaperOTResult
	err        error
}

func (m *mockAttendanceLogsRepo) CreateAttendanceImportJob(_ context.Context, user uuid.UUID, path, target string) (AttendanceImportJob, error) {
	if m.err != nil {
		return AttendanceImportJob{}, m.err
	}
	return AttendanceImportJob{ID: uuid.New(), UserID: user, Status: "processing", TargetType: target, CreatedAt: time.Now().UTC(), UpdatedAt: time.Now().UTC()}, nil
}
func (m *mockAttendanceLogsRepo) ProcessAttendanceImport(_ context.Context, _, _ uuid.UUID, _, _ string, _ []AttendanceImportRow) error {
	return m.err
}
func (m *mockAttendanceLogsRepo) GetLatestAttendanceImportJob(_ context.Context, _ *string) (*AttendanceImportJob, error) {
	return m.latestJob, m.err
}
func (m *mockAttendanceLogsRepo) GetAttendanceImportJob(_ context.Context, _ uuid.UUID) (*AttendanceImportJob, error) {
	return m.job, m.err
}
func (m *mockAttendanceLogsRepo) GetAttendanceLogErrorFile(_ context.Context, _ uuid.UUID) ([]byte, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.errorData, nil
}
func (m *mockAttendanceLogsRepo) ListAttendanceLogs(_ context.Context, _ *uuid.UUID, _, _ *time.Time, _, _ int) ([]AttendanceLogRecord, int64, error) {
	return m.logs, int64(len(m.logs)), m.err
}
func (m *mockAttendanceLogsRepo) ListRawAttendanceLogs(_ context.Context, _ *uuid.UUID, _, _ *time.Time, _, _ int) ([]RawAttendanceLogRecord, int64, error) {
	return m.rawLogs, int64(len(m.rawLogs)), m.err
}
func (m *mockAttendanceLogsRepo) ListStagingLogs(_ context.Context, _, _ int) ([]AttendanceStagingLog, int64, error) {
	return m.staging, int64(len(m.staging)), m.err
}
func (m *mockAttendanceLogsRepo) ResolveStagingLog(_ context.Context, _, _ uuid.UUID, _ *uuid.UUID) error {
	return m.err
}
func (m *mockAttendanceLogsRepo) DismissStagingLog(_ context.Context, _ uuid.UUID) (bool, error) {
	return m.dismissed, m.err
}
func (m *mockAttendanceLogsRepo) EnterPaperOT(_ context.Context, _ uuid.UUID, _ PaperOTRequest, _ uuid.UUID) (AttendanceDailySummary, error) {
	return m.summary, m.err
}
func (m *mockAttendanceLogsRepo) BulkPaperOT(_ context.Context, _ []BulkPaperOTRow, _ uuid.UUID) BulkPaperOTResult {
	return m.bulkResult
}

func TestAttendanceLogs_Upload_Forbidden(t *testing.T) {
	h := AttendanceLogsHandler{Repository: &mockAttendanceLogsRepo{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1" + "/attendance/upload", nil)
	
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.Upload(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 got %d", rec.Code)
	}
}

func createAttendanceMultipart(filename string, content []byte) (*bytes.Buffer, string) {
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)
	part, _ := writer.CreateFormFile("file", filename)
	_, _ = part.Write(content)
	_ = writer.Close()
	return body, writer.FormDataContentType()
}

func TestAttendanceLogs_Upload_Success(t *testing.T) {
	user := uuid.New()
	repo := &mockAttendanceLogsRepo{}
	h := AttendanceLogsHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}, StorageDir: t.TempDir()}

	validXLSX, _ := writeXLSX([]xlsxSheet{{Name: "Sheet1", Rows: [][]string{{"employee_code", "punch_time"}, {"EMP001", "2026-03-10 09:00:00"}}}})
	body, ctype := createAttendanceMultipart("sample.xlsx", validXLSX)
	req := httptest.NewRequest(http.MethodPost, "/api/v1" + "/attendance/upload", body)
	req.Header.Set("Content-Type", ctype)
	
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))

	rec := httptest.NewRecorder()
	h.Upload(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", rec.Code, rec.Body.String())
	}
	var res struct {
		JobID uuid.UUID `json:"job_id"`
	}
	if err := json.Unmarshal(rec.Body.Bytes(), &res); err != nil || res.JobID == uuid.Nil {
		t.Fatalf("expected valid job id response, got %s", rec.Body.String())
	}
}

func TestAttendanceLogs_UploadLogs_Success(t *testing.T) {
	user := uuid.New()
	repo := &mockAttendanceLogsRepo{}
	h := AttendanceLogsHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}, StorageDir: t.TempDir()}
	validXLSX, _ := writeXLSX([]xlsxSheet{{Name: "Sheet1", Rows: [][]string{{"employee_code", "punch_time"}, {"EMP001", "2026-03-10 09:00:00"}}}})
	body, ctype := createAttendanceMultipart("sample.xlsx", validXLSX)
	req := httptest.NewRequest(http.MethodPost, "/api/v1" + "/attendance/logs/upload", body)
	req.Header.Set("Content-Type", ctype)
	
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))

	rec := httptest.NewRecorder()
	h.UploadLogs(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAttendanceLogs_LatestAndJobStatus(t *testing.T) {
	user := uuid.New()
	jobID := uuid.New()
	sampleJob := &AttendanceImportJob{ID: jobID, UserID: user, Status: "processing", TargetType: "adjustment"}
	repo := &mockAttendanceLogsRepo{latestJob: sampleJob, job: sampleJob}
	h := AttendanceLogsHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}

	// Latest
	req := httptest.NewRequest(http.MethodGet, "/api/v1" + "/attendance/import-jobs/latest", nil)
	
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.Latest(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("latest: expected 200 got %d", rec.Code)
	}

	// Job by ID
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1" + "/attendance/import-jobs/"+jobID.String(), nil)
	
	req2.SetPathValue("id", jobID.String())
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), user))
	rec2 := httptest.NewRecorder()
	h.Job(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("job: expected 200 got %d", rec2.Code)
	}
}

func TestAttendanceLogs_ErrorFile(t *testing.T) {
	user := uuid.New()
	jobID := uuid.New()
	repo := &mockAttendanceLogsRepo{errorData: []byte("PK\x03\x04errorfile")}
	h := AttendanceLogsHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1" + "/attendance/import-jobs/"+jobID.String()+"/error-file", nil)
	
	req.SetPathValue("id", jobID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.ErrorFile(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("error-file: expected 200 got %d", rec.Code)
	}
	if rec.Header().Get("Content-Disposition") != `attachment; filename="error_`+jobID.String()+`.xlsx"` {
		t.Fatalf("unexpected content-disposition %s", rec.Header().Get("Content-Disposition"))
	}
}

func TestAttendanceLogs_LogsAndRawLogs(t *testing.T) {
	user := uuid.New()
	repo := &mockAttendanceLogsRepo{
		logs:    []AttendanceLogRecord{{ID: uuid.New(), EmployeeFullName: "Alice"}},
		rawLogs: []RawAttendanceLogRecord{{ID: uuid.New(), EmployeeFullName: "Bob"}},
	}
	h := AttendanceLogsHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}

	// Logs
	req := httptest.NewRequest(http.MethodGet, "/api/v1" + "/attendance/logs?page=1&limit=20", nil)
	
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.Logs(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("logs: expected 200 got %d", rec.Code)
	}

	// Raw logs
	req2 := httptest.NewRequest(http.MethodGet, "/api/v1" + "/attendance/raw-logs?page=1&limit=20", nil)
	
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), user))
	rec2 := httptest.NewRecorder()
	h.RawLogs(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("raw-logs: expected 200 got %d", rec2.Code)
	}
}

func TestAttendanceLogs_StagingLifecycle(t *testing.T) {
	user := uuid.New()
	stagingID := uuid.New()
	empID := uuid.New()
	repo := &mockAttendanceLogsRepo{
		staging:   []AttendanceStagingLog{{ID: stagingID, RawUserID: "U1", Status: "pending"}},
		dismissed: true,
	}
	h := AttendanceLogsHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}

	// List staging
	req := httptest.NewRequest(http.MethodGet, "/api/v1" + "/attendance/staging-logs", nil)
	
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.Staging(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("staging: expected 200 got %d", rec.Code)
	}

	// Resolve staging
	resolveBody, _ := json.Marshal(ResolveStagingRequest{EmployeeID: empID})
	req2 := httptest.NewRequest(http.MethodPost, "/api/v1" + "/attendance/staging-logs/"+stagingID.String()+"/resolve", bytes.NewReader(resolveBody))
	
	req2.SetPathValue("id", stagingID.String())
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), user))
	rec2 := httptest.NewRecorder()
	h.Resolve(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("resolve: expected 200 got %d", rec2.Code)
	}

	// Dismiss staging
	req3 := httptest.NewRequest(http.MethodDelete, "/api/v1" + "/attendance/staging-logs/"+stagingID.String(), nil)
	
	req3.SetPathValue("id", stagingID.String())
	req3 = req3.WithContext(httpinfra.WithUserIDForTest(req3.Context(), user))
	rec3 := httptest.NewRecorder()
	h.Dismiss(rec3, req3)
	if rec3.Code != http.StatusNoContent {
		t.Fatalf("dismiss: expected 204 got %d", rec3.Code)
	}
}

func TestAttendanceLogs_PaperOT(t *testing.T) {
	user := uuid.New()
	empID := uuid.New()
	repo := &mockAttendanceLogsRepo{
		summary:    AttendanceDailySummary{ID: uuid.New(), EmploymentID: empID, Date: "2026-03-10", ClaimedOTMinutes: 60, OTStatus: "pending_manager"},
		bulkResult: BulkPaperOTResult{TotalRows: 1, SuccessCount: 1},
	}
	h := AttendanceLogsHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}

	// Single Paper OT
	otBody, _ := json.Marshal(PaperOTRequest{Date: "2026-03-10", ClaimedMinutes: 60, Reason: "Urgent deployment"})
	req := httptest.NewRequest(http.MethodPost, "/api/v1" + "/attendance/enter-paper-ot/"+empID.String(), bytes.NewReader(otBody))
	
	req.SetPathValue("employee_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), user))
	rec := httptest.NewRecorder()
	h.EnterPaperOT(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("paper-ot: expected 200 got %d: %s", rec.Code, rec.Body.String())
	}

	// Bulk Paper OT CSV
	csvData := []byte("employee_id,date,minutes,reason\nEMP001,2026-03-10,60,Urgent deployment\n")
	body, ctype := createAttendanceMultipart("paper_ot.csv", csvData)
	req2 := httptest.NewRequest(http.MethodPost, "/api/v1" + "/attendance/bulk-upload-paper-ot", body)
	req2.Header.Set("Content-Type", ctype)
	
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), user))
	rec2 := httptest.NewRecorder()
	h.BulkPaperOT(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("bulk-paper-ot: expected 200 got %d: %s", rec2.Code, rec2.Body.String())
	}
}
