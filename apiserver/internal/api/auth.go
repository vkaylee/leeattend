package api

import (
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type AuthHandler struct {
	Service      service.AuthService
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
	ID       string `json:"id"`
	Email    string `json:"email"`
	FullName string `json:"full_name,omitempty"`
}
type authResponse struct {
	AccessToken  string       `json:"access_token"`
	RefreshToken string       `json:"refresh_token"`
	User         userResponse `json:"user"`
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
	h.writeResponse(w, result)
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
	h.writeResponse(w, result)
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
	clearAuthCookies(w, h.CookieDomain)
	w.WriteHeader(http.StatusOK)
}

func (h AuthHandler) Me(w http.ResponseWriter, r *http.Request) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Service.Repository == nil {
		apperror.WritePrimitiveAuthRequired(w)
		return
	}
	user, err := h.Service.Repository.FindUserByID(r.Context(), userID)
	if err != nil {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyUserNotFound))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, userResponse{
		ID:       user.ID.String(),
		Email:    user.Email,
		FullName: user.FullName,
	})
}

func clearAuthCookies(w http.ResponseWriter, domain string) {
	http.SetCookie(w, &http.Cookie{Name: "access_token", Value: "", Path: "/", Domain: domain, HttpOnly: true, SameSite: http.SameSiteNoneMode, MaxAge: -1})
	http.SetCookie(w, &http.Cookie{Name: "refresh_token", Value: "", Path: "/", Domain: domain, HttpOnly: true, SameSite: http.SameSiteNoneMode, MaxAge: -1})
}
func (h AuthHandler) writeResponse(w http.ResponseWriter, result service.AuthResponse) {
	setAuthCookies(w, result.AccessToken, result.RefreshToken, h.CookieDomain)
	apperror.WriteJSON(w, http.StatusOK, authResponse{AccessToken: result.AccessToken, RefreshToken: result.RefreshToken, User: userResponse{ID: result.User.ID.String(), Email: result.User.Email, FullName: result.User.FullName}})
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
	h.writeResponse(w, result)
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
func setAuthCookies(w http.ResponseWriter, accessToken, refreshToken, domain string) {
	http.SetCookie(w, &http.Cookie{Name: "access_token", Value: accessToken, Path: "/", Domain: domain, HttpOnly: true, Secure: true, SameSite: http.SameSiteNoneMode, MaxAge: 900})
	http.SetCookie(w, &http.Cookie{Name: "refresh_token", Value: refreshToken, Path: "/", Domain: domain, HttpOnly: true, Secure: true, SameSite: http.SameSiteNoneMode, MaxAge: 30 * 24 * 60 * 60})
}
