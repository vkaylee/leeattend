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

func (m *mockCustomFieldRepo) CreateDefinition(ctx context.Context, record CustomFieldDefinitionRecord) (CustomFieldDefinitionRecord, error) {
	if m.Defs == nil {
		m.Defs = make(map[uuid.UUID]CustomFieldDefinitionRecord)
	}
	for _, d := range m.Defs {
		if d.WorkspaceID == record.WorkspaceID && d.FieldKey == record.FieldKey {
			return CustomFieldDefinitionRecord{}, ErrCustomFieldConflict
		}
	}
	m.Defs[record.ID] = record
	return record, nil
}

func (m *mockCustomFieldRepo) GetDefinition(ctx context.Context, wsID, id uuid.UUID) (CustomFieldDefinitionRecord, error) {
	if d, ok := m.Defs[id]; ok && d.WorkspaceID == wsID {
		return d, nil
	}
	return CustomFieldDefinitionRecord{}, ErrCustomFieldNotFound
}

func (m *mockCustomFieldRepo) ListDefinitions(ctx context.Context, wsID uuid.UUID, entityType *string) ([]CustomFieldDefinitionRecord, error) {
	var list []CustomFieldDefinitionRecord
	for _, d := range m.Defs {
		if d.WorkspaceID == wsID {
			if entityType == nil || d.EntityType == *entityType {
				list = append(list, d)
			}
		}
	}
	return list, nil
}

func (m *mockCustomFieldRepo) UpdateDefinition(ctx context.Context, wsID, id uuid.UUID, req UpdateCustomFieldDefinitionRequest) (CustomFieldDefinitionRecord, error) {
	d, err := m.GetDefinition(ctx, wsID, id)
	if err != nil {
		return CustomFieldDefinitionRecord{}, err
	}
	if req.FieldLabel != nil {
		d.FieldLabel = *req.FieldLabel
	}
	m.Defs[id] = d
	return d, nil
}

func (m *mockCustomFieldRepo) DeleteDefinition(ctx context.Context, wsID, id uuid.UUID) error {
	if _, err := m.GetDefinition(ctx, wsID, id); err != nil {
		return err
	}
	delete(m.Defs, id)
	return nil
}

func (m *mockCustomFieldRepo) GetValues(ctx context.Context, wsID, entityID uuid.UUID) ([]CustomFieldValueResponse, error) {
	var list []CustomFieldValueResponse
	if entityValues, ok := m.Values[entityID]; ok {
		for _, v := range entityValues {
			list = append(list, v)
		}
	}
	return list, nil
}

