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

type mockDocAuthorizer struct {
	notMember bool
}

func (mockDocAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return true
}
func (m mockDocAuthorizer) IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error) {
	return !m.notMember, nil
}

type mockDocRepo struct {
	Folders         map[uuid.UUID]DocumentFolderDto
	Distributions   []DistributeDocumentRequest
	CustomVariables map[string]string
}

func (m *mockDocRepo) ListFolders(ctx context.Context, wsID uuid.UUID) ([]DocumentFolderDto, error) {
	var list []DocumentFolderDto
	for _, f := range m.Folders {
		list = append(list, f)
	}
	return list, nil
}

func (m *mockDocRepo) CreateFolder(ctx context.Context, wsID uuid.UUID, req CreateFolderRequest) (DocumentFolderDto, error) {
	if m.Folders == nil {
		m.Folders = make(map[uuid.UUID]DocumentFolderDto)
	}
	if req.ParentFolderID != nil {
		if _, ok := m.Folders[*req.ParentFolderID]; !ok {
			return DocumentFolderDto{}, ErrInvalidParent
		}
	}
	id := uuid.New()
	dto := DocumentFolderDto{
		ID:             id,
		Name:           req.Name,
		ParentFolderID: req.ParentFolderID,
	}
	m.Folders[id] = dto
	return dto, nil
}

func (m *mockDocRepo) GetFolder(ctx context.Context, wsID, folderID uuid.UUID) (DocumentFolderDto, error) {
	if f, ok := m.Folders[folderID]; ok {
		return f, nil
	}
	return DocumentFolderDto{}, ErrFolderNotFound
}

func (m *mockDocRepo) RenameFolder(ctx context.Context, wsID, folderID uuid.UUID, name string) (DocumentFolderDto, error) {
	if f, ok := m.Folders[folderID]; ok {
		f.Name = name
		m.Folders[folderID] = f
		return f, nil
	}
	return DocumentFolderDto{}, ErrFolderNotFound
}

func (m *mockDocRepo) DeleteFolder(ctx context.Context, wsID, folderID uuid.UUID) error {
	if _, ok := m.Folders[folderID]; ok {
		delete(m.Folders, folderID)
		return nil
	}
	return ErrFolderNotFound
}

func (m *mockDocRepo) DistributeDocument(ctx context.Context, wsID uuid.UUID, req DistributeDocumentRequest) (int, error) {
	m.Distributions = append(m.Distributions, req)
	return len(req.TargetEmployeeIDs), nil
}

func (m *mockDocRepo) GetTemplateVariables(ctx context.Context, wsID, employeeID uuid.UUID) (map[string]string, error) {
	vars := map[string]string{
		"COMPANY_NAME":       "Acme Global",
		"EMPLOYEE_NAME":      "Jane Doe",
		"EMPLOYEE_ID_NUMBER": "EMP-007",
		"HIRE_DATE":          "15/01/2024",
		"JOB_TITLE":          "Senior Engineer",
		"CONTRACT_NUMBER":    "CTR-2024-001",
		"DOCUMENT_DATE":      time.Now().UTC().Format("02/01/2006"),
	}
	for k, v := range m.CustomVariables {
		vars[k] = v
	}
	return vars, nil
}

