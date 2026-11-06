package api

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"io"
	"mime/multipart"
	"net/http"
	"net/http/httptest"
	"net/textproto"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockFileStorage struct {
	PutData map[string][]byte
}

func (mockFileStorage) Presign(ctx context.Context, method, key, contentType string, ttl time.Duration) (string, error) {
	return "https://s3.example.com/" + key + "?signed=true", nil
}

func (m mockFileStorage) Put(ctx context.Context, key, contentType string, body io.Reader) error {
	if m.PutData != nil {
		data, _ := io.ReadAll(body)
		m.PutData[key] = data
	}
	return nil
}

func (m mockFileStorage) Get(ctx context.Context, key string) (*http.Response, error) {
	data := []byte("small file content")
	if m.PutData != nil && m.PutData[key] != nil {
		data = m.PutData[key]
	}
	resp := &http.Response{
		StatusCode: http.StatusOK,
		Body:       io.NopCloser(bytes.NewReader(data)),
		Header:     make(http.Header),
	}
	return resp, nil
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

func (m *mockFileRepo) CreateFileVersion(ctx context.Context, wsID, oldFileID uuid.UUID, newFile WorkspaceFileRecord) (WorkspaceFileRecord, error) {
	if m.Files == nil {
		m.Files = make(map[uuid.UUID]WorkspaceFileRecord)
	}
	if old, ok := m.Files[oldFileID]; ok {
		old.IsLatest = false
		m.Files[oldFileID] = old
	}
	m.Files[newFile.ID] = newFile
	return newFile, nil
}

func (m *mockFileRepo) ListFileVersions(ctx context.Context, wsID, fileID uuid.UUID) ([]WorkspaceFileRecord, error) {
	var list []WorkspaceFileRecord
	curr := fileID
	for {
		f, ok := m.Files[curr]
		if !ok || f.WorkspaceID != wsID {
			break
		}
		list = append(list, f)
		if f.ParentFileID == nil {
			break
		}
		curr = *f.ParentFileID
	}
	return list, nil
}

func (m *mockFileRepo) ListFilesByOwner(ctx context.Context, wsID uuid.UUID, ownerType string, ownerID uuid.UUID) ([]WorkspaceFileRecord, error) {
	var list []WorkspaceFileRecord
	for _, f := range m.Files {
		if f.WorkspaceID == wsID && f.OwnerType == ownerType && f.OwnerID == ownerID && f.IsActive {
			list = append(list, f)
		}
	}
	return list, nil
}

type mockFileAuthorizer struct {
	notMember bool
}

func (mockFileAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return true
}
func (m mockFileAuthorizer) IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error) {
	return !m.notMember, nil
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

func TestFileUploadMultipart(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	repo := &mockFileRepo{Files: make(map[uuid.UUID]WorkspaceFileRecord)}
	storage := mockFileStorage{PutData: make(map[string][]byte)}
	handler := FileHandler{
		Repository: repo,
		Storage:    storage,
		Authorizer: mockFileAuthorizer{},
	}

	var b bytes.Buffer
	w := multipart.NewWriter(&b)
	_ = w.WriteField("category", "contract")
	_ = w.WriteField("owner_type", "employee")
	_ = w.WriteField("owner_id", empID.String())
	h := make(textproto.MIMEHeader)
	h.Set("Content-Disposition", `form-data; name="file"; filename="my_contract.pdf"`)
	h.Set("Content-Type", "application/pdf")
	fw, _ := w.CreatePart(h)
	_, _ = fw.Write([]byte("%PDF-1.4 test contract body"))
	_ = w.Close()

	req := httptest.NewRequest(http.MethodPost, "/v1/workspaces/"+wsID.String()+"/files/upload", &b)
	req.Header.Set("Content-Type", w.FormDataContentType())
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.UploadFile(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 Created, got %d: %s", rec.Code, rec.Body.String())
	}
	var recFile WorkspaceFileRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &recFile); err != nil {
		t.Fatalf("failed to unmarshal file record: %v", err)
	}
	if recFile.Category != "contract" || recFile.FileName != "my_contract.pdf" {
		t.Errorf("unexpected record fields: %+v", recFile)
	}
	if recFile.ContentHash == nil || *recFile.ContentHash == "" {
		t.Error("expected non-empty content_hash")
	}
}

