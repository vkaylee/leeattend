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
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockImportRepo struct {
	createdJob    bool
	candidates    []TerminationCandidate
	statuses      map[uuid.UUID]string
	candidatesErr error
	statusesErr   error
}

func (m *mockImportRepo) CreateEmployeeImportJob(_ context.Context, _ uuid.UUID, _ string, _ uuid.UUID) error {
	m.createdJob = true
	return nil
}

func (m *mockImportRepo) TerminationCandidates(_ context.Context, _ []string) ([]TerminationCandidate, error) {
	return m.candidates, m.candidatesErr
}

func (m *mockImportRepo) TerminationStatuses(_ context.Context, _ []uuid.UUID) (map[uuid.UUID]string, error) {
	return m.statuses, m.statusesErr
}

type mockImportQueue struct {
	enqueuedEmployee    bool
	enqueuedTermination bool
	lastEmployeeJob     EmployeeImportJob
	lastTerminationJob  TerminationImportJob
}

func (m *mockImportQueue) EnqueueEmployeeImport(_ context.Context, job EmployeeImportJob) error {
	m.enqueuedEmployee = true
	m.lastEmployeeJob = job
	return nil
}

func (m *mockImportQueue) EnqueueTerminationImport(_ context.Context, job TerminationImportJob) error {
	m.enqueuedTermination = true
	m.lastTerminationJob = job
	return nil
}

func createMultipartExcel(fieldName, filename string, payload []byte) (string, *bytes.Buffer, error) {
	var body bytes.Buffer
	writer := multipart.NewWriter(&body)
	part, err := writer.CreateFormFile(fieldName, filename)
	if err != nil {
		return "", nil, err
	}
	if _, err := part.Write(payload); err != nil {
		return "", nil, err
	}
	if err := writer.Close(); err != nil {
		return "", nil, err
	}
	return writer.FormDataContentType(), &body, nil
}

func sampleXLSX(rows [][]string) []byte {
	data, _ := writeXLSX([]xlsxSheet{{
		Name: "Sheet1",
		Rows: rows,
	}})
	return data
}

func TestEmployeeImport_Upload_Forbidden(t *testing.T) {
	h := EmployeeImportHandler{
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{}},
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/upload", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Upload(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestEmployeeImport_Upload_InvalidFormat(t *testing.T) {
	h := EmployeeImportHandler{
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesUpload): true}},
	}
	ct, body, _ := createMultipartExcel("file", "test.csv", []byte("a,b,c\n1,2,3"))
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/upload", body)
	req.Header.Set("Content-Type", ct)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Upload(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for CSV, got %d", rec.Code)
	}
}

func TestEmployeeImport_Upload_Success(t *testing.T) {
	repo := &mockImportRepo{}
	q := &mockImportQueue{}
	h := EmployeeImportHandler{
		Repository: repo,
		Queue:      q,
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesUpload): true}},
		StorageDir: t.TempDir(),
	}
	xlsxBytes := sampleXLSX([][]string{{"header"}, {"row"}})
	ct, body, _ := createMultipartExcel("file", "employees.xlsx", xlsxBytes)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/upload", body)
	req.Header.Set("Content-Type", ct)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Upload(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res struct {
		JobID uuid.UUID `json:"job_id"`
	}
	if err := json.Unmarshal(rec.Body.Bytes(), &res); err != nil || res.JobID == uuid.Nil {
		t.Fatalf("expected valid job_id, got %v", res)
	}
	if !repo.createdJob || !q.enqueuedEmployee {
		t.Fatalf("expected job created and enqueued")
	}
}

