package api

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrLocationNotFound          = errors.New("location: not found")
	ErrLocationInUse             = errors.New("location: in use by employees")
	ErrLocationHQCannotChange    = errors.New("location: headquarters flag cannot be changed")
	ErrLocationHQCannotDelete    = errors.New("location: headquarters cannot be deleted")
	ErrLocationHQCannotBeCreated = errors.New("location: headquarters cannot be created via standard endpoint")
)

type WorkLocation struct {
	ID             uuid.UUID `json:"id"`
	Name           string    `json:"name"`
	Address        *string   `json:"address"`
	Timezone       string    `json:"timezone"`
	IsHeadquarters bool      `json:"is_headquarters"`
	Status         string    `json:"status"`
	CreatedAt      string    `json:"created_at"`
	UpdatedAt      string    `json:"updated_at"`
}

type CreateLocationRequest struct {
	Name           string  `json:"name"`
	Address        *string `json:"address"`
	Timezone       *string `json:"timezone"`
	IsHeadquarters *bool   `json:"is_headquarters"`
	Status         *string `json:"status"`
}

type UpdateLocationRequest struct {
	Name           *string `json:"name"`
	Address        *string `json:"address"`
	Timezone       *string `json:"timezone"`
	IsHeadquarters *bool   `json:"is_headquarters"`
	Status         *string `json:"status"`
}

type LocationListResponse struct {
	Data      []WorkLocation `json:"data"`
	Total     int64          `json:"total"`
	Page      int            `json:"page"`
	Limit     int            `json:"limit"`
	Truncated *bool          `json:"truncated"`
}

type LocationRepository interface {
	ListLocations(ctx context.Context, page, limit int) (LocationListResponse, error)
	CreateLocation(ctx context.Context, req CreateLocationRequest) (WorkLocation, error)
	GetLocation(ctx context.Context, id uuid.UUID) (WorkLocation, error)
	UpdateLocation(ctx context.Context, id uuid.UUID, req UpdateLocationRequest) (WorkLocation, error)
	DeleteLocation(ctx context.Context, id uuid.UUID) error
}

type LocationHandler struct {
	Repository LocationRepository
	Authorizer Authorizer
}

func (h *LocationHandler) List(w http.ResponseWriter, r *http.Request) {
	if !h.isMember(r) {
		h.forbidden(w, r)
		return
	}
	p, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	res, err := h.Repository.ListLocations(r.Context(), p.Page, p.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, res)
}

func (h *LocationHandler) Create(w http.ResponseWriter, r *http.Request) {
	if !h.isMember(r) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateLocationRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.IsHeadquarters != nil && *req.IsHeadquarters {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	loc, err := h.Repository.CreateLocation(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, loc)
}

func (h *LocationHandler) Get(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !h.isMember(r) {
		h.forbidden(w, r)
		return
	}
	loc, err := h.Repository.GetLocation(r.Context(), id)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, loc)
}

func (h *LocationHandler) Update(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !h.isMember(r) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateLocationRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Name != nil {
		trimmed := strings.TrimSpace(*req.Name)
		if trimmed == "" {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		req.Name = &trimmed
	}
	loc, err := h.Repository.UpdateLocation(r.Context(), id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, loc)
}

func (h *LocationHandler) Delete(w http.ResponseWriter, r *http.Request) {
	id, idOK := pathUUID(r, "id")
	if !idOK || !h.isMember(r) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteLocation(r.Context(), id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *LocationHandler) isMember(r *http.Request) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	isMem, err := h.Authorizer.IsMember(r.Context(), userID)
	return err == nil && isMem
}

func (h *LocationHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h *LocationHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrLocationNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrLocationInUse) || errors.Is(err, ErrLocationHQCannotChange) || errors.Is(err, ErrLocationHQCannotDelete) || errors.Is(err, ErrLocationHQCannotBeCreated) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
