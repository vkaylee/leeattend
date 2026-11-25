package api

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockContractRepo struct {
	contracts             map[uuid.UUID]EmployeeContract
	contractsWithEmployee []EmployeeContractWithEmployee
}

func newMockContractRepo() *mockContractRepo {
	return &mockContractRepo{contracts: make(map[uuid.UUID]EmployeeContract)}
}

func (m *mockContractRepo) ListContracts(_ context.Context) ([]EmployeeContractWithEmployee, error) {
	return m.contractsWithEmployee, nil
}

func (m *mockContractRepo) ListEmployeeContracts(_ context.Context, _ uuid.UUID) ([]EmployeeContract, error) {
	var res []EmployeeContract
	for _, c := range m.contracts {
		res = append(res, c)
	}
	return res, nil
}

func (m *mockContractRepo) CreateContract(_ context.Context, req CreateContractRequest) (EmployeeContract, error) {
	for _, c := range m.contracts {
		if c.EmploymentID == req.EmploymentID && c.Status == "active" {
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
		EmploymentID: req.EmploymentID,
		ContractType: req.ContractType,
		Status:       "active",
		StartDate:    req.StartDate,
		EndDate:      req.EndDate,
	}
	m.contracts[id] = c
	return c, nil
}

func (m *mockContractRepo) GetContract(_ context.Context, contractID uuid.UUID) (EmployeeContract, error) {
	c, ok := m.contracts[contractID]
	if !ok {
		return EmployeeContract{}, ErrContractNotFound
	}
	return c, nil
}

func (m *mockContractRepo) UpdateContract(_ context.Context, contractID uuid.UUID, req UpdateContractRequest) (EmployeeContract, error) {
	c, ok := m.contracts[contractID]
	if !ok {
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

func (m *mockContractRepo) SoftDeleteContract(_ context.Context, contractID uuid.UUID) error {
	delete(m.contracts, contractID)
	return nil
}

type mockEmploymentRepo struct {
	employments                map[uuid.UUID]Employment
	relationships              map[uuid.UUID]EmploymentRelationship
	relationshipList           EmploymentRelationshipListResponse
	relationshipErr            error
	createdRelationship        EmploymentRelationshipCreated
	createRelationshipErr      error
	lastRelationshipEmployerID *uuid.UUID
	lastRelationshipAsOf       *time.Time
}

func newMockEmploymentRepo() *mockEmploymentRepo {
	return &mockEmploymentRepo{employments: make(map[uuid.UUID]Employment), relationships: make(map[uuid.UUID]EmploymentRelationship)}
}

func (m *mockEmploymentRepo) ListEmploymentRelationships(_ context.Context, employeeID uuid.UUID, page, limit int, employerID *uuid.UUID, asOf *time.Time) (EmploymentRelationshipListResponse, error) {
	m.lastRelationshipEmployerID = employerID
	m.lastRelationshipAsOf = asOf
	if m.relationshipErr != nil {
		return EmploymentRelationshipListResponse{}, m.relationshipErr
	}
	if m.relationshipList.Data != nil {
		return m.relationshipList, nil
	}
	data := make([]EmploymentRelationship, 0)
	for _, relationship := range m.relationships {
		if relationship.EmployeeID == employeeID {
			data = append(data, relationship)
		}
	}
	return EmploymentRelationshipListResponse{Data: data, Total: int64(len(data)), Page: page, Limit: limit}, nil
}

func (m *mockEmploymentRepo) CreateEmploymentRelationship(_ context.Context, employeeID uuid.UUID, req CreateEmploymentRelationshipRequest) (EmploymentRelationshipCreated, error) {
	if m.createRelationshipErr != nil {
		return EmploymentRelationshipCreated{}, m.createRelationshipErr
	}
	if m.createdRelationship.Relationship.ID != uuid.Nil {
		return m.createdRelationship, nil
	}
	relationship := EmploymentRelationship{ID: uuid.New(), EmployeeID: employeeID, EmployerID: req.EmployerID, Status: req.Status, StartDate: req.StartDate, EndDate: req.EndDate, TerminationReason: req.TerminationReason, Notes: req.Notes}
	segment := Employment{ID: uuid.New(), EmployeeID: employeeID, EmployerID: req.EmployerID, Status: req.Status, StartDate: req.StartDate, EndDate: req.EndDate, JobTitle: req.JobTitle, EmploymentType: req.EmploymentType, ManagerEmploymentID: req.ManagerEmploymentID, DepartmentID: req.DepartmentID, PrimaryLocationID: req.PrimaryLocationID, TerminationReason: req.TerminationReason, Notes: req.Notes}
	m.createdRelationship = EmploymentRelationshipCreated{Relationship: relationship, Segment: segment}
	return m.createdRelationship, nil
}

func (m *mockEmploymentRepo) GetEmploymentRelationship(_ context.Context, id uuid.UUID) (EmploymentRelationship, error) {
	relationship, ok := m.relationships[id]
	if !ok {
		return EmploymentRelationship{}, ErrEmploymentRelationshipNotFound
	}
	return relationship, nil
}

func (m *mockEmploymentRepo) ListEmployments(_ context.Context, employeeID uuid.UUID) ([]Employment, error) {
	var res []Employment
	for _, e := range m.employments {
		if e.EmployeeID == employeeID {
			res = append(res, e)
		}
	}
	return res, nil
}

func (m *mockEmploymentRepo) CreateEmployment(_ context.Context, req CreateEmploymentRequest) (Employment, error) {
	id := uuid.New()
	e := Employment{
		ID:                  id,
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

func (m *mockEmploymentRepo) GetEmployment(_ context.Context, id uuid.UUID) (Employment, error) {
	e, ok := m.employments[id]
	if !ok {
		return Employment{}, ErrEmploymentNotFound
	}
	return e, nil
}

func (m *mockEmploymentRepo) TerminateEmployment(_ context.Context, employeeID, employmentID uuid.UUID, req TerminateEmploymentRequest) (TerminateEmploymentResult, error) {
	e, ok := m.employments[employmentID]
	if !ok || e.EmployeeID != employeeID {
		return TerminateEmploymentResult{}, ErrEmploymentNotFound
	}
	e.Status = "terminated"
	end := req.TerminationDate
	e.EndDate = &end
	e.TerminationReason = req.TerminationReason
	e.Notes = req.Notes
	m.employments[employmentID] = e
	return TerminateEmploymentResult{
		Employment:         e,
		RelationshipClosed: true,
		TenureClosed:       true,
	}, nil
}

func (m *mockEmploymentRepo) UpdateEmployment(_ context.Context, id uuid.UUID, req UpdateEmploymentRequest) (Employment, error) {
	e, ok := m.employments[id]
	if !ok {
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

func (m *mockEmploymentRepo) SoftDeleteEmployment(_ context.Context, id uuid.UUID) error {
	delete(m.employments, id)
	return nil
}

type mockCEAuthz struct {
	isMember          bool
	deniedPermissions map[string]bool
}

func (a mockCEAuthz) IsMember(context.Context, uuid.UUID) (bool, error) {
	return a.isMember, nil
}
func (mockCEAuthz) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockCEAuthz) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (mockCEAuthz) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (a mockCEAuthz) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return !a.deniedPermissions[permission]
}
func (mockCEAuthz) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return true
}
func (mockCEAuthz) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func testRequest(method, path, body string) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
}

func TestContractsHandler_ListAndAuth(t *testing.T) {
	contractRepo := newMockContractRepo()
	contractRepo.contractsWithEmployee = []EmployeeContractWithEmployee{
		{
			EmployeeContract: EmployeeContract{ID: uuid.New(), Status: "active", StartDate: "2026-01-01"},
			EmployeeName:     "John Doe",
			EmployeeCode:     "EMP001",
			EmployeeID:       uuid.New(),
		},
	}
	handler := ContractEmploymentHandler{
		Contracts:  contractRepo,
		Authorizer: mockCEAuthz{isMember: true},
	}

	req := testRequest(http.MethodGet, "/api/v1/contracts", "")
	rec := httptest.NewRecorder()
	handler.ListContracts(rec, req)

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
	handlerNonMember.ListContracts(recForbidden, req)
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", recForbidden.Code)
	}

	handlerWithoutContractRead := ContractEmploymentHandler{
		Contracts:  contractRepo,
		Authorizer: mockCEAuthz{isMember: true, deniedPermissions: map[string]bool{"documents:contract:read": true}},
	}
	recWithoutPermission := httptest.NewRecorder()
	handlerWithoutContractRead.ListContracts(recWithoutPermission, req)
	if recWithoutPermission.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without contract read permission, got %d", recWithoutPermission.Code)
	}
}

func TestContractsHandler_CreateGetUpdateDeleteAndOverlap(t *testing.T) {
	empID := uuid.New()
	employmentID := uuid.New()
	contractRepo := newMockContractRepo()
	employmentRepo := newMockEmploymentRepo()
	employmentRepo.employments[employmentID] = Employment{ID: employmentID, EmployeeID: empID, EmployerID: uuid.New(), Status: "active", StartDate: "2026-01-01", EmploymentType: "full_time"}
	handler := ContractEmploymentHandler{
		Contracts:   contractRepo,
		Employments: employmentRepo,
		Authorizer:  mockCEAuthz{isMember: true},
	}

	// 1. Create first contract
	createPayload := fmt.Sprintf(`{
		"employment_id": "%s",
		"contract_type": "fixed_term",
		"start_date": "2026-01-01",
		"end_date": "2026-12-31"
	}`, employmentID.String())
	req := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/contracts", createPayload)
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
	reqOverlap := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/contracts", overlapPayload)
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
	reqAutoClose := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/contracts", autoClosePayload)
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
	reqGet := testRequest(http.MethodGet, "/api/v1/employees/"+empID.String()+"/contracts/"+c2.ID.String(), "")
	reqGet.SetPathValue("employee_id", empID.String())
	reqGet.SetPathValue("contract_id", c2.ID.String())
	recGet := httptest.NewRecorder()
	handler.GetContract(recGet, reqGet)
	if recGet.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", recGet.Code)
	}

	// A contract cannot be read through another employee's parent path.
	wrongEmployeeID := uuid.New()
	reqWrongParent := testRequest(http.MethodGet, "/api/v1/employees/"+wrongEmployeeID.String()+"/contracts/"+c2.ID.String(), "")
	reqWrongParent.SetPathValue("employee_id", wrongEmployeeID.String())
	reqWrongParent.SetPathValue("contract_id", c2.ID.String())
	recWrongParent := httptest.NewRecorder()
	handler.GetContract(recWrongParent, reqWrongParent)
	if recWrongParent.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for contract parent mismatch, got %d", recWrongParent.Code)
	}

	// Contract mutations require document management permission.
	withoutContractManage := ContractEmploymentHandler{
		Contracts:   contractRepo,
		Employments: employmentRepo,
		Authorizer:  mockCEAuthz{isMember: true, deniedPermissions: map[string]bool{"documents:manage": true}},
	}
	reqDeniedUpdate := testRequest(http.MethodPatch, "/api/v1/employees/"+empID.String()+"/contracts/"+c2.ID.String(), `{"status":"draft"}`)
	reqDeniedUpdate.SetPathValue("employee_id", empID.String())
	reqDeniedUpdate.SetPathValue("contract_id", c2.ID.String())
	recDeniedUpdate := httptest.NewRecorder()
	withoutContractManage.UpdateContract(recDeniedUpdate, reqDeniedUpdate)
	if recDeniedUpdate.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without contract manage permission, got %d", recDeniedUpdate.Code)
	}

	// 5. Update contract
	updatePayload := `{"status": "terminated"}`
	reqUpdate := testRequest(http.MethodPatch, "/api/v1/employees/"+empID.String()+"/contracts/"+c2.ID.String(), updatePayload)
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

	// 6. Wrong parent cannot update or delete contract.
	wrongUpdate := testRequest(http.MethodPatch, "/api/v1/employees/"+wrongEmployeeID.String()+"/contracts/"+c2.ID.String(), `{"status":"draft"}`)
	wrongUpdate.SetPathValue("employee_id", wrongEmployeeID.String())
	wrongUpdate.SetPathValue("contract_id", c2.ID.String())
	recWrongUpdate := httptest.NewRecorder()
	handler.UpdateContract(recWrongUpdate, wrongUpdate)
	if recWrongUpdate.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for contract update parent mismatch, got %d", recWrongUpdate.Code)
	}

	wrongDelete := testRequest(http.MethodDelete, "/api/v1/employees/"+wrongEmployeeID.String()+"/contracts/"+c2.ID.String(), "")
	wrongDelete.SetPathValue("employee_id", wrongEmployeeID.String())
	wrongDelete.SetPathValue("contract_id", c2.ID.String())
	recWrongDelete := httptest.NewRecorder()
	handler.DeleteContract(recWrongDelete, wrongDelete)
	if recWrongDelete.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for contract delete parent mismatch, got %d", recWrongDelete.Code)
	}

	// 7. Delete contract
	reqDel := testRequest(http.MethodDelete, "/api/v1/employees/"+empID.String()+"/contracts/"+c2.ID.String(), "")
	reqDel.SetPathValue("employee_id", empID.String())
	reqDel.SetPathValue("contract_id", c2.ID.String())
	recDel := httptest.NewRecorder()
	handler.DeleteContract(recDel, reqDel)
	if recDel.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDel.Code)
	}
}

