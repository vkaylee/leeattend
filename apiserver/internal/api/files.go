package api

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrFileValidation = errors.New("file: validation failed")
	ErrFileNotFound   = errors.New("file: not found")
)

type CreateUploadUrlRequest struct {
	FileName      string    `json:"file_name"`
	MimeType      string    `json:"mime_type"`
	FileSize      int64     `json:"file_size"`
	Category      string    `json:"category"`
	OwnerType     string    `json:"owner_type"`
	OwnerID       uuid.UUID `json:"owner_id"`
	ExpiresInSecs *int64    `json:"expires_in_secs,omitempty"`
}

type WorkspaceFileRecord struct {
	ID           uuid.UUID  `json:"id"`
	WorkspaceID  uuid.UUID  `json:"workspace_id"`
	FileName     string     `json:"file_name"`
	StorageKey   string     `json:"storage_key"`
	FileSize     int64      `json:"file_size"`
	MimeType     string     `json:"mime_type"`
	Category     string     `json:"category"`
	OwnerType    string     `json:"owner_type"`
	OwnerID      uuid.UUID  `json:"owner_id"`
	UploadedBy   *uuid.UUID `json:"uploaded_by,omitempty"`
	IsActive     bool       `json:"is_active"`
	ScanStatus   string     `json:"scan_status"`
	Version      int        `json:"version"`
	IsLatest     bool       `json:"is_latest"`
	ContentHash  *string    `json:"content_hash,omitempty"`
	FolderID     *uuid.UUID `json:"folder_id,omitempty"`
	ExpiresAt    *string    `json:"expires_at,omitempty"`
	ScannedAt    *string    `json:"scanned_at,omitempty"`
	ParentFileID *uuid.UUID `json:"parent_file_id,omitempty"`
	CreatedAt    string     `json:"created_at"`
	UpdatedAt    string     `json:"updated_at"`
}
type FileStorage interface {
	Presign(ctx context.Context, method, key, contentType string, ttl time.Duration) (string, error)
	Put(ctx context.Context, key, contentType string, body io.Reader) error
	Get(ctx context.Context, key string) (*http.Response, error)
}

type FileRepository interface {
	CreateFile(ctx context.Context, file WorkspaceFileRecord) error
	GetFile(ctx context.Context, wsID, fileID uuid.UUID) (WorkspaceFileRecord, error)
	DeleteFile(ctx context.Context, wsID, fileID uuid.UUID) error
	CreateSignatureRequest(ctx context.Context, req SignatureRequestRecord) error
	ListPendingSignatures(ctx context.Context, wsID, signerID uuid.UUID) ([]SignatureRequestRecord, error)
	SignDocument(ctx context.Context, wsID, requestID uuid.UUID) error
	CreateFileVersion(ctx context.Context, wsID, oldFileID uuid.UUID, newFile WorkspaceFileRecord) (WorkspaceFileRecord, error)
	ListFileVersions(ctx context.Context, wsID, fileID uuid.UUID) ([]WorkspaceFileRecord, error)
	ListFilesByOwner(ctx context.Context, wsID uuid.UUID, ownerType string, ownerID uuid.UUID) ([]WorkspaceFileRecord, error)
}

type CategoryConfig struct {
	MaxSize      int64
	AllowedMimes []string
}

