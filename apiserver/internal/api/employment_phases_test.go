package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockEmploymentPhaseRepo struct {
	definitions       []EmploymentPhaseDefinition
	phases            map[uuid.UUID][]EmployeePhase
	organizationQuery *uuid.UUID
}

func (m *mockEmploymentPhaseRepo) CreateDefinition(ctx context.Context, def EmploymentPhaseDefinition) (EmploymentPhaseDefinition, error) {
	m.definitions = append(m.definitions, def)
	return def, nil
}

func (m *mockEmploymentPhaseRepo) ListDefinitions(context.Context) ([]EmploymentPhaseDefinition, error) {
	return append([]EmploymentPhaseDefinition(nil), m.definitions...), nil
}
func (m *mockEmploymentPhaseRepo) ListOrganizationPhases(_ context.Context, _ int, _ int, _ string, _ string, employeeID *uuid.UUID) (EmploymentPhaseAssignmentListResponse, error) {
	m.organizationQuery = employeeID
	return EmploymentPhaseAssignmentListResponse{Data: []EmploymentPhaseAssignment{}, Page: 1, Limit: 50}, nil
}

func (m *mockEmploymentPhaseRepo) AssignPhase(ctx context.Context, employmentID uuid.UUID, req AssignEmployeePhaseRequest) (EmployeePhase, error) {
	if m.phases == nil {
		m.phases = make(map[uuid.UUID][]EmployeePhase)
	}
	startT, _ := time.Parse("2006-01-02", req.StartDate)
	autoCloseDate := startT.AddDate(0, 0, -1).Format("2006-01-02")
	existing := m.phases[employmentID]
	for i := range existing {
		if existing[i].Status == "active" && existing[i].StartDate <= req.StartDate {
			existing[i].Status = "completed"
			if existing[i].EndDate > autoCloseDate {
				existing[i].EndDate = autoCloseDate
			}
			now := time.Now().UTC().Format(time.RFC3339)
			existing[i].CompletedAt = &now
			existing[i].UpdatedAt = now
		}
	}
	now := time.Now().UTC().Format(time.RFC3339)
	newPhase := EmployeePhase{
		ID:                uuid.New(),
		EmploymentID:      employmentID,
		PhaseDefinitionID: req.PhaseDefinitionID,
		StartDate:         req.StartDate,
		EndDate:           req.EndDate,
		Status:            "active",
		OverrideConfig:    req.OverrideConfig,
		CreatedAt:         now,
		UpdatedAt:         now,
	}
	existing = append(existing, newPhase)
	m.phases[employmentID] = existing
	return newPhase, nil
}

func (m *mockEmploymentPhaseRepo) UpdatePhaseStatus(ctx context.Context, employmentID, phaseID uuid.UUID, req UpdateEmployeePhaseStatusRequest, evaluatedBy uuid.UUID) (EmployeePhase, error) {
	existing := m.phases[employmentID]
	for i := range existing {
		if existing[i].ID == phaseID {
			existing[i].Status = req.Status
			existing[i].EvaluationNotes = req.EvaluationNotes
			existing[i].EvaluatedBy = &evaluatedBy
			now := time.Now().UTC().Format(time.RFC3339)
			existing[i].CompletedAt = &now
			existing[i].UpdatedAt = now
			m.phases[employmentID] = existing
			return existing[i], nil
		}
	}
	return EmployeePhase{}, ErrEmployeePhaseNotFound
}

func (m *mockEmploymentPhaseRepo) ListPhases(ctx context.Context, employmentID uuid.UUID) ([]EmployeePhase, error) {
	return m.phases[employmentID], nil
}

func (m *mockEmploymentPhaseRepo) ResolveActiveEmployment(ctx context.Context, employeeID uuid.UUID) (uuid.UUID, error) {
	if employeeID == uuid.Nil {
		return uuid.Nil, ErrEmployeeNotFound
	}
	return employeeID, nil
}

type mockPhaseAuthorizer struct {
	allowed map[string]bool
}

