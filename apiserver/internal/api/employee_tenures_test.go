package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type scopedEmployeeAuthorizer struct {
	perms       map[string]bool
	allowedDept *uuid.UUID
	global      bool
}

func (a scopedEmployeeAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return a.perms[permission]
}
func (a scopedEmployeeAuthorizer) HasScopedPermission(ctx context.Context, userID uuid.UUID, permission string, targetDept *uuid.UUID) bool {
	if !a.HasPermission(ctx, userID, permission) {
		return false
	}
	if a.global {
		return true
	}
	return targetDept != nil && a.allowedDept != nil && *targetDept == *a.allowedDept
}
func (scopedEmployeeAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (scopedEmployeeAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (scopedEmployeeAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}
func (scopedEmployeeAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (a scopedEmployeeAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return a.global }

type stubTenureCreator struct {
	called     bool
	employeeID uuid.UUID
	req        CreateEmployeeTenureRequest
	created    EmployeeTenureCreated
	err        error
}

func (s *stubTenureCreator) CreateEmployeeTenure(_ context.Context, empID uuid.UUID, req CreateEmployeeTenureRequest) (EmployeeTenureCreated, error) {
	s.called = true
	s.employeeID = empID
	s.req = req
	if s.err != nil {
		return EmployeeTenureCreated{}, s.err
	}
	return s.created, nil
}

func newCreateReq(body string, actorID uuid.UUID) *http.Request {
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees", bytes.NewBufferString(body))
	if actorID != uuid.Nil {
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	}
	return req
}

func newTenureReq(empIDPath, body string, actorID uuid.UUID) *http.Request {
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+empIDPath+"/tenures", bytes.NewBufferString(body))
	req.SetPathValue("employee_id", empIDPath)
	if actorID != uuid.Nil {
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	}
	return req
}

func TestEmployeeHandler_CreateNormalizesIdentityAndFailsClosedWithoutSecret(t *testing.T) {
	actorID := uuid.New()
	perms := map[string]bool{string(auth.PermissionEmployeesCreate): true}
	hasher, err := identity.NewHasher(testMatchSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}

	t.Run("normalizes and hashes optional identity fields without leaking audit PII", func(t *testing.T) {
		repo := &captureCreateRepo{}
		auditSink := &stubMatchAudit{}
		h := EmployeeHandler{
			Repository:     repo,
			IdentityHasher: hasher,
			Authorizer:     employeeSliceAuthorizer{perms: perms},
			Audit:          auditSink,
		}
		body := `{"full_name":"  Nguyễn Văn A ","employee_id_number":" EMP-201 ","hire_date":"2026-01-15","phone":"+84 912-345-678","date_of_birth":"1994-05-10","national_id":" 001-234.567 "}`
		rec := httptest.NewRecorder()
		h.Create(rec, newCreateReq(body, actorID))
		if rec.Code != http.StatusCreated {
			t.Fatalf("status = %d, want 201; body=%s", rec.Code, rec.Body.String())
		}
		assertCapturedCreateIdentity(t, repo.request, auditSink)
	})

	t.Run("identity fields fail closed with 503 when hmac secret missing", func(t *testing.T) {
		repo := &captureCreateRepo{}
		h := EmployeeHandler{Repository: repo, Authorizer: employeeSliceAuthorizer{perms: perms}}
		rec := httptest.NewRecorder()
		h.Create(rec, newCreateReq(`{"full_name":"New Emp","employee_id_number":"EMP-202","hire_date":"2026-01-15","national_id":"001234567"}`, actorID))
		if rec.Code != http.StatusServiceUnavailable {
			t.Fatalf("status = %d, want 503", rec.Code)
		}
	})

	t.Run("rejects invalid identity formats and unknown fields", func(t *testing.T) {
		h := EmployeeHandler{Repository: &captureCreateRepo{}, IdentityHasher: hasher, Authorizer: employeeSliceAuthorizer{perms: perms}}
		for _, body := range []string{
			`{"full_name":"A","employee_id_number":"E1","hire_date":"not-a-date"}`,
			`{"full_name":"A","employee_id_number":"E1","hire_date":"2026-01-15","phone":"0912345678"}`,
			`{"full_name":"A","employee_id_number":"E1","hire_date":"2026-01-15","date_of_birth":"1995-02-30"}`,
			`{"full_name":"A","employee_id_number":"E1","hire_date":"2026-01-15","unknown_field":"x"}`,
		} {
			rec := httptest.NewRecorder()
			h.Create(rec, newCreateReq(body, actorID))
			if rec.Code != http.StatusBadRequest {
				t.Fatalf("body=%s status=%d, want 400", body, rec.Code)
			}
		}
	})
}

func assertStringPtrEq(t *testing.T, label string, got *string, want string) {
	t.Helper()
	if got == nil || *got != want {
		t.Fatalf("%s = %v, want %q", label, got, want)
	}
}

func assertHashPtrValid(t *testing.T, label string, got *string) string {
	t.Helper()
	if got == nil || len(*got) != 64 {
		t.Fatalf("%s = %v, want 64-char hash", label, got)
	}
	return *got
}

func assertCapturedCreateIdentity(t *testing.T, req CreateEmployeeRequest, auditSink *stubMatchAudit) {
	t.Helper()
	assertStringPtrEq(t, "phone", req.Phone, "+84912345678")
	phoneHash := assertHashPtrValid(t, "phone_hash", req.PhoneLookupHash)
	assertStringPtrEq(t, "national_id", req.NationalID, "001-234.567")
	nidHash := assertHashPtrValid(t, "national_id_hash", req.NationalIDLookupHash)
	assertStringPtrEq(t, "date_of_birth", req.DateOfBirth, "1994-05-10")
	rawAudit, _ := json.Marshal(auditSink.events)
	assertNoForbiddenSubstrings(t, "create audit", string(rawAudit), "+84912345678", "001-234.567", "1994-05-10", phoneHash, nidHash)
}

func TestEmployeeHandler_CreateTenureAuthAndScope(t *testing.T) {
	empID := uuid.New()
	employerID := uuid.New()
	deptAllowed := uuid.New()
	deptDenied := uuid.New()
	actorID := uuid.New()
	validBody := `{"start_date":"2026-07-01","employer_id":"` + employerID.String() + `"}`

	creator := &stubTenureCreator{}
	h := EmployeeHandler{
		TenureCreator: creator,
		Authorizer:    scopedEmployeeAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesRehire): true}, global: true},
	}
	rec401 := httptest.NewRecorder()
	h.CreateTenure(rec401, newTenureReq(empID.String(), validBody, uuid.Nil))
	if rec401.Code != http.StatusUnauthorized {
		t.Fatalf("unauthenticated status = %d, want 401", rec401.Code)
	}

	hNoPerm := EmployeeHandler{
		TenureCreator: creator,
		Authorizer:    scopedEmployeeAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesCreate): true}, global: true},
	}
	rec403 := httptest.NewRecorder()
	hNoPerm.CreateTenure(rec403, newTenureReq(empID.String(), validBody, actorID))
	if rec403.Code != http.StatusForbidden {
		t.Fatalf("missing employees:rehire status = %d, want 403", rec403.Code)
	}

	hScoped := EmployeeHandler{
		TenureCreator: creator,
		Authorizer:    scopedEmployeeAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesRehire): true}, allowedDept: &deptAllowed},
	}
	recScopeDenied := httptest.NewRecorder()
	hScoped.CreateTenure(recScopeDenied, newTenureReq(empID.String(), `{"start_date":"2026-07-01","employer_id":"`+employerID.String()+`","department_id":"`+deptDenied.String()+`"}`, actorID))
	if recScopeDenied.Code != http.StatusForbidden {
		t.Fatalf("out-of-scope department status = %d, want 403", recScopeDenied.Code)
	}
	if creator.called {
		t.Fatal("denied requests must not call TenureCreator")
	}
}

