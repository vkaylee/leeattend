package api

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"
	"unicode"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const (
	MatchTierHard = "hard"
	MatchTierSoft = "soft"
	MatchTierNone = "none"

	MatchReasonNationalIDExact = "national_id_exact"
	MatchReasonPhoneExact      = "phone_exact"
	MatchReasonNameDOBReview   = "name_dob_review"
	MatchReasonNamePhoneReview = "name_phone_review"

	MatchStatusActive     = "active"
	MatchStatusTerminated = "terminated"

	MatchAuditAction = "employees.identity.match"

	maxEmployeeMatchBodyBytes   = 4 << 10
	maxMatchFullNameBytes       = 255
	maxMatchDateOfBirthBytes    = 32
	maxMatchNationalIDBytes     = 128
	maxMatchPhoneBytes          = 64
	maxMatchPartialPhoneBytes   = 32
	defaultEmployeeMatchHardCap = 10
	defaultEmployeeMatchSoftCap = 5
)

type MatchEmployeeRequest struct {
	FullName     string `json:"full_name,omitempty"`
	DateOfBirth  string `json:"date_of_birth,omitempty"`
	Phone        string `json:"phone,omitempty"`
	PartialPhone string `json:"partial_phone,omitempty"`
	NationalID   string `json:"national_id,omitempty"`
}

type EmployeeMatchCandidate struct {
	EmployeeID   uuid.UUID `json:"employee_id"`
	DisplayName  string    `json:"display_name"`
	EmployeeCode string    `json:"employee_code"`
	Status       string    `json:"status"`
	Tier         string    `json:"tier"`
	Reasons      []string  `json:"reasons"`
}

type MatchEmployeeResponse struct {
	Tier       string                   `json:"tier"`
	Candidates []EmployeeMatchCandidate `json:"candidates"`
}

// EmployeeMatchQuery holds normalized, hashed, and bounded match inputs for repository lookup.
type EmployeeMatchQuery struct {
	NationalIDHash     string
	PhoneHash          string
	NormalizedFullName string
	NormalizedDOB      string
	PartialPhoneDigits string
	HardLimit          int
	SoftLimit          int
}

// HasCriteria reports whether at least one hard or soft matching rule can be evaluated.
func (q EmployeeMatchQuery) HasCriteria() bool {
	if q.NationalIDHash != "" || q.PhoneHash != "" {
		return true
	}
	return q.NormalizedFullName != "" && (q.NormalizedDOB != "" || q.PartialPhoneDigits != "")
}

type EmployeeIdentityMatcher interface {
	MatchEmployees(context.Context, EmployeeMatchQuery) ([]EmployeeMatchCandidate, error)
}

// Match handles POST /api/v1/employees/match (read-only redacted candidate lookup).
func (h *EmployeeHandler) Match(w http.ResponseWriter, r *http.Request) {
	startedAt := time.Now()
	actorID, ok := h.authorizeMatch(w, r)
	if !ok {
		return
	}
	req, ok := decodeMatchRequest(w, r)
	if !ok {
		return
	}
	if err := h.IdentityHasher.Validate(); err != nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	query, err := buildEmployeeMatchQuery(req, h.IdentityHasher)
	if err != nil {
		writeMatchBuildError(w, r, err)
		return
	}
	if h.IdentityMatcher == nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	if h.Audit == nil {
		apperror.WriteError(w, r, apperror.InternalString("employee identity match audit writer is not configured"))
		return
	}

	candidates := []EmployeeMatchCandidate{}
	if query.HasCriteria() {
		rawCandidates, matchErr := h.IdentityMatcher.MatchEmployees(r.Context(), query)
		if matchErr != nil {
			h.Metrics.ObserveMatch(r.Context(), "error", 0, time.Since(startedAt))
			apperror.WriteError(w, r, apperror.Internal(errors.New("employee identity match lookup failed")))
			return
		}
		candidates = sanitizeMatchCandidates(rawCandidates)
	}

	tier := resolveMatchOutcomeTier(candidates)
	if auditErr := h.emitMatchAudit(r.Context(), actorID, tier, len(candidates)); auditErr != nil {
		h.Metrics.ObserveMatch(r.Context(), "audit_error", 0, time.Since(startedAt))
		apperror.WriteError(w, r, apperror.Internal(errors.New("employee identity match audit failed")))
		return
	}
	h.Metrics.ObserveMatch(r.Context(), tier, len(candidates), time.Since(startedAt))

	apperror.WriteJSON(w, http.StatusOK, MatchEmployeeResponse{
		Tier:       tier,
		Candidates: candidates,
	})
}

func (h *EmployeeHandler) authorizeMatch(w http.ResponseWriter, r *http.Request) (uuid.UUID, bool) {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || userID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return uuid.Nil, false
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionEmployeesIdentityMatch)) {
		writeForbidden(w, r)
		return uuid.Nil, false
	}
	return userID, true
}

func decodeMatchRequest(w http.ResponseWriter, r *http.Request) (MatchEmployeeRequest, bool) {
	if hasDisallowedMatchQueryParams(r) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return MatchEmployeeRequest{}, false
	}
	req, err := DecodeJSON[MatchEmployeeRequest](r, maxEmployeeMatchBodyBytes)
	if err != nil {
		var maxBytesErr *http.MaxBytesError
		if errors.As(err, &maxBytesErr) {
			apperror.WriteError(w, r, apperror.PayloadTooLarge(apperror.KeyFileTooLarge))
			return MatchEmployeeRequest{}, false
		}
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return MatchEmployeeRequest{}, false
	}
	return req, true
}