func (m *mockCustomFieldRepo) SetValues(ctx context.Context, wsID, entityID uuid.UUID, entityType string, values []SetCustomFieldValueItem) ([]CustomFieldValueResponse, error) {
	if m.Values == nil {
		m.Values = make(map[uuid.UUID]map[uuid.UUID]CustomFieldValueResponse)
	}
	if m.Values[entityID] == nil {
		m.Values[entityID] = make(map[uuid.UUID]CustomFieldValueResponse)
	}
	for _, item := range values {
		def, ok := m.Defs[item.DefinitionID]
		if !ok || def.WorkspaceID != wsID {
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
	return m.GetValues(ctx, wsID, entityID)
}

type mockCustomFieldAuthorizer struct{}

func (mockCustomFieldAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return true
}

func TestCustomFieldsDefinitionsCRUD(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	repo := &mockCustomFieldRepo{Defs: make(map[uuid.UUID]CustomFieldDefinitionRecord)}
	handler := CustomFieldHandler{
		Repository: repo,
		Authorizer: mockCustomFieldAuthorizer{},
	}

	// 1. Create definition
	createBody := CreateCustomFieldDefinitionRequest{
		EntityType: "employee",
		FieldKey:   "t_shirt_size",
		FieldLabel: "T-Shirt Size",
		FieldType:  "text",
	}
	raw, _ := json.Marshal(createBody)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/definitions", bytes.NewReader(raw))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.CreateDefinition(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 Created, got %d: %s", rec.Code, rec.Body.String())
	}
	var created CustomFieldDefinitionRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &created); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	// 2. Duplicate create returns 409
	recDup := httptest.NewRecorder()
	reqDup := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/definitions", bytes.NewReader(raw))
	reqDup.SetPathValue("ws_id", wsID.String())
	reqDup = reqDup.WithContext(httpinfra.WithUserIDForTest(reqDup.Context(), userID))
	handler.CreateDefinition(recDup, reqDup)
	if recDup.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict, got %d", recDup.Code)
	}

	// 3. Get definition
	getReq := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/definitions/"+created.ID.String(), nil)
	getReq.SetPathValue("ws_id", wsID.String())
	getReq.SetPathValue("id", created.ID.String())
	getReq = getReq.WithContext(httpinfra.WithUserIDForTest(getReq.Context(), userID))
	getRec := httptest.NewRecorder()

	handler.GetDefinition(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", getRec.Code)
	}

	// 4. List definitions
	listReq := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/definitions?entity_type=employee", nil)
	listReq.SetPathValue("ws_id", wsID.String())
	listReq = listReq.WithContext(httpinfra.WithUserIDForTest(listReq.Context(), userID))
	listRec := httptest.NewRecorder()

	handler.ListDefinitions(listRec, listReq)
	if listRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", listRec.Code)
	}
	var list []CustomFieldDefinitionRecord
	if err := json.Unmarshal(listRec.Body.Bytes(), &list); err != nil || len(list) != 1 {
		t.Fatalf("expected 1 definition, got %d", len(list))
	}

	// 5. Update definition
	newLabel := "Shirt Size"
	patchBody, _ := json.Marshal(UpdateCustomFieldDefinitionRequest{FieldLabel: &newLabel})
	patchReq := httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/definitions/"+created.ID.String(), bytes.NewReader(patchBody))
	patchReq.SetPathValue("ws_id", wsID.String())
	patchReq.SetPathValue("id", created.ID.String())
	patchReq = patchReq.WithContext(httpinfra.WithUserIDForTest(patchReq.Context(), userID))
	patchRec := httptest.NewRecorder()

	handler.UpdateDefinition(patchRec, patchReq)
	if patchRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", patchRec.Code)
	}

	// 6. Delete definition
	delReq := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/definitions/"+created.ID.String(), nil)
	delReq.SetPathValue("ws_id", wsID.String())
	delReq.SetPathValue("id", created.ID.String())
	delReq = delReq.WithContext(httpinfra.WithUserIDForTest(delReq.Context(), userID))
	delRec := httptest.NewRecorder()

	handler.DeleteDefinition(delRec, delReq)
	if delRec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 No Content, got %d", delRec.Code)
	}
}

func TestCustomFieldsValues(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	empID := uuid.New()
	defID := uuid.New()
	repo := &mockCustomFieldRepo{
		Defs: map[uuid.UUID]CustomFieldDefinitionRecord{
			defID: {ID: defID, WorkspaceID: wsID, EntityType: "employee", FieldKey: "notes", FieldLabel: "Notes", FieldType: "text"},
		},
		Values: make(map[uuid.UUID]map[uuid.UUID]CustomFieldValueResponse),
	}
	handler := CustomFieldHandler{
		Repository: repo,
		Authorizer: mockCustomFieldAuthorizer{},
	}

	// Set value
	setBody := BulkSetCustomFieldValuesRequest{
		Values: []SetCustomFieldValueItem{
			{DefinitionID: defID, Value: json.RawMessage(`"Top performer"`)},
		},
	}
	raw, _ := json.Marshal(setBody)
	setReq := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/values/employee/"+empID.String(), bytes.NewReader(raw))
	setReq.SetPathValue("ws_id", wsID.String())
	setReq.SetPathValue("entity_type", "employee")
	setReq.SetPathValue("entity_id", empID.String())
	setReq = setReq.WithContext(httpinfra.WithUserIDForTest(setReq.Context(), userID))
	setRec := httptest.NewRecorder()

	handler.SetValues(setRec, setReq)
	if setRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", setRec.Code, setRec.Body.String())
	}

	// Get value
	getReq := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/custom-fields/values/employee/"+empID.String(), nil)
	getReq.SetPathValue("ws_id", wsID.String())
	getReq.SetPathValue("entity_type", "employee")
	getReq.SetPathValue("entity_id", empID.String())
	getReq = getReq.WithContext(httpinfra.WithUserIDForTest(getReq.Context(), userID))
	getRec := httptest.NewRecorder()

	handler.GetValues(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", getRec.Code)
	}
	var vals []CustomFieldValueResponse
	if err := json.Unmarshal(getRec.Body.Bytes(), &vals); err != nil || len(vals) != 1 {
		t.Fatalf("expected 1 custom field value, got %d: %v", len(vals), err)
	}
}
