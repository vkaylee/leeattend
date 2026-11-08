package api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockContractRepo struct {
	contracts          map[uuid.UUID]EmployeeContract
	workspaceContracts []EmployeeContractWithEmployee
}

func newMockContractRepo() *mockContractRepo {
	return &mockContractRepo{contracts: make(map[uuid.UUID]EmployeeContract)}
}

func (m *mockContractRepo) ListWorkspaceContracts(_ context.Context, wsID uuid.UUID) ([]EmployeeContractWithEmployee, error) {
	var res []EmployeeContractWithEmployee
	for _, c := range m.workspaceContracts {
		if c.WorkspaceID == wsID {
			res = append(res, c)
		}
	}
	return res, nil
}

func (m *mockContractRepo) ListEmployeeContracts(_ context.Context, wsID, employeeID uuid.UUID) ([]EmployeeContract, error) {
	var res []EmployeeContract
	for _, c := range m.contracts {
		if c.WorkspaceID == wsID {
			res = append(res, c)
		}
	}
	return res, nil
}

func (m *mockContractRepo) CreateContract(_ context.Context, wsID uuid.UUID, req CreateContractRequest) (EmployeeContract, error) {
	for _, c := range m.contracts {
		if c.WorkspaceID == wsID && c.EmploymentID == req.EmploymentID && c.Status == "active" {
			if rangesOverlap(req.StartDate, req.EndDate, c.StartDate, c.EndDate) {
				if req.AutoCloseOverlapping == nil || !*req.AutoCloseOverlapping {
					return EmployeeContract{}, ErrContractOverlap
				}
				// auto close existing
				c.Status = "expired"
				m.contracts[c.ID] = c
			}
		}
	}
	id := uuid.New()
	c := EmployeeContract{
		ID:           id,
		WorkspaceID:  wsID,
		EmploymentID: req.EmploymentID,
		ContractType: req.ContractType,
		Status:       "active",
		StartDate:    req.StartDate,
		EndDate:      req.EndDate,
	}
	m.contracts[id] = c
	return c, nil
}

func (m *mockContractRepo) GetContract(_ context.Context, wsID, contractID uuid.UUID) (EmployeeContract, error) {
	c, ok := m.contracts[contractID]
	if !ok || c.WorkspaceID != wsID {
		return EmployeeContract{}, ErrContractNotFound
	}
	return c, nil
}

func (m *mockContractRepo) UpdateContract(_ context.Context, wsID, contractID uuid.UUID, req UpdateContractRequest) (EmployeeContract, error) {
	c, ok := m.contracts[contractID]
	if !ok || c.WorkspaceID != wsID {
		return EmployeeContract{}, ErrContractNotFound
	}
	if req.ContractType != nil {
		c.ContractType = *req.ContractType
	}
	if req.Status != nil {
		c.Status = *req.Status
	}
	if req.StartDate != nil {
		c.StartDate = *req.StartDate
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			c.EndDate = nil
		} else {
			c.EndDate = *req.EndDate
		}
	}
	m.contracts[contractID] = c
	return c, nil
}

func (m *mockContractRepo) SoftDeleteContract(_ context.Context, wsID, contractID uuid.UUID) error {
	delete(m.contracts, contractID)
	return nil
}

type mockEmploymentRepo struct {
	employments map[uuid.UUID]Employment
}

func newMockEmploymentRepo() *mockEmploymentRepo {
	return &mockEmploymentRepo{employments: make(map[uuid.UUID]Employment)}
}

func (m *mockEmploymentRepo) ListEmployments(_ context.Context, wsID, employeeID uuid.UUID) ([]Employment, error) {
	var res []Employment
	for _, e := range m.employments {
		if e.WorkspaceID == wsID && e.EmployeeID == employeeID {
			res = append(res, e)
		}
	}
	return res, nil
}