func TestEmployeeImport_Preview_Flow(t *testing.T) {
	emp1ID := uuid.New()
	emp2ID := uuid.New()
	emp3ID := uuid.New()
	emp1EmploymentID := uuid.New()
	emp2EmploymentID := uuid.New()
	emp3EmploymentID := uuid.New()
	leaveTypeID := uuid.New()

	repo := &mockImportRepo{
		candidates: []TerminationCandidate{
			{
				EmployeeID:   emp1ID,
				EmployeeCode: "EMP001",
				FullName:     "John Active",
				Employments: []TerminationEmployment{
					{
						ID:        emp1EmploymentID,
						Status:    "active",
						StartDate: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
					},
				},
				Balances: map[int][]TerminationLeaveBalance{
					2026: {
						{
							LeaveTypeID:   leaveTypeID,
							LeaveTypeCode: "AL",
							LeaveTypeName: "Annual Leave",
							Entitlement:   12,
						},
					},
				},
			},
			{
				EmployeeID:   emp2ID,
				EmployeeCode: "EMP002",
				FullName:     "Jane Early",
				Employments: []TerminationEmployment{
					{
						ID:        emp2EmploymentID,
						Status:    "active",
						StartDate: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
					},
				},
				Balances: map[int][]TerminationLeaveBalance{},
			},
			{
				EmployeeID:   emp3ID,
				EmployeeCode: "EMP003",
				FullName:     "Bob Terminated",
				Employments: []TerminationEmployment{
					{
						ID:        emp3EmploymentID,
						Status:    "terminated",
						StartDate: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
					},
				},
				Balances: map[int][]TerminationLeaveBalance{},
			},
		},
	}

	h := EmployeeImportHandler{
		Repository: repo,
		Queue:      &mockImportQueue{},
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesUpdate): true}},
	}

	xlsxData := sampleXLSX([][]string{
		{"employee_code", "termination_date", "termination_reason", "termination_notes"},
		{"EMP001", "2026-06-30", "voluntary_resignation", "Relocating"},
		{"EMP002", "2025-12-31", "voluntary_resignation", ""},
		{"EMP003", "2026-06-30", "voluntary_resignation", ""},
		{"EMP999", "2026-06-30", "voluntary_resignation", ""},
	})
	ct, body, _ := createMultipartExcel("file", "termination.xlsx", xlsxData)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/import-termination/preview", body)
	req.Header.Set("Content-Type", ct)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.PreviewTermination(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var preview TerminationImportPreviewResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &preview); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	if len(preview.ValidRows) != 1 {
		t.Fatalf("expected 1 valid row, got %d", len(preview.ValidRows))
	}
	valid := preview.ValidRows[0]
	if valid.EmployeeCode != "EMP001" || valid.EmploymentID != emp1EmploymentID {
		t.Fatalf("unexpected valid row: %+v", valid)
	}
	if len(valid.ProratedLeaves) != 1 {
		t.Fatalf("expected 1 prorated leave preview, got %d", len(valid.ProratedLeaves))
	}
	// 181 days out of 365 days of 12 = 6 days
	if valid.ProratedLeaves[0].NewEntitlement != 6 {
		t.Fatalf("expected new entitlement 6, got %d", valid.ProratedLeaves[0].NewEntitlement)
	}

	if len(preview.InvalidRows) != 3 {
		t.Fatalf("expected 3 invalid rows, got %d", len(preview.InvalidRows))
	}
	if !preview.CanCommit {
		t.Fatalf("expected can_commit to be true")
	}
}

func TestEmployeeImport_Preview_RejectsMultipleActiveEmployments(t *testing.T) {
	employeeID := uuid.New()
	repo := &mockImportRepo{candidates: []TerminationCandidate{{
		EmployeeID:   employeeID,
		EmployeeCode: "EMP-MULTI",
		FullName:     "Concurrent Employee",
		Employments: []TerminationEmployment{
			{ID: uuid.New(), Status: "active", StartDate: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)},
			{ID: uuid.New(), Status: "active", StartDate: time.Date(2026, 2, 1, 0, 0, 0, 0, time.UTC)},
		},
		Balances: map[int][]TerminationLeaveBalance{},
	}}}
	h := EmployeeImportHandler{
		Repository: repo,
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesUpdate): true}},
	}
	data := sampleXLSX([][]string{
		{"employee_code", "termination_date", "termination_reason", "termination_notes"},
		{"EMP-MULTI", "2026-06-30", "voluntary_resignation", ""},
	})
	contentType, body, err := createMultipartExcel("file", "termination.xlsx", data)
	if err != nil {
		t.Fatalf("create workbook: %v", err)
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/import-termination/preview", body)
	req.Header.Set("Content-Type", contentType)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.PreviewTermination(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var preview TerminationImportPreviewResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &preview); err != nil {
		t.Fatalf("decode preview: %v", err)
	}
	if len(preview.ValidRows) != 0 || len(preview.InvalidRows) != 1 || preview.CanCommit {
		t.Fatalf("ambiguous termination preview = %+v, want one invalid row and no commit", preview)
	}
	if preview.InvalidRows[0].Reason != "employee number matches multiple active employments; employer context is required" {
		t.Fatalf("unexpected ambiguity reason: %q", preview.InvalidRows[0].Reason)
	}
}

