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

type mockLifecycleRepo struct {
	templates  map[uuid.UUID]LifecycleChecklistTemplate
	items      map[uuid.UUID]LifecycleChecklistTemplateItem
	checklists map[uuid.UUID]LifecycleChecklistResponse
	myTasks    []PendingTaskResponse
}

func (m *mockLifecycleRepo) CreateTemplate(ctx context.Context, wsID uuid.UUID, req CreateLifecycleChecklistTemplateRequest) (LifecycleChecklistTemplate, error) {
	if m.templates == nil {
		m.templates = make(map[uuid.UUID]LifecycleChecklistTemplate)
	}
	if req.IsDefault {
		for id, t := range m.templates {
			if t.WorkspaceID == wsID && t.ChecklistType == req.ChecklistType {
				t.IsDefault = false
				m.templates[id] = t
			}
		}
	}
	tmpl := LifecycleChecklistTemplate{
		ID:            uuid.New(),
		WorkspaceID:   wsID,
		Name:          req.Name,
		Description:   req.Description,
		ChecklistType: req.ChecklistType,
		IsDefault:     req.IsDefault,
		IsActive:      true,
	}
	m.templates[tmpl.ID] = tmpl
	return tmpl, nil
}

func (m *mockLifecycleRepo) GetTemplate(ctx context.Context, wsID, id uuid.UUID) (LifecycleChecklistTemplate, error) {
	if t, ok := m.templates[id]; ok && t.WorkspaceID == wsID {
		return t, nil
	}
	return LifecycleChecklistTemplate{}, ErrTemplateNotFound
}

func (m *mockLifecycleRepo) ListTemplates(ctx context.Context, wsID uuid.UUID, checklistType *string) ([]LifecycleChecklistTemplate, error) {
	var list []LifecycleChecklistTemplate
	for _, t := range m.templates {
		if t.WorkspaceID == wsID {
			if checklistType == nil || *checklistType == t.ChecklistType {
				list = append(list, t)
			}
		}
	}
	return list, nil
}

func (m *mockLifecycleRepo) UpdateTemplate(ctx context.Context, wsID, id uuid.UUID, req UpdateLifecycleChecklistTemplateRequest) (LifecycleChecklistTemplate, error) {
	t, ok := m.templates[id]
	if !ok || t.WorkspaceID != wsID {
		return LifecycleChecklistTemplate{}, ErrTemplateNotFound
	}
	if req.Name != nil {
		t.Name = *req.Name
	}
	if req.Description != nil {
		t.Description = req.Description
	}
	if req.IsDefault != nil && *req.IsDefault {
		for otherID, other := range m.templates {
			if other.WorkspaceID == wsID && other.ChecklistType == t.ChecklistType && otherID != id {
				other.IsDefault = false
				m.templates[otherID] = other
			}
		}
		t.IsDefault = true
	}
	m.templates[id] = t
	return t, nil
}

func (m *mockLifecycleRepo) DeleteTemplate(ctx context.Context, wsID, id uuid.UUID) error {
	delete(m.templates, id)
	return nil
}

func (m *mockLifecycleRepo) CreateTemplateItem(ctx context.Context, wsID, templateID uuid.UUID, req CreateLifecycleChecklistTemplateItemRequest) (LifecycleChecklistTemplateItem, error) {
	if m.items == nil {
		m.items = make(map[uuid.UUID]LifecycleChecklistTemplateItem)
	}
	item := LifecycleChecklistTemplateItem{
		ID:             uuid.New(),
		TemplateID:     templateID,
		Title:          req.Title,
		Description:    req.Description,
		Category:       req.Category,
		AssigneeType:   req.AssigneeType,
		AssigneeUserID: req.AssigneeUserID,
		IsRequired:     req.IsRequired,
		DueDaysOffset:  req.DueDaysOffset,
		DisplayOrder:   req.DisplayOrder,
	}
	m.items[item.ID] = item
	return item, nil
}

func (m *mockLifecycleRepo) GetTemplateItem(ctx context.Context, wsID, itemID uuid.UUID) (LifecycleChecklistTemplateItem, error) {
	if item, ok := m.items[itemID]; ok {
		return item, nil
	}
	return LifecycleChecklistTemplateItem{}, ErrTemplateItemNotFound
}

func (m *mockLifecycleRepo) ListTemplateItems(ctx context.Context, wsID, templateID uuid.UUID) ([]LifecycleChecklistTemplateItem, error) {
	var list []LifecycleChecklistTemplateItem
	for _, item := range m.items {
		if item.TemplateID == templateID {
			list = append(list, item)
		}
	}
	return list, nil
}