func (m *mockEmploymentRepo) CreateEmployment(_ context.Context, wsID uuid.UUID, req CreateEmploymentRequest) (Employment, error) {
	id := uuid.New()
	e := Employment{
		ID:                  id,
		WorkspaceID:         wsID,
		EmployeeID:          req.EmployeeID,
		EmployerID:          req.EmployerID,
		PrimaryLocationID:   req.PrimaryLocationID,
		DepartmentID:        req.DepartmentID,
		Status:              req.Status,
		StartDate:           req.StartDate,
		EndDate:             req.EndDate,
		JobTitle:            req.JobTitle,
		EmploymentType:      req.EmploymentType,
		ManagerEmploymentID: req.ManagerEmploymentID,
	}
	m.employments[id] = e
	return e, nil
}

func (m *mockEmploymentRepo) GetEmployment(_ context.Context, wsID, id uuid.UUID) (Employment, error) {
	e, ok := m.employments[id]
	if !ok || e.WorkspaceID != wsID {
		return Employment{}, ErrEmploymentNotFound
	}
	return e, nil
}

func (m *mockEmploymentRepo) UpdateEmployment(_ context.Context, wsID, id uuid.UUID, req UpdateEmploymentRequest) (Employment, error) {
	e, ok := m.employments[id]
	if !ok || e.WorkspaceID != wsID {
		return Employment{}, ErrEmploymentNotFound
	}
	if req.EmployerID != nil {
		e.EmployerID = *req.EmployerID
	}
	if req.PrimaryLocationIDPresent {
		if req.PrimaryLocationID == nil || *req.PrimaryLocationID == nil {
			e.PrimaryLocationID = nil
		} else {
			e.PrimaryLocationID = *req.PrimaryLocationID
		}
	}
	if req.DepartmentIDPresent {
		if req.DepartmentID == nil || *req.DepartmentID == nil {
			e.DepartmentID = nil
		} else {
			e.DepartmentID = *req.DepartmentID
		}
	}
	if req.Status != nil {
		e.Status = *req.Status
	}
	if req.StartDate != nil {
		e.StartDate = *req.StartDate
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			e.EndDate = nil
		} else {
			e.EndDate = *req.EndDate
		}
	}
	if req.ManagerEmploymentIDPresent {
		if req.ManagerEmploymentID == nil || *req.ManagerEmploymentID == nil {
			e.ManagerEmploymentID = nil
		} else {
			e.ManagerEmploymentID = *req.ManagerEmploymentID
		}
	}
	m.employments[id] = e
	return e, nil
}

func (m *mockEmploymentRepo) SoftDeleteEmployment(_ context.Context, wsID, id uuid.UUID) error {
	delete(m.employments, id)
	return nil
}

type mockCEAuthz struct {
	isMember bool
}

