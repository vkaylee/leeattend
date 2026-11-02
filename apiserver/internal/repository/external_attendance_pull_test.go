package repository

import (
	"context"
	"errors"
	"testing"

	"github.com/google/uuid"
)

func TestExternalAttendancePullRepositoryRequiresTenantDB(t *testing.T) {
	r := ExternalAttendancePullRepository{}
	_, err := r.GetState(context.Background(), uuid.New(), uuid.New())
	if err == nil || err.Error() != "external attendance pull repository: tenant database is not configured" {
		t.Fatalf("expected tenant database error, got %v", err)
	}
}

func TestFailureText(t *testing.T) {
	if got := failureText(nil); got != nil {
		t.Fatalf("nil failure should remain nil, got %v", got)
	}
	want := errors.New("provider unavailable")
	if got := failureText(want); got != want.Error() {
		t.Fatalf("failure text = %v, want %q", got, want.Error())
	}
}
