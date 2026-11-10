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
	ErrEmailExists        = errors.New("email already exists")
	ErrFullNameRequired   = errors.New("full name is required")
	ErrInvalidEmail       = errors.New("invalid email format")
	ErrPasswordTooShort   = errors.New("password too short")
	ErrPasswordTooLong    = errors.New("password too long")
	ErrPasswordComplexity = errors.New("password complexity")
	ErrResetTokenInvalid  = errors.New("reset token invalid")
	ErrCurrentPassword    = errors.New("current password is incorrect")
)

type UserRecord struct {
	ID                uuid.UUID
	Email             string
	PasswordHash      string
	FullName          *string
	AvatarURL         *string
	Bio               *string
	PreferredLanguage *string
	CreatedAt         time.Time
	AvatarFileID      *uuid.UUID
}

type UpdateUserProfileRequest struct {
	FullName          *string `json:"full_name"`
	AvatarURL         *string `json:"avatar_url"`
	Bio               *string `json:"bio"`
	PreferredLanguage *string `json:"preferred_language"`
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
	UpdateUserProfile(context.Context, uuid.UUID, UpdateUserProfileRequest) (UserRecord, error)
	ListOrganizationUsers(context.Context) ([]UserRecord, error)
	CreateUserWithAdminCheck(context.Context, UserRecord) (bool, error)
	UpdatePasswordHash(context.Context, uuid.UUID, string) error
	CreateSession(context.Context, SessionRecord) error
	FindSessionByTokenHash(context.Context, string) (SessionRecord, error)
	RevokeSession(context.Context, uuid.UUID, time.Time) error
	RevokeAllSessions(context.Context, uuid.UUID, time.Time) error
}

type ResetTokenStore interface {
	CreateResetToken(context.Context, string, uuid.UUID, time.Duration) error
	ConsumeResetToken(context.Context, string) (uuid.UUID, error)
	SetUserRevocation(context.Context, uuid.UUID, time.Duration) error
}

type LoginLimiter interface {
	IsLocked(context.Context, string) (bool, error)
	RecordFailure(context.Context, string) error
	ClearFailures(context.Context, string) error
}

