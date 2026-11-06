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
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
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

func (r *authTestRepository) CreateUserWithAdminCheck(_ context.Context, u service.UserRecord) (bool, error) {
	r.user = u
	return true, nil
}
func (r *authTestRepository) UpdatePasswordHash(_ context.Context, userID uuid.UUID, hash string) error {
	if r.user.ID == userID {
		r.user.PasswordHash = hash
	}
	return nil
}
func (r *authTestRepository) GetSystemRolesForUser(_ context.Context, uid uuid.UUID) ([]string, error) {
	if r.user.ID == uid {
		return []string{}, nil
	}
	return nil, service.ErrSessionNotFound
}
func (r *authTestRepository) UpdateUserProfile(_ context.Context, uid uuid.UUID, req service.UpdateUserProfileRequest) (service.UserRecord, error) {
	if r.user.ID != uid {
		return service.UserRecord{}, service.ErrSessionNotFound
	}
	if req.FullName != nil {
		r.user.FullName = req.FullName
	}
	if req.AvatarURL != nil {
		r.user.AvatarURL = req.AvatarURL
	}
	if req.Bio != nil {
		r.user.Bio = req.Bio
	}
	if req.PreferredLanguage != nil {
		r.user.PreferredLanguage = req.PreferredLanguage
	}
	return r.user, nil
}
func (r *authTestRepository) ListWorkspaceUsers(_ context.Context, _ uuid.UUID) ([]service.UserRecord, error) {
	return []service.UserRecord{r.user}, nil
}

