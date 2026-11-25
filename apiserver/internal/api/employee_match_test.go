package api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const testMatchSecret = "0123456789abcdef0123456789abcdef"

type stubIdentityMatcher struct {
	called     bool
	lastQuery  EmployeeMatchQuery
	candidates []EmployeeMatchCandidate
	err        error
}

func (s *stubIdentityMatcher) MatchEmployees(_ context.Context, query EmployeeMatchQuery) ([]EmployeeMatchCandidate, error) {
	s.called = true
	s.lastQuery = query
	if s.err != nil {
		return nil, s.err
	}
	return s.candidates, nil
}

type stubMatchAudit struct {
	events []audit.Event
	err    error
}

func (s *stubMatchAudit) Emit(_ context.Context, event audit.Event) error {
	s.events = append(s.events, event)
	return s.err
}

func newTestMatchHandler(
	t *testing.T,
	perms map[string]bool,
	matcher *stubIdentityMatcher,
	auditSink *stubMatchAudit,
) EmployeeHandler {
	t.Helper()
	hasher, err := identity.NewHasher(testMatchSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}
	return EmployeeHandler{
		Repository:      mockEmployeeRepo{},
		IdentityMatcher: matcher,
		IdentityHasher:  hasher,
		Authorizer:      employeeSliceAuthorizer{perms: perms},
		Audit:           auditSink,
	}
}

func newAuthedMatchRequest(target, body string, actorID uuid.UUID) *http.Request {
	req := httptest.NewRequest(http.MethodPost, target, bytes.NewBufferString(body))
	if actorID != uuid.Nil {
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	}
	return req
}

func TestEmployeeMatch_AuthAndPermissionFailClosed(t *testing.T) {
	matcher := &stubIdentityMatcher{}
	auditSink := &stubMatchAudit{}
	h := newTestMatchHandler(t, map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true}, matcher, auditSink)

	rec401 := httptest.NewRecorder()
	h.Match(rec401, newAuthedMatchRequest("/api/v1/employees/match", `{"national_id":"001234567"}`, uuid.Nil))
	if rec401.Code != http.StatusUnauthorized {
		t.Fatalf("unauthenticated status = %d, want 401", rec401.Code)
	}

	hNoPerm := newTestMatchHandler(t, map[string]bool{"employees:read": true, "employees:create": true}, matcher, auditSink)
	rec403 := httptest.NewRecorder()
	hNoPerm.Match(rec403, newAuthedMatchRequest("/api/v1/employees/match", `{"national_id":"001234567"}`, uuid.New()))
	if rec403.Code != http.StatusForbidden {
		t.Fatalf("missing permission status = %d, want 403", rec403.Code)
	}
	if matcher.called || len(auditSink.events) != 0 {
		t.Fatalf("unauthorized request must not query matcher or emit audit")
	}
}

func TestEmployeeMatch_ValidationRejectsUnknownFieldsAndQueryPII(t *testing.T) {
	actorID := uuid.New()
	cases := []struct {
		name   string
		target string
		body   string
	}{
		{"unknown field", "/api/v1/employees/match", `{"national_id":"001234567","extra":"nope"}`},
		{"malformed json", "/api/v1/employees/match", `{"national_id":`},
		{"trailing json", "/api/v1/employees/match", `{}{}`},
		{"query phone pii", "/api/v1/employees/match?phone=%2B84912345678", `{}`},
		{"query national_id pii", "/api/v1/employees/match?national_id=001234567", `{}`},
		{"invalid dob", "/api/v1/employees/match", `{"date_of_birth":"1995-02-30"}`},
		{"national only phone", "/api/v1/employees/match", `{"phone":"0912345678"}`},
		{"phone extension", "/api/v1/employees/match", `{"phone":"+84912345678 ext 9"}`},
		{"short partial phone", "/api/v1/employees/match", `{"full_name":"Nguyen Van A","partial_phone":"123"}`},
		{"alpha partial phone", "/api/v1/employees/match", `{"full_name":"Nguyen Van A","partial_phone":"12ab"}`},
		{"overlong full name", "/api/v1/employees/match", `{"full_name":"` + strings.Repeat("a", 256) + `"}`},
		{"control char in name", "/api/v1/employees/match", `{"full_name":"Nguyen\nVan A"}`},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			matcher := &stubIdentityMatcher{}
			auditSink := &stubMatchAudit{}
			h := newTestMatchHandler(t, map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true}, matcher, auditSink)

			rec := httptest.NewRecorder()
			h.Match(rec, newAuthedMatchRequest(tc.target, tc.body, actorID))
			if rec.Code != http.StatusBadRequest {
				t.Fatalf("%s status = %d, want 400; body=%s", tc.name, rec.Code, rec.Body.String())
			}
			if matcher.called {
				t.Fatalf("%s must not invoke matcher", tc.name)
			}
		})
	}
}

