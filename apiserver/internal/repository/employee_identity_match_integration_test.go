//go:build integration

package repository

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"github.com/vkaylee/leeattend/apiserver/internal/standalone"
)

type failingMatchAudit struct{}

func (failingMatchAudit) Emit(context.Context, audit.Event) error {
	return errors.New("simulated audit failure")
}

func TestIntegrationEmployeeIdentityMatchRBACPermissions(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	if err := standalone.EnsureDefaultRolesAndPermissions(ctx, tx); err != nil {
		t.Fatalf("EnsureDefaultRolesAndPermissions: %v", err)
	}

	authorizer := OrganizationAuthorizer{DB: tx}
	roleExpectations := []struct {
		role    string
		allowed bool
	}{
		{"admin", true},
		{"hr_manager", true},
		{"shift_manager", false},
		{"team_lead", false},
		{"employee", false},
	}

	for _, tc := range roleExpectations {
		userID := insertIntegrationUserWithRole(t, ctx, tx, tc.role)
		got := authorizer.HasPermission(ctx, userID, string(auth.PermissionEmployeesIdentityMatch))
		if got != tc.allowed {
			t.Fatalf("role %q HasPermission(%s) = %v, want %v", tc.role, auth.PermissionEmployeesIdentityMatch, got, tc.allowed)
		}
	}
	if authorizer.HasPermission(ctx, uuid.New(), string(auth.PermissionEmployeesIdentityMatch)) {
		t.Fatal("unassigned user must be denied employees:identity:match")
	}
}

func TestIntegrationEmployeeIdentityMatchEndToEnd(t *testing.T) {
	pool, ctx := requireTenureIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	if err := standalone.EnsureDefaultRolesAndPermissions(ctx, tx); err != nil {
		t.Fatalf("EnsureDefaultRolesAndPermissions: %v", err)
	}
	hrUserID := insertIntegrationUserWithRole(t, ctx, tx, "hr_manager")
	teamLeadUserID := insertIntegrationUserWithRole(t, ctx, tx, "team_lead")

	hasher, err := identity.NewHasher(testIdentityBackfillSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}
	fixtures := seedIdentityMatchFixtures(t, ctx, tx, hasher)

	handler := &api.EmployeeHandler{
		IdentityMatcher: EmployeeIdentityRepository{DB: tx},
		IdentityHasher:  hasher,
		Authorizer:      OrganizationAuthorizer{DB: tx},
		Audit:           AuditLogWriter{DB: tx},
	}

	assertIntegrationMatchAuthDenied(t, handler, teamLeadUserID)
	assertIntegrationHardMatchAndAudit(t, ctx, tx, handler, hrUserID, fixtures, hasher)
	assertIntegrationSoftAndNoneMatches(t, handler, hrUserID, fixtures)
	assertIntegrationSoftLimitCap(t, ctx, tx, handler, hrUserID)
	assertIntegrationAuditFailClosed(t, handler, hrUserID, fixtures)
}

type matchIntegrationFixtures struct {
	activeNIDEmployeeID     uuid.UUID
	terminatedNIDEmployeeID uuid.UUID
	phoneEmployeeID         uuid.UUID
	legacySoftEmployeeID    uuid.UUID
}

func seedIdentityMatchFixtures(
	t *testing.T,
	ctx context.Context,
	tx pgx.Tx,
	hasher identity.Hasher,
) matchIntegrationFixtures {
	t.Helper()
	if _, err := tx.Exec(ctx, `DELETE FROM employee_profiles`); err != nil {
		t.Fatalf("clear profiles in tx: %v", err)
	}

	f := matchIntegrationFixtures{
		activeNIDEmployeeID:     insertMatchEmployee(t, ctx, tx, "EMP-ACT-01", "Nguyễn Văn An", true, "001-234-567", "+84911111111", "1991-03-15"),
		terminatedNIDEmployeeID: insertMatchEmployee(t, ctx, tx, "EMP-TERM-02", "Nguyễn Văn An Old", false, "００１ 234.567", "", "1991-03-15"),
		phoneEmployeeID:         insertMatchEmployee(t, ctx, tx, "EMP-PHN-03", "Trần Thị Bình", true, "099-000-111", "+84 912-345-678", "1992-07-20"),
		legacySoftEmployeeID:    insertMatchEmployee(t, ctx, tx, "EMP-LEG-04", "  Nguyễn   Thị  Cúc  ", true, "", "0988-777-666", "1993-11-05"),
	}

	repo := EmployeeIdentityRepository{
		DB: tx,
		BeginTx: func(context.Context) (service.TxBeginner, error) {
			return tx, nil
		},
	}
	if _, err := repo.BackfillIdentityLookupHashes(ctx, hasher, IdentityBackfillOptions{BatchSize: 50}); err != nil {
		t.Fatalf("backfill lookup hashes: %v", err)
	}
	return f
}

