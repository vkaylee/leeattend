package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockEmployerRepo struct {
	employers map[uuid.UUID]Employer
	err       error
	inUse     bool
}

func newMockEmployerRepo() *mockEmployerRepo {
	return &mockEmployerRepo{employers: make(map[uuid.UUID]Employer)}
}

func (m *mockEmployerRepo) ListEmployers(_ context.Context, page, limit int) (EmployerListResponse, error) {
	if m.err != nil {
		return EmployerListResponse{}, m.err
	}
	items := make([]Employer, 0, len(m.employers))
	for _, e := range m.employers {
		items = append(items, e)
	}
	return EmployerListResponse{Data: items, Total: int64(len(items)), Page: page, Limit: limit}, nil
}

func (m *mockEmployerRepo) CreateEmployer(_ context.Context, req CreateEmployerRequest) (Employer, error) {
	if m.err != nil {
		return Employer{}, m.err
	}
	id := uuid.New()
	e := Employer{
		ID:                 id,
		Name:               req.Name,
		TaxCode:            req.TaxCode,
		LegalAddress:       req.LegalAddress,
		ParentID:           req.ParentID,
		RelationshipType:   req.RelationshipType,
		BusinessType:       req.BusinessType,
		RegistrationNumber: req.RegistrationNumber,
		Phone:              req.Phone,
		Email:              req.Email,
		Website:            req.Website,
		IncorporationDate:  req.IncorporationDate,
		CountryCode:        req.CountryCode,
		CurrencyCode:       req.CurrencyCode,
		CreatedAt:          "2026-06-16T00:00:00Z",
		UpdatedAt:          "2026-06-16T00:00:00Z",
	}
	m.employers[id] = e
	return e, nil
}

func (m *mockEmployerRepo) GetEmployer(_ context.Context, id uuid.UUID) (Employer, error) {
	if m.err != nil {
		return Employer{}, m.err
	}
	e, ok := m.employers[id]
	if !ok {
		return Employer{}, ErrEmployerNotFound
	}
	return e, nil
}

func (m *mockEmployerRepo) UpdateEmployer(_ context.Context, id uuid.UUID, req UpdateEmployerRequest) (Employer, error) {
	if m.err != nil {
		return Employer{}, m.err
	}
	e, ok := m.employers[id]
	if !ok {
		return Employer{}, ErrEmployerNotFound
	}
	if req.Name != nil {
		e.Name = *req.Name
	}
	if req.ParentIDPresent {
		if req.ParentID == nil {
			e.ParentID = nil
		} else {
			e.ParentID = *req.ParentID
		}
	}
	if req.RelationshipTypePresent {
		if req.RelationshipType == nil {
			e.RelationshipType = nil
		} else {
			e.RelationshipType = *req.RelationshipType
		}
	}
	m.employers[id] = e
	return e, nil
}

func (m *mockEmployerRepo) DeleteEmployer(_ context.Context, id uuid.UUID) error {
	if m.err != nil {
		return m.err
	}
	if m.inUse {
		return ErrEmployerInUse
	}
	if _, ok := m.employers[id]; !ok {
		return ErrEmployerNotFound
	}
	delete(m.employers, id)
	return nil
}

type mockEmployerAuthz struct {
	isMember bool
}

func (a mockEmployerAuthz) HasPermission(_ context.Context, _ uuid.UUID, _ string) bool {
	return a.isMember
}

func (a mockEmployerAuthz) HasScopedPermission(_ context.Context, _ uuid.UUID, _ string, _ *uuid.UUID) bool {
	return a.isMember
}

func (a mockEmployerAuthz) IsMember(_ context.Context, _ uuid.UUID) (bool, error) {
	return a.isMember, nil
}

func (a mockEmployerAuthz) EffectivePermissions(_ context.Context, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (a mockEmployerAuthz) GetUsersWithPermission(_ context.Context, _ string) ([]uuid.UUID, error) {
	return nil, nil
}

func employerRequest(method, path, body string) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	return req
}

func TestEmployerHandlerList(t *testing.T) {
	repo := newMockEmployerRepo()
	id := uuid.New()
	repo.employers[id] = Employer{ID: id, Name: "Acme"}
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: true}}

	req := employerRequest(http.MethodGet, "/api/v1/employers", "")
	rec := httptest.NewRecorder()
	h.List(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestEmployerHandlerCreateValidation(t *testing.T) {
	repo := newMockEmployerRepo()
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: true}}

	// Missing name
	req := employerRequest(http.MethodPost, "/api/v1/employers", `{"name":""}`)
	rec := httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty name, got %d", rec.Code)
	}

	// Invalid email
	req = employerRequest(http.MethodPost, "/api/v1/employers", `{"name":"Corp","email":"bad-email"}`)
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad email, got %d", rec.Code)
	}

	// Invalid country code (must be 3 ASCII letters)
	req = employerRequest(http.MethodPost, "/api/v1/employers", `{"name":"Corp","country_code":"VN"}`)
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for short country code, got %d", rec.Code)
	}

	// Invalid currency code
	req = employerRequest(http.MethodPost, "/api/v1/employers", `{"name":"Corp","currency_code":"VND1"}`)
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad currency code, got %d", rec.Code)
	}
}