func TestFileUpdateCoW(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	fileID := uuid.New()

	initialData := []byte("%PDF-1.4 initial content")
	initialHash := hex.EncodeToString(func() []byte { h := sha256.Sum256(initialData); return h[:] }())

	repo := &mockFileRepo{Files: map[uuid.UUID]WorkspaceFileRecord{
		fileID: {
			ID:          fileID,
			WorkspaceID: wsID,
			FileName:    "file.pdf",
			StorageKey:  "workspaces/" + wsID.String() + "/contract/file.pdf",
			FileSize:    int64(len(initialData)),
			MimeType:    "application/pdf",
			ContentHash: &initialHash,
			Category:    "contract",
			OwnerType:   "employee",
			OwnerID:     empID,
			IsActive:    true,
			Version:     1,
			IsLatest:    true,
		},
	}}
	storage := mockFileStorage{PutData: make(map[string][]byte)}
	handler := FileHandler{
		Repository: repo,
		Storage:    storage,
		Authorizer: mockFileAuthorizer{},
	}

	// 1. Update with identical content -> CoW Short-Circuit (no version change)
	var b1 bytes.Buffer
	w1 := multipart.NewWriter(&b1)
	fw1, _ := w1.CreateFormFile("file", "file.pdf")
	_, _ = fw1.Write(initialData)
	_ = w1.Close()

	req1 := httptest.NewRequest(http.MethodPut, "/v1/workspaces/"+wsID.String()+"/files/"+fileID.String(), &b1)
	req1.Header.Set("Content-Type", w1.FormDataContentType())
	req1.SetPathValue("ws_id", wsID.String())
	req1.SetPathValue("id", fileID.String())
	req1 = req1.WithContext(httpinfra.WithUserIDForTest(req1.Context(), userID))
	rec1 := httptest.NewRecorder()

	handler.UpdateFile(rec1, req1)
	if rec1.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for CoW bypass, got %d", rec1.Code)
	}
	var resp1 WorkspaceFileRecord
	_ = json.Unmarshal(rec1.Body.Bytes(), &resp1)
	if resp1.Version != 1 || resp1.ID != fileID {
		t.Fatalf("expected version 1 and same file ID for CoW short-circuit, got version %d id %v", resp1.Version, resp1.ID)
	}

	// 2. Update with changed content -> Version increment and new record
	newData := []byte("%PDF-1.4 revised contract content")
	var b2 bytes.Buffer
	w2 := multipart.NewWriter(&b2)
	h2 := make(textproto.MIMEHeader)
	h2.Set("Content-Disposition", `form-data; name="file"; filename="file_v2.pdf"`)
	h2.Set("Content-Type", "application/pdf")
	fw2, _ := w2.CreatePart(h2)
	_, _ = fw2.Write(newData)
	_ = w2.Close()

	req2 := httptest.NewRequest(http.MethodPut, "/v1/workspaces/"+wsID.String()+"/files/"+fileID.String(), &b2)
	req2.Header.Set("Content-Type", w2.FormDataContentType())
	req2.SetPathValue("ws_id", wsID.String())
	req2.SetPathValue("id", fileID.String())
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), userID))
	rec2 := httptest.NewRecorder()

	handler.UpdateFile(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for updated version, got %d: %s", rec2.Code, rec2.Body.String())
	}
	var resp2 WorkspaceFileRecord
	_ = json.Unmarshal(rec2.Body.Bytes(), &resp2)
	if resp2.Version != 2 {
		t.Fatalf("expected version 2 for changed file, got %d", resp2.Version)
	}
	if resp2.ParentFileID == nil || *resp2.ParentFileID != fileID {
		t.Fatalf("expected parent_file_id %v, got %v", fileID, resp2.ParentFileID)
	}
	if !repo.Files[resp2.ID].IsLatest || repo.Files[fileID].IsLatest {
		t.Errorf("expected new file to be latest and old file to not be latest")
	}
}