func insertIntegrationUserWithRole(t *testing.T, ctx context.Context, tx pgx.Tx, roleName string) uuid.UUID {
	t.Helper()
	userID := uuid.New()
	email := fmt.Sprintf("match-%s-%s@example.invalid", roleName, userID.String()[:8])
	if _, err := tx.Exec(ctx,
		`INSERT INTO users (id, email, password_hash, full_name) VALUES ($1, $2, 'hash', $3)`,
		userID, email, "Match Test "+roleName,
	); err != nil {
		t.Fatalf("insert user %s: %v", roleName, err)
	}
	if _, err := tx.Exec(ctx,
		`INSERT INTO user_roles (user_id, role_id) SELECT $1, id FROM roles WHERE name = $2`,
		userID, roleName,
	); err != nil {
		t.Fatalf("assign role %s: %v", roleName, err)
	}
	return userID
}

func insertMatchEmployee(
	t *testing.T,
	ctx context.Context,
	tx pgx.Tx,
	code, fullName string,
	active bool,
	nationalID, phone, dob string,
) uuid.UUID {
	t.Helper()
	empID := uuid.New()
	if _, err := tx.Exec(ctx,
		`INSERT INTO employees (id, full_name)
		 VALUES ($1, $2)`,
		empID, fullName,
	); err != nil {
		t.Fatalf("insert employee %s: %v", code, err)
	}
	endDate := any(nil)
	if !active {
		endDate = "2025-12-31"
	}
	if _, err := tx.Exec(ctx,
		`INSERT INTO employee_tenures (employee_id, employee_number, start_date, end_date)
		 VALUES ($1, $2, DATE '2025-01-01', $3::date)`,
		empID, code, endDate,
	); err != nil {
		t.Fatalf("insert tenure %s: %v", code, err)
	}
	var nidPtr, phonePtr, dobPtr any
	if nationalID != "" {
		nidPtr = nationalID
	}
	if phone != "" {
		phonePtr = phone
	}
	if dob != "" {
		dobPtr = dob
	}
	if _, err := tx.Exec(ctx,
		`INSERT INTO employee_profiles (employee_id, national_id, phone, date_of_birth)
		 VALUES ($1, $2, $3, $4::date)`,
		empID, nidPtr, phonePtr, dobPtr,
	); err != nil {
		t.Fatalf("insert profile %s: %v", code, err)
	}
	return empID
}

func invokeIntegrationMatch(handler *api.EmployeeHandler, actorID uuid.UUID, body string) *httptest.ResponseRecorder {
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/match", bytes.NewBufferString(body))
	if actorID != uuid.Nil {
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	}
	rec := httptest.NewRecorder()
	handler.Match(rec, req)
	return rec
}

func assertIntegrationMatchAuthDenied(t *testing.T, handler *api.EmployeeHandler, teamLeadUserID uuid.UUID) {
	t.Helper()
	if rec := invokeIntegrationMatch(handler, uuid.Nil, `{"national_id":"001234567"}`); rec.Code != http.StatusUnauthorized {
		t.Fatalf("unauthenticated status = %d, want 401", rec.Code)
	}
	if rec := invokeIntegrationMatch(handler, teamLeadUserID, `{"national_id":"001234567"}`); rec.Code != http.StatusForbidden {
		t.Fatalf("team_lead status = %d, want 403", rec.Code)
	}
}

