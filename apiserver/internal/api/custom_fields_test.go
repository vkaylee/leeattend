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

type mockCustomFieldRepo struct {
	Defs   map[uuid.UUID]CustomFieldDefinitionRecord
	Values map[uuid.UUID]map[uuid.UUID]CustomFieldValueResponse
}

func (m *mockCustomFieldRepo) CreateDefinition(_ context.Context, record CustomFieldDefinitionRecord) (CustomFieldDefinitionRecord, error) {
	if m.Defs == nil {
		m.Defs = make(map[uuid.UUID]CustomFieldDefinitionRecord)
	}
	for _, d := range m.Defs {
		if d.FieldKey == record.FieldKey {
			return CustomFieldDefinitionRecord{}, ErrCustomFieldConflict
		}
	}
	m.Defs[record.ID] = record
	return record, nil
}

func (m *mockCustomFieldRepo) GetDefinition(_ context.Context, id uuid.UUID) (CustomFieldDefinitionRecord, error) {
	if d, ok := m.Defs[id]; ok {
		return d, nil
	}
	return CustomFieldDefinitionRecord{}, ErrCustomFieldNotFound
}

func (m *mockCustomFieldRepo) ListDefinitions(_ context.Context, entityType *string) ([]CustomFieldDefinitionRecord, error) {
	var list []CustomFieldDefinitionRecord
	for _, d := range m.Defs {
		if entityType == nil || d.EntityType == *entityType {
			list = append(list, d)
		}
	}
	return list, nil
}

func (m *mockCustomFieldRepo) UpdateDefinition(ctx context.Context, id uuid.UUID, req UpdateCustomFieldDefinitionRequest) (CustomFieldDefinitionRecord, error) {
	d, err := m.GetDefinition(ctx, id)
	if err != nil {
		return CustomFieldDefinitionRecord{}, err
	}
	if req.FieldLabel != nil {
		d.FieldLabel = *req.FieldLabel
	}
	m.Defs[id] = d
	return d, nil
}

func (m *mockCustomFieldRepo) DeleteDefinition(ctx context.Context, id uuid.UUID) error {
	if _, err := m.GetDefinition(ctx, id); err != nil {
		return err
	}
	delete(m.Defs, id)
	return nil
}

func (m *mockCustomFieldRepo) GetValues(_ context.Context, entityID uuid.UUID) ([]CustomFieldValueResponse, error) {
	var list []CustomFieldValueResponse
	for _, v := range m.Values[entityID] {
		list = append(list, v)
	}
	return list, nil
}

func (m *mockCustomFieldRepo) SetValues(ctx context.Context, entityID uuid.UUID, _ string, values []SetCustomFieldValueItem) ([]CustomFieldValueResponse, error) {
	if m.Values == nil {
		m.Values = make(map[uuid.UUID]map[uuid.UUID]CustomFieldValueResponse)
	}
	if m.Values[entityID] == nil {
		m.Values[entityID] = make(map[uuid.UUID]CustomFieldValueResponse)
	}
	for _, item := range values {
		def, ok := m.Defs[item.DefinitionID]
		if !ok {
			return nil, ErrCustomFieldNotFound
		}
		m.Values[entityID][item.DefinitionID] = CustomFieldValueResponse{
			DefinitionID: item.DefinitionID,
			FieldKey:     def.FieldKey,
			FieldLabel:   def.FieldLabel,
			FieldType:    def.FieldType,
			Value:        item.Value,
		}
	}
	return m.GetValues(ctx, entityID)
}

type customFieldAuthorizer struct {
	permissions map[string]bool
}

func (a customFieldAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return a.permissions[permission]
}

func (customFieldAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}

func (customFieldAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }

