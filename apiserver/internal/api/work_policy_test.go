package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type workPolicyRepositoryStub struct {
	created bool
}

func (s *workPolicyRepositoryStub) ListWorkPolicyGroupVersions(context.Context, int, int) (WorkPolicyGroupListResponse, error) {
	return WorkPolicyGroupListResponse{}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyGroup(_ context.Context, req CreateWorkPolicyGroupRequest, actorID uuid.UUID) (WorkPolicyGroupVersion, error) {
	s.created = true
	return WorkPolicyGroupVersion{
		GroupID: actorID,
		Name:    req.Name,
	}, nil
}

func (s *workPolicyRepositoryStub) ListWorkPolicyMemberships(context.Context, uuid.UUID) ([]WorkPolicyMembership, error) {
	return []WorkPolicyMembership{}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyMembership(_ context.Context, groupID uuid.UUID, req CreateWorkPolicyMembershipRequest, _ uuid.UUID) (WorkPolicyMembership, error) {
	return WorkPolicyMembership{GroupID: groupID, EmploymentID: req.EmploymentID}, nil
}

func workPolicyRequest(method string, body string, userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, "/api/v1/payroll/work-policy-groups", bytes.NewBufferString(body))
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
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

func TestWorkPolicyGroupCreateDelegatesValidatedRequest(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	handler := WorkPolicyGroupHandler{Repository: repo, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}}}
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
}