func TestEmployeeMatch_OversizedBodyAndMissingSecretFailClosed(t *testing.T) {
	actorID := uuid.New()
	perms := map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true}

	t.Run("oversized body returns 413", func(t *testing.T) {
		matcher := &stubIdentityMatcher{}
		h := newTestMatchHandler(t, perms, matcher, &stubMatchAudit{})
		bigBody := `{"full_name":"` + strings.Repeat("a", maxEmployeeMatchBodyBytes+64) + `"}`
		rec := httptest.NewRecorder()
		h.Match(rec, newAuthedMatchRequest("/api/v1/employees/match", bigBody, actorID))
		if rec.Code != http.StatusRequestEntityTooLarge {
			t.Fatalf("status = %d, want 413", rec.Code)
		}
		if matcher.called {
			t.Fatal("oversized body must not invoke matcher")
		}
	})

	t.Run("missing secret fails closed with 503", func(t *testing.T) {
		matcher := &stubIdentityMatcher{}
		auditSink := &stubMatchAudit{}
		h := EmployeeHandler{
			IdentityMatcher: matcher,
			IdentityHasher:  identity.Hasher{},
			Authorizer:      employeeSliceAuthorizer{perms: perms},
			Audit:           auditSink,
		}
		rec := httptest.NewRecorder()
		h.Match(rec, newAuthedMatchRequest("/api/v1/employees/match", `{"national_id":"001234567"}`, actorID))
		if rec.Code != http.StatusServiceUnavailable {
			t.Fatalf("status = %d, want 503; body=%s", rec.Code, rec.Body.String())
		}
		if matcher.called || len(auditSink.events) != 0 {
			t.Fatal("missing secret must fail closed before lookup/audit")
		}
	})
}

func TestEmployeeMatch_HardMatchRedactionAndAudit(t *testing.T) {
	actorID := uuid.New()
	candidateID := uuid.New()
	rawNID := "001-234.567"
	rawPhone := "+84 912-345-678"
	rawDOB := "1995-08-19"

	matcher := &stubIdentityMatcher{
		candidates: []EmployeeMatchCandidate{{
			EmployeeID:   candidateID,
			DisplayName:  "  Nguyễn   Văn A\n",
			EmployeeCode: " EMP-001 ",
			Status:       MatchStatusActive,
			Tier:         MatchTierHard,
			Reasons:      []string{MatchReasonPhoneExact, "leaked_reason", MatchReasonNationalIDExact},
		}},
	}
	auditSink := &stubMatchAudit{}
	h := newTestMatchHandler(t, map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true}, matcher, auditSink)

	body := `{"full_name":"Nguyễn Văn A","date_of_birth":"` + rawDOB + `","phone":"` + rawPhone + `","national_id":"` + rawNID + `"}`
	rec := httptest.NewRecorder()
	h.Match(rec, newAuthedMatchRequest("/api/v1/employees/match?lang=en", body, actorID))
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200; body=%s", rec.Code, rec.Body.String())
	}

	assertHardMatchResponseAndRedaction(t, rec.Body.String(), candidateID, matcher.lastQuery, rawNID, rawPhone, rawDOB)
	assertSanitizedMatchAudit(t, auditSink.events, actorID, candidateID, matcher.lastQuery, MatchTierHard, "1", rawNID, rawPhone, rawDOB)
}

func assertNoForbiddenSubstrings(t *testing.T, label, text string, forbidden ...string) {
	t.Helper()
	for _, value := range forbidden {
		if value != "" && strings.Contains(text, value) {
			t.Fatalf("%s leaked sensitive value %q: %s", label, value, text)
		}
	}
}