func TestDocumentFolders_CRUD(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	repo := &mockDocRepo{Folders: make(map[uuid.UUID]DocumentFolderDto)}
	handler := DocumentHandler{
		Repository: repo,
		Authorizer: mockDocAuthorizer{},
	}

	// 1. List empty folders
	listReq := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/documents/folders", nil)
	listReq = listReq.WithContext(httpinfra.WithOrganizationIDForTest(listReq.Context(), wsID))
	listReq = listReq.WithContext(httpinfra.WithUserIDForTest(listReq.Context(), userID))
	listRec := httptest.NewRecorder()
	handler.ListFolders(listRec, listReq)
	if listRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", listRec.Code)
	}

	// 2. Create root folder
	createBody, _ := json.Marshal(CreateFolderRequest{Name: "Policies"})
	createReq := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/documents/folders", bytes.NewReader(createBody))
	createReq = createReq.WithContext(httpinfra.WithOrganizationIDForTest(createReq.Context(), wsID))
	createReq = createReq.WithContext(httpinfra.WithUserIDForTest(createReq.Context(), userID))
	createRec := httptest.NewRecorder()
	handler.CreateFolder(createRec, createReq)
	if createRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", createRec.Code)
	}
	var rootFolder DocumentFolderDto
	_ = json.Unmarshal(createRec.Body.Bytes(), &rootFolder)
	if rootFolder.Name != "Policies" || rootFolder.ParentFolderID != nil {
		t.Errorf("unexpected root folder: %+v", rootFolder)
	}

	// 3. Create subfolder with invalid parent -> 400
	badParent := uuid.New()
	badSubBody, _ := json.Marshal(CreateFolderRequest{Name: "2024", ParentFolderID: &badParent})
	badSubReq := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/documents/folders", bytes.NewReader(badSubBody))
	badSubReq = badSubReq.WithContext(httpinfra.WithOrganizationIDForTest(badSubReq.Context(), wsID))
	badSubReq = badSubReq.WithContext(httpinfra.WithUserIDForTest(badSubReq.Context(), userID))
	badSubRec := httptest.NewRecorder()
	handler.CreateFolder(badSubRec, badSubReq)
	if badSubRec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 Bad Request for bad parent, got %d", badSubRec.Code)
	}

	// 4. Create subfolder with valid parent -> 200
	subBody, _ := json.Marshal(CreateFolderRequest{Name: "2024", ParentFolderID: &rootFolder.ID})
	subReq := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/documents/folders", bytes.NewReader(subBody))
	subReq = subReq.WithContext(httpinfra.WithOrganizationIDForTest(subReq.Context(), wsID))
	subReq = subReq.WithContext(httpinfra.WithUserIDForTest(subReq.Context(), userID))
	subRec := httptest.NewRecorder()
	handler.CreateFolder(subRec, subReq)
	if subRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for valid subfolder, got %d", subRec.Code)
	}
	var subFolder DocumentFolderDto
	_ = json.Unmarshal(subRec.Body.Bytes(), &subFolder)
	if subFolder.ParentFolderID == nil || *subFolder.ParentFolderID != rootFolder.ID {
		t.Errorf("expected parent_folder_id %v, got %v", rootFolder.ID, subFolder.ParentFolderID)
	}

	// 5. Get folder
	getReq := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/documents/folders/"+subFolder.ID.String(), nil)
	getReq = getReq.WithContext(httpinfra.WithOrganizationIDForTest(getReq.Context(), wsID))
	getReq.SetPathValue("id", subFolder.ID.String())
	getReq = getReq.WithContext(httpinfra.WithUserIDForTest(getReq.Context(), userID))
	getRec := httptest.NewRecorder()
	handler.GetFolder(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for get, got %d", getRec.Code)
	}

	// 6. Rename folder
	renameBody, _ := json.Marshal(RenameFolderRequest{Name: "2024-Archived"})
	renameReq := httptest.NewRequest(http.MethodPut, "/v1/workspaces/"+wsID.String()+"/documents/folders/"+subFolder.ID.String(), bytes.NewReader(renameBody))
	renameReq = renameReq.WithContext(httpinfra.WithOrganizationIDForTest(renameReq.Context(), wsID))
	renameReq.SetPathValue("id", subFolder.ID.String())
	renameReq = renameReq.WithContext(httpinfra.WithUserIDForTest(renameReq.Context(), userID))
	renameRec := httptest.NewRecorder()
	handler.RenameFolder(renameRec, renameReq)
	if renameRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for rename, got %d", renameRec.Code)
	}
	var renamedFolder DocumentFolderDto
	_ = json.Unmarshal(renameRec.Body.Bytes(), &renamedFolder)
	if renamedFolder.Name != "2024-Archived" {
		t.Errorf("expected renamed name '2024-Archived', got '%s'", renamedFolder.Name)
	}

	// 7. Delete folder
	delReq := httptest.NewRequest(http.MethodDelete, "/v1/workspaces/"+wsID.String()+"/documents/folders/"+subFolder.ID.String(), nil)
	delReq = delReq.WithContext(httpinfra.WithOrganizationIDForTest(delReq.Context(), wsID))
	delReq.SetPathValue("id", subFolder.ID.String())
	delReq = delReq.WithContext(httpinfra.WithUserIDForTest(delReq.Context(), userID))
	delRec := httptest.NewRecorder()
	handler.DeleteFolder(delRec, delReq)
	if delRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for delete, got %d", delRec.Code)
	}

	// 8. Get deleted folder -> 404
	getDeletedReq := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/documents/folders/"+subFolder.ID.String(), nil)
	getDeletedReq = getDeletedReq.WithContext(httpinfra.WithOrganizationIDForTest(getDeletedReq.Context(), wsID))
	getDeletedReq.SetPathValue("id", subFolder.ID.String())
	getDeletedReq = getDeletedReq.WithContext(httpinfra.WithUserIDForTest(getDeletedReq.Context(), userID))
	getDeletedRec := httptest.NewRecorder()
	handler.GetFolder(getDeletedRec, getDeletedReq)
	if getDeletedRec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 Not Found for deleted folder, got %d", getDeletedRec.Code)
	}
}

