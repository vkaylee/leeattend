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

type mockPatternRepository struct {
	createErr     error
	listErr       error
	detailsErr    error
	updateErr     error
	deleteErr     error
	assignErr     error
	bulkAssignErr error
	historyErr    error
}

func (m *mockPatternRepository) CreatePattern(_ context.Context, workspaceID uuid.UUID, req CreateWorkingPatternRequest) (WorkingPattern, error) {
	if m.createErr != nil {
		return WorkingPattern{}, m.createErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return WorkingPattern{
		ID:              uuid.New(),
		WorkspaceID:     workspaceID,
		Name:            req.Name,
		Description:     req.Description,
		IsActive:        true,
		PatternType:     req.PatternType,
		CycleLengthDays: req.CycleLengthDays,
		CreatedAt:       now,
		UpdatedAt:       now,
	}, nil
}

func (m *mockPatternRepository) ListPatterns(_ context.Context, workspaceID uuid.UUID) ([]WorkingPattern, error) {
	if m.listErr != nil {
		return nil, m.listErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return []WorkingPattern{
		{ID: uuid.New(), WorkspaceID: workspaceID, Name: "Standard 5/2", IsActive: true, PatternType: "weekly", CreatedAt: now, UpdatedAt: now},
	}, nil
}

func (m *mockPatternRepository) GetPatternDetails(_ context.Context, _ uuid.UUID, patternID uuid.UUID) ([]WorkingPatternDetail, error) {
	if m.detailsErr != nil {
		return nil, m.detailsErr
	}
	shiftID := uuid.New()
	return []WorkingPatternDetail{
		{ID: uuid.New(), PatternID: patternID, ScheduleDay: 1, ShiftID: &shiftID},
	}, nil
}

func (m *mockPatternRepository) UpdatePattern(_ context.Context, workspaceID, patternID uuid.UUID, req CreateWorkingPatternRequest) (WorkingPattern, error) {
	if m.updateErr != nil {
		return WorkingPattern{}, m.updateErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return WorkingPattern{
		ID:              patternID,
		WorkspaceID:     workspaceID,
		Name:            req.Name,
		Description:     req.Description,
		IsActive:        true,
		PatternType:     req.PatternType,
		CycleLengthDays: req.CycleLengthDays,
		CreatedAt:       now,
		UpdatedAt:       now,
	}, nil
}

func (m *mockPatternRepository) DeletePattern(_ context.Context, _, _ uuid.UUID) error {
	return m.deleteErr
}

func (m *mockPatternRepository) AssignPattern(_ context.Context, workspaceID, employmentID uuid.UUID, req AssignPatternRequest) (EmployeeWorkingPattern, error) {
	if m.assignErr != nil {
		return EmployeeWorkingPattern{}, m.assignErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return EmployeeWorkingPattern{
		ID:           uuid.New(),
		WorkspaceID:  workspaceID,
		EmploymentID: employmentID,
		PatternID:    req.PatternID,
		StartDate:    req.StartDate,
		EndDate:      req.EndDate,
		AnchorDate:   req.AnchorDate,
		CreatedAt:    now,
		UpdatedAt:    now,
	}, nil
}

func (m *mockPatternRepository) BulkAssignPattern(_ context.Context, _ uuid.UUID, req BulkAssignPatternRequest) (BulkAssignPatternResponse, error) {
	if m.bulkAssignErr != nil {
		return BulkAssignPatternResponse{}, m.bulkAssignErr
	}
	return BulkAssignPatternResponse{
		SuccessCount:    len(req.EmploymentIDs),
		FailedEmployees: []uuid.UUID{},
	}, nil
}

func (m *mockPatternRepository) EmploymentPatternHistory(_ context.Context, workspaceID, employmentID uuid.UUID) ([]EmployeeWorkingPattern, error) {
	if m.historyErr != nil {
		return nil, m.historyErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return []EmployeeWorkingPattern{
		{
			ID:           uuid.New(),
			WorkspaceID:  workspaceID,
			EmploymentID: employmentID,
			PatternID:    uuid.New(),
			StartDate:    "2026-10-01",
			CreatedAt:    now,
			UpdatedAt:    now,
		},
	}, nil
}

func TestWorkingPatternHandler_Create_RequiresPermission(t *testing.T) {
	wsID := uuid.New()
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	body := []byte(`{"name":"5/2 Standard","pattern_type":"weekly","details":[{"schedule_day":1}]}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/working-patterns", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestWorkingPatternHandler_Create_Success(t *testing.T) {
	wsID := uuid.New()
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"name":"5/2 Standard","pattern_type":"weekly","details":[{"schedule_day":1},{"schedule_day":2}]}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/working-patterns", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var pattern WorkingPattern
	if err := json.Unmarshal(rec.Body.Bytes(), &pattern); err != nil {
		t.Fatal(err)
	}
	if pattern.Name != "5/2 Standard" || pattern.WorkspaceID != wsID {
		t.Fatalf("unexpected pattern: %+v", pattern)
	}
}

func TestWorkingPatternHandler_List_Success(t *testing.T) {
	wsID := uuid.New()
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/working-patterns", nil)
	req.SetPathValue("ws_id", wsID.String())
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var patterns []WorkingPattern
	if err := json.Unmarshal(rec.Body.Bytes(), &patterns); err != nil {
		t.Fatal(err)
	}
	if len(patterns) != 1 {
		t.Fatalf("expected 1 pattern, got %d", len(patterns))
	}
}

func TestWorkingPatternHandler_Assign_Success(t *testing.T) {
	wsID := uuid.New()
	empID := uuid.New()
	patID := uuid.New()
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"pattern_id":"` + patID.String() + `","start_date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employments/"+empID.String()+"/working-patterns", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("employment_id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Assign(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var assignment EmployeeWorkingPattern
	if err := json.Unmarshal(rec.Body.Bytes(), &assignment); err != nil {
		t.Fatal(err)
	}
	if assignment.EmploymentID != empID || assignment.PatternID != patID {
		t.Fatalf("unexpected assignment: %+v", assignment)
	}
}

func TestWorkingPatternHandler_BulkAssign_Success(t *testing.T) {
	wsID := uuid.New()
	patID := uuid.New()
	emp1 := uuid.New()
	emp2 := uuid.New()
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"pattern_id":"` + patID.String() + `","employment_ids":["` + emp1.String() + `","` + emp2.String() + `"],"start_date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/working-patterns/bulk-assign", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.BulkAssign(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res BulkAssignPatternResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &res); err != nil {
		t.Fatal(err)
	}
	if res.SuccessCount != 2 || len(res.FailedEmployees) != 0 {
		t.Fatalf("unexpected bulk response: %+v", res)
	}
}