func TestEmploymentRelationshipsHandler_Create(t *testing.T) {
	employeeID := uuid.New()
	employerID := uuid.New()
	employmentRepo := newMockEmploymentRepo()
	handler := ContractEmploymentHandler{Employments: employmentRepo, Authorizer: mockCEAuthz{isMember: true}}
	body := fmt.Sprintf(`{"employer_id":"%s","status":"active","start_date":"2026-01-01","employment_type":"full_time"}`, employerID.String())
	req := testRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/employment-relationships", body)
	req.SetPathValue("employee_id", employeeID.String())
	rec := httptest.NewRecorder()
	handler.CreateEmploymentRelationship(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var created EmploymentRelationshipCreated
	if err := json.Unmarshal(rec.Body.Bytes(), &created); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if created.Relationship.EmployeeID != employeeID || created.Relationship.EmployerID != employerID || created.Segment.EmployeeID != employeeID || created.Segment.EmployerID != employerID {
		t.Fatalf("unexpected created relationship: %+v", created)
	}

	denied := ContractEmploymentHandler{Employments: employmentRepo, Authorizer: mockCEAuthz{isMember: true, deniedPermissions: map[string]bool{"employees:create": true}}}
	deniedRec := httptest.NewRecorder()
	denied.CreateEmploymentRelationship(deniedRec, req)
	if deniedRec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without employee create permission, got %d", deniedRec.Code)
	}

	invalidBody := testRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/employment-relationships", `{"employer_id":"bad"}`)
	invalidBody.SetPathValue("employee_id", employeeID.String())
	invalidRec := httptest.NewRecorder()
	handler.CreateEmploymentRelationship(invalidRec, invalidBody)
	if invalidRec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid create payload, got %d", invalidRec.Code)
	}
}