func TestEmployeeImport_Confirm_ValidationAndSuccess(t *testing.T) {
	activeEmpID := uuid.New()
	terminatedEmpID := uuid.New()

	repo := &mockImportRepo{
		statuses: map[uuid.UUID]string{
			activeEmpID:     "active",
			terminatedEmpID: "terminated",
		},
	}
	q := &mockImportQueue{}
	h := EmployeeImportHandler{
		Repository: repo,
		Queue:      q,
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesUpdate): true}},
	}

	// 1. Confirm with already terminated employment should fail 400
	payloadBad := ConfirmTerminationImportRequest{
		Rows: []ConfirmTerminationImportRow{
			{
				EmploymentID:      terminatedEmpID,
				TerminationDate:   "2026-06-30",
				TerminationReason: "voluntary_resignation",
			},
		},
	}
	badBody, _ := json.Marshal(payloadBad)
	reqBad := httptest.NewRequest(http.MethodPost, "/api/v1/employees/import-termination/confirm", bytes.NewReader(badBody))
	reqBad = reqBad.WithContext(httpinfra.WithUserIDForTest(reqBad.Context(), uuid.New()))
	recBad := httptest.NewRecorder()
	h.ConfirmTermination(recBad, reqBad)
	if recBad.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for terminated employment, got %d: %s", recBad.Code, recBad.Body.String())
	}

	// 2. Confirm with valid row should succeed
	payloadGood := ConfirmTerminationImportRequest{
		Rows: []ConfirmTerminationImportRow{
			{
				EmploymentID:      activeEmpID,
				TerminationDate:   "2026-06-30",
				TerminationReason: "voluntary_resignation",
			},
		},
	}
	goodBody, _ := json.Marshal(payloadGood)
	reqGood := httptest.NewRequest(http.MethodPost, "/api/v1/employees/import-termination/confirm", bytes.NewReader(goodBody))
	reqGood = reqGood.WithContext(httpinfra.WithUserIDForTest(reqGood.Context(), uuid.New()))
	recGood := httptest.NewRecorder()
	h.ConfirmTermination(recGood, reqGood)
	if recGood.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recGood.Code, recGood.Body.String())
	}

	var res ConfirmTerminationImportResponse
	if err := json.Unmarshal(recGood.Body.Bytes(), &res); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if res.SuccessCount != 1 || res.JobID == uuid.Nil {
		t.Fatalf("unexpected confirm response: %+v", res)
	}
	if !q.enqueuedTermination {
		t.Fatalf("expected termination import enqueued")
	}
}

func TestEmployeeImport_Template_Download(t *testing.T) {
	h := EmployeeImportHandler{
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesUpdate): true}},
	}

	// English template
	reqEn := httptest.NewRequest(http.MethodGet, "/api/v1/employees/import-termination/template", nil)
	reqEn = reqEn.WithContext(httpinfra.WithUserIDForTest(reqEn.Context(), uuid.New()))
	recEn := httptest.NewRecorder()
	h.TerminationTemplate(recEn, reqEn)

	if recEn.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recEn.Code)
	}
	if ct := recEn.Header().Get("Content-Type"); ct != "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" {
		t.Fatalf("unexpected Content-Type: %s", ct)
	}
	if cd := recEn.Header().Get("Content-Disposition"); cd != `attachment; filename="employee_termination_template.xlsx"` {
		t.Fatalf("unexpected Content-Disposition: %s", cd)
	}
	if !isXLSX(recEn.Body.Bytes()) {
		t.Fatalf("response body is not a valid zip/xlsx archive")
	}

	// Vietnamese template
	reqVi := httptest.NewRequest(http.MethodGet, "/api/v1/employees/import-termination/template?lang=vi", nil)
	reqVi = reqVi.WithContext(httpinfra.WithUserIDForTest(reqVi.Context(), uuid.New()))
	recVi := httptest.NewRecorder()
	h.TerminationTemplate(recVi, reqVi)
	if recVi.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recVi.Code)
	}
	wbVi, err := parseXLSX(recVi.Body.Bytes())
	if err != nil {
		t.Fatalf("failed to parse generated VI template: %v", err)
	}
	if len(wbVi.sheets) != 2 || wbVi.sheets[0].name != "Mẫu nhập" || wbVi.sheets[1].name != "Hướng dẫn" {
		t.Fatalf("unexpected VI sheets: %+v", wbVi.sheets)
	}
}
