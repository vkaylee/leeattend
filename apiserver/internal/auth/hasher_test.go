package auth

import (
	"testing"
)

func TestHashPassword_FormatAndVerification(t *testing.T) {
	password := "secure-test-pass-123!"
	hash, err := HashPassword(password)
	if err != nil {
		t.Fatalf("HashPassword failed: %v", err)
	}

	valid, needsRehash := VerifyPassword(password, hash)
	if !valid || needsRehash {
		t.Fatalf("expected valid=true needsRehash=false, got valid=%v needsRehash=%v", valid, needsRehash)
	}

	valid, _ = VerifyPassword("wrong-pass", hash)
	if valid {
		t.Fatal("expected invalid for wrong password")
	}

	valid, _ = VerifyPassword(password, "invalid-hash")
	if valid {
		t.Fatal("expected invalid for bad hash format")
	}
}
