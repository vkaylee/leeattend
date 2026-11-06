package api

import (
	"context"
	"net/http"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type SystemHealthAuthorizer interface {
	HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool
}

type SystemHealthHandler struct {
	Authorizer SystemHealthAuthorizer
}

type systemHealthResponse struct {
	Status       string `json:"status"`
	ViewerUserID string `json:"viewer_user_id"`
}

func (h SystemHealthHandler) Get(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	workspaceID, hasWorkspace := httpinfra.WorkspaceIDFromContext(r.Context())
	if !ok || !hasWorkspace {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyUnauthorized))
		return
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, string(auth.PermissionInfraMonitor)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, systemHealthResponse{
		Status:       "ok",
		ViewerUserID: userID.String(),
	})
}
