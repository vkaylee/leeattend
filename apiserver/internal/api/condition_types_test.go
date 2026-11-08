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

func (m *mockConditionTypeRepo) List(ctx context.Context, wsID uuid.UUID, activeOnly bool) ([]ConditionTypeRecord, error) {
	var list []ConditionTypeRecord
	for _, r := range m.Records {
		if r.WorkspaceID == wsID {
			if !activeOnly || r.IsActive {
				list = append(list, r)
			}
		}
	}
	return list, nil
}

func (m *mockConditionTypeRepo) Get(ctx context.Context, wsID, id uuid.UUID) (ConditionTypeRecord, error) {
	if r, ok := m.Records[id]; ok && r.WorkspaceID == wsID {
		return r, nil
	}
	return ConditionTypeRecord{}, ErrConditionTypeNotFound
}

func (m *mockConditionTypeRepo) Create(ctx context.Context, record ConditionTypeRecord) (ConditionTypeRecord, error) {
	if m.Records == nil {
		m.Records = make(map[uuid.UUID]ConditionTypeRecord)
	}
	for _, existing := range m.Records {
		if existing.WorkspaceID == record.WorkspaceID && existing.Code == record.Code {
			return ConditionTypeRecord{}, ErrConditionTypeConflict
		}
	}
	m.Records[record.ID] = record
	return record, nil
}

func (m *mockConditionTypeRepo) Update(ctx context.Context, wsID, id uuid.UUID, req UpdateConditionTypeRequest) (ConditionTypeRecord, error) {
	r, err := m.Get(ctx, wsID, id)
	if err != nil {
		return ConditionTypeRecord{}, err
	}
	if req.Category != nil {
		r.Category = *req.Category
	}
	m.Records[id] = r
	return r, nil
}

func (m *mockConditionTypeRepo) Delete(ctx context.Context, wsID, id uuid.UUID) error {
	r, err := m.Get(ctx, wsID, id)
	if err != nil {
		return err
	}
	if r.IsSystem {
		return ErrConditionTypeSystem
	}
	delete(m.Records, id)
	return nil
}

type mockConditionTypeAuthorizer struct{}

func (mockConditionTypeAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return true
}

func TestConditionTypeCRUD(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	repo := &mockConditionTypeRepo{Records: make(map[uuid.UUID]ConditionTypeRecord)}
	handler := ConditionTypeHandler{
		Repository: repo,
		Authorizer: mockConditionTypeAuthorizer{},
	}

	// 1. Create
	createBody := CreateConditionTypeRequest{
		Code:         "REMOTE_WORK",
		NameI18N:     json.RawMessage(`{"en":"Remote Work"}`),
		Category:     "working_hours",
		AffectsRules: json.RawMessage(`{"grace_period_minutes":true}`),
	}
	raw, _ := json.Marshal(createBody)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/condition-types", bytes.NewReader(raw))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 Created, got %d: %s", rec.Code, rec.Body.String())
	}
	var created ConditionTypeRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &created); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	// 2. Duplicate create returns 409
	recDup := httptest.NewRecorder()
	reqDup := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/condition-types", bytes.NewReader(raw))
	reqDup = reqDup.WithContext(httpinfra.WithOrganizationIDForTest(reqDup.Context(), wsID))
	reqDup = reqDup.WithContext(httpinfra.WithUserIDForTest(reqDup.Context(), userID))
	handler.Create(recDup, reqDup)
	if recDup.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict, got %d", recDup.Code)
	}

	// 3. Get
	getReq := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/condition-types/"+created.ID.String(), nil)
	getReq = getReq.WithContext(httpinfra.WithOrganizationIDForTest(getReq.Context(), wsID))
	getReq.SetPathValue("id", created.ID.String())
	getReq = getReq.WithContext(httpinfra.WithUserIDForTest(getReq.Context(), userID))
	getRec := httptest.NewRecorder()

	handler.Get(getRec, getReq)
	if getRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", getRec.Code)
	}

	// 4. Update
	newCat := "remote"
	patchBody, _ := json.Marshal(UpdateConditionTypeRequest{Category: &newCat})
	patchReq := httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/condition-types/"+created.ID.String(), bytes.NewReader(patchBody))
	patchReq = patchReq.WithContext(httpinfra.WithOrganizationIDForTest(patchReq.Context(), wsID))
	patchReq.SetPathValue("id", created.ID.String())
	patchReq = patchReq.WithContext(httpinfra.WithUserIDForTest(patchReq.Context(), userID))
	patchRec := httptest.NewRecorder()

	handler.Update(patchRec, patchReq)
	if patchRec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", patchRec.Code)
	}

	// 5. Delete
	delReq := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/condition-types/"+created.ID.String(), nil)
	delReq = delReq.WithContext(httpinfra.WithOrganizationIDForTest(delReq.Context(), wsID))
	delReq.SetPathValue("id", created.ID.String())
	delReq = delReq.WithContext(httpinfra.WithUserIDForTest(delReq.Context(), userID))
	delRec := httptest.NewRecorder()

	handler.Delete(delRec, delReq)
	if delRec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 No Content, got %d", delRec.Code)
	}
}

func TestConditionTypeSystemCannotBeDeleted(t *testing.T) {
	wsID := uuid.New()
	sysID := uuid.New()
	repo := &mockConditionTypeRepo{Records: map[uuid.UUID]ConditionTypeRecord{
		sysID: {ID: sysID, WorkspaceID: wsID, Code: "SYSTEM", IsSystem: true},
	}}
	handler := ConditionTypeHandler{
		Repository: repo,
		Authorizer: mockConditionTypeAuthorizer{},
	}

	req := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/condition-types/"+sysID.String(), nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", sysID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	handler.Delete(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 Forbidden on system type delete, got %d", rec.Code)
	}
}