func TestEmployeeHandler_CreateTenureValidation(t *testing.T) {
	empID := uuid.New()
	employerID := uuid.New()
	actorID := uuid.New()
	authz := scopedEmployeeAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesRehire): true}, global: true}

	h := EmployeeHandler{TenureCreator: &stubTenureCreator{}, Authorizer: authz}
	cases := []struct {
		empPath string
		body    string
	}{
		{"not-a-uuid", `{"start_date":"2026-07-01","employer_id":"` + employerID.String() + `"}`},
		{empID.String(), `{"start_date":"2026-07-01","employer_id":"` + employerID.String() + `","employee_code":"MUTATE"}`},
		{empID.String(), `{"start_date":"2026-02-30","employer_id":"` + employerID.String() + `"}`},
		{empID.String(), `{"start_date":"2026-07-01"}`},
		{empID.String(), `{"start_date":"2026-07-01","employer_id":"` + employerID.String() + `","expected_status":"bogus"}`},
	}
	for _, tc := range cases {
		rec := httptest.NewRecorder()
		h.CreateTenure(rec, newTenureReq(tc.empPath, tc.body, actorID))
		if rec.Code != http.StatusBadRequest {
			t.Fatalf("path=%s body=%s status=%d, want 400", tc.empPath, tc.body, rec.Code)
		}
	}
}

