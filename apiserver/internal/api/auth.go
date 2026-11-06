package api

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type WorkspaceMemberAuthorizer interface {
	IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error)
	EffectivePermissions(context.Context, uuid.UUID, uuid.UUID) ([]string, []string, error)
}

type AuthHandler struct {
	Service      service.AuthService
	Authorizer   WorkspaceMemberAuthorizer
	CookieDomain string
}

type loginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}
type refreshRequest struct {
	RefreshToken string `json:"refresh_token"`
}
type userResponse struct {
	ID                string    `json:"id"`
	Email             string    `json:"email"`
	FullName          *string   `json:"full_name"`
	AvatarURL         *string   `json:"avatar_url"`
	Bio               *string   `json:"bio"`
	PreferredLanguage *string   `json:"preferred_language"`
	CreatedAt         time.Time `json:"created_at"`
	SystemRoles       []string  `json:"system_roles"`
	AvatarFileID      *string   `json:"avatar_file_id"`
}
type updateMeRequest struct {
	FullName          *string `json:"full_name"`
	AvatarURL         *string `json:"avatar_url"`
	Bio               *string `json:"bio"`
	PreferredLanguage *string `json:"preferred_language"`
}
type authResponse struct {
	AccessToken  string       `json:"access_token"`
	RefreshToken string       `json:"refresh_token"`
	User         userResponse `json:"user"`
}
type permissionsResponse struct {
	Permissions          []string `json:"permissions"`
	Roles                []string `json:"roles"`
	ManagedDepartmentIDs []string `json:"managed_department_ids"`
}

func (h AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	request, err := DecodeJSON[loginRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Service.Login(r.Context(), request.Email, request.Password)
	if err != nil {
		h.writeAuthError(w, r, err)
		return
	}
	h.writeResponse(w, r, result)
}

func (h AuthHandler) Refresh(w http.ResponseWriter, r *http.Request) {
	request, err := DecodeJSON[refreshRequest](r, 1<<20)
	if err != nil {
		request.RefreshToken = refreshCookie(r)
	}
	if strings.TrimSpace(request.RefreshToken) == "" {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyNoRefreshToken))
		return
	}
	result, err := h.Service.Refresh(r.Context(), request.RefreshToken)
	if err != nil {
		h.writeAuthError(w, r, err)
		return
	}
	h.writeResponse(w, r, result)
}

func (h AuthHandler) Logout(w http.ResponseWriter, r *http.Request) {
	refreshToken := ""
	if request, err := DecodeJSON[refreshRequest](r, 1<<20); err == nil {
		refreshToken = request.RefreshToken
	}
	if refreshToken == "" {
		refreshToken = refreshCookie(r)
	}
	if refreshToken != "" && h.Service.Repository != nil {
		if session, err := h.Service.Repository.FindSessionByTokenHash(r.Context(), auth.HashRefreshToken(refreshToken)); err == nil {
			_ = h.Service.Repository.RevokeSession(r.Context(), session.ID, time.Now().UTC())
		}
	}
	clearAuthCookies(w, r, h.CookieDomain)
	w.WriteHeader(http.StatusOK)
}

func (h AuthHandler) Me(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Service.Repository == nil {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	user, err := h.Service.Profile(r.Context(), userID)
	if err != nil {
		if errors.Is(err, service.ErrInvalidCredentials) || errors.Is(err, service.ErrSessionNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyUserNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, toUserResponse(user))
}

func (h AuthHandler) UpdateMe(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	request, err := DecodeJSON[updateMeRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	user, err := h.Service.UpdateProfile(r.Context(), userID, service.UpdateUserProfileRequest{
		FullName: request.FullName, AvatarURL: request.AvatarURL, Bio: request.Bio, PreferredLanguage: request.PreferredLanguage,
	})
	if err != nil {
		if errors.Is(err, service.ErrInvalidCredentials) || errors.Is(err, service.ErrSessionNotFound) {
			apperror.WriteError(w, r, apperror.NotFound(apperror.KeyUserNotFound))
			return
		}
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	user.SystemRoles = []string{}
	apperror.WriteJSON(w, http.StatusOK, toUserResponse(user))
}

func toUserResponse(user service.UserRecord) userResponse {
	var avatarFileID *string
	if user.AvatarFileID != nil {
		value := user.AvatarFileID.String()
		avatarFileID = &value
	}
	roles := user.SystemRoles
	if roles == nil {
		roles = []string{}
	}
	return userResponse{
		ID: user.ID.String(), Email: user.Email, FullName: user.FullName, AvatarURL: user.AvatarURL,
		Bio: user.Bio, PreferredLanguage: user.PreferredLanguage, CreatedAt: user.CreatedAt,
		SystemRoles: roles, AvatarFileID: avatarFileID,
	}
}

func (h AuthHandler) MyPermissions(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	workspaceID, ok := httpinfra.WorkspaceIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	if h.Authorizer == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("auth handler: nil authorizer")))
		return
	}
	member, err := h.Authorizer.IsMember(r.Context(), userID, workspaceID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	if !member {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotWorkspaceMember))
		return
	}
	permissions, roles, err := h.Authorizer.EffectivePermissions(r.Context(), userID, workspaceID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, permissionsResponse{
		Permissions:          permissions,
		Roles:                roles,
		ManagedDepartmentIDs: nil,
	})
}

