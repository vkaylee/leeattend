package api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"sort"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type assignmentSliceAuthorizer struct {
	isMember    bool
	permissions map[string]bool
}

func (a assignmentSliceAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return a.permissions[permission]
}
func (a assignmentSliceAuthorizer) HasScopedPermission(ctx context.Context, userID uuid.UUID, permission string, _ *uuid.UUID) bool {
	return a.HasPermission(ctx, userID, permission)
}
func (a assignmentSliceAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return a.isMember, nil
}
func (assignmentSliceAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (assignmentSliceAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}
func (assignmentSliceAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (assignmentSliceAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool {
	return false
}

type mockEmployeeRepoForAssignments struct {
	employees map[uuid.UUID]EmployeeRecord
}

func (m *mockEmployeeRepoForAssignments) CreateEmployee(context.Context, CreateEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{}, nil
}
func (m *mockEmployeeRepoForAssignments) ListEmployees(context.Context, int, int, string) (EmployeeListResponse, error) {
	return EmployeeListResponse{}, nil
}
func (m *mockEmployeeRepoForAssignments) GetEmployee(_ context.Context, id uuid.UUID) (EmployeeRecord, error) {
	e, ok := m.employees[id]
	if !ok {
		return EmployeeRecord{}, pgx.ErrNoRows
	}
	return e, nil
}
func (m *mockEmployeeRepoForAssignments) UpdateEmployee(context.Context, uuid.UUID, UpdateEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{}, nil
}
func (m *mockEmployeeRepoForAssignments) SoftDeleteEmployee(context.Context, uuid.UUID) error { return nil }
func (m *mockEmployeeRepoForAssignments) RehireEmployee(context.Context, uuid.UUID, RehireEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{}, nil
}

type mockEmployeeAssignmentRepo struct {
	assignments      map[uuid.UUID]EmployeeAssignment
	employeeByAssign map[uuid.UUID]uuid.UUID
	activeEmps       map[uuid.UUID]ActiveEmployment
}

func newMockEmployeeAssignmentRepo() *mockEmployeeAssignmentRepo {
	return &mockEmployeeAssignmentRepo{
		assignments:      make(map[uuid.UUID]EmployeeAssignment),
		employeeByAssign: make(map[uuid.UUID]uuid.UUID),
		activeEmps:       make(map[uuid.UUID]ActiveEmployment),
	}
}

func (m *mockEmployeeAssignmentRepo) ListAssignments(context.Context) ([]EmployeeAssignmentWithEmployee, error) {
	res := make([]EmployeeAssignmentWithEmployee, 0, len(m.assignments))
	for _, a := range m.assignments {
		res = append(res, EmployeeAssignmentWithEmployee{
			EmployeeAssignment: a,
			EmployeeID:         m.employeeByAssign[a.ID],
			EmployeeName:       "Test Employee",
			EmployeeCode:       "EMP-001",
			HomeLocationName:   "HQ",
			HostLocationName:   "Branch 1",
		})
	}
	sort.Slice(res, func(i, j int) bool { return res[i].StartDate > res[j].StartDate })
	return res, nil
}
func (m *mockEmployeeAssignmentRepo) ListEmployeeAssignments(_ context.Context, employeeID uuid.UUID) ([]EmployeeAssignment, error) {
	res := make([]EmployeeAssignment, 0)
	for id, a := range m.assignments {
		if m.employeeByAssign[id] == employeeID {
			res = append(res, a)
		}
	}
	sort.Slice(res, func(i, j int) bool { return res[i].StartDate > res[j].StartDate })
	return res, nil
}
func (m *mockEmployeeAssignmentRepo) GetEmployeeAssignment(_ context.Context, employeeID, assignmentID uuid.UUID) (EmployeeAssignment, error) {
	a, ok := m.assignments[assignmentID]
	if !ok || m.employeeByAssign[assignmentID] != employeeID {
		return EmployeeAssignment{}, ErrEmployeeAssignmentNotFound
	}
	return a, nil
}
func (m *mockEmployeeAssignmentRepo) GetActiveEmployment(_ context.Context, employeeID uuid.UUID, _ string) (ActiveEmployment, error) {
	employment, ok := m.activeEmps[employeeID]
	if !ok {
		return ActiveEmployment{}, ErrNoActiveEmployment
	}
	return employment, nil
}
func (m *mockEmployeeAssignmentRepo) CreateEmployeeAssignment(_ context.Context, employmentID, homeLocationID uuid.UUID, req CreateEmployeeAssignmentRequest) (EmployeeAssignment, error) {
	id := uuid.New()
	now := time.Now().UTC().Format(time.RFC3339)
	a := EmployeeAssignment{
		ID:             id,
		HomeLocationID: homeLocationID,
		HostLocationID: req.HostLocationID,
		StartDate:      req.StartDate,
		EndDate:        req.EndDate,
		Reason:         req.Reason,
		Status:         AssignmentStatusActive,
		CreatedAt:      now,
		UpdatedAt:      now,
		EmploymentID:   employmentID,
	}
	m.assignments[id] = a
	for employeeID, active := range m.activeEmps {
		if active.ID == employmentID {
			m.employeeByAssign[id] = employeeID
			break
		}
	}
	return a, nil
}
func (m *mockEmployeeAssignmentRepo) UpdateEmployeeAssignment(_ context.Context, employeeID, assignmentID uuid.UUID, req UpdateEmployeeAssignmentRequest) (EmployeeAssignment, error) {
	a, ok := m.assignments[assignmentID]
	if !ok || m.employeeByAssign[assignmentID] != employeeID {
		return EmployeeAssignment{}, ErrEmployeeAssignmentNotFound
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			a.EndDate = nil
		} else {
			a.EndDate = *req.EndDate
		}
	}
	if req.Status != nil {
		a.Status = *req.Status
	}
	if req.ReasonPresent {
		if req.Reason == nil || *req.Reason == nil {
			a.Reason = nil
		} else {
			a.Reason = *req.Reason
		}
	}
	a.UpdatedAt = time.Now().UTC().Format(time.RFC3339)
	m.assignments[assignmentID] = a
	return a, nil
}
func (m *mockEmployeeAssignmentRepo) DeleteEmployeeAssignment(_ context.Context, employeeID, assignmentID uuid.UUID) error {
	if _, ok := m.assignments[assignmentID]; !ok || m.employeeByAssign[assignmentID] != employeeID {
		return ErrEmployeeAssignmentNotFound
	}
	delete(m.assignments, assignmentID)
	delete(m.employeeByAssign, assignmentID)
	return nil
}

type mockAssignmentRuleRepo struct {
	rules map[uuid.UUID]AssignmentRule
}

func newMockAssignmentRuleRepo() *mockAssignmentRuleRepo {
	return &mockAssignmentRuleRepo{rules: make(map[uuid.UUID]AssignmentRule)}
}
func (m *mockAssignmentRuleRepo) ListAssignmentRules(context.Context) ([]AssignmentRule, error) {
	res := make([]AssignmentRule, 0, len(m.rules))
	for _, rule := range m.rules {
		res = append(res, rule)
	}
	sort.Slice(res, func(i, j int) bool {
		if res[i].Priority != res[j].Priority {
			return res[i].Priority > res[j].Priority
		}
		return res[i].CreatedAt < res[j].CreatedAt
	})
	return res, nil
}
func (m *mockAssignmentRuleRepo) CreateAssignmentRule(_ context.Context, req CreateAssignmentRuleRequest) (AssignmentRule, error) {
	id := uuid.New()
	now := time.Now().UTC().Format(time.RFC3339)
	rule := AssignmentRule{ID: id, PatternID: req.PatternID, Name: req.Name, Priority: req.Priority, Conditions: req.Conditions, IsActive: true, CreatedAt: now, UpdatedAt: now, StartDate: req.StartDate, EndDate: req.EndDate}
	m.rules[id] = rule
	return rule, nil
}
func (m *mockAssignmentRuleRepo) UpdateAssignmentRule(_ context.Context, ruleID uuid.UUID, req UpdateAssignmentRuleRequest) (AssignmentRule, error) {
	rule, ok := m.rules[ruleID]
	if !ok {
		return AssignmentRule{}, ErrAssignmentRuleNotFound
	}
	if req.PatternID != nil {
		rule.PatternID = *req.PatternID
	}
	if req.Name != nil {
		rule.Name = *req.Name
	}
	if req.Priority != nil {
		rule.Priority = *req.Priority
	}
	if req.Conditions != nil {
		rule.Conditions = *req.Conditions
	}
	if req.IsActive != nil {
		rule.IsActive = *req.IsActive
	}
	if req.StartDatePresent {
		if req.StartDate == nil || *req.StartDate == nil {
			rule.StartDate = nil
		} else {
			rule.StartDate = *req.StartDate
		}
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			rule.EndDate = nil
		} else {
			rule.EndDate = *req.EndDate
		}
	}
	rule.UpdatedAt = time.Now().UTC().Format(time.RFC3339)
	m.rules[ruleID] = rule
	return rule, nil
}
func (m *mockAssignmentRuleRepo) DeleteAssignmentRule(_ context.Context, ruleID uuid.UUID) error {
	delete(m.rules, ruleID)
	return nil
}

type mockAssignmentHistoryRepo struct {
	items []AssignmentHistory
}

func (m *mockAssignmentHistoryRepo) ListAssignmentHistory(_ context.Context, page, limit int) ([]AssignmentHistory, int64, error) {
	total := int64(len(m.items))
	offset := (page - 1) * limit
	if offset >= len(m.items) {
		return []AssignmentHistory{}, total, nil
	}
	end := offset + limit
	if end > len(m.items) {
		end = len(m.items)
	}
	return m.items[offset:end], total, nil
}

func makeAssignmentRequest(method, path, body string) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	req.Header.Set("Accept-Language", "en")
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
}

