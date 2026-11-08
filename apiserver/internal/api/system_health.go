package api

import (
	"net/http"

	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type SystemHealthHandler struct {
	Authorizer Authorizer
}

type systemHealthResponse struct {
	Status       string `json:"status"`
	ViewerUserID string `json:"viewer_user_id"`
}

func (h SystemHealthHandler) Get(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyUnauthorized))
		return
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionInfraMonitor)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, systemHealthResponse{
		Status:       "ok",
		ViewerUserID: userID.String(),
	})
}