func (h AuthHandler) WorkspaceUsers(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	workspaceID, ok := httpinfra.WorkspaceIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyMissingWorkspace))
		return
	}
	if h.Authorizer != nil {
		member, err := h.Authorizer.IsMember(r.Context(), userID, workspaceID)
		if err != nil {
			apperror.WriteError(w, r, apperror.Internal(err))
			return
		}
		if !member {
			apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotWorkspaceMember))
			return
		}
	}
	users, err := h.Service.WorkspaceUsers(r.Context(), workspaceID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return
	}
	response := make([]userResponse, 0, len(users))
	for _, user := range users {
		response = append(response, toUserResponse(user))
	}
	apperror.WriteJSON(w, http.StatusOK, response)
}

func clearAuthCookies(w http.ResponseWriter, r *http.Request, domain string) {
	secure, sameSite := authCookiePolicy(r)
	http.SetCookie(w, &http.Cookie{Name: "access_token", Value: "", Path: "/", Domain: domain, HttpOnly: true, Secure: secure, SameSite: sameSite, MaxAge: -1})
	http.SetCookie(w, &http.Cookie{Name: "refresh_token", Value: "", Path: "/", Domain: domain, HttpOnly: true, Secure: secure, SameSite: sameSite, MaxAge: -1})
}
func (h AuthHandler) writeResponse(w http.ResponseWriter, r *http.Request, result service.AuthResponse) {
	setAuthCookies(w, r, result.AccessToken, result.RefreshToken, h.CookieDomain)
	apperror.WriteJSON(w, http.StatusOK, authResponse{AccessToken: result.AccessToken, RefreshToken: result.RefreshToken, User: toUserResponse(result.User)})
}

func (h AuthHandler) writeAuthError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, service.ErrAccountLocked):
		apperror.WriteError(w, r, apperror.New(apperror.CodeTooManyRequests, http.StatusTooManyRequests, apperror.KeyLoginLocked))
	case errors.Is(err, service.ErrInvalidCredentials):
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyInvalidEmailOrPassword))
	case errors.Is(err, service.ErrRefreshReuse):
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyInvalidToken))
	case errors.Is(err, service.ErrEmailExists):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyEmailAlreadyExists))
	case errors.Is(err, service.ErrFullNameRequired):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyFullNameRequired))
	case errors.Is(err, service.ErrInvalidEmail):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidEmailFormat))
	case errors.Is(err, service.ErrPasswordTooShort):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyPasswordTooShort))
	case errors.Is(err, service.ErrPasswordTooLong):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyPasswordTooLong))
	case errors.Is(err, service.ErrPasswordComplexity):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyPasswordComplexity))
	case errors.Is(err, service.ErrResetTokenInvalid):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyResetTokenInvalid))
	case errors.Is(err, service.ErrCurrentPassword):
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyCurrentPasswordIncorrect))
	case errors.Is(err, service.ErrSessionNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyUserNotFound))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}

func refreshCookie(r *http.Request) string {
	cookie, err := r.Cookie("refresh_token")
	if err != nil {
		return ""
	}
	return cookie.Value
}

type registerRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
	FullName string `json:"full_name"`
}
type forgotPasswordRequest struct {
	Email string `json:"email"`
}
type resetPasswordRequest struct {
	Token       string `json:"token"`
	NewPassword string `json:"new_password"`
}
type changePasswordRequest struct {
	OldPassword string `json:"old_password"`
	NewPassword string `json:"new_password"`
}
type messageResponse struct {
	Message string `json:"message"`
}

func (h AuthHandler) Register(w http.ResponseWriter, r *http.Request) {
	request, err := DecodeJSON[registerRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Service.Register(r.Context(), request.Email, request.Password, request.FullName)
	if err != nil {
		h.writeAuthError(w, r, err)
		return
	}
	h.writeResponse(w, r, result)
}

func (h AuthHandler) ForgotPassword(w http.ResponseWriter, r *http.Request) {
	request, err := DecodeJSON[forgotPasswordRequest](r, 1<<20)
	if err == nil {
		_ = h.Service.ForgotPassword(r.Context(), request.Email)
	}
	apperror.WriteJSON(w, http.StatusOK, messageResponse{Message: "If this email is registered, a reset link will be sent."})
}

func (h AuthHandler) ResetPassword(w http.ResponseWriter, r *http.Request) {
	request, err := DecodeJSON[resetPasswordRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Service.ResetPassword(r.Context(), request.Token, request.NewPassword); err != nil {
		h.writeAuthError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h AuthHandler) ChangePassword(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	request, err := DecodeJSON[changePasswordRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Service.ChangePassword(r.Context(), userID, request.OldPassword, request.NewPassword); err != nil {
		h.writeAuthError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}
func setAuthCookies(w http.ResponseWriter, r *http.Request, accessToken, refreshToken, domain string) {
	secure, sameSite := authCookiePolicy(r)
	http.SetCookie(w, &http.Cookie{Name: "access_token", Value: accessToken, Path: "/", Domain: domain, HttpOnly: true, Secure: secure, SameSite: sameSite, MaxAge: 900})
	http.SetCookie(w, &http.Cookie{Name: "refresh_token", Value: refreshToken, Path: "/", Domain: domain, HttpOnly: true, Secure: secure, SameSite: sameSite, MaxAge: 30 * 24 * 60 * 60})
}

func authCookiePolicy(r *http.Request) (bool, http.SameSite) {
	forwardedProto := strings.TrimSpace(strings.Split(r.Header.Get("X-Forwarded-Proto"), ",")[0])
	secure := r.TLS != nil || strings.EqualFold(forwardedProto, "https")
	if secure {
		return true, http.SameSiteNoneMode
	}
	return false, http.SameSiteLaxMode
}
