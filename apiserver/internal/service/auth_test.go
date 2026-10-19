package service

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

type memoryAuthRepo struct {
	user     UserRecord
	sessions map[string]SessionRecord
}

func (m *memoryAuthRepo) FindUserByEmail(_ context.Context, email string) (UserRecord, error) {
	if email == "" || email == m.user.Email {
		return m.user, nil
	}
	return UserRecord{}, ErrInvalidCredentials
}

func (m *memoryAuthRepo) CreateSession(_ context.Context, s SessionRecord) error {
	m.sessions[s.TokenHash] = s
	return nil
}

func (m *memoryAuthRepo) FindUserByID(_ context.Context, id uuid.UUID) (UserRecord, error) {
	if id != m.user.ID {
		return UserRecord{}, ErrSessionNotFound
	}
	return m.user, nil
}
func (m *memoryAuthRepo) FindSessionByTokenHash(_ context.Context, hash string) (SessionRecord, error) {
	s, ok := m.sessions[hash]
	if !ok {
		return SessionRecord{}, ErrSessionNotFound
	}
	return s, nil
}

func (m *memoryAuthRepo) RevokeSession(_ context.Context, id uuid.UUID, at time.Time) error {
	for k, v := range m.sessions {
		if v.ID == id {
			v.RevokedAt = &at
			m.sessions[k] = v
		}
	}
	return nil
}

func (m *memoryAuthRepo) RevokeAllSessions(_ context.Context, uid uuid.UUID, at time.Time) error {
	for k, v := range m.sessions {
		if v.UserID == uid {
			v.RevokedAt = &at
			m.sessions[k] = v
		}
	}
	return nil
}

func TestAuthService_LoginAndRotateSession(t *testing.T) {
	pass := "my-secret-password-123!"
	h, err := auth.HashPassword(pass)
	if err != nil {
		t.Fatal(err)
	}

	repo := &memoryAuthRepo{
		user: UserRecord{
			ID:           uuid.New(),
			Email:        "user@example.com",
			PasswordHash: h,
		},
		sessions: make(map[string]SessionRecord),
	}

	svc := AuthService{
		Repository: repo,
		JWTSecret:  "jwt-secret-with-sufficient-length-32-bytes!!",
		AccessTTL:  15 * time.Minute,
		RefreshTTL: 7 * 24 * time.Hour,
	}

	res, err := svc.Login(context.Background(), "user@example.com", pass)
	if err != nil {
		t.Fatalf("login failed: %v", err)
	}
	if res.AccessToken == "" || res.RefreshToken == "" {
		t.Fatalf("expected tokens, got %+v", res)
	}

	// Rotate token
	refreshed, err := svc.Refresh(context.Background(), res.RefreshToken)
	if err != nil {
		t.Fatalf("refresh failed: %v", err)
	}
	if refreshed.AccessToken == "" || refreshed.RefreshToken == res.RefreshToken {
		t.Fatalf("expected new refresh token, got %+v", refreshed)
	}

	// Reuse detection
	_, err = svc.Refresh(context.Background(), res.RefreshToken)
	if err != ErrRefreshReuse {
		t.Fatalf("expected ErrRefreshReuse, got %v", err)
	}
}
