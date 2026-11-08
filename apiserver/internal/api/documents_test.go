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

type documentTestAuthorizer struct{ notMember bool }

func (documentTestAuthorizer) HasPermission(context.Context, uuid.UUID, string) bool { return true }
func (documentTestAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (a documentTestAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return !a.notMember, nil
}
func (documentTestAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (documentTestAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

type mockDocRepo struct {
	Folders         map[uuid.UUID]DocumentFolderDto
	Distributions   []DistributeDocumentRequest
	CustomVariables map[string]string
}

func (m *mockDocRepo) ListFolders(context.Context) ([]DocumentFolderDto, error) {
	var list []DocumentFolderDto
	for _, f := range m.Folders {
		list = append(list, f)
	}
	return list, nil
}
func (m *mockDocRepo) CreateFolder(_ context.Context, req CreateFolderRequest) (DocumentFolderDto, error) {
	if m.Folders == nil {
		m.Folders = make(map[uuid.UUID]DocumentFolderDto)
	}
	if req.ParentFolderID != nil {
		if _, ok := m.Folders[*req.ParentFolderID]; !ok {
			return DocumentFolderDto{}, ErrInvalidParent
		}
	}
	id := uuid.New()
	folder := DocumentFolderDto{ID: id, Name: req.Name, ParentFolderID: req.ParentFolderID}
	m.Folders[id] = folder
	return folder, nil
}
func (m *mockDocRepo) GetFolder(_ context.Context, folderID uuid.UUID) (DocumentFolderDto, error) {
	if f, ok := m.Folders[folderID]; ok {
		return f, nil
	}
	return DocumentFolderDto{}, ErrFolderNotFound
}
func (m *mockDocRepo) RenameFolder(_ context.Context, folderID uuid.UUID, name string) (DocumentFolderDto, error) {
	if f, ok := m.Folders[folderID]; ok {
		f.Name = name
		m.Folders[folderID] = f
		return f, nil
	}
	return DocumentFolderDto{}, ErrFolderNotFound
}
func (m *mockDocRepo) DeleteFolder(_ context.Context, folderID uuid.UUID) error {
	if _, ok := m.Folders[folderID]; ok {
		delete(m.Folders, folderID)
		return nil
	}
	return ErrFolderNotFound
}
func (m *mockDocRepo) DistributeDocument(_ context.Context, req DistributeDocumentRequest) (int, error) {
	m.Distributions = append(m.Distributions, req)
	return len(req.TargetEmployeeIDs), nil
}
func (m *mockDocRepo) GetTemplateVariables(_ context.Context, _ uuid.UUID) (map[string]string, error) {
	vars := map[string]string{
		"COMPANY_NAME": "Acme Global", "EMPLOYEE_NAME": "Jane Doe", "EMPLOYEE_ID_NUMBER": "EMP-007",
		"HIRE_DATE": "15/01/2024", "JOB_TITLE": "Senior Engineer", "CONTRACT_NUMBER": "CTR-2024-001",
		"DOCUMENT_DATE": time.Now().UTC().Format("02/01/2006"),
	}
	for k, v := range m.CustomVariables {
		vars[k] = v
	}
	return vars, nil
}

func documentTestRequest(method, target string, body []byte, userID uuid.UUID) *http.Request {
	var req *http.Request
	if body == nil {
		req = httptest.NewRequest(method, target, nil)
	} else {
		req = httptest.NewRequest(method, target, bytes.NewReader(body))
	}
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestDocumentFoldersCRUDAndParentValidation(t *testing.T) {
	userID := uuid.New()
	repo := &mockDocRepo{Folders: make(map[uuid.UUID]DocumentFolderDto)}
	handler := DocumentHandler{Repository: repo, Authorizer: documentTestAuthorizer{}}

	listRec := httptest.NewRecorder()
	handler.ListFolders(listRec, documentTestRequest(http.MethodGet, "/api/v1/documents/folders", nil, userID))
	if listRec.Code != http.StatusOK {
		t.Fatalf("expected empty folder list, got %d", listRec.Code)
	}

	body, _ := json.Marshal(CreateFolderRequest{Name: "Policies"})
	createRec := httptest.NewRecorder()
	handler.CreateFolder(createRec, documentTestRequest(http.MethodPost, "/api/v1/documents/folders", body, userID))
	if createRec.Code != http.StatusOK {
		t.Fatalf("expected root folder create, got %d: %s", createRec.Code, createRec.Body.String())
	}
	var parent DocumentFolderDto
	if err := json.Unmarshal(createRec.Body.Bytes(), &parent); err != nil {
		t.Fatalf("decode root folder: %v", err)
	}

	invalidParent := uuid.New()
	badBody, _ := json.Marshal(CreateFolderRequest{Name: "2024", ParentFolderID: &invalidParent})
	badRec := httptest.NewRecorder()
	handler.CreateFolder(badRec, documentTestRequest(http.MethodPost, "/api/v1/documents/folders", badBody, userID))
	if badRec.Code != http.StatusBadRequest {
		t.Fatalf("expected invalid parent rejection, got %d", badRec.Code)
	}

	subBody, _ := json.Marshal(CreateFolderRequest{Name: "2024", ParentFolderID: &parent.ID})
	subRec := httptest.NewRecorder()
	handler.CreateFolder(subRec, documentTestRequest(http.MethodPost, "/api/v1/documents/folders", subBody, userID))
	if subRec.Code != http.StatusOK {
		t.Fatalf("expected subfolder create, got %d: %s", subRec.Code, subRec.Body.String())
	}
	var child DocumentFolderDto
	if err := json.Unmarshal(subRec.Body.Bytes(), &child); err != nil {
		t.Fatalf("decode child folder: %v", err)
	}
	if child.ParentFolderID == nil || *child.ParentFolderID != parent.ID {
		t.Fatalf("child parent mismatch: %+v", child)
	}

	getReq := documentTestRequest(http.MethodGet, "/api/v1/documents/folders/"+child.ID.String(), nil, userID)
	getReq.SetPathValue("id", child.ID.String())
	getRec := httptest.NewRecorder()
	handler.GetFolder(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected folder lookup, got %d", getRec.Code)
	}

	renameBody, _ := json.Marshal(RenameFolderRequest{Name: "2024-Archived"})
	renameReq := documentTestRequest(http.MethodPut, "/api/v1/documents/folders/"+child.ID.String(), renameBody, userID)
	renameReq.SetPathValue("id", child.ID.String())
	renameRec := httptest.NewRecorder()
	handler.RenameFolder(renameRec, renameReq)
	if renameRec.Code != http.StatusOK || repo.Folders[child.ID].Name != "2024-Archived" {
		t.Fatalf("expected rename, got %d: %+v", renameRec.Code, repo.Folders[child.ID])
	}

	delReq := documentTestRequest(http.MethodDelete, "/api/v1/documents/folders/"+child.ID.String(), nil, userID)
	delReq.SetPathValue("id", child.ID.String())
	delRec := httptest.NewRecorder()
	handler.DeleteFolder(delRec, delReq)
	if delRec.Code != http.StatusOK {
		t.Fatalf("expected delete, got %d", delRec.Code)
	}
	if _, exists := repo.Folders[child.ID]; exists {
		t.Fatal("deleted folder remains")
	}
}

func TestDocumentDistributionRequiresMember(t *testing.T) {
	userID, docID, emp1, emp2 := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	repo := &mockDocRepo{}
	handler := DocumentHandler{Repository: repo, Authorizer: documentTestAuthorizer{}}
	body, _ := json.Marshal(DistributeDocumentRequest{DocumentID: docID, TargetEmployeeIDs: []uuid.UUID{emp1, emp2}})
	rec := httptest.NewRecorder()
	handler.Distribute(rec, documentTestRequest(http.MethodPost, "/api/v1/documents/distributions", body, userID))
	var result map[string]any
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &result) != nil || result["count"] != float64(2) {
		t.Fatalf("expected two document distributions, got %d: %s", rec.Code, rec.Body.String())
	}
	if len(repo.Distributions) != 1 || len(repo.Distributions[0].TargetEmployeeIDs) != 2 {
		t.Fatalf("distribution request not recorded: %+v", repo.Distributions)
	}

	forbidden := DocumentHandler{Repository: repo, Authorizer: documentTestAuthorizer{notMember: true}}
	forbiddenRec := httptest.NewRecorder()
	forbidden.Distribute(forbiddenRec, documentTestRequest(http.MethodPost, "/api/v1/documents/distributions", body, userID))
	if forbiddenRec.Code != http.StatusForbidden {
		t.Fatalf("expected non-member denial, got %d", forbiddenRec.Code)
	}
}

func TestDocumentTemplatesPreviewAndGenerate(t *testing.T) {
	userID, empID, templateID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockDocRepo{}
	handler := DocumentHandler{Repository: repo, Authorizer: documentTestAuthorizer{}}
	templateContent := "Agreement between {{COMPANY_NAME}} and {{EMPLOYEE_NAME}}. Role: {{JOB_TITLE}}, Custom: {{CUSTOM_VAR}}, Missing: {{UNSET_KEY}}."
	body, _ := json.Marshal(TemplatePreviewRequest{TemplateContent: templateContent, EmployeeID: empID, ManualVariables: map[string]string{
		"CUSTOM_VAR": "CustomValue123", "JOB_TITLE": "Lead Architect",
	}})
	previewRec := httptest.NewRecorder()
	handler.PreviewTemplate(previewRec, documentTestRequest(http.MethodPost, "/api/v1/documents/templates/preview", body, userID))
	var preview TemplatePreviewResponse
	if previewRec.Code != http.StatusOK || json.Unmarshal(previewRec.Body.Bytes(), &preview) != nil {
		t.Fatalf("expected template preview, got %d: %s", previewRec.Code, previewRec.Body.String())
	}
	for _, expected := range []string{"Acme Global and Jane Doe", "Lead Architect", "CustomValue123", "{{UNSET_KEY}}"} {
		if !bytes.Contains([]byte(preview.FilledContent), []byte(expected)) {
			t.Errorf("preview missing %q: %s", expected, preview.FilledContent)
		}
	}

	genReq := documentTestRequest(http.MethodPost, "/api/v1/documents/templates/"+templateID.String()+"/generate", body, userID)
	genReq.SetPathValue("id", templateID.String())
	genRec := httptest.NewRecorder()
	handler.GenerateTemplate(genRec, genReq)
	var result map[string]string
	if genRec.Code != http.StatusOK || json.Unmarshal(genRec.Body.Bytes(), &result) != nil || result["status"] != "generated" {
		t.Fatalf("expected successful template generation, got %d: %s", genRec.Code, genRec.Body.String())
	}
}
