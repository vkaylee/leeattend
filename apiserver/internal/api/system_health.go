package api

import (
	"context"
	"net/http"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type SystemRoleRepository interface {
	GetSystemRolesForUser(context.Context, uuid.UUID) ([]string, error)
}

type SystemHealthHandler struct {
	Roles SystemRoleRepository
}

type systemHealthResponse struct {
	Status       string   `json:"status"`
	ViewerUserID string   `json:"viewer_user_id"`
	ViewerRoles  []string `json:"viewer_roles"`
}

func (h SystemHealthHandler) Get(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Roles == nil {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyUnauthorized))
		return
	}
	roles, err := h.Roles.GetSystemRolesForUser(r.Context(), userID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	known := make([]string, 0, len(roles))
	allowed := false
	for _, role := range roles {
		switch role {
		case "admin", "platform_admin":
			known = append(known, role)
			allowed = true
		case "support":
			known = append(known, role)
			allowed = true
		case "billing_admin":
			known = append(known, role)
		}
	}
	if !allowed {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, systemHealthResponse{
		Status:       "ok",
		ViewerUserID: userID.String(),
		ViewerRoles:  known,
	})
}
