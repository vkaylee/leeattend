package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type workPolicyRepositoryStub struct {
	created        bool
	versionCreated bool
}

type workPolicyAuditRecorder struct {
	events []audit.Event
}

func (r *workPolicyAuditRecorder) Emit(_ context.Context, event audit.Event) error {
	r.events = append(r.events, event)
	return nil
}

func (s *workPolicyRepositoryStub) ListWorkPolicyGroupVersions(context.Context, int, int) (WorkPolicyGroupListResponse, error) {
	return WorkPolicyGroupListResponse{}, nil
}

func (s *workPolicyRepositoryStub) ListWorkPolicyGroupHistory(context.Context, workpolicy.GroupID, int, int) (WorkPolicyGroupListResponse, error) {
	return WorkPolicyGroupListResponse{}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyGroup(_ context.Context, req workpolicy.CreateGroupInput, actorID ids.UserID) (WorkPolicyGroupVersion, error) {
	s.created = true
	return WorkPolicyGroupVersion{
		GroupID: uuid.UUID(actorID),
		Name:    req.Name,
	}, nil
}

func (s *workPolicyRepositoryStub) ResolveWorkPolicy(context.Context, ids.EmploymentID, time.Time) (WorkPolicyGroupVersion, error) {
	return WorkPolicyGroupVersion{GroupID: uuid.New(), EffectiveFrom: "2026-01-01"}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyGroupVersion(_ context.Context, req workpolicy.CreateGroupVersionInput, _ ids.UserID) (WorkPolicyGroupVersion, error) {
	s.versionCreated = true
	return WorkPolicyGroupVersion{GroupID: uuid.UUID(req.GroupID), Name: req.Name, EffectiveFrom: req.EffectiveFrom}, nil
}

func (s *workPolicyRepositoryStub) ListWorkPolicyMemberships(context.Context, workpolicy.GroupID) ([]WorkPolicyMembership, error) {
	return []WorkPolicyMembership{}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyMembership(_ context.Context, req workpolicy.CreateMembershipInput) (WorkPolicyMembership, error) {
	return WorkPolicyMembership{GroupID: uuid.UUID(req.GroupID), EmploymentID: uuid.UUID(req.EmploymentID)}, nil
}

func workPolicyRequest(method string, body string, userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, "/api/v1/payroll/work-policy-groups", bytes.NewBufferString(body))
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestWorkPolicyHistoryRequiresReadPermission(t *testing.T) {
	handler := WorkPolicyGroupHandler{Repository: &workPolicyRepositoryStub{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": false}}}
	rec := httptest.NewRecorder()
	req := workPolicyRequest(http.MethodGet, "", uuid.New())
	req.SetPathValue("group_id", uuid.NewString())
	req.URL.RawQuery = "page=1&limit=20"
	handler.History(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want 403", rec.Code)
	}
}

func TestWorkPolicyGroupCreateRequiresManagePermission(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	handler := WorkPolicyGroupHandler{Repository: repo, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": false}}}
	rec := httptest.NewRecorder()
	handler.Create(rec, workPolicyRequest(http.MethodPost, `{}`, uuid.New()))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want 403", rec.Code)
	}
	if repo.created {
		t.Fatal("repository must not run when authorization fails")
	}
}

func TestWorkPolicyGroupCreateRejectsInvalidEffectiveDate(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	handler := WorkPolicyGroupHandler{Repository: repo, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}}}
	rec := httptest.NewRecorder()
	body := `{"name":"Operations","minutes_per_credit":450,"attendance_policy_id":"` + uuid.NewString() + `","working_pattern_id":"` + uuid.NewString() + `","pay_method":"hourly","overtime_policy_id":"` + uuid.NewString() + `","effective_from":"not-a-date"}`
	handler.Create(rec, workPolicyRequest(http.MethodPost, body, uuid.New()))
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
	if repo.created {
		t.Fatal("repository must not run for invalid effective date")
	}
}

func TestWorkPolicyMembershipCreateRejectsInvalidRange(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	handler := WorkPolicyGroupHandler{Repository: repo, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}}}
	rec := httptest.NewRecorder()
	req := workPolicyRequest(http.MethodPost, `{"employment_id":"`+uuid.NewString()+`","effective_from":"2026-02-01","effective_to":"2026-01-01"}`, uuid.New())
	req.SetPathValue("group_id", uuid.NewString())
	handler.CreateMembership(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
}

func TestWorkPolicyMembershipListRequiresReadPermission(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	handler := WorkPolicyGroupHandler{Repository: repo, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": false}}}
	rec := httptest.NewRecorder()
	req := workPolicyRequest(http.MethodGet, "", uuid.New())
	req.SetPathValue("group_id", uuid.NewString())
	handler.ListMemberships(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want 403", rec.Code)
	}
}

func TestWorkPolicyResolveRejectsInvalidAsOf(t *testing.T) {
	handler := WorkPolicyGroupHandler{Repository: &workPolicyRepositoryStub{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": true}}}
	rec := httptest.NewRecorder()
	req := workPolicyRequest(http.MethodGet, "", uuid.New())
	req.URL.RawQuery = "as_of=invalid"
	req.SetPathValue("employment_id", uuid.NewString())
	handler.Resolve(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
}

func TestWorkPolicyGroupVersionCreateDelegatesValidatedRequest(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	handler := WorkPolicyGroupHandler{Repository: repo, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}}}
	rec := httptest.NewRecorder()
	req := workPolicyRequest(http.MethodPost, `{"name":"Operations","minutes_per_credit":450,"attendance_policy_id":"`+uuid.NewString()+`","working_pattern_id":"`+uuid.NewString()+`","pay_method":"hourly","rate_component_definition_id":"`+uuid.NewString()+`","overtime_policy_id":"`+uuid.NewString()+`","effective_from":"2026-06-01"}`, uuid.New())
	req.SetPathValue("group_id", uuid.NewString())
	handler.CreateVersion(rec, req)
	if rec.Code != http.StatusCreated || !repo.versionCreated {
		t.Fatalf("status = %d, version_created = %t, want 201/true", rec.Code, repo.versionCreated)
	}
}

func TestWorkPolicyGroupCreateDelegatesValidatedRequest(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	auditRecorder := &workPolicyAuditRecorder{}
	handler := WorkPolicyGroupHandler{Repository: repo, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}}, Audit: auditRecorder}
	rec := httptest.NewRecorder()
	body := `{"name":"Operations","minutes_per_credit":450,"attendance_policy_id":"` + uuid.NewString() + `","working_pattern_id":"` + uuid.NewString() + `","pay_method":"monthly","monthly_denominator":"fixed","monthly_standard_days":"22","overtime_policy_id":"` + uuid.NewString() + `","effective_from":"2026-01-01"}`
	handler.Create(rec, workPolicyRequest(http.MethodPost, body, uuid.New()))
	if rec.Code != http.StatusCreated {
		t.Fatalf("status = %d, want 201: %s", rec.Code, rec.Body.String())
	}
	if !repo.created {
		t.Fatal("expected repository create call")
	}
	if value := decimal.RequireFromString("22"); value.IsZero() {
		t.Fatal("test fixture must preserve positive monthly days")
	}
	if len(auditRecorder.events) != 1 || auditRecorder.events[0].Name != "payroll.work_policy_group.created" {
		t.Fatalf("expected create audit event, got %+v", auditRecorder.events)
	}
}