type AuthService struct {
	Repository AuthRepository
	Limiter    LoginLimiter
	TokenStore ResetTokenStore
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

func (s AuthService) Register(ctx context.Context, email, password, fullName string) (AuthResponse, error) {
	if strings.TrimSpace(fullName) == "" {
		return AuthResponse{}, ErrFullNameRequired
	}
	if err := ValidateEmail(email); err != nil {
		return AuthResponse{}, err
	}
	if err := ValidatePassword(password); err != nil {
		return AuthResponse{}, err
	}
	if s.Repository == nil {
		return AuthResponse{}, errors.New("auth service: nil repository")
	}
	existing, err := s.Repository.FindUserByEmail(ctx, strings.ToLower(strings.TrimSpace(email)))
	if err == nil && existing.ID != uuid.Nil {
		return AuthResponse{}, ErrEmailExists
	}
	hash, err := auth.HashPassword(password)
	if err != nil {
		return AuthResponse{}, fmt.Errorf("hash password: %w", err)
	}
	fn := strings.TrimSpace(fullName)
	user := UserRecord{
		ID:           uuid.New(),
		Email:        strings.ToLower(strings.TrimSpace(email)),
		PasswordHash: hash,
		FullName:     &fn,
		CreatedAt:    time.Now().UTC(),
	}
	if _, err := s.Repository.CreateUserWithAdminCheck(ctx, user); err != nil {
		if errors.Is(err, ErrEmailExists) {
			return AuthResponse{}, ErrEmailExists
		}
		return AuthResponse{}, fmt.Errorf("create user: %w", err)
	}
	return s.issueSession(ctx, user)
}

func (s AuthService) ForgotPassword(ctx context.Context, email string) error {
	trimmed := strings.ToLower(strings.TrimSpace(email))
	if !strings.Contains(trimmed, "@") || trimmed == "" || s.Repository == nil {
		return nil
	}
	user, err := s.Repository.FindUserByEmail(ctx, trimmed)
	if err != nil || user.ID == uuid.Nil {
		return nil
	}
	if s.TokenStore != nil {
		token := uuid.New().String()
		_ = s.TokenStore.CreateResetToken(ctx, token, user.ID, 15*time.Minute)
	}
	return nil
}

func (s AuthService) ResetPassword(ctx context.Context, token, newPassword string) error {
	if strings.TrimSpace(token) == "" || s.TokenStore == nil || s.Repository == nil {
		return ErrResetTokenInvalid
	}
	if err := ValidatePassword(newPassword); err != nil {
		return err
	}
	userID, err := s.TokenStore.ConsumeResetToken(ctx, strings.TrimSpace(token))
	if err != nil || userID == uuid.Nil {
		return ErrResetTokenInvalid
	}
	newHash, err := auth.HashPassword(newPassword)
	if err != nil {
		return fmt.Errorf("hash new password: %w", err)
	}
	if err := s.Repository.UpdatePasswordHash(ctx, userID, newHash); err != nil {
		return fmt.Errorf("update password hash: %w", err)
	}
	_ = s.Repository.RevokeAllSessions(ctx, userID, time.Now().UTC())
	_ = s.TokenStore.SetUserRevocation(ctx, userID, 15*time.Minute)
	return nil
}

func (s AuthService) ChangePassword(ctx context.Context, userID uuid.UUID, oldPassword, newPassword string) error {
	if s.Repository == nil {
		return errors.New("auth service: nil repository")
	}
	user, err := s.Repository.FindUserByID(ctx, userID)
	if err != nil || user.ID != userID {
		return ErrSessionNotFound
	}
	valid, _ := auth.VerifyPassword(oldPassword, user.PasswordHash)
	if !valid {
		return ErrCurrentPassword
	}
	if err := ValidatePassword(newPassword); err != nil {
		return err
	}
	newHash, err := auth.HashPassword(newPassword)
	if err != nil {
		return fmt.Errorf("hash new password: %w", err)
	}
	if err := s.Repository.UpdatePasswordHash(ctx, userID, newHash); err != nil {
		return fmt.Errorf("update password hash: %w", err)
	}
	_ = s.Repository.RevokeAllSessions(ctx, userID, time.Now().UTC())
	if s.TokenStore != nil {
		_ = s.TokenStore.SetUserRevocation(ctx, userID, 15*time.Minute)
	}
	return nil
}

func ValidatePassword(password string) error {
	if len(password) < 8 {
		return ErrPasswordTooShort
	}
	if len(password) > 128 {
		return ErrPasswordTooLong
	}
	hasDigitOrPunct := false
	for _, r := range password {
		if (r >= '0' && r <= '9') || (r >= 33 && r <= 47) || (r >= 58 && r <= 64) || (r >= 91 && r <= 96) || (r >= 123 && r <= 126) {
			hasDigitOrPunct = true
			break
		}
	}
	if !hasDigitOrPunct {
		return ErrPasswordComplexity
	}
	return nil
}

func ValidateEmail(email string) error {
	trimmed := strings.TrimSpace(email)
	if len(trimmed) == 0 || len(trimmed) > 254 {
		return ErrInvalidEmail
	}
	parts := strings.Split(trimmed, "@")
	if len(parts) != 2 {
		return ErrInvalidEmail
	}
	local, domain := parts[0], parts[1]
	if len(local) == 0 || len(local) > 64 {
		return ErrInvalidEmail
	}
	if len(domain) == 0 || !strings.Contains(domain, ".") || strings.HasPrefix(domain, ".") || strings.HasSuffix(domain, ".") || strings.Contains(domain, "..") {
		return ErrInvalidEmail
	}
	return nil
}
func (s AuthService) Profile(ctx context.Context, userID uuid.UUID) (UserRecord, error) {
	if s.Repository == nil {
		return UserRecord{}, errors.New("auth service: nil repository")
	}
	user, err := s.Repository.FindUserByID(ctx, userID)
	if err != nil {
		return UserRecord{}, err
	}
	if user.ID != userID {
		return UserRecord{}, ErrSessionNotFound
	}
	return user, nil
}

func (s AuthService) UpdateProfile(ctx context.Context, userID uuid.UUID, req UpdateUserProfileRequest) (UserRecord, error) {
	if s.Repository == nil {
		return UserRecord{}, errors.New("auth service: nil repository")
	}
	return s.Repository.UpdateUserProfile(ctx, userID, req)
}

func (s AuthService) OrganizationUsers(ctx context.Context) ([]UserRecord, error) {
	if s.Repository == nil {
		return nil, errors.New("auth service: nil repository")
	}
	return s.Repository.ListOrganizationUsers(ctx)
}