func newAuthTestHandler(t *testing.T) (AuthHandler, *authTestRepository) {
	t.Helper()
	hash, err := auth.HashPassword("correct-password")
	if err != nil {
		t.Fatalf("hash password: %v", err)
	}
	member := "Member"
	repo := &authTestRepository{
		user:     service.UserRecord{ID: uuid.New(), Email: "member@example.com", PasswordHash: hash, FullName: &member},
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
	for _, cookie := range rec.Result().Cookies() {
		if cookie.Secure || cookie.SameSite != http.SameSiteLaxMode {
			t.Fatalf("expected HTTP auth cookie to be non-secure and Lax, got %s secure=%v sameSite=%v", cookie.Name, cookie.Secure, cookie.SameSite)
		}
	}
}

func TestAuthCookiePolicyUsesRequestTransport(t *testing.T) {
	httpRequest := httptest.NewRequest(http.MethodPost, "http://localhost/api/v1/auth/login", nil)
	secure, sameSite := authCookiePolicy(httpRequest)
	if secure || sameSite != http.SameSiteLaxMode {
		t.Fatalf("expected HTTP cookies to be non-secure and Lax, got secure=%v sameSite=%v", secure, sameSite)
	}

	forwardedRequest := httptest.NewRequest(http.MethodPost, "http://localhost/api/v1/auth/login", nil)
	forwardedRequest.Header.Set("X-Forwarded-Proto", "https")
	secure, sameSite = authCookiePolicy(forwardedRequest)
	if !secure || sameSite != http.SameSiteNoneMode {
		t.Fatalf("expected forwarded HTTPS cookies to be secure and SameSite=None, got secure=%v sameSite=%v", secure, sameSite)
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

type recordingTokenStore struct {
	tokens map[string]uuid.UUID
}

func (s *recordingTokenStore) CreateResetToken(_ context.Context, token string, uid uuid.UUID, _ time.Duration) error {
	s.tokens[token] = uid
	return nil
}
func (s *recordingTokenStore) ConsumeResetToken(_ context.Context, token string) (uuid.UUID, error) {
	uid, ok := s.tokens[token]
	if !ok {
		return uuid.Nil, service.ErrResetTokenInvalid
	}
	delete(s.tokens, token)
	return uid, nil
}
func (s *recordingTokenStore) SetUserRevocation(_ context.Context, _ uuid.UUID, _ time.Duration) error {
	return nil
}

func TestAuthHandler_SelfService(t *testing.T) {
	store := &recordingTokenStore{tokens: make(map[string]uuid.UUID)}
	repo := &authTestRepository{sessions: make(map[string]service.SessionRecord)}
	h := AuthHandler{Service: service.AuthService{Repository: repo, TokenStore: store, JWTSecret: "test-secret-at-least-32-chars-long-for-jwt!!"}}

	// 1. Register validates weakly
	req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/register", bytes.NewBufferString(`{"email":"bad","password":"short","full_name":"X"}`))
	rec := httptest.NewRecorder()
	h.Register(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid register, got %d", rec.Code)
	}

	// 2. Register succeeds
	req = httptest.NewRequest(http.MethodPost, "/api/v1/auth/register", bytes.NewBufferString(`{"email":"new@example.com","password":"validPass1!","full_name":"New User"}`))
	rec = httptest.NewRecorder()
	h.Register(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for register, got %d: %s", rec.Code, rec.Body.String())
	}

	// 3. Forgot password always 200
	req = httptest.NewRequest(http.MethodPost, "/api/v1/auth/forgot-password", bytes.NewBufferString(`{"email":"unknown@example.com"}`))
	rec = httptest.NewRecorder()
	h.ForgotPassword(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for forgot password, got %d", rec.Code)
	}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/auth/forgot-password", bytes.NewBufferString(`{"email":"new@example.com"}`))
	rec = httptest.NewRecorder()
	h.ForgotPassword(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for known email forgot password, got %d", rec.Code)
	}

	// 4. Reset password
	var token string
	for k := range store.tokens {
		token = k
	}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/auth/reset-password", bytes.NewBufferString(`{"token":"`+token+`","new_password":"newSecret2!"}`))
	rec = httptest.NewRecorder()
	h.ResetPassword(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for reset password, got %d: %s", rec.Code, rec.Body.String())
	}

	// 5. Change password requires user context
	req = httptest.NewRequest(http.MethodPost, "/api/v1/auth/change-password", bytes.NewBufferString(`{"old_password":"newSecret2!","new_password":"otherSecret3!"}`))
	rec = httptest.NewRecorder()
	h.ChangePassword(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 without user context, got %d", rec.Code)
	}
}

type profileAuthorizer struct {
	member bool
}

func (a profileAuthorizer) IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error) {
	return a.member, nil
}
func (profileAuthorizer) EffectivePermissions(context.Context, uuid.UUID, uuid.UUID) ([]string, []string, error) {
	return []string{"employees:read"}, []string{uuid.New().String()}, nil
}

func TestAuthHandler_ProfileUsersAndPermissions(t *testing.T) {
	h, repo := newAuthTestHandler(t)
	h.Authorizer = profileAuthorizer{member: true}
	userID := repo.user.ID
	workspaceID := uuid.New()

	req := httptest.NewRequest(http.MethodGet, "/api/v1/me", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.Me(rec, req)
	if rec.Code != http.StatusOK || !bytes.Contains(rec.Body.Bytes(), []byte(`"system_roles":[]`)) {
		t.Fatalf("expected full profile response, got %d: %s", rec.Code, rec.Body.String())
	}

	req = httptest.NewRequest(http.MethodPatch, "/api/v1/me", bytes.NewBufferString(`{"preferred_language":"vi"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.UpdateMe(rec, req)
	if rec.Code != http.StatusOK || !bytes.Contains(rec.Body.Bytes(), []byte(`"preferred_language":"vi"`)) {
		t.Fatalf("expected updated profile response, got %d: %s", rec.Code, rec.Body.String())
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/users", nil)
	ctx := httpinfra.WithUserIDForTest(req.Context(), userID)
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, workspaceID)
	req = req.WithContext(ctx)
	rec = httptest.NewRecorder()
	h.WorkspaceUsers(rec, req)
	if rec.Code != http.StatusOK || !bytes.HasPrefix(rec.Body.Bytes(), []byte("[")) {
		t.Fatalf("expected workspace user array, got %d: %s", rec.Code, rec.Body.String())
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/me/permissions", nil)
	req = req.WithContext(ctx)
	rec = httptest.NewRecorder()
	h.MyPermissions(rec, req)
	if rec.Code != http.StatusOK || !bytes.Contains(rec.Body.Bytes(), []byte(`"employees:read"`)) {
		t.Fatalf("expected permissions response, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAuthHandler_WorkspaceRoutesRequireMembershipContext(t *testing.T) {
	h, repo := newAuthTestHandler(t)
	h.Authorizer = profileAuthorizer{member: false}
	ctx := httpinfra.WithUserIDForTest(context.Background(), repo.user.ID)
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, uuid.New())
	req := httptest.NewRequest(http.MethodGet, "/api/v1/users", nil).WithContext(ctx)
	rec := httptest.NewRecorder()
	h.WorkspaceUsers(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for non-member, got %d", rec.Code)
	}
}
