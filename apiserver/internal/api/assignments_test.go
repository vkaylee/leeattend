package api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"sort"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAssignmentAuthz struct {
	isMember    bool
	permissions map[string]bool
}

func (a mockAssignmentAuthz) IsMember(_ context.Context, _, _ uuid.UUID) (bool, error) {
	return a.isMember, nil
}

func (a mockAssignmentAuthz) EffectivePermissions(_ context.Context, _, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (a mockAssignmentAuthz) HasPermission(_ context.Context, _, _ uuid.UUID, perm string) bool {
	return a.permissions[perm]
}

type mockEmployeeRepoForAssignments struct {
	employees map[uuid.UUID]EmployeeRecord
}

func (m *mockEmployeeRepoForAssignments) CreateEmployee(context.Context, uuid.UUID, CreateEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{}, nil
}

func (m *mockEmployeeRepoForAssignments) ListEmployees(context.Context, uuid.UUID, int, int, string) (EmployeeListResponse, error) {
	return EmployeeListResponse{}, nil
}

func (m *mockEmployeeRepoForAssignments) GetEmployee(_ context.Context, wsID, id uuid.UUID) (EmployeeRecord, error) {
	e, ok := m.employees[id]
	if !ok || e.WorkspaceID != wsID {
		return EmployeeRecord{}, pgx.ErrNoRows
	}
	return e, nil
}

func (m *mockEmployeeRepoForAssignments) UpdateEmployee(context.Context, uuid.UUID, uuid.UUID, UpdateEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{}, nil
}

func (m *mockEmployeeRepoForAssignments) SoftDeleteEmployee(context.Context, uuid.UUID, uuid.UUID) error {
	return nil
}

func (m *mockEmployeeRepoForAssignments) RehireEmployee(context.Context, uuid.UUID, uuid.UUID, RehireEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{}, nil
}

type mockEmployeeAssignmentRepo struct {
	assignments map[uuid.UUID]EmployeeAssignment
	activeEmps  map[uuid.UUID]ActiveEmployment
}

func newMockEmployeeAssignmentRepo() *mockEmployeeAssignmentRepo {
	return &mockEmployeeAssignmentRepo{
		assignments: make(map[uuid.UUID]EmployeeAssignment),
		activeEmps:  make(map[uuid.UUID]ActiveEmployment),
	}
}

func (m *mockEmployeeAssignmentRepo) ListEmployeeAssignments(_ context.Context, wsID, employeeID uuid.UUID) ([]EmployeeAssignment, error) {
	res := make([]EmployeeAssignment, 0)
	for _, a := range m.assignments {
		if a.WorkspaceID == wsID {
			res = append(res, a)
		}
	}
	sort.Slice(res, func(i, j int) bool {
		return res[i].StartDate > res[j].StartDate
	})
	return res, nil
}

func (m *mockEmployeeAssignmentRepo) GetEmployeeAssignment(_ context.Context, wsID, employeeID, assignmentID uuid.UUID) (EmployeeAssignment, error) {
	a, ok := m.assignments[assignmentID]
	if !ok || a.WorkspaceID != wsID {
		return EmployeeAssignment{}, ErrEmployeeAssignmentNotFound
	}
	return a, nil
}

func (m *mockEmployeeAssignmentRepo) GetActiveEmployment(_ context.Context, wsID, employeeID uuid.UUID, _ string) (ActiveEmployment, error) {
	emp, ok := m.activeEmps[employeeID]
	if !ok {
		return ActiveEmployment{}, ErrNoActiveEmployment
	}
	return emp, nil
}

func (m *mockEmployeeAssignmentRepo) CreateEmployeeAssignment(_ context.Context, wsID, employmentID, homeLocationID uuid.UUID, req CreateEmployeeAssignmentRequest) (EmployeeAssignment, error) {
	id := uuid.New()
	now := time.Now().UTC().Format(time.RFC3339)
	a := EmployeeAssignment{
		ID:             id,
		WorkspaceID:    wsID,
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
	return a, nil
}

func (m *mockEmployeeAssignmentRepo) UpdateEmployeeAssignment(_ context.Context, wsID, employeeID, assignmentID uuid.UUID, req UpdateEmployeeAssignmentRequest) (EmployeeAssignment, error) {
	a, ok := m.assignments[assignmentID]
	if !ok || a.WorkspaceID != wsID {
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

func (m *mockEmployeeAssignmentRepo) DeleteEmployeeAssignment(_ context.Context, wsID, employeeID, assignmentID uuid.UUID) error {
	a, ok := m.assignments[assignmentID]
	if !ok || a.WorkspaceID != wsID {
		return ErrEmployeeAssignmentNotFound
	}
	delete(m.assignments, assignmentID)
	return nil
}

type mockAssignmentRuleRepo struct {
	rules map[uuid.UUID]AssignmentRule
}

func newMockAssignmentRuleRepo() *mockAssignmentRuleRepo {
	return &mockAssignmentRuleRepo{
		rules: make(map[uuid.UUID]AssignmentRule),
	}
}

func (m *mockAssignmentRuleRepo) ListAssignmentRules(_ context.Context, wsID uuid.UUID) ([]AssignmentRule, error) {
	res := make([]AssignmentRule, 0)
	for _, r := range m.rules {
		if r.WorkspaceID == wsID {
			res = append(res, r)
		}
	}
	sort.Slice(res, func(i, j int) bool {
		if res[i].Priority != res[j].Priority {
			return res[i].Priority > res[j].Priority
		}
		return res[i].CreatedAt < res[j].CreatedAt
	})
	return res, nil
}

func (m *mockAssignmentRuleRepo) CreateAssignmentRule(_ context.Context, wsID uuid.UUID, req CreateAssignmentRuleRequest) (AssignmentRule, error) {
	id := uuid.New()
	now := time.Now().UTC().Format(time.RFC3339)
	r := AssignmentRule{
		ID:          id,
		WorkspaceID: wsID,
		PatternID:   req.PatternID,
		Name:        req.Name,
		Priority:    req.Priority,
		Conditions:  req.Conditions,
		IsActive:    true,
		CreatedAt:   now,
		UpdatedAt:   now,
		StartDate:   req.StartDate,
		EndDate:     req.EndDate,
	}
	m.rules[id] = r
	return r, nil
}

func (m *mockAssignmentRuleRepo) UpdateAssignmentRule(_ context.Context, wsID, ruleID uuid.UUID, req UpdateAssignmentRuleRequest) (AssignmentRule, error) {
	r, ok := m.rules[ruleID]
	if !ok || r.WorkspaceID != wsID {
		return AssignmentRule{}, ErrAssignmentRuleNotFound
	}
	if req.PatternID != nil {
		r.PatternID = *req.PatternID
	}
	if req.Name != nil {
		r.Name = *req.Name
	}
	if req.Priority != nil {
		r.Priority = *req.Priority
	}
	if req.Conditions != nil {
		r.Conditions = *req.Conditions
	}
	if req.IsActive != nil {
		r.IsActive = *req.IsActive
	}
	if req.StartDatePresent {
		if req.StartDate == nil || *req.StartDate == nil {
			r.StartDate = nil
		} else {
			r.StartDate = *req.StartDate
		}
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			r.EndDate = nil
		} else {
			r.EndDate = *req.EndDate
		}
	}
	r.UpdatedAt = time.Now().UTC().Format(time.RFC3339)
	m.rules[ruleID] = r
	return r, nil
}

func (m *mockAssignmentRuleRepo) DeleteAssignmentRule(_ context.Context, wsID, ruleID uuid.UUID) error {
	delete(m.rules, ruleID)
	return nil
}

type mockAssignmentHistoryRepo struct {
	items []AssignmentHistory
}

func (m *mockAssignmentHistoryRepo) ListAssignmentHistory(_ context.Context, _ uuid.UUID, page, limit int) ([]AssignmentHistory, int64, error) {
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

func makeAssignmentRequest(method, path, body string, wsID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	req.Header.Set("Accept-Language", "en")
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	return req
}

func TestEmployeeAssignments_ValidationAndCRUD(t *testing.T) {
	wsID := uuid.New()
	employeeID := uuid.New()
	homeLocID := uuid.New()
	hostLocID := uuid.New()
	empID := uuid.New()

	assignmentRepo := newMockEmployeeAssignmentRepo()
	employeeRepo := &mockEmployeeRepoForAssignments{
		employees: map[uuid.UUID]EmployeeRecord{
			employeeID: {ID: employeeID, WorkspaceID: wsID, FullName: "Alice"},
		},
	}
	handler := AssignmentHandler{
		Assignments: assignmentRepo,
		Employees:   employeeRepo,
		Authorizer:  mockAssignmentAuthz{isMember: true},
	}

	// 1. Validation: No active employment
	createBody := fmt.Sprintf(`{"host_location_id": "%s", "start_date": "2026-06-01"}`, hostLocID)
	req := makeAssignmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments", createBody, wsID)
	req.SetPathValue("employee_id", employeeID.String())
	rec := httptest.NewRecorder()
	handler.CreateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for no active employment, got %d: %s", rec.Code, rec.Body.String())
	}
	if !strings.Contains(rec.Body.String(), "No active employment found") {
		t.Fatalf("expected active employment error message, got: %s", rec.Body.String())
	}

	// 2. Validation: Active employment without primary location
	assignmentRepo.activeEmps[employeeID] = ActiveEmployment{ID: empID, PrimaryLocationID: nil}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments", createBody, wsID)
	req.SetPathValue("employee_id", employeeID.String())
	handler.CreateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for missing primary location, got %d", rec.Code)
	}
	if !strings.Contains(rec.Body.String(), "Employee active employment has no primary location assigned") {
		t.Fatalf("expected details message about primary location, got: %s", rec.Body.String())
	}

	// 3. Validation: Home and host location same
	assignmentRepo.activeEmps[employeeID] = ActiveEmployment{ID: empID, PrimaryLocationID: &hostLocID}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments", createBody, wsID)
	req.SetPathValue("employee_id", employeeID.String())
	handler.CreateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for same home/host location, got %d", rec.Code)
	}
	if !strings.Contains(rec.Body.String(), "Home location and host location cannot be the same") {
		t.Fatalf("expected home/host location error message, got: %s", rec.Body.String())
	}

	// 4. Create assignment: returns 200 (not 201)
	assignmentRepo.activeEmps[employeeID] = ActiveEmployment{ID: empID, PrimaryLocationID: &homeLocID}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments", createBody, wsID)
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

	// 5. Get assignment by ID: 200 found, 404 missing
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), "", wsID)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.GetEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get assignment, got %d", rec.Code)
	}

	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments/"+uuid.New().String(), "", wsID)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", uuid.New().String())
	handler.GetEmployeeAssignment(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for missing assignment, got %d", rec.Code)
	}
	if !strings.Contains(rec.Body.String(), "not found") {
		t.Fatalf("expected not found response, got: %s", rec.Body.String())
	}

	// 6. List assignments: 200 []EmployeeAssignment
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments", "", wsID)
	req.SetPathValue("employee_id", employeeID.String())
	handler.ListEmployeeAssignments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for list assignments, got %d", rec.Code)
	}
	var list []EmployeeAssignment
	if err := json.Unmarshal(rec.Body.Bytes(), &list); err != nil || len(list) != 1 {
		t.Fatalf("expected 1 assignment in list, got %d (err: %v)", len(list), err)
	}

	// 7. PATCH: Status enum valid and invalid
	patchInvalidStatus := `{"status": "nonexistent"}`
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), patchInvalidStatus, wsID)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.UpdateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid status enum, got %d", rec.Code)
	}

	patchValidStatus := `{"status": "completed"}`
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), patchValidStatus, wsID)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.UpdateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for valid status enum patch, got %d", rec.Code)
	}
	var patched EmployeeAssignment
	_ = json.Unmarshal(rec.Body.Bytes(), &patched)
	if patched.Status != AssignmentStatusCompleted {
		t.Fatalf("expected completed status, got %s", patched.Status)
	}

	// 8. PATCH: nested null semantics (explicit null clears field; omitted keeps field)
	patchWithNull := `{"end_date": null, "reason": null}`
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), patchWithNull, wsID)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.UpdateEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for null patch, got %d", rec.Code)
	}
	_ = json.Unmarshal(rec.Body.Bytes(), &patched)
	if patched.EndDate != nil || patched.Reason != nil {
		t.Fatalf("expected null end_date and reason, got end_date=%v reason=%v", patched.EndDate, patched.Reason)
	}

	// 9. DELETE: returns 200 JSON null (not 204)
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments/"+created.ID.String(), "", wsID)
	req.SetPathValue("employee_id", employeeID.String())
	req.SetPathValue("assignment_id", created.ID.String())
	handler.DeleteEmployeeAssignment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for delete assignment, got %d: %s", rec.Code, rec.Body.String())
	}
	if strings.TrimSpace(rec.Body.String()) != "null" {
		t.Fatalf("expected 'null' body, got: %s", rec.Body.String())
	}

	// 10. Workspace Isolation / Auth checks
	nonMemberHandler := AssignmentHandler{
		Assignments: assignmentRepo,
		Authorizer:  mockAssignmentAuthz{isMember: false},
	}
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/assignments", "", wsID)
	req.SetPathValue("employee_id", employeeID.String())
	nonMemberHandler.ListEmployeeAssignments(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for non-member, got %d", rec.Code)
	}
}

