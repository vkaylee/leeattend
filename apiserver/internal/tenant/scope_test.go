package tenant

import (
	"errors"
	"testing"

	"github.com/google/uuid"
)

func TestWorkspaceScope_Valid(t *testing.T) {
	id := uuid.New()
	scope, err := NewWorkspaceScope(id)
	if err != nil || scope.ID() != id {
		t.Fatalf("unexpected scope: %v, %v", scope, err)
	}
	pred, args, err := scope.Predicate("workspace_id")
	if err != nil || pred != "workspace_id = $1" || len(args) != 1 || args[0] != id {
		t.Fatalf("unexpected predicate: %s %v %v", pred, args, err)
	}
}

func TestWorkspaceScope_NilFails(t *testing.T) {
	_, err := NewWorkspaceScope(uuid.Nil)
	if !errors.Is(err, ErrMissingWorkspace) {
		t.Fatalf("expected ErrMissingWorkspace, got %v", err)
	}
}
