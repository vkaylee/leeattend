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

type mockConditionTypeRepo struct {
	Records map[uuid.UUID]ConditionTypeRecord
}

func (m *mockConditionTypeRepo) List(_ context.Context, activeOnly bool) ([]ConditionTypeRecord, error) {
	var list []ConditionTypeRecord
	for _, r := range m.Records {
		if !activeOnly || r.IsActive {
			list = append(list, r)
		}
	}
	return list, nil
}

func (m *mockConditionTypeRepo) Get(_ context.Context, id uuid.UUID) (ConditionTypeRecord, error) {
	if r, ok := m.Records[id]; ok {
		return r, nil
	}
	return ConditionTypeRecord{}, ErrConditionTypeNotFound
}

func (m *mockConditionTypeRepo) Create(_ context.Context, record ConditionTypeRecord) (ConditionTypeRecord, error) {
	if m.Records == nil {
		m.Records = make(map[uuid.UUID]ConditionTypeRecord)
	}
	for _, existing := range m.Records {
		if existing.Code == record.Code {
			return ConditionTypeRecord{}, ErrConditionTypeConflict
		}
	}
	m.Records[record.ID] = record
	return record, nil
}

func (m *mockConditionTypeRepo) Update(ctx context.Context, id uuid.UUID, req UpdateConditionTypeRequest) (ConditionTypeRecord, error) {
	r, err := m.Get(ctx, id)
	if err != nil {
		return ConditionTypeRecord{}, err
	}
	if req.Category != nil {
		r.Category = *req.Category
	}
	m.Records[id] = r
	return r, nil
}

func (m *mockConditionTypeRepo) Delete(ctx context.Context, id uuid.UUID) error {
	r, err := m.Get(ctx, id)
	if err != nil {
		return err
	}
	if r.IsSystem {
		return ErrConditionTypeSystem
	}
	delete(m.Records, id)
	return nil
}

// conditionTypeAuthorizer is the slice-local fake for the shared Authorizer contract.
type conditionTypeAuthorizer struct {
	permissions map[string]bool
}

func (a conditionTypeAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return a.permissions[permission]
}

func (conditionTypeAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}

func (conditionTypeAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }

func (conditionTypeAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (conditionTypeAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func conditionTypeHandlerWith(repo ConditionTypeRepository, permissions ...string) ConditionTypeHandler {
	grants := make(map[string]bool, len(permissions))
	for _, p := range permissions {
		grants[p] = true
	}
	return ConditionTypeHandler{Repository: repo, Authorizer: conditionTypeAuthorizer{permissions: grants}}
}

func conditionTypeRequest(method, target string, body []byte, userID uuid.UUID) *http.Request {
	var req *http.Request
	if body == nil {
		req = httptest.NewRequest(method, target, nil)
	} else {
		req = httptest.NewRequest(method, target, bytes.NewReader(body))
	}
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestConditionTypeCRUD(t *testing.T) {
	userID := uuid.New()
	repo := &mockConditionTypeRepo{Records: make(map[uuid.UUID]ConditionTypeRecord)}
	handler := conditionTypeHandlerWith(repo, "organization:settings:read", "organization:settings:write")

	createBody := CreateConditionTypeRequest{
		Code:         "REMOTE_WORK",
		NameI18N:     json.RawMessage(`{"en":"Remote Work"}`),
		Category:     "working_hours",
		AffectsRules: json.RawMessage(`{"grace_period_minutes":true}`),
	}
	raw, _ := json.Marshal(createBody)
	rec := httptest.NewRecorder()
	handler.Create(rec, conditionTypeRequest(http.MethodPost, "/api/v1/condition-types", raw, userID))
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 Created, got %d: %s", rec.Code, rec.Body.String())
	}
	var created ConditionTypeRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &created); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	recDup := httptest.NewRecorder()
	handler.Create(recDup, conditionTypeRequest(http.MethodPost, "/api/v1/condition-types", raw, userID))
	if recDup.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict, got %d", recDup.Code)
	}

	getReq := conditionTypeRequest(http.MethodGet, "/api/v1/condition-types/"+created.ID.String(), nil, userID)
	getReq.SetPathValue("id", created.ID.String())
	getRec := httptest.NewRecorder()
	handler.Get(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", getRec.Code)
	}

	newCat := "remote"
	patchBody, _ := json.Marshal(UpdateConditionTypeRequest{Category: &newCat})
	patchReq := conditionTypeRequest(http.MethodPatch, "/api/v1/condition-types/"+created.ID.String(), patchBody, userID)
	patchReq.SetPathValue("id", created.ID.String())
	patchRec := httptest.NewRecorder()
	handler.Update(patchRec, patchReq)
	if patchRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", patchRec.Code)
	}

	delReq := conditionTypeRequest(http.MethodDelete, "/api/v1/condition-types/"+created.ID.String(), nil, userID)
	delReq.SetPathValue("id", created.ID.String())
	delRec := httptest.NewRecorder()
	handler.Delete(delRec, delReq)
	if delRec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 No Content, got %d", delRec.Code)
	}
	if _, ok := repo.Records[created.ID]; ok {
		t.Fatal("expected deleted condition type to be removed")
	}
}

func TestConditionTypeRequiresSettingsPermission(t *testing.T) {
	userID := uuid.New()
	repo := &mockConditionTypeRepo{Records: make(map[uuid.UUID]ConditionTypeRecord)}

	readOnly := conditionTypeHandlerWith(repo, "organization:settings:read")
	createBody, _ := json.Marshal(CreateConditionTypeRequest{
		Code:     "SHIFT_WORK",
		NameI18N: json.RawMessage(`{"en":"Shift Work"}`),
	})
	recCreate := httptest.NewRecorder()
	readOnly.Create(recCreate, conditionTypeRequest(http.MethodPost, "/api/v1/condition-types", createBody, userID))
	if recCreate.Code != http.StatusForbidden {
		t.Fatalf("expected 403 when write permission is absent, got %d", recCreate.Code)
	}

	anonymous := ConditionTypeHandler{Repository: repo}
	recList := httptest.NewRecorder()
	anonymous.List(recList, conditionTypeRequest(http.MethodGet, "/api/v1/condition-types", nil, userID))
	if recList.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without an authorizer, got %d", recList.Code)
	}

	reader := conditionTypeHandlerWith(repo, "organization:settings:read")
	recRead := httptest.NewRecorder()
	reader.List(recRead, conditionTypeRequest(http.MethodGet, "/api/v1/condition-types", nil, userID))
	if recRead.Code != http.StatusOK {
		t.Fatalf("expected 200 with settings read permission, got %d", recRead.Code)
	}
}

func TestConditionTypeSystemCannotBeDeleted(t *testing.T) {
	sysID := uuid.New()
	repo := &mockConditionTypeRepo{Records: map[uuid.UUID]ConditionTypeRecord{
		sysID: {ID: sysID, Code: "SYSTEM", IsSystem: true, IsActive: true},
	}}
	handler := conditionTypeHandlerWith(repo, "organization:settings:write")

	req := conditionTypeRequest(http.MethodDelete, "/api/v1/condition-types/"+sysID.String(), nil, uuid.New())
	req.SetPathValue("id", sysID.String())
	rec := httptest.NewRecorder()
	handler.Delete(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden on system type delete, got %d", rec.Code)
	}
	if _, ok := repo.Records[sysID]; !ok {
		t.Fatal("system condition type must remain after rejected delete")
	}
}
