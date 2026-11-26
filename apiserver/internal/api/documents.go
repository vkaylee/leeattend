package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"regexp"
	"strings"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

var (
	ErrFolderNotFound = errors.New("document folder: not found")
	ErrInvalidParent  = errors.New("document folder: invalid parent")
)

type DocumentFolderDto struct {
	ID             uuid.UUID  `json:"id"`
	Name           string     `json:"name"`
	ParentFolderID *uuid.UUID `json:"parent_folder_id"`
}

type CreateFolderRequest struct {
	Name           string     `json:"name"`
	ParentFolderID *uuid.UUID `json:"parent_folder_id,omitempty"`
}

type RenameFolderRequest struct {
	Name string `json:"name"`
}

type DistributeDocumentRequest struct {
	DocumentID        uuid.UUID   `json:"document_id"`
	TargetEmployeeIDs []uuid.UUID `json:"target_employee_ids"`
}

type TemplatePreviewRequest struct {
	TemplateContent string            `json:"template_content"`
	EmployeeID      uuid.UUID         `json:"employee_id"`
	ManualVariables map[string]string `json:"manual_variables,omitempty"`
}

type TemplatePreviewResponse struct {
	FilledContent string `json:"filled_content"`
}

type DocumentRepository interface {
	ListFolders(ctx context.Context) ([]DocumentFolderDto, error)
	CreateFolder(ctx context.Context, req CreateFolderRequest) (DocumentFolderDto, error)
	GetFolder(ctx context.Context, folderID uuid.UUID) (DocumentFolderDto, error)
	RenameFolder(ctx context.Context, folderID uuid.UUID, name string) (DocumentFolderDto, error)
	DeleteFolder(ctx context.Context, folderID uuid.UUID) error
	DistributeDocument(ctx context.Context, req DistributeDocumentRequest) (int, error)
	GetTemplateVariables(ctx context.Context, employeeID uuid.UUID) (map[string]string, error)
}

type DocumentHandler struct {
	Repository DocumentRepository
	Authorizer Authorizer
}

func (h *DocumentHandler) ListFolders(w http.ResponseWriter, r *http.Request) {
	if !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []DocumentFolderDto{})
		return
	}
	folders, err := h.Repository.ListFolders(r.Context())
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, folders)
}

func (h *DocumentHandler) CreateFolder(w http.ResponseWriter, r *http.Request) {
	if !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	var req CreateFolderRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	if req.Name == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, DocumentFolderDto{
			ID:             uuid.New(),
			Name:           req.Name,
			ParentFolderID: req.ParentFolderID,
		})
		return
	}
	folder, err := h.Repository.CreateFolder(r.Context(), req)
	if err != nil {
		if errors.Is(err, ErrInvalidParent) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, folder)
}

func (h *DocumentHandler) GetFolder(w http.ResponseWriter, r *http.Request) {
	folderID, idOK := pathUUID(r, "id")
	if !idOK || !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	folder, err := h.Repository.GetFolder(r.Context(), folderID)
	if err != nil {
		if errors.Is(err, ErrFolderNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, folder)
}

func (h *DocumentHandler) RenameFolder(w http.ResponseWriter, r *http.Request) {
	folderID, idOK := pathUUID(r, "id")
	if !idOK || !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	var req RenameFolderRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	if req.Name == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	folder, err := h.Repository.RenameFolder(r.Context(), folderID, req.Name)
	if err != nil {
		if errors.Is(err, ErrFolderNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, folder)
}

func (h *DocumentHandler) DeleteFolder(w http.ResponseWriter, r *http.Request) {
	folderID, idOK := pathUUID(r, "id")
	if !idOK || !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if err := h.Repository.DeleteFolder(r.Context(), folderID); err != nil {
		if errors.Is(err, ErrFolderNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]string{"status": "success"})
}

func (h *DocumentHandler) Distribute(w http.ResponseWriter, r *http.Request) {
	if !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	var req DistributeDocumentRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || req.DocumentID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	count := len(req.TargetEmployeeIDs)
	if h.Repository != nil && count > 0 {
		var err error
		count, err = h.Repository.DistributeDocument(r.Context(), req)
		if err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]any{
		"status": "success",
		"count":  count,
	})
}

var templateVarRegex = regexp.MustCompile(`\{\{([A-Z0-9_]+)\}\}`)

func FillTemplate(template string, vars map[string]string) string {
	return templateVarRegex.ReplaceAllStringFunc(template, func(match string) string {
		sub := templateVarRegex.FindStringSubmatch(match)
		if len(sub) == 2 {
			if val, ok := vars[sub[1]]; ok {
				return val
			}
		}
		return match
	})
}

func (h *DocumentHandler) PreviewTemplate(w http.ResponseWriter, r *http.Request) {
	if !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	var req TemplatePreviewRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || req.EmployeeID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	vars := make(map[string]string)
	if h.Repository != nil {
		if repoVars, err := h.Repository.GetTemplateVariables(r.Context(), req.EmployeeID); err == nil {
			for k, v := range repoVars {
				vars[k] = v
			}
		}
	}
	for k, v := range req.ManualVariables {
		vars[k] = v
	}
	filled := FillTemplate(req.TemplateContent, vars)
	apperror.WriteJSON(w, http.StatusOK, TemplatePreviewResponse{
		FilledContent: filled,
	})
}

func (h *DocumentHandler) GenerateTemplate(w http.ResponseWriter, r *http.Request) {
	_, idOK := pathUUID(r, "id")
	if !idOK || !isMember(r, h.Authorizer) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]string{"status": "generated"})
}