func (customFieldAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (customFieldAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}
func (customFieldAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (customFieldAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }

func customFieldHandler(repo CustomFieldRepository, permissions ...string) CustomFieldHandler {
	grants := make(map[string]bool, len(permissions))
	for _, p := range permissions {
		grants[p] = true
	}
	return CustomFieldHandler{Repository: repo, Authorizer: customFieldAuthorizer{permissions: grants}}
}

func customFieldRequest(method, target string, body []byte, userID uuid.UUID) *http.Request {
	var req *http.Request
	if body == nil {
		req = httptest.NewRequest(method, target, nil)
	} else {
		req = httptest.NewRequest(method, target, bytes.NewReader(body))
	}
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestCustomFieldDefinitionsCRUD(t *testing.T) {
	userID := uuid.New()
	repo := &mockCustomFieldRepo{Defs: make(map[uuid.UUID]CustomFieldDefinitionRecord)}
	handler := customFieldHandler(repo, "organization:settings:read", "organization:settings:write")

	createBody, _ := json.Marshal(CreateCustomFieldDefinitionRequest{
		EntityType: "employee", FieldKey: "t_shirt_size", FieldLabel: "T-Shirt Size", FieldType: "text",
	})
	createRec := httptest.NewRecorder()
	handler.CreateDefinition(createRec, customFieldRequest(http.MethodPost, "/api/v1/custom-fields/definitions", createBody, userID))
	if createRec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", createRec.Code, createRec.Body.String())
	}
	var created CustomFieldDefinitionRecord
	if err := json.Unmarshal(createRec.Body.Bytes(), &created); err != nil {
		t.Fatalf("decode created definition: %v", err)
	}

	dupRec := httptest.NewRecorder()
	handler.CreateDefinition(dupRec, customFieldRequest(http.MethodPost, "/api/v1/custom-fields/definitions", createBody, userID))
	if dupRec.Code != http.StatusConflict {
		t.Fatalf("expected duplicate definition conflict, got %d", dupRec.Code)
	}

	getReq := customFieldRequest(http.MethodGet, "/api/v1/custom-fields/definitions/"+created.ID.String(), nil, userID)
	getReq.SetPathValue("id", created.ID.String())
	getRec := httptest.NewRecorder()
	handler.GetDefinition(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", getRec.Code)
	}

	listRec := httptest.NewRecorder()
	handler.ListDefinitions(listRec, customFieldRequest(http.MethodGet, "/api/v1/custom-fields/definitions?entity_type=employee", nil, userID))
	var list []CustomFieldDefinitionRecord
	if listRec.Code != http.StatusOK || json.Unmarshal(listRec.Body.Bytes(), &list) != nil || len(list) != 1 {
		t.Fatalf("expected one employee field, got %d: %s", listRec.Code, listRec.Body.String())
	}

	newLabel := "Shirt Size"
	patchBody, _ := json.Marshal(UpdateCustomFieldDefinitionRequest{FieldLabel: &newLabel})
	patchReq := customFieldRequest(http.MethodPatch, "/api/v1/custom-fields/definitions/"+created.ID.String(), patchBody, userID)
	patchReq.SetPathValue("id", created.ID.String())
	patchRec := httptest.NewRecorder()
	handler.UpdateDefinition(patchRec, patchReq)
	if patchRec.Code != http.StatusOK || repo.Defs[created.ID].FieldLabel != newLabel {
		t.Fatalf("expected label update, got %d: %+v", patchRec.Code, repo.Defs[created.ID])
	}

	delReq := customFieldRequest(http.MethodDelete, "/api/v1/custom-fields/definitions/"+created.ID.String(), nil, userID)
	delReq.SetPathValue("id", created.ID.String())
	delRec := httptest.NewRecorder()
	handler.DeleteDefinition(delRec, delReq)
	if delRec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", delRec.Code)
	}
	if _, exists := repo.Defs[created.ID]; exists {
		t.Fatal("deleted definition remains")
	}
}

func TestCustomFieldValuesRequireEntityPermissionAndPersistValue(t *testing.T) {
	userID, empID, defID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockCustomFieldRepo{
		Defs: map[uuid.UUID]CustomFieldDefinitionRecord{
			defID: {ID: defID, EntityType: "employee", FieldKey: "notes", FieldLabel: "Notes", FieldType: "text"},
		},
		Values: make(map[uuid.UUID]map[uuid.UUID]CustomFieldValueResponse),
	}
	body, _ := json.Marshal(BulkSetCustomFieldValuesRequest{Values: []SetCustomFieldValueItem{{DefinitionID: defID, Value: json.RawMessage(`"Top performer"`)}}})
	setReq := customFieldRequest(http.MethodPut, "/api/v1/custom-fields/values/employee/"+empID.String(), body, userID)
	setReq.SetPathValue("entity_type", "employee")
	setReq.SetPathValue("entity_id", empID.String())
	setRec := httptest.NewRecorder()
	customFieldHandler(repo, "employees:update").SetValues(setRec, setReq)
	if setRec.Code != http.StatusOK {
		t.Fatalf("expected employee update permission to allow set, got %d", setRec.Code)
	}

	getReq := customFieldRequest(http.MethodGet, "/api/v1/custom-fields/values/employee/"+empID.String(), nil, userID)
	getReq.SetPathValue("entity_type", "employee")
	getReq.SetPathValue("entity_id", empID.String())
	getRec := httptest.NewRecorder()
	customFieldHandler(repo, "employees:read").GetValues(getRec, getReq)
	var values []CustomFieldValueResponse
	if getRec.Code != http.StatusOK || json.Unmarshal(getRec.Body.Bytes(), &values) != nil || len(values) != 1 || string(values[0].Value) != `"Top performer"` {
		t.Fatalf("expected stored custom field value, got %d: %s", getRec.Code, getRec.Body.String())
	}

	forbiddenReq := customFieldRequest(http.MethodGet, "/api/v1/custom-fields/values/employee/"+empID.String(), nil, userID)
	forbiddenReq.SetPathValue("entity_type", "employee")
	forbiddenReq.SetPathValue("entity_id", empID.String())
	forbiddenRec := httptest.NewRecorder()
	customFieldHandler(repo, "departments:read").GetValues(forbiddenRec, forbiddenReq)
	if forbiddenRec.Code != http.StatusForbidden {
		t.Fatalf("expected unrelated department permission to deny employee data, got %d", forbiddenRec.Code)
	}
}

func TestCustomFieldListPermissions(t *testing.T) {
	userID := uuid.New()
	repo := &mockCustomFieldRepo{Defs: map[uuid.UUID]CustomFieldDefinitionRecord{
		uuid.New(): {ID: uuid.New(), EntityType: "employee", FieldKey: "badge_code", FieldLabel: "Badge Code"},
	}}

	reader := customFieldHandler(repo, "employees:read")
	req := customFieldRequest(http.MethodGet, "/api/v1/custom-fields/definitions?entity_type=employee", nil, userID)
	rec := httptest.NewRecorder()
	reader.ListDefinitions(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected employee-scoped read permission to allow employee definitions, got %d", rec.Code)
	}

	req = customFieldRequest(http.MethodGet, "/api/v1/custom-fields/definitions", nil, userID)
	rec = httptest.NewRecorder()
	reader.ListDefinitions(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected employee permission without entity filter to deny, got %d", rec.Code)
	}

	admin := customFieldHandler(repo, "organization:settings:read")
	rec = httptest.NewRecorder()
	admin.ListDefinitions(rec, customFieldRequest(http.MethodGet, "/api/v1/custom-fields/definitions", nil, userID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected organization settings permission to list all definitions, got %d", rec.Code)
	}
}