func TestEmployeeAssignments_ValidationAndCRUD(t *testing.T) {
	employeeID := uuid.New()
	homeLocID := uuid.New()
	hostLocID := uuid.New()
	employmentID := uuid.New()

	assignmentRepo := newMockEmployeeAssignmentRepo()
	assignmentRepo.activeEmps[employeeID] = ActiveEmployment{ID: employmentID}
	employeeRepo := &mockEmployeeRepoForAssignments{employees: map[uuid.UUID]EmployeeRecord{employeeID: {ID: employeeID, FullName: "Alice"}}}
	handler := AssignmentHandler{Assignments: assignmentRepo, Employees: employeeRepo, Authorizer: assignmentSliceAuthorizer{isMember: true}}

	createBody := fmt.Sprintf(`{"host_location_id":"%s","start_date":"2026-06-01"}`, hostLocID)
	assignmentRepo.activeEmps = map[uuid.UUID]ActiveEmployment{}
	req := makeAssignmentRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/assignments", createBody)
	req.SetPathValue("employee_id", employeeID.String())
	rec := httptest.NewRecorder()
	handler.CreateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for no active employment, got %d: %s", rec.Code, rec.Body.String())
	}

	assignmentRepo.activeEmps[employeeID] = ActiveEmployment{ID: employmentID}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/assignments", createBody)
	req.SetPathValue("employee_id", employeeID.String())
	handler.CreateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing primary location, got %d", rec.Code)
	}

	assignmentRepo.activeEmps[employeeID] = ActiveEmployment{ID: employmentID, PrimaryLocationID: &hostLocID}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/assignments", createBody)
	req.SetPathValue("employee_id", employeeID.String())
	handler.CreateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for same home/host location, got %d", rec.Code)
	}

	assignmentRepo.activeEmps[employeeID] = ActiveEmployment{ID: employmentID, PrimaryLocationID: &homeLocID}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/assignments", createBody)
	req.SetPathValue("employee_id", employeeID.String())
	handler.CreateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for created assignment, got %d: %s", rec.Code, rec.Body.String())
	}
	var created EmployeeAssignment
	if err := json.Unmarshal(rec.Body.Bytes(), &created); err != nil {
		t.Fatalf("json parse error: %v", err)
	}
	if created.Status != AssignmentStatusActive || created.HomeLocationID != homeLocID || created.HostLocationID != hostLocID {
		t.Fatalf("unexpected assignment fields: %+v", created)
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), "")
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.GetEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get assignment, got %d", rec.Code)
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/assignments", "")
	req.SetPathValue("employee_id", employeeID.String())
	handler.ListEmployeeAssignments(rec, req)
	var listed []EmployeeAssignment
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &listed) != nil || len(listed) != 1 {
		t.Fatalf("expected one listed assignment, got %d: %s", rec.Code, rec.Body.String())
	}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/assignments", "")
	handler.ListAssignments(rec, req)
	var allAssignments []EmployeeAssignmentWithEmployee
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &allAssignments) != nil || len(allAssignments) != 1 {
		t.Fatalf("expected one assignment in collection, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPatch, "/api/v1/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), `{"status":"completed"}`)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.UpdateEmployeeAssignment(rec, req)
	var updated EmployeeAssignment
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &updated) != nil || updated.Status != AssignmentStatusCompleted {
		t.Fatalf("expected completed assignment, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPatch, "/api/v1/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), `{"end_date":null,"reason":null}`)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.UpdateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &updated) != nil || updated.EndDate != nil || updated.Reason != nil {
		t.Fatalf("expected explicit null to clear nullable fields, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodDelete, "/api/v1/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), "")
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.DeleteEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK || string(bytes.TrimSpace(rec.Body.Bytes())) != "null" {
		t.Fatalf("expected successful delete with null body, got %d: %s", rec.Code, rec.Body.String())
	}

	nonMemberHandler := AssignmentHandler{Assignments: assignmentRepo, Authorizer: assignmentSliceAuthorizer{isMember: false}}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/assignments", "")
	req.SetPathValue("employee_id", employeeID.String())
	nonMemberHandler.ListEmployeeAssignments(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for non-member, got %d", rec.Code)
	}
}

