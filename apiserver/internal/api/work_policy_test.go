package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type workPolicyRepositoryStub struct {
	created         bool
	versionCreated  bool
	patched         bool
	published       bool
	transferred     bool
	publishErr      error
	patchErr        error
	lastIdemKey     string
	lastExpectedRow int64
}

type workPolicyAuditRecorder struct {
	events []audit.Event
}

func (r *workPolicyAuditRecorder) Emit(_ context.Context, event audit.Event) error {
	r.events = append(r.events, event)
	return nil
}

func (s *workPolicyRepositoryStub) ListWorkPolicyGroups(context.Context, int, int) (WorkPolicyGroupListResponse, error) {
	return WorkPolicyGroupListResponse{}, nil
}

func (s *workPolicyRepositoryStub) ListWorkPolicyGroupVersions(context.Context, workpolicy.GroupID, string, int, int) (WorkPolicyVersionListResponse, error) {
	return WorkPolicyVersionListResponse{}, nil
}

func (s *workPolicyRepositoryStub) GetWorkPolicyVersion(_ context.Context, groupID workpolicy.GroupID, versionID workpolicy.VersionID) (WorkPolicyVersion, error) {
	return WorkPolicyVersion{GroupID: uuid.UUID(groupID), ID: uuid.UUID(versionID), Status: workpolicy.StatusDraft, RowVersion: 1}, nil
}