func (a mockCEAuthz) IsMember(_ context.Context, _, _ uuid.UUID) (bool, error) {
	return a.isMember, nil
}
func (a mockCEAuthz) EffectivePermissions(_ context.Context, _, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func testRequest(method, path, body string, wsID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	return req
}

func TestContractsHandler_WorkspaceListAndAuth(t *testing.T) {
	wsID := uuid.New()
	contractRepo := newMockContractRepo()
	contractRepo.workspaceContracts = []EmployeeContractWithEmployee{
		{
			EmployeeContract: EmployeeContract{ID: uuid.New(), WorkspaceID: wsID, Status: "active", StartDate: "2026-01-01"},
			EmployeeName:     "John Doe",
			EmployeeCode:     "EMP001",
			EmployeeID:       uuid.New(),
		},
	}
	handler := ContractEmploymentHandler{
		Contracts:  contractRepo,
		Authorizer: mockCEAuthz{isMember: true},
	}

	req := testRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/contracts", "", wsID)
	rec := httptest.NewRecorder()
	handler.ListWorkspaceContracts(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res []EmployeeContractWithEmployee
	if err := json.Unmarshal(rec.Body.Bytes(), &res); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if len(res) != 1 || res[0].EmployeeName != "John Doe" {
		t.Fatalf("unexpected result: %+v", res)
	}

	// Test forbidden non-member
	handlerNonMember := ContractEmploymentHandler{
		Contracts:  contractRepo,
		Authorizer: mockCEAuthz{isMember: false},
	}
	recForbidden := httptest.NewRecorder()
	handlerNonMember.ListWorkspaceContracts(recForbidden, req)
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", recForbidden.Code)
	}
}

func TestContractsHandler_CreateGetUpdateDeleteAndOverlap(t *testing.T) {
	wsID := uuid.New()
	empID := uuid.New()
	employmentID := uuid.New()
	contractRepo := newMockContractRepo()
	handler := ContractEmploymentHandler{
		Contracts:  contractRepo,
		Authorizer: mockCEAuthz{isMember: true},
	}

	// 1. Create first contract
	createPayload := fmt.Sprintf(`{
		"employment_id": "%s",
		"contract_type": "fixed_term",
		"start_date": "2026-01-01",
		"end_date": "2026-12-31"
	}`, employmentID.String())
	req := testRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/contracts", createPayload, wsID)
	req.SetPathValue("employee_id", empID.String())
	rec := httptest.NewRecorder()
	handler.CreateContract(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var c1 EmployeeContract
	if err := json.Unmarshal(rec.Body.Bytes(), &c1); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if c1.Status != "active" {
		t.Fatalf("expected active status, got %s", c1.Status)
	}

	// 2. Attempt overlapping create without auto_close -> 409 Conflict
	overlapPayload := fmt.Sprintf(`{
		"employment_id": "%s",
		"contract_type": "fixed_term",
		"start_date": "2026-06-01",
		"end_date": "2026-12-31",
		"auto_close_overlapping": false
	}`, employmentID.String())
	reqOverlap := testRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/contracts", overlapPayload, wsID)
	reqOverlap.SetPathValue("employee_id", empID.String())
	recOverlap := httptest.NewRecorder()
	handler.CreateContract(recOverlap, reqOverlap)
	if recOverlap.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict for overlap, got %d: %s", recOverlap.Code, recOverlap.Body.String())
	}

	// 3. Overlapping create with auto_close -> 201 Created
	autoClosePayload := fmt.Sprintf(`{
		"employment_id": "%s",
		"contract_type": "indefinite",
		"start_date": "2026-06-01",
		"auto_close_overlapping": true
	}`, employmentID.String())
	reqAutoClose := testRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/contracts", autoClosePayload, wsID)
	reqAutoClose.SetPathValue("employee_id", empID.String())
	recAutoClose := httptest.NewRecorder()
	handler.CreateContract(recAutoClose, reqAutoClose)
	if recAutoClose.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", recAutoClose.Code, recAutoClose.Body.String())
	}
	var c2 EmployeeContract
	_ = json.Unmarshal(recAutoClose.Body.Bytes(), &c2)

	// Verify c1 was marked expired
	if contractRepo.contracts[c1.ID].Status != "expired" {
		t.Fatalf("expected c1 to be expired, got %s", contractRepo.contracts[c1.ID].Status)
	}

	// 4. Get contract
	reqGet := testRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/contracts/"+c2.ID.String(), "", wsID)
	reqGet.SetPathValue("employee_id", empID.String())
	reqGet.SetPathValue("contract_id", c2.ID.String())
	recGet := httptest.NewRecorder()
	handler.GetContract(recGet, reqGet)
	if recGet.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recGet.Code)
	}

	// 5. Update contract
	updatePayload := `{"status": "terminated"}`
	reqUpdate := testRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/contracts/"+c2.ID.String(), updatePayload, wsID)
	reqUpdate.SetPathValue("employee_id", empID.String())
	reqUpdate.SetPathValue("contract_id", c2.ID.String())
	recUpdate := httptest.NewRecorder()
	handler.UpdateContract(recUpdate, reqUpdate)
	if recUpdate.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recUpdate.Code, recUpdate.Body.String())
	}
	if contractRepo.contracts[c2.ID].Status != "terminated" {
		t.Fatalf("expected status terminated, got %s", contractRepo.contracts[c2.ID].Status)
	}

	// 6. Delete contract
	reqDel := testRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/contracts/"+c2.ID.String(), "", wsID)
	reqDel.SetPathValue("employee_id", empID.String())
	reqDel.SetPathValue("contract_id", c2.ID.String())
	recDel := httptest.NewRecorder()
	handler.DeleteContract(recDel, reqDel)
	if recDel.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDel.Code)
	}
}