func hasDisallowedMatchQueryParams(r *http.Request) bool {
	if r == nil || r.URL == nil || r.URL.RawQuery == "" {
		return false
	}
	for key := range r.URL.Query() {
		if key != "lang" {
			return true
		}
	}
	return false
}

func writeMatchBuildError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, identity.ErrIdentityHashSecretMissing) {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
}

func buildEmployeeMatchQuery(req MatchEmployeeRequest, hasher identity.Hasher) (EmployeeMatchQuery, error) {
	if err := validateMatchInputBounds(req); err != nil {
		return EmployeeMatchQuery{}, err
	}
	dob, err := identity.NormalizeDOB(req.DateOfBirth)
	if err != nil {
		return EmployeeMatchQuery{}, ErrInvalidInput
	}
	normalizedPhone, err := identity.NormalizePhone(req.Phone)
	if err != nil {
		return EmployeeMatchQuery{}, ErrInvalidInput
	}
	partialPhone, err := identity.NormalizePartialPhone(req.PartialPhone)
	if err != nil {
		return EmployeeMatchQuery{}, ErrInvalidInput
	}
	if partialPhone == "" && normalizedPhone != "" {
		partialPhone = identity.PhoneSoftSuffix(normalizedPhone)
	}

	nidHash, _, err := hasher.HashNationalID(req.NationalID)
	if err != nil {
		return EmployeeMatchQuery{}, err
	}
	phoneHash, _, err := hasher.HashPhone(normalizedPhone)
	if err != nil {
		return EmployeeMatchQuery{}, err
	}

	return EmployeeMatchQuery{
		NationalIDHash:     nidHash,
		PhoneHash:          phoneHash,
		NormalizedFullName: identity.NormalizeName(req.FullName),
		NormalizedDOB:      dob,
		PartialPhoneDigits: partialPhone,
		HardLimit:          defaultEmployeeMatchHardCap,
		SoftLimit:          defaultEmployeeMatchSoftCap,
	}, nil
}

func validateMatchInputBounds(req MatchEmployeeRequest) error {
	checks := []struct {
		value  string
		maxLen int
	}{
		{req.FullName, maxMatchFullNameBytes},
		{req.DateOfBirth, maxMatchDateOfBirthBytes},
		{req.NationalID, maxMatchNationalIDBytes},
		{req.Phone, maxMatchPhoneBytes},
		{req.PartialPhone, maxMatchPartialPhoneBytes},
	}
	for _, c := range checks {
		if len(c.value) > c.maxLen || containsControlChar(c.value) {
			return ErrInvalidInput
		}
	}
	return nil
}

func containsControlChar(s string) bool {
	for _, r := range s {
		if (r < 0x20 && r != '\t') || r == 0x7f {
			return true
		}
	}
	return false
}

func (h *EmployeeHandler) emitMatchAudit(ctx context.Context, actorID uuid.UUID, tier string, candidateCount int) error {
	return h.Audit.Emit(ctx, audit.Event{
		Name:     MatchAuditAction,
		ActorID:  actorID.String(),
		Resource: "employees",
		Outcome:  tier,
		Details: map[string]string{
			"outcome_tier":    tier,
			"candidate_count": strconv.Itoa(candidateCount),
		},
		At: time.Now().UTC(),
	})
}

func sanitizeMatchCandidates(raw []EmployeeMatchCandidate) []EmployeeMatchCandidate {
	out := make([]EmployeeMatchCandidate, 0, len(raw))
	for _, candidate := range raw {
		reasons := filterAllowedMatchReasons(candidate.Reasons)
		if candidate.EmployeeID == uuid.Nil || len(reasons) == 0 {
			continue
		}
		out = append(out, EmployeeMatchCandidate{
			EmployeeID:   candidate.EmployeeID,
			DisplayName:  sanitizeMatchText(candidate.DisplayName),
			EmployeeCode: sanitizeMatchText(candidate.EmployeeCode),
			Status:       normalizeMatchStatus(candidate.Status),
			Tier:         resolveCandidateTier(reasons),
			Reasons:      reasons,
		})
	}
	return out
}

func sanitizeMatchText(raw string) string {
	cleaned := strings.Map(func(r rune) rune {
		if unicode.IsControl(r) {
			return ' '
		}
		return r
	}, raw)
	return strings.Join(strings.Fields(cleaned), " ")
}

func normalizeMatchStatus(status string) string {
	if status == MatchStatusActive {
		return MatchStatusActive
	}
	return MatchStatusTerminated
}

func filterAllowedMatchReasons(reasons []string) []string {
	allowedOrder := []string{
		MatchReasonNationalIDExact,
		MatchReasonPhoneExact,
		MatchReasonNameDOBReview,
		MatchReasonNamePhoneReview,
	}
	present := make(map[string]bool, len(reasons))
	for _, reason := range reasons {
		present[reason] = true
	}
	filtered := make([]string, 0, len(reasons))
	for _, reason := range allowedOrder {
		if present[reason] {
			filtered = append(filtered, reason)
		}
	}
	return filtered
}

func resolveCandidateTier(reasons []string) string {
	for _, reason := range reasons {
		if reason == MatchReasonNationalIDExact || reason == MatchReasonPhoneExact {
			return MatchTierHard
		}
	}
	return MatchTierSoft
}

func resolveMatchOutcomeTier(candidates []EmployeeMatchCandidate) string {
	if len(candidates) == 0 {
		return MatchTierNone
	}
	for _, candidate := range candidates {
		if candidate.Tier == MatchTierHard {
			return MatchTierHard
		}
	}
	return MatchTierSoft
}