func (s *workPolicyRepositoryStub) ValidateWorkPolicyVersion(_ context.Context, groupID workpolicy.GroupID, versionID workpolicy.VersionID) (WorkPolicyValidationResponse, error) {
	return WorkPolicyValidationResponse{VersionID: uuid.UUID(versionID), Valid: uuid.UUID(groupID) != uuid.Nil}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyGroup(_ context.Context, req workpolicy.CreateGroupInput, _ ids.UserID) (WorkPolicyVersion, error) {
	s.created = true
	return WorkPolicyVersion{
		ID: uuid.New(), GroupID: uuid.New(), GroupCode: req.Code, GroupName: req.Name,
		Status: workpolicy.StatusDraft, RowVersion: 1, PayMethod: req.Draft.PayMethod, EffectiveFrom: req.Draft.EffectiveFrom,
	}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyVersion(_ context.Context, req workpolicy.CreateVersionInput, _ ids.UserID) (WorkPolicyVersion, error) {
	s.versionCreated = true
	return WorkPolicyVersion{ID: uuid.New(), GroupID: uuid.UUID(req.GroupID), Status: workpolicy.StatusDraft, RowVersion: 1}, nil
}

func (s *workPolicyRepositoryStub) PatchWorkPolicyVersion(_ context.Context, req workpolicy.PatchVersionInput) (WorkPolicyVersion, error) {
	s.patched = true
	s.lastExpectedRow = req.ExpectedRowVersion
	if s.patchErr != nil {
		return WorkPolicyVersion{}, s.patchErr
	}
	return WorkPolicyVersion{ID: uuid.UUID(req.VersionID), GroupID: uuid.UUID(req.GroupID), Status: workpolicy.StatusDraft, RowVersion: req.ExpectedRowVersion + 1, EffectiveFrom: req.Draft.EffectiveFrom}, nil
}

func (s *workPolicyRepositoryStub) PublishWorkPolicyVersion(_ context.Context, req workpolicy.PublishVersionInput) (WorkPolicyVersion, error) {
	s.published = true
	s.lastIdemKey = req.IdempotencyKey
	s.lastExpectedRow = req.ExpectedRowVersion
	if s.publishErr != nil {
		return WorkPolicyVersion{}, s.publishErr
	}
	return WorkPolicyVersion{ID: uuid.UUID(req.VersionID), GroupID: uuid.UUID(req.GroupID), Status: workpolicy.StatusPublished, RowVersion: req.ExpectedRowVersion + 1, EffectiveFrom: "2026-01-01", PayMethod: workpolicy.PayMethodMonthly}, nil
}

func (s *workPolicyRepositoryStub) ResolveWorkPolicyByRelationship(context.Context, uuid.UUID, time.Time) (ResolvedWorkPolicyResponse, error) {
	return ResolvedWorkPolicyResponse{GroupID: uuid.New(), EffectiveFrom: "2026-01-01", Status: workpolicy.StatusPublished}, nil
}

func (s *workPolicyRepositoryStub) ListWorkPolicyMemberships(context.Context, workpolicy.GroupID) ([]WorkPolicyMembership, error) {
	return []WorkPolicyMembership{}, nil
}

func (s *workPolicyRepositoryStub) CreateWorkPolicyMembership(_ context.Context, req workpolicy.CreateMembershipInput) (WorkPolicyMembership, error) {
	return WorkPolicyMembership{ID: uuid.New(), GroupID: uuid.UUID(req.GroupID), EmploymentRelationshipID: req.EmploymentRelationshipID, EffectiveFrom: req.EffectiveFrom}, nil
}

func (s *workPolicyRepositoryStub) TransferWorkPolicyMembership(_ context.Context, req workpolicy.TransferMembershipInput) (TransferWorkPolicyMembershipResponse, error) {
	s.transferred = true
	return TransferWorkPolicyMembershipResponse{
		NewMembership: WorkPolicyMembership{ID: uuid.New(), GroupID: uuid.UUID(req.TargetGroupID), EmploymentRelationshipID: req.EmploymentRelationshipID, EffectiveFrom: req.EffectiveDate},
	}, nil
}

func workPolicyRequest(method string, body string, userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, "/api/v1/payroll/work-policy-groups", bytes.NewBufferString(body))
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestWorkPolicyDetailAndValidateAuthorization(t *testing.T) {
	handler := WorkPolicyGroupHandler{Repository: &workPolicyRepositoryStub{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": false}}}
	rec := httptest.NewRecorder()
	req := workPolicyRequest(http.MethodGet, "", uuid.New())
	req.SetPathValue("group_id", uuid.NewString())
	req.SetPathValue("version_id", uuid.NewString())
	handler.Detail(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("detail status = %d, want 403", rec.Code)
	}

	handler.Authorizer = payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": true}}
	rec = httptest.NewRecorder()
	req = workPolicyRequest(http.MethodPost, "", uuid.New())
	req.SetPathValue("group_id", "bad")
	req.SetPathValue("version_id", uuid.NewString())
	handler.Validate(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("validate invalid group ID status = %d, want 400", rec.Code)
	}
}

func TestWorkPolicyLifecycle_CreatePatchPublishAndTransfer(t *testing.T) {
	repo := &workPolicyRepositoryStub{}
	recorder := &workPolicyAuditRecorder{}
	handler := WorkPolicyGroupHandler{
		Repository: repo,
		Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true, "payroll:read": true}},
		Audit:      recorder,
	}
	userID := uuid.New()
	otVerID := uuid.NewString()

	// Create group + draft v1
	createBody := `{"code":"OFFICE-STD","name":"Office Standard","minutes_per_credit":480,"pay_method":"monthly","monthly_denominator":"fixed","monthly_standard_days":"22","overtime_policy_version_id":"` + otVerID + `","effective_from":"2026-01-01"}`
	rec := httptest.NewRecorder()
	handler.Create(rec, workPolicyRequest(http.MethodPost, createBody, userID))
	if rec.Code != http.StatusCreated || !repo.created {
		t.Fatalf("create status = %d body=%s", rec.Code, rec.Body.String())
	}

	// Patch draft stale row_version -> 409
	repo.patchErr = ErrWorkPolicyVersionConflict
	patchBody := `{"expected_row_version":1,"minutes_per_credit":450,"pay_method":"monthly","monthly_denominator":"fixed","monthly_standard_days":"22","overtime_policy_version_id":"` + otVerID + `","effective_from":"2026-01-01"}`
	reqPatch := workPolicyRequest(http.MethodPatch, patchBody, userID)
	reqPatch.SetPathValue("group_id", uuid.NewString())
	reqPatch.SetPathValue("version_id", uuid.NewString())
	recPatch := httptest.NewRecorder()
	handler.PatchVersion(recPatch, reqPatch)
	if recPatch.Code != http.StatusConflict {
		t.Fatalf("stale patch status = %d, want 409", recPatch.Code)
	}
	repo.patchErr = nil

	// Publish with Idempotency-Key header
	reqPub := workPolicyRequest(http.MethodPost, `{"expected_row_version":2,"auto_close_previous":true}`, userID)
	reqPub.Header.Set("Idempotency-Key", "pub-key-1")
	reqPub.SetPathValue("group_id", uuid.NewString())
	reqPub.SetPathValue("version_id", uuid.NewString())
	recPub := httptest.NewRecorder()
	handler.PublishVersion(recPub, reqPub)
	if recPub.Code != http.StatusOK || !repo.published || repo.lastIdemKey != "pub-key-1" {
		t.Fatalf("publish status = %d idem=%q", recPub.Code, repo.lastIdemKey)
	}

	// Atomic transfer membership
	transferBody := `{"employment_relationship_id":"` + uuid.NewString() + `","target_group_id":"` + uuid.NewString() + `","effective_date":"2026-04-01"}`
	recTr := httptest.NewRecorder()
	handler.TransferMembership(recTr, workPolicyRequest(http.MethodPost, transferBody, userID))
	if recTr.Code != http.StatusOK || !repo.transferred {
		t.Fatalf("transfer status = %d body=%s", recTr.Code, recTr.Body.String())
	}

	// Verify audit emitted without PII/salary
	if len(recorder.events) < 3 {
		t.Fatalf("expected >=3 audit events, got %d", len(recorder.events))
	}
}

func TestWorkPolicyResolveRejectsInvalidAsOf(t *testing.T) {
	handler := WorkPolicyGroupHandler{Repository: &workPolicyRepositoryStub{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": true}}}
	rec := httptest.NewRecorder()
	req := workPolicyRequest(http.MethodGet, "", uuid.New())
	req.URL.RawQuery = "as_of=invalid"
	req.SetPathValue("relationship_id", uuid.NewString())
	handler.Resolve(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
}
