package service

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

var (
	ErrInvalidCredentials = errors.New("invalid email or password")
	ErrAccountLocked      = errors.New("account is temporarily locked")
	ErrSessionNotFound    = errors.New("session not found")
	ErrSessionRevoked     = errors.New("session is revoked")
	ErrRefreshReuse       = errors.New("refresh token reuse detected")
)

type UserRecord struct {
	ID           uuid.UUID
	Email        string
	PasswordHash string
	FullName     string
}

type SessionRecord struct {
	ID        uuid.UUID
	UserID    uuid.UUID
	TokenHash string
	ExpiresAt time.Time
	RevokedAt *time.Time
}

type AuthRepository interface {
	FindUserByEmail(context.Context, string) (UserRecord, error)
	FindUserByID(context.Context, uuid.UUID) (UserRecord, error)
	CreateSession(context.Context, SessionRecord) error
	FindSessionByTokenHash(context.Context, string) (SessionRecord, error)
	RevokeSession(context.Context, uuid.UUID, time.Time) error
	RevokeAllSessions(context.Context, uuid.UUID, time.Time) error
}

type LoginLimiter interface {
	IsLocked(context.Context, string) (bool, error)
	RecordFailure(context.Context, string) error
	ClearFailures(context.Context, string) error
}

type AuthService struct {
	Repository AuthRepository
	Limiter    LoginLimiter
	JWTSecret  string
	AccessTTL  time.Duration
	RefreshTTL time.Duration
}

type AuthResponse struct {
	AccessToken  string
	RefreshToken string
	User         UserRecord
}

func (s AuthService) Login(ctx context.Context, email, password string) (AuthResponse, error) {
	email = strings.ToLower(strings.TrimSpace(email))
	if email == "" || password == "" || s.Repository == nil {
		return AuthResponse{}, ErrInvalidCredentials
	}
	if s.Limiter != nil {
		locked, err := s.Limiter.IsLocked(ctx, email)
		if err != nil {
			return AuthResponse{}, fmt.Errorf("check login lockout: %w", err)
		}
		if locked {
			return AuthResponse{}, ErrAccountLocked
		}
	}
	user, err := s.Repository.FindUserByEmail(ctx, email)
	if err != nil {
		return AuthResponse{}, fmt.Errorf("find user: %w", err)
	}
	valid, _ := auth.VerifyPassword(password, user.PasswordHash)
	if !valid {
		if s.Limiter != nil {
			_ = s.Limiter.RecordFailure(ctx, email)
		}
		return AuthResponse{}, ErrInvalidCredentials
	}
	if s.Limiter != nil {
		_ = s.Limiter.ClearFailures(ctx, email)
	}
	return s.issueSession(ctx, user)
}

func (s AuthService) Refresh(ctx context.Context, refreshToken string) (AuthResponse, error) {
	if refreshToken == "" || s.Repository == nil {
		return AuthResponse{}, ErrSessionNotFound
	}
	hash := hashRefreshToken(refreshToken)
	session, err := s.Repository.FindSessionByTokenHash(ctx, hash)
	if err != nil {
		return AuthResponse{}, fmt.Errorf("find session: %w", err)
	}
	if session.RevokedAt != nil {
		_ = s.Repository.RevokeAllSessions(ctx, session.UserID, time.Now().UTC())
		return AuthResponse{}, ErrRefreshReuse
	}
	if !time.Now().UTC().Before(session.ExpiresAt) {
		return AuthResponse{}, ErrSessionNotFound
	}
	if err := s.Repository.RevokeSession(ctx, session.ID, time.Now().UTC()); err != nil {
		return AuthResponse{}, fmt.Errorf("rotate session: %w", err)
	}
	user, err := s.Repository.FindUserByID(ctx, session.UserID)
	if err != nil || user.ID != session.UserID {
		return AuthResponse{}, ErrSessionNotFound
	}
	return s.issueSession(ctx, user)
}

func (s AuthService) issueSession(ctx context.Context, user UserRecord) (AuthResponse, error) {
	accessTTL := s.AccessTTL
	if accessTTL <= 0 {
		accessTTL = auth.AccessTokenDuration
	}
	refreshTTL := s.RefreshTTL
	if refreshTTL <= 0 {
		refreshTTL = auth.RefreshTokenDuration
	}
	accessToken, err := auth.GenerateToken(user.ID, s.JWTSecret, accessTTL)
	if err != nil {
		return AuthResponse{}, fmt.Errorf("generate access token: %w", err)
	}
	refreshToken, err := randomRefreshToken()
	if err != nil {
		return AuthResponse{}, fmt.Errorf("generate refresh token: %w", err)
	}
	now := time.Now().UTC()
	if err := s.Repository.CreateSession(ctx, SessionRecord{ID: uuid.New(), UserID: user.ID, TokenHash: hashRefreshToken(refreshToken), ExpiresAt: now.Add(refreshTTL)}); err != nil {
		return AuthResponse{}, fmt.Errorf("create session: %w", err)
	}
	return AuthResponse{AccessToken: accessToken, RefreshToken: refreshToken, User: user}, nil
}

func randomRefreshToken() (string, error) {
	buffer := make([]byte, 32)
	if _, err := rand.Read(buffer); err != nil {
		return "", err
	}
	return hex.EncodeToString(buffer), nil
}
func hashRefreshToken(value string) string { return auth.HashRefreshToken(value) }
