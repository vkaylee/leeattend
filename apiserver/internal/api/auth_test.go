package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type authTestRepository struct {
	user     service.UserRecord
	sessions map[string]service.SessionRecord
}

func (r *authTestRepository) FindUserByEmail(_ context.Context, email string) (service.UserRecord, error) {
	if email != r.user.Email {
		return service.UserRecord{}, service.ErrInvalidCredentials
	}
	return r.user, nil
}
func (r *authTestRepository) FindUserByID(_ context.Context, id uuid.UUID) (service.UserRecord, error) {
	if id != r.user.ID {
		return service.UserRecord{}, service.ErrSessionNotFound
	}
	return r.user, nil
}
func (r *authTestRepository) CreateSession(_ context.Context, session service.SessionRecord) error {
	r.sessions[session.TokenHash] = session
	return nil
}
func (r *authTestRepository) FindSessionByTokenHash(_ context.Context, tokenHash string) (service.SessionRecord, error) {
	session, ok := r.sessions[tokenHash]
	if !ok {
		return service.SessionRecord{}, service.ErrSessionNotFound
	}
	return session, nil
}
func (r *authTestRepository) RevokeSession(_ context.Context, id uuid.UUID, revokedAt time.Time) error {
	for hash, session := range r.sessions {
		if session.ID == id {
			session.RevokedAt = &revokedAt
			r.sessions[hash] = session
		}
	}
	return nil
}
func (r *authTestRepository) RevokeAllSessions(_ context.Context, userID uuid.UUID, revokedAt time.Time) error {
	for hash, session := range r.sessions {
		if session.UserID == userID {
			session.RevokedAt = &revokedAt
			r.sessions[hash] = session
		}
	}
	return nil
}

func newAuthTestHandler(t *testing.T) (AuthHandler, *authTestRepository) {
	t.Helper()
	hash, err := auth.HashPassword("correct-password")
	if err != nil {
		t.Fatalf("hash password: %v", err)
	}
	repo := &authTestRepository{
		user:     service.UserRecord{ID: uuid.New(), Email: "member@example.com", PasswordHash: hash, FullName: "Member"},
		sessions: make(map[string]service.SessionRecord),
	}
	return AuthHandler{Service: service.AuthService{Repository: repo, JWTSecret: "test-secret-at-least-32-chars-long-for-jwt!!"}}, repo
}

func TestAuthHandler_LoginSuccessSetsCookies(t *testing.T) {
	h, _ := newAuthTestHandler(t)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewBufferString(`{"email":"MEMBER@example.com","password":"correct-password"}`))
	rec := httptest.NewRecorder()

	h.Login(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if len(rec.Result().Cookies()) != 2 {
		t.Fatalf("expected access and refresh cookies, got %d", len(rec.Result().Cookies()))
	}
}

func TestAuthHandler_RefreshRequiresToken(t *testing.T) {
	h := AuthHandler{}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/refresh", bytes.NewBufferString(`{}`))
	rec := httptest.NewRecorder()

	h.Refresh(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for empty refresh, got %d", rec.Code)
	}
}

func TestAuthHandler_LogoutRevokesSessionAndClearsCookies(t *testing.T) {
	h, repo := newAuthTestHandler(t)
	loginReq := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", bytes.NewBufferString(`{"email":"member@example.com","password":"correct-password"}`))
	loginRec := httptest.NewRecorder()
	h.Login(loginRec, loginReq)
	refreshToken := loginRec.Result().Cookies()[1].Value

	logoutReq := httptest.NewRequest(http.MethodPost, "/api/v1/auth/logout", bytes.NewBufferString(`{"refresh_token":"`+refreshToken+`"}`))
	logoutRec := httptest.NewRecorder()
	h.Logout(logoutRec, logoutReq)
	if logoutRec.Code != http.StatusOK {
		t.Fatalf("expected logout 200, got %d", logoutRec.Code)
	}
	session := repo.sessions[auth.HashRefreshToken(refreshToken)]
	if session.RevokedAt == nil {
		t.Fatal("expected logout to revoke refresh session")
	}
	if len(logoutRec.Result().Cookies()) != 2 || logoutRec.Result().Cookies()[0].MaxAge != -1 {
		t.Fatal("expected both auth cookies to be cleared")
	}
}
