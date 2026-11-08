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

type mockPatternComplianceChecker struct {
	assignmentErr error
	bulkErr       error
	assignCalls   int
	bulkCalls     int
}

func (m *mockPatternComplianceChecker) CheckPatternCompliance(_ context.Context, _ uuid.UUID, _ AssignPatternRequest) error {
	m.assignCalls++
	return m.assignmentErr
}

func (m *mockPatternComplianceChecker) CheckBulkPatternCompliance(_ context.Context, _ BulkAssignPatternRequest) error {
	m.bulkCalls++
	return m.bulkErr
}

func newPatternAssignRequest(empID uuid.UUID, body string) *http.Request {
	req := httptest.NewRequest(http.MethodPost, "/api/v1" + "/employments/"+empID.String()+"/working-patterns", bytes.NewReader([]byte(body)))
	req.SetPathValue("employment_id", empID.String())
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
}

func newPatternBulkAssignRequest(body string) *http.Request {
	req := httptest.NewRequest(http.MethodPost, "/api/v1" + "/working-patterns/bulk-assign", bytes.NewReader([]byte(body)))
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
}

func TestWorkingPatternHandler_Assign_HardComplianceViolation(t *testing.T) {
	empID, patID := uuid.New(), uuid.New()
	checker := &mockPatternComplianceChecker{assignmentErr: &ShiftComplianceError{Hard: true}}
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}, Compliance: checker}

	rec := httptest.NewRecorder()
	handler.Assign(rec, newPatternAssignRequest(empID, `{"pattern_id":"`+patID.String()+`","start_date":"2026-10-20"}`))

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
	if checker.assignCalls != 1 {
		t.Fatalf("expected compliance check once, got %d", checker.assignCalls)
	}
}

func TestWorkingPatternHandler_Assign_SoftComplianceViolation(t *testing.T) {
	empID, patID := uuid.New(), uuid.New()
	checker := &mockPatternComplianceChecker{assignmentErr: &ShiftComplianceError{Hard: false}}
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}, Compliance: checker}

	rec := httptest.NewRecorder()
	handler.Assign(rec, newPatternAssignRequest(empID, `{"pattern_id":"`+patID.String()+`","start_date":"2026-10-20"}`))

	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestWorkingPatternHandler_BulkAssign_ComplianceViolation(t *testing.T) {
	emp1, patID := uuid.New(), uuid.New()
	checker := &mockPatternComplianceChecker{bulkErr: &ShiftComplianceError{Hard: true}}
	handler := WorkingPatternHandler{Repository: &mockPatternRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}, Compliance: checker}

	rec := httptest.NewRecorder()
	handler.BulkAssign(rec, newPatternBulkAssignRequest(`{"pattern_id":"`+patID.String()+`","employment_ids":["`+emp1.String()+`"],"start_date":"2026-10-20"}`))

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
	if checker.bulkCalls != 1 {
		t.Fatalf("expected bulk compliance check once, got %d", checker.bulkCalls)
	}
}

func TestWorkingPatternHandler_Assign_CompliancePasses(t *testing.T) {
	empID, patID := uuid.New(), uuid.New()
	checker := &mockPatternComplianceChecker{}
	repo := &mockPatternRepository{}
	handler := WorkingPatternHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}, Compliance: checker}

	rec := httptest.NewRecorder()
	handler.Assign(rec, newPatternAssignRequest(empID, `{"pattern_id":"`+patID.String()+`","start_date":"2026-10-20"}`))

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if checker.assignCalls != 1 {
		t.Fatalf("expected compliance check once, got %d", checker.assignCalls)
	}
}