var categoryConfigs = map[string]CategoryConfig{
	"employee_avatar": {
		MaxSize:      5 * 1024 * 1024,
		AllowedMimes: []string{"image/jpeg", "image/png", "image/webp"},
	},
	"user_avatar": {
		MaxSize:      5 * 1024 * 1024,
		AllowedMimes: []string{"image/jpeg", "image/png", "image/webp"},
	},
	"workspace_logo": {
		MaxSize:      5 * 1024 * 1024,
		AllowedMimes: []string{"image/jpeg", "image/png", "image/webp", "image/svg+xml"},
	},
	"attendance_photo": {
		MaxSize:      10 * 1024 * 1024,
		AllowedMimes: []string{"image/jpeg", "image/png", "image/webp"},
	},
	"employee_doc": {
		MaxSize:      50 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf", "image/jpeg", "image/png", "image/webp", "application/msword", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"},
	},
	"onboarding_attachment": {
		MaxSize:      50 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf", "image/jpeg", "image/png", "image/webp", "application/msword", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"},
	},
	"contract": {
		MaxSize:      50 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf", "application/msword", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"},
	},
	"leave_attachment": {
		MaxSize:      20 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf"},
	},
	"attendance_adjustment_attachment": {
		MaxSize:      20 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf"},
	},
	"overtime_request_attachment": {
		MaxSize:      20 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf"},
	},
	"ot_claim_attachment": {
		MaxSize:      20 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf"},
	},
	"payslip": {
		MaxSize:      10 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf"},
	},
	"report_export": {
		MaxSize:      100 * 1024 * 1024,
		AllowedMimes: []string{"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "text/csv"},
	},
	"import_file": {
		MaxSize:      20 * 1024 * 1024,
		AllowedMimes: []string{"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "application/vnd.ms-excel"},
	},
	"policy": {
		MaxSize:      50 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf"},
	},
	"template_output": {
		MaxSize:      50 * 1024 * 1024,
		AllowedMimes: []string{"application/pdf", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"},
	},
	"system_asset": {
		MaxSize:      50 * 1024 * 1024,
		AllowedMimes: []string{"image/jpeg", "image/png", "image/webp", "image/svg+xml", "application/pdf"},
	},
}

func ValidateCategory(category string, size int64, mimeType string) error {
	cfg, ok := categoryConfigs[category]
	if !ok {
		return errors.New("invalid file category")
	}
	if size > cfg.MaxSize {
		return errors.New("file size exceeds maximum allowed limit")
	}
	allowed := false
	for _, m := range cfg.AllowedMimes {
		if strings.HasSuffix(m, "/*") {
			prefix := strings.TrimSuffix(m, "*")
			if strings.HasPrefix(mimeType, prefix) {
				allowed = true
				break
			}
		} else if mimeType == m {
			allowed = true
			break
		}
	}
	if !allowed {
		return errors.New("mime type not allowed for category")
	}
	return nil
}

var validOwnerTypes = map[string]bool{
	"employee":                 true,
	"employment":               true,
	"leave_request":            true,
	"workspace":                true,
	"user":                     true,
	"system":                   true,
	"payroll":                  true,
	"attendance_adjustment":    true,
	"overtime_request":         true,
	"attendance_daily_summary": true,
}

func sanitizeFileName(name string) string {
	ext := filepath.Ext(name)
	stem := strings.TrimSuffix(name, ext)
	if stem == "" {
		stem = "file"
	}
	var b strings.Builder
	for _, r := range stem {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '-' || r == '_' {
			b.WriteRune(r)
		} else {
			b.WriteRune('_')
		}
	}
	sanitizedStem := b.String()
	if ext == "" {
		return sanitizedStem
	}
	var eb strings.Builder
	for _, r := range ext {
		if (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9') || r == '.' {
			eb.WriteRune(r)
		}
	}
	return sanitizedStem + eb.String()
}

func generateStorageKey(wsID uuid.UUID, category, fileName string) string {
	dateStr := time.Now().UTC().Format("2006/01")
	fileUUID := uuid.New()
	sanitized := sanitizeFileName(fileName)
	return fmt.Sprintf("workspaces/%s/%s/%s/%s_%s", wsID.String(), category, dateStr, fileUUID.String(), sanitized)
}

type CreateUploadUrlResponse struct {
	File      WorkspaceFileRecord `json:"file"`
	UploadURL string              `json:"upload_url"`
}

type FileURLResponse struct {
	URL string `json:"url"`
}

type SignatureRequestRecord struct {
	ID          uuid.UUID `json:"id"`
	WorkspaceID uuid.UUID `json:"workspace_id"`
	DocumentID  uuid.UUID `json:"document_id"`
	SignerID    uuid.UUID `json:"signer_id"`
	Status      string    `json:"status"`
	SignedAt    *string   `json:"signed_at,omitempty"`
	CreatedAt   string    `json:"created_at"`
	UpdatedAt   string    `json:"updated_at"`
}

type CreateSignatureRequestBody struct {
	SignerID uuid.UUID `json:"signer_id"`
}

type FileHandler struct {
	Repository FileRepository
	Storage    FileStorage
	Authorizer WorkspaceAuthorizer
}

func (h FileHandler) CreateUploadURL(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || !h.allowed(r, wsID, "files:manage") {
		h.forbidden(w, r)
		return
	}
	var req CreateUploadUrlRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		h.validation(w, r)
		return
	}
	req.FileName = strings.TrimSpace(req.FileName)
	req.MimeType = strings.TrimSpace(req.MimeType)
	if req.FileName == "" || req.MimeType == "" || req.FileSize <= 0 || req.FileSize > 50*1024*1024 || req.OwnerID == uuid.Nil {
		h.validation(w, r)
		return
	}
	fileID := uuid.New()
	storageKey := fmt.Sprintf("workspaces/%s/%s/%s", wsID.String(), fileID.String(), req.FileName)
	now := time.Now().UTC()
	file := WorkspaceFileRecord{
		ID:          fileID,
		WorkspaceID: wsID,
		FileName:    req.FileName,
		StorageKey:  storageKey,
		FileSize:    req.FileSize,
		MimeType:    req.MimeType,
		Category:    req.Category,
		OwnerType:   req.OwnerType,
		OwnerID:     req.OwnerID,
		UploadedBy:  &userID,
		IsActive:    true,
		ScanStatus:  "pending",
		Version:     1,
		IsLatest:    true,
		CreatedAt:   now.Format(time.RFC3339),
		UpdatedAt:   now.Format(time.RFC3339),
	}
	if h.Repository != nil {
		if err := h.Repository.CreateFile(r.Context(), file); err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
	}
	uploadURL := "http://localhost:9000/leeattend-files/" + storageKey
	if h.Storage != nil {
		ttl := 15 * time.Minute
		if req.ExpiresInSecs != nil && *req.ExpiresInSecs > 0 {
			ttl = time.Duration(*req.ExpiresInSecs) * time.Second
		}
		if signed, err := h.Storage.Presign(r.Context(), http.MethodPut, storageKey, req.MimeType, ttl); err == nil {
			uploadURL = signed
		}
	}
	apperror.WriteJSON(w, http.StatusOK, CreateUploadUrlResponse{File: file, UploadURL: uploadURL})
}

func (h FileHandler) GetMetadata(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	fileID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "files:read") {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	file, err := h.Repository.GetFile(r.Context(), wsID, fileID)
	if err != nil {
		if errors.Is(err, ErrFileNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, file)
}

func (h FileHandler) GetURL(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	fileID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "files:read") {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	file, err := h.Repository.GetFile(r.Context(), wsID, fileID)
	if err != nil {
		if errors.Is(err, ErrFileNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	downloadURL := "http://localhost:9000/leeattend-files/" + file.StorageKey
	if h.Storage != nil {
		if signed, err := h.Storage.Presign(r.Context(), http.MethodGet, file.StorageKey, "", 15*time.Minute); err == nil {
			downloadURL = signed
		}
	}
	apperror.WriteJSON(w, http.StatusOK, FileURLResponse{URL: downloadURL})
}

func (h FileHandler) DeleteFile(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	fileID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.allowed(r, wsID, "files:manage") {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if err := h.Repository.DeleteFile(r.Context(), wsID, fileID); err != nil {
		if errors.Is(err, ErrFileNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct{}{})
}

func (h FileHandler) CreateSignatureRequest(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	docID, docOK := pathUUID(r, "document_id")
	if !ok || !docOK || !h.allowed(r, wsID, "documents:manage") {
		h.forbidden(w, r)
		return
	}
	var req CreateSignatureRequestBody
	if json.NewDecoder(r.Body).Decode(&req) != nil || req.SignerID == uuid.Nil {
		h.validation(w, r)
		return
	}
	now := time.Now().UTC()
	sigReq := SignatureRequestRecord{
		ID:          uuid.New(),
		WorkspaceID: wsID,
		DocumentID:  docID,
		SignerID:    req.SignerID,
		Status:      "pending",
		CreatedAt:   now.Format(time.RFC3339),
		UpdatedAt:   now.Format(time.RFC3339),
	}
	if h.Repository != nil {
		if err := h.Repository.CreateSignatureRequest(r.Context(), sigReq); err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
	}
	apperror.WriteJSON(w, http.StatusOK, sigReq)
}

func (h FileHandler) ListPendingSignatures(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []SignatureRequestRecord{})
		return
	}
	list, err := h.Repository.ListPendingSignatures(r.Context(), wsID, userID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, list)
}

func (h FileHandler) SignDocument(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	reqID, reqOK := pathUUID(r, "request_id")
	if !ok || !reqOK {
		h.forbidden(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if err := h.Repository.SignDocument(r.Context(), wsID, reqID); err != nil {
		if errors.Is(err, ErrFileNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, struct{}{})
}

func (h FileHandler) allowed(r *http.Request, wsID uuid.UUID, p string) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), u, wsID, p)
}

func (h FileHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h FileHandler) validation(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
}

func (h FileHandler) isMember(r *http.Request, wsID uuid.UUID) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	if checker, ok := h.Authorizer.(interface {
		IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error)
	}); ok {
		m, err := checker.IsMember(r.Context(), u, wsID)
		return err == nil && m
	}
	return h.Authorizer.HasPermission(r.Context(), u, wsID, "files:read") ||
		h.Authorizer.HasPermission(r.Context(), u, wsID, "files:manage")
}

func (h FileHandler) UploadFile(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || !h.isMember(r, wsID) {
		h.forbidden(w, r)
		return
	}

	const maxUploadBytes = 20 * 1024 * 1024
	r.Body = http.MaxBytesReader(w, r.Body, maxUploadBytes)
	if err := r.ParseMultipartForm(maxUploadBytes); err != nil {
		h.validation(w, r)
		return
	}

	category := strings.TrimSpace(r.FormValue("category"))
	ownerType := strings.TrimSpace(r.FormValue("owner_type"))
	ownerIDStr := strings.TrimSpace(r.FormValue("owner_id"))
	ownerID, err := uuid.Parse(ownerIDStr)
	if err != nil || ownerID == uuid.Nil || !validOwnerTypes[ownerType] {
		h.validation(w, r)
		return
	}

	file, header, err := r.FormFile("file")
	if err != nil {
		h.validation(w, r)
		return
	}
	defer file.Close()

	fileName := strings.TrimSpace(header.Filename)
	if fileName == "" {
		fileName = "upload"
	}
	mimeType := header.Header.Get("Content-Type")
	if mimeType == "" {
		mimeType = "application/octet-stream"
	}

	data, err := io.ReadAll(file)
	if err != nil || len(data) == 0 {
		h.validation(w, r)
		return
	}

	fileSize := int64(len(data))
	if err := ValidateCategory(category, fileSize, mimeType); err != nil {
		h.validation(w, r)
		return
	}

	hashSum := sha256.Sum256(data)
	contentHash := hex.EncodeToString(hashSum[:])
	storageKey := generateStorageKey(wsID, category, fileName)

	if h.Storage != nil {
		if err := h.Storage.Put(r.Context(), storageKey, mimeType, bytes.NewReader(data)); err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
	}

	var expiresAt *string
	if expStr := r.FormValue("expires_in_secs"); expStr != "" {
		if secs, err := strconv.ParseInt(expStr, 10, 64); err == nil && secs > 0 {
			expTime := time.Now().UTC().Add(time.Duration(secs) * time.Second).Format(time.RFC3339)
			expiresAt = &expTime
		}
	}

	now := time.Now().UTC().Format(time.RFC3339)
	fileID := uuid.New()
	record := WorkspaceFileRecord{
		ID:          fileID,
		WorkspaceID: wsID,
		FileName:    fileName,
		StorageKey:  storageKey,
		FileSize:    fileSize,
		MimeType:    mimeType,
		ContentHash: &contentHash,
		Category:    category,
		OwnerType:   ownerType,
		OwnerID:     ownerID,
		UploadedBy:  &userID,
		IsActive:    true,
		ExpiresAt:   expiresAt,
		ScanStatus:  "pending",
		Version:     1,
		IsLatest:    true,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	if h.Repository != nil {
		if err := h.Repository.CreateFile(r.Context(), record); err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
	}

	apperror.WriteJSON(w, http.StatusCreated, record)
}

func (h FileHandler) UpdateFile(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	fileID, idOK := pathUUID(r, "id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !idOK || !userOK || !h.isMember(r, wsID) {
		h.forbidden(w, r)
		return
	}

	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	currentFile, err := h.Repository.GetFile(r.Context(), wsID, fileID)
	if err != nil {
		if errors.Is(err, ErrFileNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	const maxUploadBytes = 20 * 1024 * 1024
	r.Body = http.MaxBytesReader(w, r.Body, maxUploadBytes)
	if err := r.ParseMultipartForm(maxUploadBytes); err != nil {
		h.validation(w, r)
		return
	}

	file, header, err := r.FormFile("file")
	if err != nil {
		h.validation(w, r)
		return
	}
	defer file.Close()

	fileName := strings.TrimSpace(header.Filename)
	if fileName == "" {
		fileName = currentFile.FileName
	}
	mimeType := header.Header.Get("Content-Type")
	if mimeType == "" {
		mimeType = currentFile.MimeType
	}

	data, err := io.ReadAll(file)
	if err != nil || len(data) == 0 {
		h.validation(w, r)
		return
	}

	fileSize := int64(len(data))
	hashSum := sha256.Sum256(data)
	contentHash := hex.EncodeToString(hashSum[:])

	// CoW Short-Circuit Optimization
	if currentFile.ContentHash != nil && *currentFile.ContentHash == contentHash {
		apperror.WriteJSON(w, http.StatusOK, currentFile)
		return
	}

	if err := ValidateCategory(currentFile.Category, fileSize, mimeType); err != nil {
		h.validation(w, r)
		return
	}

	newStorageKey := generateStorageKey(wsID, currentFile.Category, fileName)
	if h.Storage != nil {
		if err := h.Storage.Put(r.Context(), newStorageKey, mimeType, bytes.NewReader(data)); err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
	}

	now := time.Now().UTC().Format(time.RFC3339)
	newRecord := WorkspaceFileRecord{
		ID:           uuid.New(),
		WorkspaceID:  wsID,
		FileName:     fileName,
		StorageKey:   newStorageKey,
		FileSize:     fileSize,
		MimeType:     mimeType,
		ContentHash:  &contentHash,
		Category:     currentFile.Category,
		FolderID:     currentFile.FolderID,
		OwnerType:    currentFile.OwnerType,
		OwnerID:      currentFile.OwnerID,
		UploadedBy:   &userID,
		IsActive:     true,
		ExpiresAt:    currentFile.ExpiresAt,
		ScanStatus:   "pending",
		Version:      currentFile.Version + 1,
		ParentFileID: &currentFile.ID,
		IsLatest:     true,
		CreatedAt:    now,
		UpdatedAt:    now,
	}

	created, err := h.Repository.CreateFileVersion(r.Context(), wsID, currentFile.ID, newRecord)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	apperror.WriteJSON(w, http.StatusOK, created)
}

func (h FileHandler) DownloadFile(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	fileID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.isMember(r, wsID) {
		h.forbidden(w, r)
		return
	}

	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}

	file, err := h.Repository.GetFile(r.Context(), wsID, fileID)
	if err != nil {
		if errors.Is(err, ErrFileNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}

	const proxyThreshold = 1024 * 1024 // 1MB
	if file.FileSize <= proxyThreshold && h.Storage != nil {
		resp, err := h.Storage.Get(r.Context(), file.StorageKey)
		if err != nil || resp == nil {
			apperror.WriteError(w, r, apperror.Internal(fmt.Errorf("file download failed: %w", err)))
			return
		}
		defer resp.Body.Close()

		w.Header().Set("Content-Type", file.MimeType)
		w.Header().Set("Content-Disposition", fmt.Sprintf(`attachment; filename="%s"`, file.FileName))
		w.WriteHeader(http.StatusOK)
		_, _ = io.Copy(w, resp.Body)
		return
	}

	downloadURL := "http://localhost:9000/leeattend-files/" + file.StorageKey
	if h.Storage != nil {
		if signed, err := h.Storage.Presign(r.Context(), http.MethodGet, file.StorageKey, "", 15*time.Minute); err == nil {
			downloadURL = signed
		}
	}
	http.Redirect(w, r, downloadURL, http.StatusTemporaryRedirect)
}

func (h FileHandler) ListVersions(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	fileID, idOK := pathUUID(r, "id")
	if !ok || !idOK || !h.isMember(r, wsID) {
		h.forbidden(w, r)
		return
	}

	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []WorkspaceFileRecord{})
		return
	}

	versions, err := h.Repository.ListFileVersions(r.Context(), wsID, fileID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, versions)
}

func (h FileHandler) ListByOwner(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	ownerType := r.PathValue("owner_type")
	ownerIDStr := r.PathValue("owner_id")
	ownerID, idErr := uuid.Parse(ownerIDStr)
	if !ok || idErr != nil || !validOwnerTypes[ownerType] || !h.isMember(r, wsID) {
		h.validation(w, r)
		return
	}

	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []WorkspaceFileRecord{})
		return
	}

	files, err := h.Repository.ListFilesByOwner(r.Context(), wsID, ownerType, ownerID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, files)
}
