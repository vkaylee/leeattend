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

func (m *memoryAuthRepo) CreateUserWithAdminCheck(_ context.Context, u UserRecord) (bool, error) {
	if m.user.Email == u.Email {
		return false, ErrEmailExists
	}
	m.user = u
	return true, nil
}

func (m *memoryAuthRepo) UpdatePasswordHash(_ context.Context, uid uuid.UUID, hash string) error {
	if m.user.ID == uid {
		m.user.PasswordHash = hash
		return nil
	}
	return ErrSessionNotFound
}

type memoryTokenStore struct {
	tokens      map[string]uuid.UUID
	revocations map[uuid.UUID]time.Duration
}

func (m *memoryTokenStore) CreateResetToken(_ context.Context, token string, uid uuid.UUID, _ time.Duration) error {
	m.tokens[token] = uid
	return nil
}

func (m *memoryTokenStore) ConsumeResetToken(_ context.Context, token string) (uuid.UUID, error) {
	uid, ok := m.tokens[token]
	if !ok {
		return uuid.Nil, ErrResetTokenInvalid
	}
	delete(m.tokens, token)
	return uid, nil
}

func (m *memoryTokenStore) SetUserRevocation(_ context.Context, uid uuid.UUID, ttl time.Duration) error {
	m.revocations[uid] = ttl
	return nil
}

func (m *memoryAuthRepo) UpdateUserProfile(_ context.Context, uid uuid.UUID, req UpdateUserProfileRequest) (UserRecord, error) {
	if m.user.ID != uid {
		return UserRecord{}, ErrSessionNotFound
	}
	if req.FullName != nil {
		m.user.FullName = req.FullName
	}
	if req.AvatarURL != nil {
		m.user.AvatarURL = req.AvatarURL
	}
	if req.Bio != nil {
		m.user.Bio = req.Bio
	}
	if req.PreferredLanguage != nil {
		m.user.PreferredLanguage = req.PreferredLanguage
	}
	return m.user, nil
}

func (m *memoryAuthRepo) ListOrganizationUsers(_ context.Context) ([]UserRecord, error) {
	return []UserRecord{m.user}, nil
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

func TestAuthService_RegisterAndSelfService(t *testing.T) {
	repo := &memoryAuthRepo{
		sessions: make(map[string]SessionRecord),
	}
	tokenStore := &memoryTokenStore{
		tokens:      make(map[string]uuid.UUID),
		revocations: make(map[uuid.UUID]time.Duration),
	}
	svc := AuthService{
		Repository: repo,
		TokenStore: tokenStore,
		JWTSecret:  "jwt-secret-with-sufficient-length-32-bytes!!",
	}

	// 1. Validation failures
	if _, err := svc.Register(context.Background(), "invalid-email", "validPass1!", "User"); err != ErrInvalidEmail {
		t.Fatalf("expected ErrInvalidEmail, got %v", err)
	}
	if _, err := svc.Register(context.Background(), "user@example.com", "short", "User"); err != ErrPasswordTooShort {
		t.Fatalf("expected ErrPasswordTooShort, got %v", err)
	}
	if _, err := svc.Register(context.Background(), "user@example.com", "nodigitsorpunct", "User"); err != ErrPasswordComplexity {
		t.Fatalf("expected ErrPasswordComplexity, got %v", err)
	}

	// 2. Successful registration
	resp, err := svc.Register(context.Background(), "user@example.com", "validPass1!", "New User")
	if err != nil {
		t.Fatalf("unexpected register error: %v", err)
	}
	if resp.User.Email != "user@example.com" || resp.AccessToken == "" {
		t.Fatalf("unexpected register response: %+v", resp)
	}

	// 3. Duplicate email
	if _, err := svc.Register(context.Background(), "user@example.com", "validPass1!", "Duplicate"); err != ErrEmailExists {
		t.Fatalf("expected ErrEmailExists, got %v", err)
	}

	// 4. Change password
	if err := svc.ChangePassword(context.Background(), resp.User.ID, "wrongPass1!", "newSecretPass2!"); err != ErrCurrentPassword {
		t.Fatalf("expected ErrCurrentPassword, got %v", err)
	}
	if err := svc.ChangePassword(context.Background(), resp.User.ID, "validPass1!", "newSecretPass2!"); err != nil {
		t.Fatalf("unexpected change password error: %v", err)
	}

	// 5. Forgot password creates reset token
	if err := svc.ForgotPassword(context.Background(), "user@example.com"); err != nil {
		t.Fatalf("unexpected forgot password error: %v", err)
	}
	if len(tokenStore.tokens) != 1 {
		t.Fatalf("expected 1 reset token, got %d", len(tokenStore.tokens))
	}
	var resetTok string
	for k := range tokenStore.tokens {
		resetTok = k
	}

	// 6. Reset password with token
	if err := svc.ResetPassword(context.Background(), resetTok, "anotherSecret3!"); err != nil {
		t.Fatalf("unexpected reset password error: %v", err)
	}
	// Token should be consumed
	if len(tokenStore.tokens) != 0 {
		t.Fatalf("expected reset token to be consumed, got %d", len(tokenStore.tokens))
	}
	// Old token cannot be reused
	if err := svc.ResetPassword(context.Background(), resetTok, "anotherSecret3!"); err != ErrResetTokenInvalid {
		t.Fatalf("expected ErrResetTokenInvalid for consumed token, got %v", err)
	}
}