func TestDocumentDistribution(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	docID := uuid.New()
	emp1 := uuid.New()
	emp2 := uuid.New()

	repo := &mockDocRepo{}
	handler := DocumentHandler{
		Repository: repo,
		Authorizer: mockDocAuthorizer{},
	}

	reqBody, _ := json.Marshal(DistributeDocumentRequest{
		DocumentID:        docID,
		TargetEmployeeIDs: []uuid.UUID{emp1, emp2},
	})
	req := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/documents/distributions", bytes.NewReader(reqBody))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Distribute(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp map[string]any
	_ = json.Unmarshal(rec.Body.Bytes(), &resp)
	if resp["status"] != "success" || resp["count"] != float64(2) {
		t.Errorf("unexpected distribute response: %+v", resp)
	}
}

func TestDocumentTemplates_PreviewAndGenerate(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	templateID := uuid.New()

	repo := &mockDocRepo{}
	handler := DocumentHandler{
		Repository: repo,
		Authorizer: mockDocAuthorizer{},
	}

	templateContent := "Employment Agreement between {{COMPANY_NAME}} and {{EMPLOYEE_NAME}} (ID: {{EMPLOYEE_ID_NUMBER}}). Role: {{JOB_TITLE}}, Start Date: {{HIRE_DATE}}, Contract: {{CONTRACT_NUMBER}}, Date: {{DOCUMENT_DATE}}, Custom: {{CUSTOM_VAR}}, Missing: {{UNSET_KEY}}."

	// 1. Preview template
	reqBody, _ := json.Marshal(TemplatePreviewRequest{
		TemplateContent: templateContent,
		EmployeeID:      empID,
		ManualVariables: map[string]string{
			"CUSTOM_VAR": "CustomValue123",
			"JOB_TITLE":  "Lead Architect", // manual override of job title
		},
	})
	previewReq := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/documents/templates/preview", bytes.NewReader(reqBody))
	previewReq = previewReq.WithContext(httpinfra.WithOrganizationIDForTest(previewReq.Context(), wsID))
	previewReq = previewReq.WithContext(httpinfra.WithUserIDForTest(previewReq.Context(), userID))
	previewRec := httptest.NewRecorder()

	handler.PreviewTemplate(previewRec, previewReq)
	if previewRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", previewRec.Code, previewRec.Body.String())
	}
	var previewResp TemplatePreviewResponse
	_ = json.Unmarshal(previewRec.Body.Bytes(), &previewResp)

	// Verify standard variables substituted
	expectedSnippets := []string{
		"Employment Agreement between Acme Global and Jane Doe",
		"ID: EMP-007",
		"Role: Lead Architect", // overridden
		"Start Date: 15/01/2024",
		"Contract: CTR-2024-001",
		"Custom: CustomValue123",
		"Missing: {{UNSET_KEY}}", // unmatched kept intact
	}
	for _, snip := range expectedSnippets {
		if !bytes.Contains([]byte(previewResp.FilledContent), []byte(snip)) {
			t.Errorf("expected filled content to contain %q, but got:\n%s", snip, previewResp.FilledContent)
		}
	}

	// 2. Generate template
	genReq := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/documents/templates/"+templateID.String()+"/generate", bytes.NewReader(reqBody))
	genReq = genReq.WithContext(httpinfra.WithOrganizationIDForTest(genReq.Context(), wsID))
	genReq.SetPathValue("id", templateID.String())
	genReq = genReq.WithContext(httpinfra.WithUserIDForTest(genReq.Context(), userID))
	genRec := httptest.NewRecorder()

	handler.GenerateTemplate(genRec, genReq)
	if genRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", genRec.Code, genRec.Body.String())
	}
	var genResp map[string]string
	_ = json.Unmarshal(genRec.Body.Bytes(), &genResp)
	if genResp["status"] != "generated" {
		t.Errorf("expected status 'generated', got %s", genResp["status"])
	}
}

func TestDocumentAuth_Forbidden(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	handler := DocumentHandler{
		Repository: &mockDocRepo{},
		Authorizer: mockDocAuthorizer{notMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/documents/folders", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.ListFolders(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden for non-member, got %d", rec.Code)
	}
}