func TestEmploymentRelationshipsHandler_ListGetAndValidation(t *testing.T) {
	employeeID := uuid.New()
	relationshipID := uuid.New()
	employmentRepo := newMockEmploymentRepo()
	employmentRepo.relationships[relationshipID] = EmploymentRelationship{ID: relationshipID, EmployeeID: employeeID, EmployerID: uuid.New(), Status: "active", StartDate: "2026-01-01"}
	handler := ContractEmploymentHandler{Employments: employmentRepo, Authorizer: mockCEAuthz{isMember: true}}

	req := testRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/employment-relationships?page=2&limit=10&as_of=2026-06-01", "")
	req.SetPathValue("employee_id", employeeID.String())
	rec := httptest.NewRecorder()
	handler.ListEmploymentRelationships(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var list EmploymentRelationshipListResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &list); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}
	if list.Total != 1 || list.Page != 2 || list.Limit != 10 || len(list.Data) != 1 {
		t.Fatalf("unexpected relationship list: %+v", list)
	}
	if employmentRepo.lastRelationshipAsOf == nil || employmentRepo.lastRelationshipAsOf.Format("2006-01-02") != "2026-06-01" {
		t.Fatalf("expected as_of filter to reach repository, got %v", employmentRepo.lastRelationshipAsOf)
	}

	employerID := employmentRepo.relationships[relationshipID].EmployerID
	employerFilter := testRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/employment-relationships?employer_id="+employerID.String(), "")
	employerFilter.SetPathValue("employee_id", employeeID.String())
	employerFilterRec := httptest.NewRecorder()
	handler.ListEmploymentRelationships(employerFilterRec, employerFilter)
	if employerFilterRec.Code != http.StatusOK || employmentRepo.lastRelationshipEmployerID == nil || *employmentRepo.lastRelationshipEmployerID != employerID {
		t.Fatalf("expected employer filter to reach repository: status=%d employer=%v", employerFilterRec.Code, employmentRepo.lastRelationshipEmployerID)
	}

	getReq := testRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/employment-relationships/"+relationshipID.String(), "")
	getReq.SetPathValue("employee_id", employeeID.String())
	getReq.SetPathValue("relationship_id", relationshipID.String())
	getRec := httptest.NewRecorder()
	handler.GetEmploymentRelationship(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", getRec.Code, getRec.Body.String())
	}

	invalidEmployer := testRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/employment-relationships?employer_id=invalid", "")
	invalidEmployer.SetPathValue("employee_id", employeeID.String())
	invalidEmployerRec := httptest.NewRecorder()
	handler.ListEmploymentRelationships(invalidEmployerRec, invalidEmployer)
	if invalidEmployerRec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid employer filter, got %d", invalidEmployerRec.Code)
	}

	invalidPage := testRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/employment-relationships?limit=0", "")
	invalidPage.SetPathValue("employee_id", employeeID.String())
	invalidPageRec := httptest.NewRecorder()
	handler.ListEmploymentRelationships(invalidPageRec, invalidPage)
	if invalidPageRec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid pagination, got %d", invalidPageRec.Code)
	}

	wrongEmployeeID := uuid.New()
	wrongParent := testRequest(http.MethodGet, "/api/v1/employees/"+wrongEmployeeID.String()+"/employment-relationships/"+relationshipID.String(), "")
	wrongParent.SetPathValue("employee_id", wrongEmployeeID.String())
	wrongParent.SetPathValue("relationship_id", relationshipID.String())
	wrongParentRec := httptest.NewRecorder()
	handler.GetEmploymentRelationship(wrongParentRec, wrongParent)
	if wrongParentRec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for relationship parent mismatch, got %d", wrongParentRec.Code)
	}

	denied := ContractEmploymentHandler{Employments: employmentRepo, Authorizer: mockCEAuthz{isMember: true, deniedPermissions: map[string]bool{"employees:read": true}}}
	deniedRec := httptest.NewRecorder()
	denied.ListEmploymentRelationships(deniedRec, req)
	if deniedRec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without employee read permission, got %d", deniedRec.Code)
	}
}