func decodeIntegrationMatchResponse(t *testing.T, rec *httptest.ResponseRecorder, label string) api.MatchEmployeeResponse {
	t.Helper()
	if rec.Code != http.StatusOK {
		t.Fatalf("%s status = %d, want 200; body=%s", label, rec.Code, rec.Body.String())
	}
	var resp api.MatchEmployeeResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode %s: %v", label, err)
	}
	return resp
}

func assertNoIntegrationLeak(t *testing.T, label, raw string, forbidden ...string) {
	t.Helper()
	for _, value := range forbidden {
		if value != "" && strings.Contains(raw, value) {
			t.Fatalf("%s leaked sensitive value %q: %s", label, value, raw)
		}
	}
}

func assertIntegrationHardMatchAndAudit(
	t *testing.T,
	ctx context.Context,
	tx pgx.Tx,
	handler *api.EmployeeHandler,
	hrUserID uuid.UUID,
	f matchIntegrationFixtures,
	hasher identity.Hasher,
) {
	t.Helper()
	rec := invokeIntegrationMatch(handler, hrUserID, `{"national_id":"001/234_567"}`)
	resp := decodeIntegrationMatchResponse(t, rec, "hard NID")
	assertHardNIDCandidates(t, resp, f)

	nidHash, _, _ := hasher.HashNationalID("001234567")
	assertNoIntegrationLeak(t, "hard response", rec.Body.String(), "001-234-567", "001234567", "+84911111111", "1991-03-15", nidHash)
	assertPersistedMatchAudit(t, ctx, tx, hrUserID, api.MatchTierHard, "2", "001234567", nidHash, f.activeNIDEmployeeID.String(), f.terminatedNIDEmployeeID.String())

	recPhone := invokeIntegrationMatch(handler, hrUserID, `{"phone":"+84912345678"}`)
	phoneResp := decodeIntegrationMatchResponse(t, recPhone, "hard phone")
	if phoneResp.Tier != api.MatchTierHard || len(phoneResp.Candidates) != 1 || phoneResp.Candidates[0].EmployeeID != f.phoneEmployeeID {
		t.Fatalf("unexpected hard phone match response: %+v", phoneResp)
	}
}

func assertHardNIDCandidates(t *testing.T, resp api.MatchEmployeeResponse, f matchIntegrationFixtures) {
	t.Helper()
	if resp.Tier != api.MatchTierHard || len(resp.Candidates) != 2 {
		t.Fatalf("expected 2 hard NID candidates (active+terminated), got %+v", resp)
	}
	statuses := map[uuid.UUID]string{}
	for _, c := range resp.Candidates {
		statuses[c.EmployeeID] = c.Status
		if len(c.Reasons) != 1 || c.Reasons[0] != api.MatchReasonNationalIDExact {
			t.Fatalf("unexpected hard NID reasons: %+v", c.Reasons)
		}
	}
	if statuses[f.activeNIDEmployeeID] != api.MatchStatusActive || statuses[f.terminatedNIDEmployeeID] != api.MatchStatusTerminated {
		t.Fatalf("unexpected candidate statuses: %+v", statuses)
	}
}

func assertPersistedMatchAudit(
	t *testing.T,
	ctx context.Context,
	tx pgx.Tx,
	actorID uuid.UUID,
	wantTier, wantCount string,
	forbidden ...string,
) {
	t.Helper()
	var detailsRaw string
	if err := tx.QueryRow(ctx,
		`SELECT details::text FROM audit_logs WHERE actor_id = $1 AND action = $2 ORDER BY created_at DESC LIMIT 1`,
		actorID, api.MatchAuditAction,
	).Scan(&detailsRaw); err != nil {
		t.Fatalf("query persisted audit log: %v", err)
	}
	var details map[string]string
	if err := json.Unmarshal([]byte(detailsRaw), &details); err != nil {
		t.Fatalf("unmarshal audit details: %v", err)
	}
	if len(details) != 2 || details["outcome_tier"] != wantTier || details["candidate_count"] != wantCount {
		t.Fatalf("unexpected persisted audit details: %+v", details)
	}
	assertNoIntegrationLeak(t, "audit_logs row", detailsRaw, forbidden...)
}