func TestAssignmentRules_PermissionsOrderingCRUD(t *testing.T) {
	wsID := uuid.New()
	patternID := uuid.New()
	ruleRepo := newMockAssignmentRuleRepo()

	handler := AssignmentHandler{
		Rules: ruleRepo,
		Authorizer: mockAssignmentAuthz{
			isMember:    true,
			permissions: map[string]bool{"assignment_rules:manage": true},
		},
	}

	// 1. Permission check: forbidden when lacking assignment_rules:manage
	unauthHandler := AssignmentHandler{
		Rules: ruleRepo,
		Authorizer: mockAssignmentAuthz{
			isMember:    true,
			permissions: map[string]bool{},
		},
	}
	req := makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/assignment-rules", "", wsID)
	rec := httptest.NewRecorder()
	unauthHandler.ListAssignmentRules(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without permission, got %d", rec.Code)
	}

	// 2. Create rules: returns 201, active=true
	createBody1 := fmt.Sprintf(`{
		"pattern_id": "%s",
		"name": "Rule Low Priority",
		"priority": 10,
		"conditions": {"tags": ["shift_a"]}
	}`, patternID)
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/assignment-rules", createBody1, wsID)
	handler.CreateAssignmentRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for rule creation, got %d: %s", rec.Code, rec.Body.String())
	}
	var r1 AssignmentRule
	_ = json.Unmarshal(rec.Body.Bytes(), &r1)
	if !r1.IsActive || r1.Priority != 10 {
		t.Fatalf("unexpected rule: %+v", r1)
	}

	createBody2 := fmt.Sprintf(`{
		"pattern_id": "%s",
		"name": "Rule High Priority",
		"priority": 50,
		"conditions": {"tags": ["critical"]}
	}`, patternID)
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/assignment-rules", createBody2, wsID)
	handler.CreateAssignmentRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}
	var r2 AssignmentRule
	_ = json.Unmarshal(rec.Body.Bytes(), &r2)

	// 3. List rules: priority DESC, created_at ASC
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/assignment-rules", "", wsID)
	handler.ListAssignmentRules(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var rules []AssignmentRule
	_ = json.Unmarshal(rec.Body.Bytes(), &rules)
	if len(rules) != 2 || rules[0].Priority < rules[1].Priority {
		t.Fatalf("expected priority DESC ordering: %+v", rules)
	}

	// 4. PATCH rule: update priority and is_active
	patchBody := `{"priority": 99, "is_active": false}`
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/assignment-rules/"+r1.ID.String(), patchBody, wsID)
	req.SetPathValue("rule_id", r1.ID.String())
	handler.UpdateAssignmentRule(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for patch rule, got %d: %s", rec.Code, rec.Body.String())
	}
	var patched AssignmentRule
	_ = json.Unmarshal(rec.Body.Bytes(), &patched)
	if patched.Priority != 99 || patched.IsActive != false {
		t.Fatalf("expected patched values, got: %+v", patched)
	}

	// 5. DELETE rule: returns 204 even if missing row
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/assignment-rules/"+r1.ID.String(), "", wsID)
	req.SetPathValue("rule_id", r1.ID.String())
	handler.DeleteAssignmentRule(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for delete rule, got %d", rec.Code)
	}

	missingID := uuid.New()
	rec = httptest.NewRecorder()
	req = makeAssignmentRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/assignment-rules/"+missingID.String(), "", wsID)
	req.SetPathValue("rule_id", missingID.String())
	handler.DeleteAssignmentRule(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 even for missing rule, got %d", rec.Code)
	}
}