func (m mockPhaseAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, perm string) bool {
	return m.allowed[perm]
}
func (mockPhaseAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (mockPhaseAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockPhaseAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (mockPhaseAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (mockPhaseAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (mockPhaseAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func TestEmploymentPhaseDefinitionsAndPermissions(t *testing.T) {
	userID := uuid.New()
	repo := &mockEmploymentPhaseRepo{}
	handler := &EmploymentPhaseHandler{
		Repository: repo,
		Authorizer: mockPhaseAuthorizer{allowed: map[string]bool{
			"employment_phases:manage": true,
			"employment_phases:read":   true,
		}},
	}

	body := CreateEmploymentPhaseDefinitionRequest{
		Name:         "Probation",
		PhaseType:    "probation",
		RuleConfig:   json.RawMessage(`{"duration_months":2}`),
		DisplayOrder: func() *int { v := 1; return &v }(),
	}
	raw, _ := json.Marshal(body)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employment-phases", bytes.NewReader(raw))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.CreateDefinition(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	listReq := httptest.NewRequest(http.MethodGet, "/api/v1/employment-phases", nil)
	listReq = listReq.WithContext(httpinfra.WithUserIDForTest(listReq.Context(), userID))
	listRec := httptest.NewRecorder()
	handler.ListDefinitions(listRec, listReq)
	if listRec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", listRec.Code)
	}
}

func TestEmploymentPhaseListUsesReadPermission(t *testing.T) {
	userID := uuid.New()
	handler := &EmploymentPhaseHandler{Repository: &mockEmploymentPhaseRepo{}, Authorizer: mockPhaseAuthorizer{allowed: map[string]bool{"employment_phases:read": true}}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employment-phase-assignments?page=1&limit=10", nil).WithContext(httpinfra.WithUserIDForTest(context.Background(), userID))
	rec := httptest.NewRecorder()
	handler.ListOrganization(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestEmploymentPhaseListValidatesEmployeeFilter(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employment-phase-assignments?employee_id=bad", nil).WithContext(httpinfra.WithUserIDForTest(context.Background(), uuid.New()))
	rec := httptest.NewRecorder()
	(&EmploymentPhaseHandler{Repository: &mockEmploymentPhaseRepo{}, Authorizer: mockPhaseAuthorizer{allowed: map[string]bool{"employment_phases:read": true}}}).ListOrganization(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected validation error, got %d", rec.Code)
	}
}

func TestEmploymentPhaseListPassesEmployeeFilter(t *testing.T) {
	employeeID := uuid.New()
	repo := &mockEmploymentPhaseRepo{}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employment-phase-assignments?employee_id="+employeeID.String(), nil).WithContext(httpinfra.WithUserIDForTest(context.Background(), uuid.New()))
	rec := httptest.NewRecorder()
	(&EmploymentPhaseHandler{Repository: repo, Authorizer: mockPhaseAuthorizer{allowed: map[string]bool{"employment_phases:read": true}}}).ListOrganization(rec, req)
	if rec.Code != http.StatusOK || repo.organizationQuery == nil || *repo.organizationQuery != employeeID {
		t.Fatalf("expected employee filter to reach repository, status=%d", rec.Code)
	}
}

func TestEmploymentPhaseListForbiddenWithoutReadPermission(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employment-phase-assignments", nil).WithContext(httpinfra.WithUserIDForTest(context.Background(), uuid.New()))
	rec := httptest.NewRecorder()
	(&EmploymentPhaseHandler{Repository: &mockEmploymentPhaseRepo{}, Authorizer: mockPhaseAuthorizer{allowed: map[string]bool{}}}).ListOrganization(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestEmploymentPhaseAssignOverlapTransition(t *testing.T) {
	empID := uuid.New()
	userID := uuid.New()
	defID := uuid.New()
	repo := &mockEmploymentPhaseRepo{}
	handler := &EmploymentPhaseHandler{
		Repository: repo,
		Authorizer: mockPhaseAuthorizer{allowed: map[string]bool{"employment_phases:manage": true}},
	}

	assign1 := AssignEmployeePhaseRequest{
		PhaseDefinitionID: defID,
		StartDate:         "2026-06-01",
		EndDate:           "2026-07-31",
		OverrideConfig:    json.RawMessage(`{}`),
	}
	raw1, _ := json.Marshal(assign1)
	req1 := httptest.NewRequest(http.MethodPost, "/api/v1/employments/"+empID.String()+"/phases", bytes.NewReader(raw1))
	req1.SetPathValue("employment_id", empID.String())
	req1 = req1.WithContext(httpinfra.WithUserIDForTest(req1.Context(), userID))
	rec1 := httptest.NewRecorder()
	handler.AssignEmployment(rec1, req1)
	if rec1.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec1.Code, rec1.Body.String())
	}

	assign2 := AssignEmployeePhaseRequest{
		PhaseDefinitionID: defID,
		StartDate:         "2026-07-01",
		EndDate:           "2026-12-31",
		OverrideConfig:    json.RawMessage(`{}`),
	}
	raw2, _ := json.Marshal(assign2)
	req2 := httptest.NewRequest(http.MethodPost, "/api/v1/employments/"+empID.String()+"/phases", bytes.NewReader(raw2))
	req2.SetPathValue("employment_id", empID.String())
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), userID))
	rec2 := httptest.NewRecorder()
	handler.AssignEmployment(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec2.Code, rec2.Body.String())
	}

	phases := repo.phases[empID]
	if len(phases) != 2 {
		t.Fatalf("expected 2 phases, got %d", len(phases))
	}
	if phases[0].Status != "completed" || phases[0].EndDate != "2026-06-30" {
		t.Fatalf("expected old phase auto-closed to 2026-06-30, got %+v", phases[0])
	}
	if phases[1].Status != "active" || phases[1].StartDate != "2026-07-01" {
		t.Fatalf("expected new phase active, got %+v", phases[1])
	}
}