func TestEmploymentsHandler_CRUDAndOverlap(t *testing.T) {
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
	req := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/employments", createPayload)
	req.SetPathValue("employee_id", empID.String())
	rec := httptest.NewRecorder()
	handler.CreateEmployment(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var emp Employment
	_ = json.Unmarshal(rec.Body.Bytes(), &emp)

	// A request cannot create employment for a different employee parent.
	wrongEmployeeID := uuid.New()
	wrongCreate := fmt.Sprintf(`{
		"employee_id": "%s",
		"employer_id": "%s",
		"status": "active",
		"employment_type": "part_time",
		"start_date": "2027-01-01",
		"end_date": "2027-12-31"
	}`, wrongEmployeeID.String(), employer1.String())
	reqWrongCreate := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/employments", wrongCreate)
	reqWrongCreate.SetPathValue("employee_id", empID.String())
	recWrongCreate := httptest.NewRecorder()
	handler.CreateEmployment(recWrongCreate, reqWrongCreate)
	if recWrongCreate.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for employment create parent mismatch, got %d", recWrongCreate.Code)
	}

	withoutEmployeeCreate := ContractEmploymentHandler{
		Employments: employmentRepo,
		Authorizer:  mockCEAuthz{isMember: true, deniedPermissions: map[string]bool{"employees:create": true}},
	}
	deniedCreatePayload := fmt.Sprintf(`{"employee_id":"%s","employer_id":"%s","status":"active","employment_type":"part_time","start_date":"2027-01-01"}`, empID.String(), employer2.String())
	reqDeniedCreate := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/employments", deniedCreatePayload)
	reqDeniedCreate.SetPathValue("employee_id", empID.String())
	recDeniedCreate := httptest.NewRecorder()
	withoutEmployeeCreate.CreateEmployment(recDeniedCreate, reqDeniedCreate)
	if recDeniedCreate.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without employee create permission, got %d", recDeniedCreate.Code)
	}

	// 2. Overlap with same employer -> 409 Conflict
	sameEmployerOverlap := fmt.Sprintf(`{
		"employee_id": "%s",
		"employer_id": "%s",
		"status": "active",
		"employment_type": "part_time",
		"start_date": "2026-06-01",
		"end_date": "2026-12-31"
	}`, empID.String(), employer1.String())
	reqConflict := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/employments", sameEmployerOverlap)
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
	reqDiff := testRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/employments", diffEmployerOverlap)
	reqDiff.SetPathValue("employee_id", empID.String())
	recDiff := httptest.NewRecorder()
	handler.CreateEmployment(recDiff, reqDiff)
	if recDiff.Code != http.StatusCreated {
		t.Fatalf("expected 201 for different employer overlap, got %d: %s", recDiff.Code, recDiff.Body.String())
	}

	// 4. Wrong parent cannot read, update, or delete employment.
	wrongGet := testRequest(http.MethodGet, "/api/v1/employees/"+wrongEmployeeID.String()+"/employments/"+emp.ID.String(), "")
	wrongGet.SetPathValue("employee_id", wrongEmployeeID.String())
	wrongGet.SetPathValue("employment_id", emp.ID.String())
	recWrongGet := httptest.NewRecorder()
	handler.GetEmployment(recWrongGet, wrongGet)
	if recWrongGet.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for employment get parent mismatch, got %d", recWrongGet.Code)
	}

	wrongPatch := testRequest(http.MethodPatch, "/api/v1/employees/"+wrongEmployeeID.String()+"/employments/"+emp.ID.String(), `{"status":"inactive"}`)
	wrongPatch.SetPathValue("employee_id", wrongEmployeeID.String())
	wrongPatch.SetPathValue("employment_id", emp.ID.String())
	recWrongPatch := httptest.NewRecorder()
	handler.UpdateEmployment(recWrongPatch, wrongPatch)
	if recWrongPatch.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for employment update parent mismatch, got %d", recWrongPatch.Code)
	}

	wrongDelete := testRequest(http.MethodDelete, "/api/v1/employees/"+wrongEmployeeID.String()+"/employments/"+emp.ID.String(), "")
	wrongDelete.SetPathValue("employee_id", wrongEmployeeID.String())
	wrongDelete.SetPathValue("employment_id", emp.ID.String())
	recWrongDelete := httptest.NewRecorder()
	handler.DeleteEmployment(recWrongDelete, wrongDelete)
	if recWrongDelete.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for employment delete parent mismatch, got %d", recWrongDelete.Code)
	}

	// 5. Update with double option null to clear end_date
	updatePayload := `{"end_date": null}`
	reqPatch := testRequest(http.MethodPatch, "/api/v1/employees/"+empID.String()+"/employments/"+emp.ID.String(), updatePayload)
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

	// 6. Delete employment -> 204 No Content
	reqDel := testRequest(http.MethodDelete, "/api/v1/employees/"+empID.String()+"/employments/"+emp.ID.String(), "")
	reqDel.SetPathValue("employee_id", empID.String())
	reqDel.SetPathValue("employment_id", emp.ID.String())
	recDel := httptest.NewRecorder()
	handler.DeleteEmployment(recDel, reqDel)
	if recDel.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", recDel.Code)
	}
}