func (m *mockLifecycleRepo) UpdateTemplateItem(ctx context.Context, wsID, itemID uuid.UUID, req UpdateLifecycleChecklistTemplateItemRequest) (LifecycleChecklistTemplateItem, error) {
	item, ok := m.items[itemID]
	if !ok {
		return LifecycleChecklistTemplateItem{}, ErrTemplateItemNotFound
	}
	if req.Title != nil {
		item.Title = *req.Title
	}
	if req.AssigneeIDPresent {
		if req.AssigneeUserID == nil || *req.AssigneeUserID == nil {
			item.AssigneeUserID = nil
		} else {
			item.AssigneeUserID = *req.AssigneeUserID
		}
	}
	m.items[itemID] = item
	return item, nil
}

func (m *mockLifecycleRepo) DeleteTemplateItem(ctx context.Context, wsID, itemID uuid.UUID) error {
	delete(m.items, itemID)
	return nil
}

func (m *mockLifecycleRepo) CreateChecklistFromTemplate(ctx context.Context, wsID, employmentID, templateID uuid.UUID, actorID *uuid.UUID, usersResolver func(string) []uuid.UUID) (LifecycleChecklistResponse, error) {
	if m.checklists == nil {
		m.checklists = make(map[uuid.UUID]LifecycleChecklistResponse)
	}
	var templateItems []LifecycleChecklistTemplateItem
	for _, it := range m.items {
		if it.TemplateID == templateID {
			templateItems = append(templateItems, it)
		}
	}
	status := "in_progress"
	progress := 0
	if len(templateItems) == 0 {
		status = "completed"
		progress = 100
	}
	clID := uuid.New()
	res := LifecycleChecklistResponse{
		ID:              clID,
		WorkspaceID:     wsID,
		EmploymentID:    employmentID,
		TemplateID:      &templateID,
		ChecklistType:   "onboarding",
		Status:          status,
		ProgressPercent: progress,
		Items:           make([]LifecycleChecklistItem, 0, len(templateItems)),
	}
	for _, ti := range templateItems {
		item := LifecycleChecklistItem{
			ID:             uuid.New(),
			ChecklistID:    clID,
			TemplateItemID: &ti.ID,
			Title:          ti.Title,
			AssigneeUserID: ti.AssigneeUserID,
			Status:         "pending",
		}
		res.Items = append(res.Items, item)
	}
	m.checklists[clID] = res
	return res, nil
}

func (m *mockLifecycleRepo) GetChecklist(ctx context.Context, wsID, checklistID uuid.UUID) (LifecycleChecklistResponse, error) {
	if cl, ok := m.checklists[checklistID]; ok && cl.WorkspaceID == wsID {
		return cl, nil
	}
	return LifecycleChecklistResponse{}, ErrChecklistNotFound
}

func (m *mockLifecycleRepo) ListChecklistsForEmployment(ctx context.Context, wsID, employmentID uuid.UUID) ([]LifecycleChecklistResponse, error) {
	var list []LifecycleChecklistResponse
	for _, cl := range m.checklists {
		if cl.WorkspaceID == wsID && cl.EmploymentID == employmentID {
			list = append(list, cl)
		}
	}
	return list, nil
}

func (m *mockLifecycleRepo) UpdateChecklistItemStatus(ctx context.Context, wsID, itemID uuid.UUID, status string, notes *string, completedBy *uuid.UUID) (LifecycleChecklistResponse, error) {
	for clID, cl := range m.checklists {
		for i := range cl.Items {
			if cl.Items[i].ID == itemID {
				cl.Items[i].Status = status
				cl.Items[i].Notes = notes
				done := 0
				for _, it := range cl.Items {
					if it.Status == "completed" || it.Status == "skipped" {
						done++
					}
				}
				progress := int(float64(done) / float64(len(cl.Items)) * 100.0)
				cl.ProgressPercent = progress
				if progress == 100 {
					cl.Status = "completed"
				} else {
					cl.Status = "in_progress"
				}
				m.checklists[clID] = cl
				return cl, nil
			}
		}
	}
	return LifecycleChecklistResponse{}, ErrChecklistItemNotFound
}

func (m *mockLifecycleRepo) GetItemAssigneeAndChecklist(ctx context.Context, wsID, itemID uuid.UUID) (*uuid.UUID, uuid.UUID, error) {
	for clID, cl := range m.checklists {
		for _, it := range cl.Items {
			if it.ID == itemID {
				return it.AssigneeUserID, clID, nil
			}
		}
	}
	return nil, uuid.Nil, ErrChecklistItemNotFound
}

func (m *mockLifecycleRepo) GetPendingTasksForUser(ctx context.Context, wsID, userID uuid.UUID) ([]PendingTaskResponse, error) {
	return m.myTasks, nil
}

type mockLifecycleAuth struct {
	allowed map[string]bool
}

func (m mockLifecycleAuth) IsSystemAdmin(context.Context, uuid.UUID) bool { return false }
func (m mockLifecycleAuth) HasPermission(ctx context.Context, userID, workspaceID uuid.UUID, perm string) bool {
	return m.allowed[perm]
}