func TestEmployerHandlerCreateSuccess(t *testing.T) {
	repo := newMockEmployerRepo()
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: true}}

	body := `{"name":"Acme Corp","country_code":"VNM","currency_code":"VND","email":"hr@acme.com"}`
	req := employerRequest(http.MethodPost, "/api/v1/employers", body)
	rec := httptest.NewRecorder()
	h.Create(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestEmployerHandlerGetAndNotFound(t *testing.T) {
	repo := newMockEmployerRepo()
	id := uuid.New()
	repo.employers[id] = Employer{ID: id, Name: "Acme"}
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: true}}

	// Found
	req := employerRequest(http.MethodGet, "/api/v1/employers/"+id.String(), "")
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	// Not Found
	missingID := uuid.New()
	req = employerRequest(http.MethodGet, "/api/v1/employers/"+missingID.String(), "")
	req.SetPathValue("id", missingID.String())
	rec = httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestEmployerHandlerUpdateNullVsOmitted(t *testing.T) {
	repo := newMockEmployerRepo()
	parentID := uuid.New()
	repo.employers[parentID] = Employer{ID: parentID, Name: "Parent Corp"}

	id := uuid.New()
	rel := "subsidiary"
	repo.employers[id] = Employer{ID: id, Name: "Child", ParentID: &parentID, RelationshipType: &rel}
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: true}}

	// Case 1: omitted fields do not clear parent
	req := employerRequest(http.MethodPut, "/api/v1/employers/"+id.String(), `{"name":"Child Updated"}`)
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res1 Employer
	_ = json.Unmarshal(rec.Body.Bytes(), &res1)
	if res1.ParentID == nil || *res1.ParentID != parentID {
		t.Fatalf("expected parent_id to be preserved when omitted, got %v", res1.ParentID)
	}

	// Case 2: explicit null clears parent
	req = employerRequest(http.MethodPut, "/api/v1/employers/"+id.String(), `{"parent_id":null,"relationship_type":null}`)
	req.SetPathValue("id", id.String())
	rec = httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res2 Employer
	_ = json.Unmarshal(rec.Body.Bytes(), &res2)
	if res2.ParentID != nil {
		t.Fatalf("expected parent_id to be cleared on explicit null, got %v", res2.ParentID)
	}
}

func TestEmployerHandlerCircularParentRejection(t *testing.T) {
	repo := newMockEmployerRepo()
	idA, idB, idC := uuid.New(), uuid.New(), uuid.New()

	// A -> B -> C
	repo.employers[idA] = Employer{ID: idA, Name: "A"}
	repo.employers[idB] = Employer{ID: idB, Name: "B", ParentID: &idA}
	repo.employers[idC] = Employer{ID: idC, Name: "C", ParentID: &idB}
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: true}}

	// Self-parent
	req := employerRequest(http.MethodPut, "/api/v1/employers/"+idA.String(), `{"parent_id":"`+idA.String()+`"}`)
	req.SetPathValue("id", idA.String())
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for self-parent, got %d", rec.Code)
	}

	// Setting A's parent to C causes cycle A -> C -> B -> A
	req = employerRequest(http.MethodPut, "/api/v1/employers/"+idA.String(), `{"parent_id":"`+idC.String()+`"}`)
	req.SetPathValue("id", idA.String())
	rec = httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for cycle, got %d", rec.Code)
	}
}

func TestEmployerHandlerDeleteAndInUseGuard(t *testing.T) {
	repo := newMockEmployerRepo()
	id := uuid.New()
	repo.employers[id] = Employer{ID: id, Name: "To Delete"}
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: true}}

	// Guard: in use by active employees
	repo.inUse = true
	req := employerRequest(http.MethodDelete, "/api/v1/employers/"+id.String(), "")
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Delete(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 when in use, got %d", rec.Code)
	}

	// Success when not in use
	repo.inUse = false
	req = employerRequest(http.MethodDelete, "/api/v1/employers/"+id.String(), "")
	req.SetPathValue("id", id.String())
	rec = httptest.NewRecorder()
	h.Delete(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
}

func TestEmployerHandlerNonMemberForbidden(t *testing.T) {
	repo := newMockEmployerRepo()
	h := EmployerHandler{Repository: repo, Authorizer: mockEmployerAuthz{isMember: false}}

	req := employerRequest(http.MethodGet, "/api/v1/employers", "")
	rec := httptest.NewRecorder()
	h.List(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
