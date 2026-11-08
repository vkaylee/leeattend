package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockRecruitmentRepo struct {
	sources map[uuid.UUID]RecruitmentSource
	err     error
}

func newMockRecruitmentRepo() *mockRecruitmentRepo {
	return &mockRecruitmentRepo{sources: make(map[uuid.UUID]RecruitmentSource)}
}

func (m *mockRecruitmentRepo) ListRecruitmentSources(_ context.Context, _ uuid.UUID, page, limit int) (RecruitmentSourceListResponse, error) {
	if m.err != nil {
		return RecruitmentSourceListResponse{}, m.err
	}
	items := make([]RecruitmentSource, 0, len(m.sources))
	for _, s := range m.sources {
		items = append(items, s)
	}
	return RecruitmentSourceListResponse{Data: items, Total: int64(len(items)), Page: page, Limit: limit}, nil
}

func (m *mockRecruitmentRepo) CreateRecruitmentSource(_ context.Context, wsID uuid.UUID, req CreateRecruitmentSourceRequest) (RecruitmentSource, error) {
	if m.err != nil {
		return RecruitmentSource{}, m.err
	}
	id := uuid.New()
	s := RecruitmentSource{
		ID:                  id,
		WorkspaceID:         wsID,
		SourceType:          req.SourceType,
		Name:                req.Name,
		ReferrerEmployeeID:  req.ReferrerEmployeeID,
		ReferralBonusAmount: req.ReferralBonusAmount,
		AgencyContactName:   req.AgencyContactName,
		AgencyContactEmail:  req.AgencyContactEmail,
		AgencyFeeAmount:     req.AgencyFeeAmount,
		Notes:               req.Notes,
		CreatedAt:           "2026-06-16T00:00:00Z",
		UpdatedAt:           "2026-06-16T00:00:00Z",
	}
	m.sources[id] = s
	return s, nil
}

func (m *mockRecruitmentRepo) UpdateRecruitmentSource(_ context.Context, _, id uuid.UUID, req UpdateRecruitmentSourceRequest) (RecruitmentSource, error) {
	if m.err != nil {
		return RecruitmentSource{}, m.err
	}
	s, ok := m.sources[id]
	if !ok {
		return RecruitmentSource{}, ErrRecruitmentSourceNotFound
	}
	if req.Name != nil {
		s.Name = *req.Name
	}
	if req.ReferralBonusPaid != nil {
		s.ReferralBonusPaid = *req.ReferralBonusPaid
	}
	if req.AgencyFeePaid != nil {
		s.AgencyFeePaid = *req.AgencyFeePaid
	}
	m.sources[id] = s
	return s, nil
}

func (m *mockRecruitmentRepo) DeleteRecruitmentSource(_ context.Context, _, id uuid.UUID) error {
	if m.err != nil {
		return m.err
	}
	if _, ok := m.sources[id]; !ok {
		return ErrRecruitmentSourceNotFound
	}
	delete(m.sources, id)
	return nil
}

func (m *mockRecruitmentRepo) RecruitmentSourceBreakdown(_ context.Context, _ uuid.UUID) ([]RecruitmentSourceBreakdown, error) {
	if m.err != nil {
		return nil, m.err
	}
	return []RecruitmentSourceBreakdown{
		{SourceType: "direct", SourceName: "Direct Hire", Count: 10},
		{SourceType: "referral", SourceName: "Employee Referral", Count: 5},
	}, nil
}

type mockRecruitmentAuthz struct {
	isMember    bool
	permissions map[string]bool
}