func TestAssignmentHistory_PaginationAndTypes(t *testing.T) {
	wsID := uuid.New()
	empID := uuid.New()

	historyItems := make([]AssignmentHistory, 75)
	for i := range 75 {
		historyItems[i] = AssignmentHistory{
			ID:             uuid.New(),
			EmployeeID:     empID,
			EmployeeName:   fmt.Sprintf("Emp %d", i),
			AssignmentType: "fixed",
			DateRange:      "2026-01-01 -> 2026-06-30",
			CreatedAt:      time.Now().UTC().Format(time.RFC3339),
		}
	}

	historyRepo := &mockAssignmentHistoryRepo{items: historyItems}
	handler := AssignmentHandler{
		History:    historyRepo,
		Authorizer: mockAssignmentAuthz{isMember: true},
	}

	// 1. Default pagination: page=1, limit=50 -> 50 items returned, total=75
	req := makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/assignment-history", "", wsID)
	rec := httptest.NewRecorder()
	handler.AssignmentHistory(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var resp PaginatedAssignmentHistory
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if resp.Total != 75 || len(resp.Data) != 50 || resp.Page != 1 || resp.Limit != 50 {
		t.Fatalf("unexpected pagination response: total=%d len=%d page=%d limit=%d", resp.Total, len(resp.Data), resp.Page, resp.Limit)
	}

	// 2. Page 2 pagination: limit=50 -> remaining 25 items
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/assignment-history?page=2&limit=50", "", wsID)
	rec = httptest.NewRecorder()
	handler.AssignmentHistory(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	_ = json.Unmarshal(rec.Body.Bytes(), &resp)
	if len(resp.Data) != 25 || resp.Page != 2 {
		t.Fatalf("expected 25 items on page 2, got %d", len(resp.Data))
	}

	// 3. Max limit clamped to 3000
	req = makeAssignmentRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/assignment-history?limit=5000", "", wsID)
	rec = httptest.NewRecorder()
	handler.AssignmentHistory(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	_ = json.Unmarshal(rec.Body.Bytes(), &resp)
	if resp.Limit != 3000 {
		t.Fatalf("expected limit clamped to 3000, got %d", resp.Limit)
	}
}