func TestAssignmentRules_PermissionsOrderingCRUD(t *testing.T) {
	patternID := uuid.New()
	ruleRepo := newMockAssignmentRuleRepo()
	handler := AssignmentHandler{Rules: ruleRepo, Authorizer: assignmentSliceAuthorizer{isMember: true, permissions: map[string]bool{"assignment_rules:manage": true}}}
	unauthHandler := AssignmentHandler{Rules: ruleRepo, Authorizer: assignmentSliceAuthorizer{isMember: true, permissions: map[string]bool{}}}
	req := makeAssignmentRequest(http.MethodGet, "/api/v1/assignment-rules", "")
	rec := httptest.NewRecorder()
	unauthHandler.ListAssignmentRules(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without permission, got %d", rec.Code)
	}

	createBody := fmt.Sprintf(`{"pattern_id":"%s","name":"Rule Low Priority","priority":10,"conditions":{"tags":["shift_a"]}}`, patternID)
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/assignment-rules", createBody)
	handler.CreateAssignmentRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for rule creation, got %d: %s", rec.Code, rec.Body.String())
	}
	var lowRule AssignmentRule
	if err := json.Unmarshal(rec.Body.Bytes(), &lowRule); err != nil || !lowRule.IsActive {
		t.Fatalf("expected active created rule, got %+v, err %v", lowRule, err)
	}

	createBody = fmt.Sprintf(`{"pattern_id":"%s","name":"Rule High Priority","priority":50,"conditions":{"tags":["critical"]}}`, patternID)
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/assignment-rules", createBody)
	handler.CreateAssignmentRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/assignment-rules", "")
	handler.ListAssignmentRules(rec, req)
	var rules []AssignmentRule
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &rules) != nil || len(rules) != 2 || rules[0].Priority < rules[1].Priority {
		t.Fatalf("expected priority-descending rules, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPatch, "/api/v1/assignment-rules/"+lowRule.ID.String(), `{"priority":99,"is_active":false}`)
	req.SetPathValue("rule_id", lowRule.ID.String())
	handler.UpdateAssignmentRule(rec, req)
	var patched AssignmentRule
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &patched) != nil || patched.Priority != 99 || patched.IsActive {
		t.Fatalf("expected rule patch, got %d: %s", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodDelete, "/api/v1/assignment-rules/"+lowRule.ID.String(), "")
	req.SetPathValue("rule_id", lowRule.ID.String())
	handler.DeleteAssignmentRule(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for delete rule, got %d", rec.Code)
	}
}

func TestAssignmentHistory_PaginationAndTypes(t *testing.T) {
	employeeID := uuid.New()
	historyItems := make([]AssignmentHistory, 75)
	for i := range 75 {
		historyItems[i] = AssignmentHistory{
			ID:             uuid.New(),
			EmployeeID:     employeeID,
			EmployeeName:   fmt.Sprintf("Emp %d", i),
			AssignmentType: "fixed",
			DateRange:      "2026-01-01 -> 2026-06-30",
			CreatedAt:      time.Now().UTC().Format(time.RFC3339),
		}
	}
	handler := AssignmentHandler{History: &mockAssignmentHistoryRepo{items: historyItems}, Authorizer: assignmentSliceAuthorizer{isMember: true}}
	req := makeAssignmentRequest(http.MethodGet, "/api/v1/assignment-history", "")
	rec := httptest.NewRecorder()
	handler.AssignmentHistory(rec, req)
	var resp PaginatedAssignmentHistory
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &resp) != nil || resp.Total != 75 || len(resp.Data) != 50 || resp.Page != 1 || resp.Limit != 50 {
		t.Fatalf("unexpected pagination response: %d %s", rec.Code, rec.Body.String())
	}

	req = makeAssignmentRequest(http.MethodGet, "/api/v1/assignment-history?page=2&limit=50", "")
	rec = httptest.NewRecorder()
	handler.AssignmentHistory(rec, req)
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &resp) != nil || len(resp.Data) != 25 || resp.Page != 2 {
		t.Fatalf("unexpected page two response: %d %s", rec.Code, rec.Body.String())
	}

	req = makeAssignmentRequest(http.MethodGet, "/api/v1/assignment-history?limit=5000", "")
	rec = httptest.NewRecorder()
	handler.AssignmentHistory(rec, req)
	if rec.Code != http.StatusOK || json.Unmarshal(rec.Body.Bytes(), &resp) != nil || resp.Limit != 3000 {
		t.Fatalf("expected limit clamped to 3000, got %d %s", rec.Code, rec.Body.String())
	}
}
