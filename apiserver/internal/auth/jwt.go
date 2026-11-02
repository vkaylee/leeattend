package auth

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

// HashRefreshToken returns a one-way SHA-256 digest suitable for session storage.
func HashRefreshToken(token string) string {
	digest := sha256.Sum256([]byte(token))
	return hex.EncodeToString(digest[:])
}

// Inviolable Token Lifetimes per auth-permission-matrix.md Section 6.
const (
	AccessTokenDuration  = 15 * time.Minute
	RefreshTokenDuration = 7 * 24 * time.Hour
)

var (
	ErrInvalidToken   = errors.New("invalid token")
	ErrExpiredToken   = errors.New("token has expired")
	ErrInvalidSubject = errors.New("invalid token subject")
	ErrTokenRevoked   = errors.New("token has been revoked")
)

// Claims matches Rust Claims (sub, workspace_id, exp, iat).
// ponytail: HMAC-SHA256 symmetric secret matches Rust apiserver. Upgrade to RS256/EdDSA asymmetric keys if public key verification needed without sharing secret.
type Claims struct {
	jwt.RegisteredClaims
	WorkspaceID *uuid.UUID `json:"workspace_id,omitempty"`
}

// GenerateToken creates an HMAC-SHA256 access token for the given user ID.
func GenerateToken(userID uuid.UUID, secret string, duration time.Duration) (string, error) {
	return GenerateTokenWithWorkspace(userID, nil, secret, duration)
}

// GenerateTokenWithWorkspace creates an HMAC-SHA256 access token with optional workspace context.
func GenerateTokenWithWorkspace(userID uuid.UUID, workspaceID *uuid.UUID, secret string, duration time.Duration) (string, error) {
	if duration <= 0 {
		duration = AccessTokenDuration
	}
	now := time.Now().UTC()
	return GenerateTokenWithTimes(userID, workspaceID, secret, now, now.Add(duration))
}

// GenerateTokenWithTimes creates an HMAC-SHA256 access token with explicit issued-at and expiration timestamps.
func GenerateTokenWithTimes(userID uuid.UUID, workspaceID *uuid.UUID, secret string, issuedAt, expiresAt time.Time) (string, error) {
	claims := Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   userID.String(),
			ExpiresAt: jwt.NewNumericDate(expiresAt),
			IssuedAt:  jwt.NewNumericDate(issuedAt),
		},
		WorkspaceID: workspaceID,
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	signed, err := token.SignedString([]byte(secret))
	if err != nil {
		return "", apperror.UnauthorizedArgs(apperror.KeyInvalidToken, map[string]string{
			"details": fmt.Sprintf("Token generation failed: %v", err),
		})
	}
	return signed, nil
}

// VerifyTokenClaims parses and validates an HMAC-SHA256 JWT, checking signature and expiration.
func VerifyTokenClaims(tokenStr string, secret string) (*Claims, error) {
	token, err := jwt.ParseWithClaims(tokenStr, &Claims{}, func(t *jwt.Token) (any, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		return []byte(secret), nil
	}, jwt.WithValidMethods([]string{"HS256"}))

	if err != nil {
		if errors.Is(err, jwt.ErrTokenExpired) {
			return nil, ErrExpiredToken
		}
		return nil, ErrInvalidToken
	}

	claims, ok := token.Claims.(*Claims)
	if !ok || !token.Valid {
		return nil, ErrInvalidToken
	}

	return claims, nil
}

// VerifyToken validates the token and returns the parsed user UUID from the subject.
func VerifyToken(tokenStr string, secret string) (uuid.UUID, error) {
	claims, err := VerifyTokenClaims(tokenStr, secret)
	if err != nil {
		return uuid.Nil, err
	}
	userID, err := uuid.Parse(claims.Subject)
	if err != nil {
		return uuid.Nil, ErrInvalidSubject
	}
	return userID, nil
}

// ExtractTokenSignature returns the signature segment of the JWT for blocklist checking.
func ExtractTokenSignature(tokenStr string) string {
	parts := strings.Split(tokenStr, ".")
	if len(parts) == 3 {
		return parts[2]
	}
	return ""
}

// RefreshTokenMetadata holds refresh token lifecycle information per auth contract.
type RefreshTokenMetadata struct {
	ID        uuid.UUID  `json:"id"`
	UserID    uuid.UUID  `json:"user_id"`
	Token     string     `json:"token"`
	ExpiresAt time.Time  `json:"expires_at"`
	CreatedAt time.Time  `json:"created_at"`
	RevokedAt *time.Time `json:"revoked_at,omitempty"`
}

// NewRefreshToken generates a fresh UUID v4 refresh token string.
func NewRefreshToken() string {
	return uuid.NewString()
}

// RotatedTokenKey returns Redis key for refresh token reuse detection: jwt:rotated:{token}.
func RotatedTokenKey(refreshToken string) string {
	return "jwt:rotated:" + refreshToken
}

// RevokedSignatureKey returns Redis key for individual signature blocklist: jwt:revoked:{sig}.
func RevokedSignatureKey(tokenStr string) string {
	return "jwt:revoked:" + ExtractTokenSignature(tokenStr)
}

// UserRevokedAtKey returns Redis key for user timestamp revocation: user:revoked_at:{user_id}.
func UserRevokedAtKey(userID uuid.UUID) string {
	return "user:revoked_at:" + userID.String()
}

// IsTokenRevoked checks if token issued-at instant precedes user's global revocation timestamp.
func IsTokenRevoked(iat time.Time, revokedAt time.Time) bool {
	return iat.Before(revokedAt)
}