func (a mockRecruitmentAuthz) IsMember(_ context.Context, _, _ uuid.UUID) (bool, error) {
	return a.isMember, nil
}
func (a mockRecruitmentAuthz) EffectivePermissions(_ context.Context, _, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (a mockRecruitmentAuthz) HasPermission(_ context.Context, _, _ uuid.UUID, perm string) bool {
	return a.permissions[perm]
}

func recruitmentRequest(method, path, body string, wsID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	return req
}

func TestRecruitmentSourceHandlerList(t *testing.T) {
	wsID := uuid.New()
	repo := newMockRecruitmentRepo()
	id := uuid.New()
	repo.sources[id] = RecruitmentSource{ID: id, WorkspaceID: wsID, Name: "Campus Hiring", SourceType: "direct"}
	h := RecruitmentSourceHandler{
		Repository: repo,
		Authorizer: mockRecruitmentAuthz{isMember: true},
	}

	req := recruitmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources", "", wsID)
	rec := httptest.NewRecorder()
	h.List(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestRecruitmentSourceHandlerCreateValidationAndPermission(t *testing.T) {
	wsID := uuid.New()
	repo := newMockRecruitmentRepo()

	// Forbidden when recruitment:manage missing
	hNoPerm := RecruitmentSourceHandler{
		Repository: repo,
		Authorizer: mockRecruitmentAuthz{isMember: true, permissions: map[string]bool{}},
	}
	req := recruitmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources", `{"source_type":"direct","name":"Job Fair"}`, wsID)
	rec := httptest.NewRecorder()
	hNoPerm.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 when lacking recruitment:manage, got %d", rec.Code)
	}

	// Permitted
	h := RecruitmentSourceHandler{
		Repository: repo,
		Authorizer: mockRecruitmentAuthz{isMember: true, permissions: map[string]bool{"recruitment:manage": true}},
	}

	// Invalid source_type
	req = recruitmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources", `{"source_type":"unknown","name":"Job Fair"}`, wsID)
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid source_type, got %d", rec.Code)
	}

	// Missing name
	req = recruitmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources", `{"source_type":"direct","name":""}`, wsID)
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty name, got %d", rec.Code)
	}

	// Success
	req = recruitmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources", `{"source_type":"direct","name":"Job Fair"}`, wsID)
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestRecruitmentSourceHandlerUpdate(t *testing.T) {
	wsID := uuid.New()
	repo := newMockRecruitmentRepo()
	id := uuid.New()
	repo.sources[id] = RecruitmentSource{ID: id, WorkspaceID: wsID, Name: "Headhunter Agency", SourceType: "agency"}

	h := RecruitmentSourceHandler{
		Repository: repo,
		Authorizer: mockRecruitmentAuthz{isMember: true, permissions: map[string]bool{"recruitment:manage": true}},
	}

	// Success
	req := recruitmentRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources/"+id.String(), `{"name":"Elite Staffing","agency_fee_paid":true}`, wsID)
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// Not Found
	missingID := uuid.New()
	req = recruitmentRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources/"+missingID.String(), `{"name":"Elite Staffing"}`, wsID)
	req.SetPathValue("id", missingID.String())
	rec = httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestRecruitmentSourceHandlerDelete(t *testing.T) {
	wsID := uuid.New()
	repo := newMockRecruitmentRepo()
	id := uuid.New()
	repo.sources[id] = RecruitmentSource{ID: id, WorkspaceID: wsID, Name: "Agency", SourceType: "agency"}

	h := RecruitmentSourceHandler{
		Repository: repo,
		Authorizer: mockRecruitmentAuthz{isMember: true, permissions: map[string]bool{"recruitment:manage": true}},
	}

	// Success
	req := recruitmentRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources/"+id.String(), "", wsID)
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Delete(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}

	// Not Found on second delete
	req = recruitmentRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/recruitment-sources/"+id.String(), "", wsID)
	req.SetPathValue("id", id.String())
	rec = httptest.NewRecorder()
	h.Delete(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 on repeat delete, got %d", rec.Code)
	}
}

func TestRecruitmentSourceHandlerDashboard(t *testing.T) {
	wsID := uuid.New()
	repo := newMockRecruitmentRepo()

	// Forbidden without dashboard:read
	hNoPerm := RecruitmentSourceHandler{
		Repository: repo,
		Authorizer: mockRecruitmentAuthz{isMember: true, permissions: map[string]bool{}},
	}
	req := recruitmentRequest(http.MethodGet, "/api/v1/dashboard/recruitment-sources", "", wsID)
	rec := httptest.NewRecorder()
	hNoPerm.Dashboard(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without dashboard:read, got %d", rec.Code)
	}

	// Allowed with dashboard:read
	h := RecruitmentSourceHandler{
		Repository: repo,
		Authorizer: mockRecruitmentAuthz{isMember: true, permissions: map[string]bool{"dashboard:read": true}},
	}
	req = recruitmentRequest(http.MethodGet, "/api/v1/dashboard/recruitment-sources", "", wsID)
	rec = httptest.NewRecorder()
	h.Dashboard(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}