func TestFileDownload(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()

	smallID := uuid.New()
	largeID := uuid.New()

	repo := &mockFileRepo{Files: map[uuid.UUID]WorkspaceFileRecord{
		smallID: {
			ID:          smallID,
			WorkspaceID: wsID,
			FileName:    "small.pdf",
			StorageKey:  "small.pdf",
			FileSize:    500, // <= 1MB
			MimeType:    "application/pdf",
			IsActive:    true,
		},
		largeID: {
			ID:          largeID,
			WorkspaceID: wsID,
			FileName:    "large.pdf",
			StorageKey:  "large.pdf",
			FileSize:    5 * 1024 * 1024, // > 1MB
			MimeType:    "application/pdf",
			IsActive:    true,
		},
	}}
	storage := mockFileStorage{PutData: map[string][]byte{"small.pdf": []byte("small file content")}}
	handler := FileHandler{
		Repository: repo,
		Storage:    storage,
		Authorizer: mockFileAuthorizer{},
	}

	// Small file: streamed proxy bytes with 200 OK
	reqSmall := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/files/"+smallID.String()+"/download", nil)
	reqSmall.SetPathValue("ws_id", wsID.String())
	reqSmall.SetPathValue("id", smallID.String())
	reqSmall = reqSmall.WithContext(httpinfra.WithUserIDForTest(reqSmall.Context(), userID))
	recSmall := httptest.NewRecorder()

	handler.DownloadFile(recSmall, reqSmall)
	if recSmall.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for small file proxy, got %d", recSmall.Code)
	}
	if recSmall.Header().Get("Content-Type") != "application/pdf" {
		t.Errorf("unexpected content type: %s", recSmall.Header().Get("Content-Type"))
	}
	if recSmall.Body.String() != "small file content" {
		t.Errorf("unexpected streamed body: %s", recSmall.Body.String())
	}

	// Large file: 307 Temporary Redirect
	reqLarge := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/files/"+largeID.String()+"/download", nil)
	reqLarge.SetPathValue("ws_id", wsID.String())
	reqLarge.SetPathValue("id", largeID.String())
	reqLarge = reqLarge.WithContext(httpinfra.WithUserIDForTest(reqLarge.Context(), userID))
	recLarge := httptest.NewRecorder()

	handler.DownloadFile(recLarge, reqLarge)
	if recLarge.Code != http.StatusTemporaryRedirect {
		t.Fatalf("expected 307 Temporary Redirect for large file, got %d", recLarge.Code)
	}
}

func TestFileListVersionsAndByOwner(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	v1ID := uuid.New()
	v2ID := uuid.New()

	repo := &mockFileRepo{Files: map[uuid.UUID]WorkspaceFileRecord{
		v1ID: {
			ID:          v1ID,
			WorkspaceID: wsID,
			FileName:    "v1.pdf",
			Category:    "contract",
			OwnerType:   "employee",
			OwnerID:     empID,
			IsActive:    true,
			Version:     1,
			IsLatest:    false,
		},
		v2ID: {
			ID:           v2ID,
			WorkspaceID:  wsID,
			FileName:     "v2.pdf",
			Category:     "contract",
			OwnerType:    "employee",
			OwnerID:      empID,
			ParentFileID: &v1ID,
			IsActive:     true,
			Version:      2,
			IsLatest:     true,
		},
	}}
	handler := FileHandler{
		Repository: repo,
		Authorizer: mockFileAuthorizer{},
	}

	// Test versions
	reqVer := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/files/"+v2ID.String()+"/versions", nil)
	reqVer.SetPathValue("ws_id", wsID.String())
	reqVer.SetPathValue("id", v2ID.String())
	reqVer = reqVer.WithContext(httpinfra.WithUserIDForTest(reqVer.Context(), userID))
	recVer := httptest.NewRecorder()

	handler.ListVersions(recVer, reqVer)
	if recVer.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for versions, got %d", recVer.Code)
	}
	var versions []WorkspaceFileRecord
	_ = json.Unmarshal(recVer.Body.Bytes(), &versions)
	if len(versions) != 2 {
		t.Fatalf("expected 2 versions in lineage, got %d", len(versions))
	}

	// Test by-owner
	reqOwner := httptest.NewRequest(http.MethodGet, "/v1/workspaces/"+wsID.String()+"/files/by-owner/employee/"+empID.String(), nil)
	reqOwner.SetPathValue("ws_id", wsID.String())
	reqOwner.SetPathValue("owner_type", "employee")
	reqOwner.SetPathValue("owner_id", empID.String())
	reqOwner = reqOwner.WithContext(httpinfra.WithUserIDForTest(reqOwner.Context(), userID))
	recOwner := httptest.NewRecorder()

	handler.ListByOwner(recOwner, reqOwner)
	if recOwner.Code != http.StatusOK {
		t.Fatalf("expected 200 OK for by-owner, got %d", recOwner.Code)
	}
	var ownerFiles []WorkspaceFileRecord
	_ = json.Unmarshal(recOwner.Body.Bytes(), &ownerFiles)
	if len(ownerFiles) != 2 {
		t.Fatalf("expected 2 files for employee, got %d", len(ownerFiles))
	}
}