func TestEmploymentsHandler_CRUDAndOverlap(t *testing.T) {
	wsID := uuid.New()
	empID := uuid.New()
	employer1 := uuid.New()
	employer2 := uuid.New()
	employmentRepo := newMockEmploymentRepo()
	handler := ContractEmploymentHandler{
		Employments: employmentRepo,
		Authorizer:  mockCEAuthz{isMember: true},
	}

	// 1. Create employment
	createPayload := fmt.Sprintf(`{
		"employee_id": "%s",
		"employer_id": "%s",
		"status": "active",
		"employment_type": "full_time",
		"start_date": "2026-01-01",
		"end_date": "2026-12-31"
	}`, empID.String(), employer1.String())
	req := testRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/employments", createPayload, wsID)
	req.SetPathValue("employee_id", empID.String())
	rec := httptest.NewRecorder()
	handler.CreateEmployment(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var emp Employment
	_ = json.Unmarshal(rec.Body.Bytes(), &emp)

	// 2. Overlap with same employer -> 409 Conflict
	sameEmployerOverlap := fmt.Sprintf(`{
		"employee_id": "%s",
		"employer_id": "%s",
		"status": "active",
		"employment_type": "part_time",
		"start_date": "2026-06-01",
		"end_date": "2026-12-31"
	}`, empID.String(), employer1.String())
	reqConflict := testRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/employments", sameEmployerOverlap, wsID)
	reqConflict.SetPathValue("employee_id", empID.String())
	recConflict := httptest.NewRecorder()
	handler.CreateEmployment(recConflict, reqConflict)
	if recConflict.Code != http.StatusConflict {
		t.Fatalf("expected 409 for same employer overlap, got %d: %s", recConflict.Code, recConflict.Body.String())
	}

	// 3. Overlap with different employer -> 201 Success
	diffEmployerOverlap := fmt.Sprintf(`{
		"employee_id": "%s",
		"employer_id": "%s",
		"status": "active",
		"employment_type": "part_time",
		"start_date": "2026-06-01",
		"end_date": "2026-12-31"
	}`, empID.String(), employer2.String())
	reqDiff := testRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/employments", diffEmployerOverlap, wsID)
	reqDiff.SetPathValue("employee_id", empID.String())
	recDiff := httptest.NewRecorder()
	handler.CreateEmployment(recDiff, reqDiff)
	if recDiff.Code != http.StatusCreated {
		t.Fatalf("expected 201 for different employer overlap, got %d: %s", recDiff.Code, recDiff.Body.String())
	}

	// 4. Update with double option null to clear end_date
	updatePayload := `{"end_date": null}`
	reqPatch := testRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/employments/"+emp.ID.String(), updatePayload, wsID)
	reqPatch.SetPathValue("employee_id", empID.String())
	reqPatch.SetPathValue("employment_id", emp.ID.String())
	recPatch := httptest.NewRecorder()
	handler.UpdateEmployment(recPatch, reqPatch)
	if recPatch.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recPatch.Code, recPatch.Body.String())
	}
	if employmentRepo.employments[emp.ID].EndDate != nil {
		t.Fatalf("expected EndDate to be nil after null update, got %v", *employmentRepo.employments[emp.ID].EndDate)
	}

	// 5. Delete employment -> 204 No Content
	reqDel := testRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/employments/"+emp.ID.String(), "", wsID)
	reqDel.SetPathValue("employee_id", empID.String())
	reqDel.SetPathValue("employment_id", emp.ID.String())
	recDel := httptest.NewRecorder()
	handler.DeleteEmployment(recDel, reqDel)
	if recDel.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDel.Code)
	}
}