func assertSingleSoftCandidate(t *testing.T, resp api.MatchEmployeeResponse, wantID uuid.UUID, wantReason string) {
	t.Helper()
	if resp.Tier != api.MatchTierSoft || len(resp.Candidates) != 1 {
		t.Fatalf("unexpected soft response: %+v", resp)
	}
	c := resp.Candidates[0]
	if c.EmployeeID != wantID || len(c.Reasons) != 1 || c.Reasons[0] != wantReason {
		t.Fatalf("soft candidate = %+v, want id=%s reason=%s", c, wantID, wantReason)
	}
}

func assertIntegrationSoftAndNoneMatches(
	t *testing.T,
	handler *api.EmployeeHandler,
	hrUserID uuid.UUID,
	f matchIntegrationFixtures,
) {
	t.Helper()
	// Unicode NFC input matches stored NFD full_name + DOB -> name_dob_review.
	dobResp := decodeIntegrationMatchResponse(t, invokeIntegrationMatch(handler, hrUserID, `{"full_name":"nguyễn thị cúc","date_of_birth":"1993-11-05"}`), "soft DOB")
	assertSingleSoftCandidate(t, dobResp, f.legacySoftEmployeeID, api.MatchReasonNameDOBReview)

	// Normalized full_name + partial phone matches legacy unhashed phone -> name_phone_review.
	partialResp := decodeIntegrationMatchResponse(t, invokeIntegrationMatch(handler, hrUserID, `{"full_name":"Nguyễn Thị Cúc","partial_phone":"777-666"}`), "soft partial phone")
	assertSingleSoftCandidate(t, partialResp, f.legacySoftEmployeeID, api.MatchReasonNamePhoneReview)

	// DOB alone must never hard-match or soft-match.
	recNone := invokeIntegrationMatch(handler, hrUserID, `{"date_of_birth":"1991-03-15"}`)
	if recNone.Code != http.StatusOK || strings.TrimSpace(recNone.Body.String()) != `{"tier":"none","candidates":[]}` {
		t.Fatalf("DOB alone = (%d, %s), want (200, none)", recNone.Code, recNone.Body.String())
	}
}

func assertIntegrationSoftLimitCap(t *testing.T, ctx context.Context, tx pgx.Tx, handler *api.EmployeeHandler, hrUserID uuid.UUID) {
	t.Helper()
	for i := 0; i < 7; i++ {
		insertMatchEmployee(t, ctx, tx, fmt.Sprintf("EMP-CAP-%02d", i), "Lê Văn Hùng", true, "", "", "1988-04-04")
	}
	rec := invokeIntegrationMatch(handler, hrUserID, `{"full_name":"lê văn hùng","date_of_birth":"1988-04-04"}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("soft cap status = %d, want 200", rec.Code)
	}
	var resp api.MatchEmployeeResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("decode soft cap: %v", err)
	}
	if resp.Tier != api.MatchTierSoft || len(resp.Candidates) != 5 {
		t.Fatalf("soft cap response = (tier=%q, count=%d), want (soft, 5)", resp.Tier, len(resp.Candidates))
	}
}

func assertIntegrationAuditFailClosed(
	t *testing.T,
	handler *api.EmployeeHandler,
	hrUserID uuid.UUID,
	f matchIntegrationFixtures,
) {
	t.Helper()
	broken := *handler
	broken.Audit = failingMatchAudit{}
	rec := invokeIntegrationMatch(&broken, hrUserID, `{"national_id":"001234567"}`)
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("audit failure status = %d, want 500", rec.Code)
	}
	if strings.Contains(rec.Body.String(), f.activeNIDEmployeeID.String()) || strings.Contains(rec.Body.String(), "EMP-ACT-01") {
		t.Fatalf("audit failure leaked candidate payload: %s", rec.Body.String())
	}
}
