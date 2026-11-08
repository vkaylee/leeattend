package api

import (
	"context"

	"github.com/google/uuid"
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