func assertHardMatchResponseAndRedaction(
	t *testing.T,
	rawJSON string,
	candidateID uuid.UUID,
	query EmployeeMatchQuery,
	rawNID, rawPhone, rawDOB string,
) {
	t.Helper()
	var resp MatchEmployeeResponse
	if err := json.Unmarshal([]byte(rawJSON), &resp); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	if resp.Tier != MatchTierHard || len(resp.Candidates) != 1 {
		t.Fatalf("unexpected response: %+v", resp)
	}
	c := resp.Candidates[0]
	want := EmployeeMatchCandidate{
		EmployeeID:   candidateID,
		DisplayName:  "Nguyễn Văn A",
		EmployeeCode: "EMP-001",
		Status:       MatchStatusActive,
		Tier:         MatchTierHard,
		Reasons:      []string{MatchReasonNationalIDExact, MatchReasonPhoneExact},
	}
	if c.EmployeeID != want.EmployeeID || c.DisplayName != want.DisplayName || c.EmployeeCode != want.EmployeeCode || c.Status != want.Status || strings.Join(c.Reasons, ",") != strings.Join(want.Reasons, ",") {
		t.Fatalf("candidate = %+v, want %+v", c, want)
	}
	assertNoForbiddenSubstrings(t, "response", rawJSON, rawNID, "001234567", rawPhone, "+84912345678", rawDOB, query.NationalIDHash, query.PhoneHash, "leaked_reason")
}

func assertSanitizedMatchAudit(
	t *testing.T,
	events []audit.Event,
	actorID, candidateID uuid.UUID,
	query EmployeeMatchQuery,
	wantTier, wantCount string,
	sensitive ...string,
) {
	t.Helper()
	if len(events) != 1 {
		t.Fatalf("audit event count = %d, want 1", len(events))
	}
	ev := events[0]
	if ev.ActorID != actorID.String() || ev.Name != MatchAuditAction || ev.Outcome != wantTier {
		t.Fatalf("unexpected audit envelope: %+v", ev)
	}
	if len(ev.Details) != 2 || ev.Details["outcome_tier"] != wantTier || ev.Details["candidate_count"] != wantCount {
		t.Fatalf("unexpected audit details: %+v", ev.Details)
	}
	encoded, _ := json.Marshal(ev)
	forbidden := append(sensitive, candidateID.String(), query.NationalIDHash, query.PhoneHash)
	assertNoForbiddenSubstrings(t, "audit", string(encoded), forbidden...)
}

func TestEmployeeMatch_SoftMatchNormalization(t *testing.T) {
	actorID := uuid.New()
	perms := map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true}
	matcher := &stubIdentityMatcher{
		candidates: []EmployeeMatchCandidate{{
			EmployeeID:   uuid.New(),
			DisplayName:  "Nguyễn Văn A",
			EmployeeCode: "EMP-002",
			Status:       MatchStatusTerminated,
			Tier:         MatchTierSoft,
			Reasons:      []string{MatchReasonNameDOBReview, MatchReasonNamePhoneReview},
		}},
	}
	h := newTestMatchHandler(t, perms, matcher, &stubMatchAudit{})
	rec := httptest.NewRecorder()
	h.Match(rec, newAuthedMatchRequest("/api/v1/employees/match", `{"full_name":"  Nguyễn  Văn A ","date_of_birth":"1990-01-02","partial_phone":"345-678"}`, actorID))
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200", rec.Code)
	}
	q := matcher.lastQuery
	if q.NormalizedFullName != "nguyễn văn a" || q.NormalizedDOB != "1990-01-02" || q.PartialPhoneDigits != "345678" {
		t.Fatalf("unexpected soft query: %+v", q)
	}
	var resp MatchEmployeeResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if resp.Tier != MatchTierSoft || len(resp.Candidates) != 1 {
		t.Fatalf("unexpected soft response: %+v", resp)
	}
}

func TestEmployeeMatch_EmptyAndDOBOnlyReturnNoneWithoutQuery(t *testing.T) {
	actorID := uuid.New()
	perms := map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true}
	bodies := []string{
		`{}`,
		`{"national_id":" --- ","phone":"  "}`,
		`{"date_of_birth":"1990-01-02"}`,
		`{"full_name":"Nguyễn Văn A"}`,
		`{"partial_phone":"345678"}`,
	}
	for _, body := range bodies {
		matcher := &stubIdentityMatcher{}
		auditSink := &stubMatchAudit{}
		h := newTestMatchHandler(t, perms, matcher, auditSink)
		rec := httptest.NewRecorder()
		h.Match(rec, newAuthedMatchRequest("/api/v1/employees/match", body, actorID))
		assertNoneMatchOutcome(t, body, rec, matcher, auditSink)
	}
}

