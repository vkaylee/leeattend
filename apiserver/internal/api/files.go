package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
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
	ID          uuid.UUID  `json:"id"`
	WorkspaceID uuid.UUID  `json:"workspace_id"`
	FileName    string     `json:"file_name"`
	StorageKey  string     `json:"storage_key"`
	FileSize    int64      `json:"file_size"`
	MimeType    string     `json:"mime_type"`
	Category    string     `json:"category"`
	OwnerType   string     `json:"owner_type"`
	OwnerID     uuid.UUID  `json:"owner_id"`
	UploadedBy  *uuid.UUID `json:"uploaded_by,omitempty"`
	IsActive    bool       `json:"is_active"`
	ScanStatus  string     `json:"scan_status"`
	Version     int        `json:"version"`
	IsLatest    bool       `json:"is_latest"`
	CreatedAt   string     `json:"created_at"`
	UpdatedAt   string     `json:"updated_at"`
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

type FileStorage interface {
	Presign(ctx context.Context, method, key, contentType string, ttl time.Duration) (string, error)
}

type FileRepository interface {
	CreateFile(ctx context.Context, file WorkspaceFileRecord) error
	GetFile(ctx context.Context, wsID, fileID uuid.UUID) (WorkspaceFileRecord, error)
	DeleteFile(ctx context.Context, wsID, fileID uuid.UUID) error
	CreateSignatureRequest(ctx context.Context, req SignatureRequestRecord) error
	ListPendingSignatures(ctx context.Context, wsID, signerID uuid.UUID) ([]SignatureRequestRecord, error)
	SignDocument(ctx context.Context, wsID, requestID uuid.UUID) error
}

type FileHandler struct {
	Repository FileRepository
	Storage    FileStorage
	Authorizer WorkspaceAuthorizer
}

func (h FileHandler) CreateUploadURL(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
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
	wsID, ok := pathUUID(r, "ws_id")
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
	wsID, ok := pathUUID(r, "ws_id")
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
	wsID, ok := pathUUID(r, "ws_id")
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
	wsID, ok := pathUUID(r, "ws_id")
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
	wsID, ok := pathUUID(r, "ws_id")
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
	wsID, ok := pathUUID(r, "ws_id")
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