func TestEmployeeHandler_CreateTenureOutcomes(t *testing.T) {
	empID := uuid.New()
	employerID := uuid.New()
	actorID := uuid.New()
	authz := scopedEmployeeAuthorizer{perms: map[string]bool{string(auth.PermissionEmployeesRehire): true}, global: true}

	t.Run("expected_status active and repo conflict/not-found return deterministic errors", func(t *testing.T) {
		hConflict := EmployeeHandler{TenureCreator: &stubTenureCreator{err: ErrConflict}, Authorizer: authz}
		recActive := httptest.NewRecorder()
		hConflict.CreateTenure(recActive, newTenureReq(empID.String(), `{"start_date":"2026-07-01","employer_id":"`+employerID.String()+`","expected_status":"active"}`, actorID))
		if recActive.Code != http.StatusConflict {
			t.Fatalf("expected_status active = %d, want 409", recActive.Code)
		}

		recRepoConflict := httptest.NewRecorder()
		hConflict.CreateTenure(recRepoConflict, newTenureReq(empID.String(), `{"start_date":"2026-07-01","employer_id":"`+employerID.String()+`"}`, actorID))
		if recRepoConflict.Code != http.StatusConflict {
			t.Fatalf("repo conflict = %d, want 409", recRepoConflict.Code)
		}

		hNotFound := EmployeeHandler{TenureCreator: &stubTenureCreator{err: ErrNotFound}, Authorizer: authz}
		rec404 := httptest.NewRecorder()
		hNotFound.CreateTenure(rec404, newTenureReq(empID.String(), `{"start_date":"2026-07-01","employer_id":"`+employerID.String()+`"}`, actorID))
		if rec404.Code != http.StatusNotFound {
			t.Fatalf("repo not found = %d, want 404", rec404.Code)
		}
	})

	t.Run("success returns 201 and emits sanitized audit", func(t *testing.T) {
		expected := EmployeeTenureCreated{
			TenureID:       uuid.New(),
			EmployeeID:     empID,
			EmployeeCode:   "EMP-001",
			StartDate:      "2026-07-01",
			RelationshipID: uuid.New(),
			EmploymentID:   uuid.New(),
			EmployerID:     employerID,
			Status:         MatchStatusActive,
		}
		creator := &stubTenureCreator{created: expected}
		auditSink := &stubMatchAudit{}
		h := EmployeeHandler{TenureCreator: creator, Authorizer: authz, Audit: auditSink}
		rec := httptest.NewRecorder()
		h.CreateTenure(rec, newTenureReq(empID.String(), `{"start_date":"2026-07-01","employer_id":"`+employerID.String()+`","expected_employee_code":"EMP-001","expected_status":"terminated"}`, actorID))
		if rec.Code != http.StatusCreated {
			t.Fatalf("status = %d, want 201; body=%s", rec.Code, rec.Body.String())
		}
		var got EmployeeTenureCreated
		if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil {
			t.Fatalf("decode: %v", err)
		}
		if got != expected {
			t.Fatalf("got %+v, want %+v", got, expected)
		}
		if len(auditSink.events) != 1 || auditSink.events[0].Name != TenureCreatedAuditAction {
			t.Fatalf("unexpected audit events: %+v", auditSink.events)
		}
	})
}

func TestEmployeeTenures_OpenAPIContract(t *testing.T) {
	raw, err := os.ReadFile(filepath.Join("..", "..", "openapi.yaml"))
	if err != nil {
		t.Fatalf("read openapi.yaml: %v", err)
	}
	spec := string(raw)
	for _, required := range []string{
		`"/api/v1/employees/{employee_id}/tenures":`,
		`operationId: post_api_v1_employees_employee_id_tenures`,
		`$ref: '#/components/schemas/CreateEmployeeTenureRequest'`,
		`$ref: '#/components/schemas/EmployeeTenureCreated'`,
		`CreateEmployeeTenureRequest:`,
		`EmployeeTenureCreated:`,
	} {
		if !strings.Contains(spec, required) {
			t.Fatalf("openapi.yaml missing tenure contract clause %q", required)
		}
	}
}