func assertNoneMatchOutcome(
	t *testing.T,
	body string,
	rec *httptest.ResponseRecorder,
	matcher *stubIdentityMatcher,
	auditSink *stubMatchAudit,
) {
	t.Helper()
	if rec.Code != http.StatusOK {
		t.Fatalf("body=%s status=%d, want 200", body, rec.Code)
	}
	if matcher.called {
		t.Fatalf("body=%s must not query matcher", body)
	}
	if strings.TrimSpace(rec.Body.String()) != `{"tier":"none","candidates":[]}` {
		t.Fatalf("body=%s response=%s, want constant-shape none", body, rec.Body.String())
	}
	if len(auditSink.events) != 1 || auditSink.events[0].Details["outcome_tier"] != MatchTierNone || auditSink.events[0].Details["candidate_count"] != "0" {
		t.Fatalf("body=%s unexpected audit: %+v", body, auditSink.events)
	}
}

func TestEmployeeMatch_AuditFailureBlocksResponse(t *testing.T) {
	actorID := uuid.New()
	candidateID := uuid.New()
	perms := map[string]bool{string(auth.PermissionEmployeesIdentityMatch): true}
	matcher := &stubIdentityMatcher{
		candidates: []EmployeeMatchCandidate{{
			EmployeeID:   candidateID,
			DisplayName:  "Secret Candidate",
			EmployeeCode: "EMP-999",
			Status:       MatchStatusActive,
			Tier:         MatchTierHard,
			Reasons:      []string{MatchReasonNationalIDExact},
		}},
	}

	h := newTestMatchHandler(t, perms, matcher, &stubMatchAudit{err: errors.New("audit db down")})
	rec := httptest.NewRecorder()
	h.Match(rec, newAuthedMatchRequest("/api/v1/employees/match", `{"national_id":"001234567"}`, actorID))
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("audit failure status = %d, want 500", rec.Code)
	}
	for _, leaked := range []string{candidateID.String(), "Secret Candidate", "EMP-999"} {
		if strings.Contains(rec.Body.String(), leaked) {
			t.Fatalf("failed audit must block candidate payload %q: %s", leaked, rec.Body.String())
		}
	}

	hNilAudit := newTestMatchHandler(t, perms, matcher, nil)
	hNilAudit.Audit = nil
	recNil := httptest.NewRecorder()
	hNilAudit.Match(recNil, newAuthedMatchRequest("/api/v1/employees/match", `{"national_id":"001234567"}`, actorID))
	if recNil.Code != http.StatusInternalServerError {
		t.Fatalf("nil audit writer status = %d, want 500", recNil.Code)
	}
}

func TestEmployeeMatch_OpenAPIContract(t *testing.T) {
	specPath := filepath.Join("..", "..", "openapi.yaml")
	raw, err := os.ReadFile(specPath)
	if err != nil {
		t.Fatalf("read openapi.yaml: %v", err)
	}
	spec := string(raw)
	for _, required := range []string{
		`"/api/v1/employees/match":`,
		`operationId: post_api_v1_employees_match`,
		`$ref: '#/components/schemas/MatchEmployeeRequest'`,
		`$ref: '#/components/schemas/MatchEmployeeResponse'`,
		`MatchEmployeeRequest:`,
		`MatchEmployeeResponse:`,
		`EmployeeMatchCandidate:`,
		`- national_id_exact`,
		`- phone_exact`,
		`- name_dob_review`,
		`- name_phone_review`,
	} {
		if !strings.Contains(spec, required) {
			t.Fatalf("openapi.yaml missing contract clause %q", required)
		}
	}

	start := strings.Index(spec, "    EmployeeMatchCandidate:")
	end := strings.Index(spec[start:], "    EmployeePhase:")
	if start < 0 || end <= 0 {
		t.Fatal("failed to locate EmployeeMatchCandidate schema block")
	}
	candidateBlock := spec[start : start+end]
	for _, forbidden := range []string{"phone:", "national_id:", "date_of_birth:", "phone_lookup_hash:", "national_id_lookup_hash:"} {
		if strings.Contains(candidateBlock, forbidden) {
			t.Fatalf("EmployeeMatchCandidate schema must not expose %q", forbidden)
		}
	}
}
