package auth

import (
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
)

const testSecret = "test-jwt-secret-with-minimum-entropy-32-chars!!"

func TestGenerateAndVerifyToken(t *testing.T) {
	userID := uuid.New()
	wsID := uuid.New()

	tokenStr, err := GenerateTokenWithWorkspace(userID, &wsID, testSecret, 15*time.Minute)
	if err != nil {
		t.Fatalf("GenerateToken failed: %v", err)
	}

	claims, err := VerifyTokenClaims(tokenStr, testSecret)
	if err != nil {
		t.Fatalf("VerifyTokenClaims failed: %v", err)
	}

	if claims.Subject != userID.String() {
		t.Errorf("got subject %s, want %s", claims.Subject, userID.String())
	}
	if claims.WorkspaceID == nil || *claims.WorkspaceID != wsID {
		t.Errorf("got workspace %v, want %s", claims.WorkspaceID, wsID)
	}

	parsedID, err := VerifyToken(tokenStr, testSecret)
	if err != nil {
		t.Fatalf("VerifyToken failed: %v", err)
	}
	if parsedID != userID {
		t.Errorf("got parsed ID %s, want %s", parsedID, userID)
	}
}

func TestExpiredTokenRejection(t *testing.T) {
	userID := uuid.New()
	now := time.Now().UTC()
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   userID.String(),
			ExpiresAt: jwt.NewNumericDate(now.Add(-time.Minute)),
			IssuedAt:  jwt.NewNumericDate(now.Add(-2 * time.Minute)),
		},
	})
	tokenStr, err := token.SignedString([]byte(testSecret))
	if err != nil {
		t.Fatalf("failed to create expired token: %v", err)
	}

	_, err = VerifyTokenClaims(tokenStr, testSecret)
	if err == nil {
		t.Fatal("expected expired token error, got nil")
	}
	if !errors.Is(err, ErrExpiredToken) {
		t.Errorf("expected ErrExpiredToken, got: %v", err)
	}
}

func TestInvalidSignatureRejection(t *testing.T) {
	userID := uuid.New()
	tokenStr, err := GenerateToken(userID, testSecret, 15*time.Minute)
	if err != nil {
		t.Fatalf("failed to generate token: %v", err)
	}

	// 1. Verify with different secret
	_, err = VerifyTokenClaims(tokenStr, "wrong-secret-key-that-does-not-match")
	if err == nil {
		t.Fatal("expected error with wrong secret, got nil")
	}
	if !errors.Is(err, ErrInvalidToken) {
		t.Errorf("expected ErrInvalidToken, got: %v", err)
	}

	// 2. Tampered token payload
	parts := strings.Split(tokenStr, ".")
	if len(parts) == 3 {
		tampered := parts[0] + ".e30." + parts[2]
		_, err = VerifyTokenClaims(tampered, testSecret)
		if !errors.Is(err, ErrInvalidToken) {
			t.Errorf("expected ErrInvalidToken for tampered token, got: %v", err)
		}
	}
}

func TestInvalidSubjectRejection(t *testing.T) {
	claims := Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "not-a-valid-uuid",
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(10 * time.Minute)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
		},
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenStr, _ := token.SignedString([]byte(testSecret))

	_, err := VerifyToken(tokenStr, testSecret)
	if !errors.Is(err, ErrInvalidSubject) {
		t.Errorf("expected ErrInvalidSubject, got: %v", err)
	}
}

func TestRefreshTokenMetadataAndKeyFormatting(t *testing.T) {
	refreshToken := NewRefreshToken()
	if _, err := uuid.Parse(refreshToken); err != nil {
		t.Errorf("NewRefreshToken should produce valid UUID v4: %v", err)
	}

	if key := RotatedTokenKey(refreshToken); key != "jwt:rotated:"+refreshToken {
		t.Errorf("RotatedTokenKey mismatch: %s", key)
	}

	dummyJWT := "header.payload.signaturesig123"
	if sig := ExtractTokenSignature(dummyJWT); sig != "signaturesig123" {
		t.Errorf("ExtractTokenSignature mismatch: %s", sig)
	}

	if key := RevokedSignatureKey(dummyJWT); key != "jwt:revoked:signaturesig123" {
		t.Errorf("RevokedSignatureKey mismatch: %s", key)
	}

	userID := uuid.New()
	if key := UserRevokedAtKey(userID); key != "user:revoked_at:"+userID.String() {
		t.Errorf("UserRevokedAtKey mismatch: %s", key)
	}

	// Revocation timestamp comparison: if iat < revokedAt -> revoked
	iat := time.Now().Add(-10 * time.Minute)
	revokedAt := time.Now().Add(-5 * time.Minute)
	if !IsTokenRevoked(iat, revokedAt) {
		t.Error("expected token to be marked revoked when iat < revokedAt")
	}

	iatAfter := time.Now().Add(1 * time.Minute)
	if IsTokenRevoked(iatAfter, revokedAt) {
		t.Error("token issued after revocation should not be revoked")
	}
}
