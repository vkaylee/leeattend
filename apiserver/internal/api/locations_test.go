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

type mockLocationRepo struct {
	locations []WorkLocation
	err       error
}

func (m *mockLocationRepo) ListLocations(_ context.Context, page, limit int) (LocationListResponse, error) {
	if m.err != nil {
		return LocationListResponse{}, m.err
	}
	return LocationListResponse{Data: m.locations, Total: int64(len(m.locations)), Page: page, Limit: limit}, nil
}
func (m *mockLocationRepo) CreateLocation(_ context.Context, req CreateLocationRequest) (WorkLocation, error) {
	if m.err != nil {
		return WorkLocation{}, m.err
	}
	return WorkLocation{ID: uuid.New(), Name: req.Name, Timezone: "UTC", Status: "active"}, nil
}
func (m *mockLocationRepo) GetLocation(_ context.Context, id uuid.UUID) (WorkLocation, error) {
	if m.err != nil {
		return WorkLocation{}, m.err
	}
	return WorkLocation{ID: id, Name: "Office", Timezone: "UTC", Status: "active"}, nil
}
func (m *mockLocationRepo) UpdateLocation(_ context.Context, id uuid.UUID, req UpdateLocationRequest) (WorkLocation, error) {
	if m.err != nil {
		return WorkLocation{}, m.err
	}
	name := "Office"
	if req.Name != nil {
		name = *req.Name
	}
	return WorkLocation{ID: id, Name: name, Timezone: "UTC", Status: "active"}, nil
}
func (m *mockLocationRepo) DeleteLocation(_ context.Context, _ uuid.UUID) error {
	return m.err
}

type mockLocationAuthz struct {
	isMember bool
}

func (a mockLocationAuthz) IsMember(_ context.Context, _ uuid.UUID) (bool, error) {
	return a.isMember, nil
}
func (a mockLocationAuthz) HasPermission(context.Context, uuid.UUID, string) bool { return a.isMember }
func (a mockLocationAuthz) EffectivePermissions(_ context.Context, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockLocationAuthz) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (mockLocationAuthz) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (mockLocationAuthz) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (mockLocationAuthz) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func locationRequest(method, path, body string) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	return req
}

func TestLocationHandlerList(t *testing.T) {
	h := LocationHandler{
		Repository: &mockLocationRepo{locations: []WorkLocation{{ID: uuid.New(), Name: "HQ", IsHeadquarters: true}}},
		Authorizer: mockLocationAuthz{isMember: true},
	}
	req := locationRequest(http.MethodGet, "/api/v1/locations", "")
	rec := httptest.NewRecorder()
	h.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLocationHandlerCreateRejectsHQ(t *testing.T) {
	h := LocationHandler{
		Repository: &mockLocationRepo{},
		Authorizer: mockLocationAuthz{isMember: true},
	}
	req := locationRequest(http.MethodPost, "/api/v1/locations", `{"name":"Branch","is_headquarters":true}`)
	rec := httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for HQ create, got %d", rec.Code)
	}
}

func TestLocationHandlerCreateSuccess(t *testing.T) {
	h := LocationHandler{
		Repository: &mockLocationRepo{},
		Authorizer: mockLocationAuthz{isMember: true},
	}
	req := locationRequest(http.MethodPost, "/api/v1/locations", `{"name":"Branch Office"}`)
	rec := httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestLocationHandlerGet(t *testing.T) {
	id := uuid.New()
	h := LocationHandler{
		Repository: &mockLocationRepo{},
		Authorizer: mockLocationAuthz{isMember: true},
	}
	req := locationRequest(http.MethodGet, "/api/v1/locations/"+id.String(), "")
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestLocationHandlerDelete(t *testing.T) {
	id := uuid.New()
	h := LocationHandler{
		Repository: &mockLocationRepo{},
		Authorizer: mockLocationAuthz{isMember: true},
	}
	req := locationRequest(http.MethodDelete, "/api/v1/locations/"+id.String(), "")
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Delete(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
}

func TestLocationHandlerNonMemberForbidden(t *testing.T) {
	id := uuid.New()
	h := LocationHandler{
		Repository: &mockLocationRepo{},
		Authorizer: mockLocationAuthz{isMember: false},
	}
	req := locationRequest(http.MethodGet, "/api/v1/locations/"+id.String(), "")
	req.SetPathValue("id", id.String())
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
