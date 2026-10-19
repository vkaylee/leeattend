package tenant

import (
	"errors"
	"fmt"

	"github.com/google/uuid"
)

var ErrMissingWorkspace = errors.New("workspace scope is required")

// WorkspaceScope is a validated tenant boundary.
type WorkspaceScope struct{ id uuid.UUID }

func NewWorkspaceScope(id uuid.UUID) (WorkspaceScope, error) {
	if id == uuid.Nil {
		return WorkspaceScope{}, ErrMissingWorkspace
	}
	return WorkspaceScope{id: id}, nil
}

func (s WorkspaceScope) ID() uuid.UUID { return s.id }

// Predicate returns a fixed SQL predicate and argument. Column is allowlisted
// by callers through fixed constants; arbitrary identifiers are rejected.
func (s WorkspaceScope) Predicate(column string) (string, []any, error) {
	if s.id == uuid.Nil {
		return "", nil, ErrMissingWorkspace
	}
	switch column {
	case "workspace_id":
		return "workspace_id = $1", []any{s.id}, nil
	default:
		return "", nil, fmt.Errorf("invalid tenant column %q", column)
	}
}