func TestTemplateDefaultReplacement(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	repo := &mockLifecycleRepo{}
	handler := LifecycleHandler{
		Repository: repo,
		Authorizer: mockLifecycleAuth{allowed: map[string]bool{"workspace:settings:write": true}},
	}

	t1, _ := repo.CreateTemplate(context.Background(), wsID, CreateLifecycleChecklistTemplateRequest{
		Name:          "Template 1",
		ChecklistType: "onboarding",
		IsDefault:     true,
	})
	if !repo.templates[t1.ID].IsDefault {
		t.Fatalf("expected template 1 default true")
	}

	raw, _ := json.Marshal(CreateLifecycleChecklistTemplateRequest{
		Name:          "Template 2",
		ChecklistType: "onboarding",
		IsDefault:     true,
	})
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/lifecycle-templates", bytes.NewReader(raw))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.CreateTemplate(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	if repo.templates[t1.ID].IsDefault {
		t.Fatalf("template 1 default flag should have been cleared")
	}
}

func TestTemplateItemNestedNullPatch(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	assigneeID := uuid.New()
	repo := &mockLifecycleRepo{}
	handler := LifecycleHandler{
		Repository: repo,
		Authorizer: mockLifecycleAuth{allowed: map[string]bool{"workspace:settings:write": true}},
	}

	item, _ := repo.CreateTemplateItem(context.Background(), wsID, uuid.New(), CreateLifecycleChecklistTemplateItemRequest{
		Title:          "Setup Laptop",
		Category:       "it_setup",
		AssigneeType:   "specific_user",
		AssigneeUserID: &assigneeID,
		IsRequired:     true,
	})
	if item.AssigneeUserID == nil || *item.AssigneeUserID != assigneeID {
		t.Fatalf("initial assignee id not set")
	}

	clearPayload := `{"assignee_user_id": null}`
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/lifecycle-templates/"+item.TemplateID.String()+"/items/"+item.ID.String(), bytes.NewReader([]byte(clearPayload)))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("tid", item.TemplateID.String())
	req.SetPathValue("iid", item.ID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	handler.UpdateTemplateItem(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.items[item.ID].AssigneeUserID != nil {
		t.Fatalf("expected assignee_user_id cleared to nil")
	}
}

func TestChecklistProgressAndAssigneeAuth(t *testing.T) {
	wsID := uuid.New()
	empID := uuid.New()
	assigneeUser := uuid.New()
	nonAssigneeUser := uuid.New()
	repo := &mockLifecycleRepo{}
	auth := mockLifecycleAuth{allowed: map[string]bool{"employees:update": false}}
	handler := LifecycleHandler{
		Repository: repo,
		Authorizer: auth,
	}

	tmplID := uuid.New()
	_, _ = repo.CreateTemplateItem(context.Background(), wsID, tmplID, CreateLifecycleChecklistTemplateItemRequest{
		Title:          "Sign Contract",
		Category:       "documentation",
		AssigneeType:   "specific_user",
		AssigneeUserID: &assigneeUser,
		IsRequired:     true,
	})
	cl, _ := repo.CreateChecklistFromTemplate(context.Background(), wsID, empID, tmplID, nil, nil)
	checklistItemID := cl.Items[0].ID

	reqForbidden := httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/lifecycle/"+cl.ID.String()+"/items/"+checklistItemID.String(), bytes.NewReader([]byte(`{"status":"completed"}`)))
	reqForbidden.SetPathValue("ws_id", wsID.String())
	reqForbidden.SetPathValue("id", cl.ID.String())
	reqForbidden.SetPathValue("iid", checklistItemID.String())
	reqForbidden = reqForbidden.WithContext(httpinfra.WithUserIDForTest(reqForbidden.Context(), nonAssigneeUser))
	recForbidden := httptest.NewRecorder()
	handler.UpdateItemStatus(recForbidden, reqForbidden)
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("non-assignee without permission should be 403, got %d", recForbidden.Code)
	}

	reqAllowed := httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/lifecycle/"+cl.ID.String()+"/items/"+checklistItemID.String(), bytes.NewReader([]byte(`{"status":"completed"}`)))
	reqAllowed.SetPathValue("ws_id", wsID.String())
	reqAllowed.SetPathValue("id", cl.ID.String())
	reqAllowed.SetPathValue("iid", checklistItemID.String())
	reqAllowed = reqAllowed.WithContext(httpinfra.WithUserIDForTest(reqAllowed.Context(), assigneeUser))
	recAllowed := httptest.NewRecorder()
	handler.UpdateItemStatus(recAllowed, reqAllowed)
	if recAllowed.Code != http.StatusOK {
		t.Fatalf("assignee should be allowed, got %d: %s", recAllowed.Code, recAllowed.Body.String())
	}
	var updatedCl LifecycleChecklistResponse
	_ = json.Unmarshal(recAllowed.Body.Bytes(), &updatedCl)
	if updatedCl.ProgressPercent != 100 || updatedCl.Status != "completed" {
		t.Fatalf("expected 100 percent progress and completed status, got progress %d status %s", updatedCl.ProgressPercent, updatedCl.Status)
	}
}
