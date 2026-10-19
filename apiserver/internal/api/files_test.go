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

type mockFileStorage struct{}

func (mockFileStorage) Presign(ctx context.Context, method, key, contentType string, ttl time.Duration) (string, error) {
	return "https://s3.example.com/" + key + "?signed=true", nil
}

type mockFileRepo struct {
	Files      map[uuid.UUID]WorkspaceFileRecord
	Signatures map[uuid.UUID]SignatureRequestRecord
}

func (m *mockFileRepo) CreateFile(ctx context.Context, file WorkspaceFileRecord) error {
	if m.Files == nil {
		m.Files = make(map[uuid.UUID]WorkspaceFileRecord)
	}
	m.Files[file.ID] = file
	return nil
}

func (m *mockFileRepo) GetFile(ctx context.Context, wsID, fileID uuid.UUID) (WorkspaceFileRecord, error) {
	if f, ok := m.Files[fileID]; ok && f.WorkspaceID == wsID && f.IsActive {
		return f, nil
	}
	return WorkspaceFileRecord{}, ErrFileNotFound
}

func (m *mockFileRepo) DeleteFile(ctx context.Context, wsID, fileID uuid.UUID) error {
	if f, ok := m.Files[fileID]; ok && f.WorkspaceID == wsID && f.IsActive {
		f.IsActive = false
		m.Files[fileID] = f
		return nil
	}
	return ErrFileNotFound
}

func (m *mockFileRepo) CreateSignatureRequest(ctx context.Context, req SignatureRequestRecord) error {
	if m.Signatures == nil {
		m.Signatures = make(map[uuid.UUID]SignatureRequestRecord)
	}
	m.Signatures[req.ID] = req
	return nil
}

func (m *mockFileRepo) ListPendingSignatures(ctx context.Context, wsID, signerID uuid.UUID) ([]SignatureRequestRecord, error) {
	var list []SignatureRequestRecord
	for _, s := range m.Signatures {
		if s.WorkspaceID == wsID && s.SignerID == signerID && s.Status == "pending" {
			list = append(list, s)
		}
	}
	return list, nil
}

func (m *mockFileRepo) SignDocument(ctx context.Context, wsID, requestID uuid.UUID) error {
	if s, ok := m.Signatures[requestID]; ok && s.WorkspaceID == wsID && s.Status == "pending" {
		s.Status = "signed"
		m.Signatures[requestID] = s
		return nil
	}
	return ErrFileNotFound
}

type mockFileAuthorizer struct{}

func (mockFileAuthorizer) IsSystemAdmin(context.Context, uuid.UUID) bool { return false }
func (mockFileAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return true
}

func TestFilePresignedUploadAndMetadata(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	repo := &mockFileRepo{}
	handler := FileHandler{
		Repository: repo,
		Storage:    mockFileStorage{},
		Authorizer: mockFileAuthorizer{},
	}

	reqBody := CreateUploadUrlRequest{
		FileName:  "contract.pdf",
		MimeType:  "application/pdf",
		FileSize:  1024,
		Category:  "contract",
		OwnerType: "employee",
		OwnerID:   uuid.New(),
	}
	raw, _ := json.Marshal(reqBody)
	req := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/files/presigned", bytes.NewReader(raw))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.CreateUploadURL(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp CreateUploadUrlResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if resp.File.FileName != "contract.pdf" || resp.UploadURL == "" {
		t.Fatalf("invalid presigned response: %+v", resp)
	}

	// Fetch metadata
	metaReq := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/files/"+resp.File.ID.String(), nil)
	metaReq.SetPathValue("ws_id", wsID.String())
	metaReq.SetPathValue("id", resp.File.ID.String())
	metaReq = metaReq.WithContext(httpinfra.WithUserIDForTest(metaReq.Context(), userID))
	metaRec := httptest.NewRecorder()

	handler.GetMetadata(metaRec, metaReq)
	if metaRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for metadata, got %d", metaRec.Code)
	}

	// Get URL
	urlReq := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/files/"+resp.File.ID.String()+"/url", nil)
	urlReq.SetPathValue("ws_id", wsID.String())
	urlReq.SetPathValue("id", resp.File.ID.String())
	urlReq = urlReq.WithContext(httpinfra.WithUserIDForTest(urlReq.Context(), userID))
	urlRec := httptest.NewRecorder()

	handler.GetURL(urlRec, urlReq)
	if urlRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for download url, got %d", urlRec.Code)
	}

	// Delete
	delReq := httptest.NewRequest(http.MethodDelete, "/v1/workspaces/"+wsID.String()+"/files/"+resp.File.ID.String(), nil)
	delReq.SetPathValue("ws_id", wsID.String())
	delReq.SetPathValue("id", resp.File.ID.String())
	delReq = delReq.WithContext(httpinfra.WithUserIDForTest(delReq.Context(), userID))
	delRec := httptest.NewRecorder()

	handler.DeleteFile(delRec, delReq)
	if delRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for delete, got %d", delRec.Code)
	}
}

func TestSignatureRequestLifecycle(t *testing.T) {
	wsID := uuid.New()
	signerID := uuid.New()
	docID := uuid.New()
	repo := &mockFileRepo{}
	handler := FileHandler{
		Repository: repo,
		Authorizer: mockFileAuthorizer{},
	}

	body, _ := json.Marshal(CreateSignatureRequestBody{SignerID: signerID})
	req := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/documents/"+docID.String()+"/signature-requests", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("document_id", docID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.CreateSignatureRequest(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var sigReq SignatureRequestRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &sigReq); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	// List pending
	listReq := httptest.NewRequest(http.MethodGet, "/v1/ess/workspaces/"+wsID.String()+"/signatures/pending", nil)
	listReq.SetPathValue("ws_id", wsID.String())
	listReq = listReq.WithContext(httpinfra.WithUserIDForTest(listReq.Context(), signerID))
	listRec := httptest.NewRecorder()

	handler.ListPendingSignatures(listRec, listReq)
	if listRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for pending list, got %d", listRec.Code)
	}
	var list []SignatureRequestRecord
	if err := json.Unmarshal(listRec.Body.Bytes(), &list); err != nil || len(list) != 1 {
		t.Fatalf("expected 1 pending signature, got %d: %v", len(list), err)
	}

	// Sign
	signReq := httptest.NewRequest(http.MethodPost, "/v1/ess/workspaces/"+wsID.String()+"/signatures/"+sigReq.ID.String()+"/sign", nil)
	signReq.SetPathValue("ws_id", wsID.String())
	signReq.SetPathValue("request_id", sigReq.ID.String())
	signReq = signReq.WithContext(httpinfra.WithUserIDForTest(signReq.Context(), signerID))
	signRec := httptest.NewRecorder()

	handler.SignDocument(signRec, signReq)
	if signRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for sign, got %d", signRec.Code)
	}
}
