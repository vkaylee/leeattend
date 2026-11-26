package api

import (
	"context"
	"errors"
	"net/http"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

// Authorizer evaluates current single-organization grants; database failures deny access.
type Authorizer interface {
	IsMember(context.Context, uuid.UUID) (bool, error)
	HasPermission(context.Context, uuid.UUID, string) bool
	EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error)
	GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error)
	IsGlobalManager(context.Context, uuid.UUID) bool
	HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool
	GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error)
}

// AuditEmitter records security and domain audit events.
type AuditEmitter interface {
	Emit(context.Context, audit.Event) error
}

func hasPermission(r *http.Request, authorizer Authorizer, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && authorizer != nil && authorizer.HasPermission(r.Context(), userID, permission)
}

func writeForbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func writeMappedError(w http.ResponseWriter, r *http.Request, err error, mapper func(error) *apperror.AppError) {
	var appErr *apperror.AppError
	if errors.As(err, &appErr) {
		apperror.WriteError(w, r, appErr)
		return
	}
	if mapper != nil {
		if mapped := mapper(err); mapped != nil {
			apperror.WriteError(w, r, mapped)
			return
		}
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
