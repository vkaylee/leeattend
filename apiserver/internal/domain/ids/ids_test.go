package ids

import (
	"testing"

	"github.com/google/uuid"
)

func TestUserIDParseAndUUIDRoundTrip(t *testing.T) {
	want := uuid.New()
	got, err := ParseUserID(want.String())
	if err != nil {
		t.Fatalf("ParseUserID failed: %v", err)
	}
	if got.UUID() != want {
		t.Fatalf("UUID mismatch: got %s, want %s", got.UUID(), want)
	}
	if got.String() != want.String() {
		t.Fatalf("string mismatch: got %s, want %s", got, want)
	}
}

func TestParseUserIDRejectsMalformedUUID(t *testing.T) {
	if _, err := ParseUserID("not-a-uuid"); err == nil {
		t.Fatal("ParseUserID accepted malformed UUID")
	}
}

func TestUserIDZeroValueKeepsUUIDSemantics(t *testing.T) {
	var id UserID
	if id.UUID() != uuid.Nil {
		t.Fatalf("zero UserID UUID mismatch: got %s", id.UUID())
	}
	if id.String() != uuid.Nil.String() {
		t.Fatalf("zero UserID string mismatch: got %s", id)
	}
}
