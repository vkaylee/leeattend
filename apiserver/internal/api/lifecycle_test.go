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

func (m *mockLifecycleRepo) CreateTemplate(_ context.Context, req CreateLifecycleChecklistTemplateRequest) (LifecycleChecklistTemplate, error) {
	if m.templates == nil {
		m.templates = make(map[uuid.UUID]LifecycleChecklistTemplate)
	}
	if req.IsDefault {
		for id, template := range m.templates {
			if template.ChecklistType == req.ChecklistType {
				template.IsDefault = false
				m.templates[id] = template
			}
		}
	}
	template := LifecycleChecklistTemplate{ID: uuid.New(), Name: req.Name, Description: req.Description, ChecklistType: req.ChecklistType, IsDefault: req.IsDefault, IsActive: true}
	m.templates[template.ID] = template
	return template, nil
}

func (m *mockLifecycleRepo) GetTemplate(_ context.Context, id uuid.UUID) (LifecycleChecklistTemplate, error) {
	template, ok := m.templates[id]
	if !ok {
		return LifecycleChecklistTemplate{}, ErrTemplateNotFound
	}
	return template, nil
}

func (m *mockLifecycleRepo) ListTemplates(_ context.Context, checklistType *string) ([]LifecycleChecklistTemplate, error) {
	var list []LifecycleChecklistTemplate
	for _, template := range m.templates {
		if checklistType == nil || *checklistType == template.ChecklistType {
			list = append(list, template)
		}
	}
	return list, nil
}

func (m *mockLifecycleRepo) UpdateTemplate(_ context.Context, id uuid.UUID, req UpdateLifecycleChecklistTemplateRequest) (LifecycleChecklistTemplate, error) {
	template, ok := m.templates[id]
	if !ok {
		return LifecycleChecklistTemplate{}, ErrTemplateNotFound
	}
	if req.Name != nil {
		template.Name = *req.Name
	}
	if req.Description != nil {
		template.Description = req.Description
	}
	if req.IsDefault != nil && *req.IsDefault {
		for otherID, other := range m.templates {
			if other.ChecklistType == template.ChecklistType && otherID != id {
				other.IsDefault = false
				m.templates[otherID] = other
			}
		}
		template.IsDefault = true
	}
	m.templates[id] = template
	return template, nil
}

func (m *mockLifecycleRepo) DeleteTemplate(_ context.Context, id uuid.UUID) error {
	delete(m.templates, id)
	return nil
}

func (m *mockLifecycleRepo) CreateTemplateItem(_ context.Context, templateID uuid.UUID, req CreateLifecycleChecklistTemplateItemRequest) (LifecycleChecklistTemplateItem, error) {
	if m.items == nil {
		m.items = make(map[uuid.UUID]LifecycleChecklistTemplateItem)
	}
	item := LifecycleChecklistTemplateItem{ID: uuid.New(), TemplateID: templateID, Title: req.Title, Description: req.Description, Category: req.Category, AssigneeType: req.AssigneeType, AssigneeUserID: req.AssigneeUserID, IsRequired: req.IsRequired, DueDaysOffset: req.DueDaysOffset, DisplayOrder: req.DisplayOrder}
	m.items[item.ID] = item
	return item, nil
}

func (m *mockLifecycleRepo) GetTemplateItem(_ context.Context, itemID uuid.UUID) (LifecycleChecklistTemplateItem, error) {
	item, ok := m.items[itemID]
	if !ok {
		return LifecycleChecklistTemplateItem{}, ErrTemplateItemNotFound
	}
	return item, nil
}

func (m *mockLifecycleRepo) ListTemplateItems(_ context.Context, templateID uuid.UUID) ([]LifecycleChecklistTemplateItem, error) {
	var list []LifecycleChecklistTemplateItem
	for _, item := range m.items {
		if item.TemplateID == templateID {
			list = append(list, item)
		}
	}
	return list, nil
}

func (m *mockLifecycleRepo) UpdateTemplateItem(_ context.Context, itemID uuid.UUID, req UpdateLifecycleChecklistTemplateItemRequest) (LifecycleChecklistTemplateItem, error) {
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

func (m *mockLifecycleRepo) DeleteTemplateItem(_ context.Context, itemID uuid.UUID) error {
	delete(m.items, itemID)
	return nil
}

func (m *mockLifecycleRepo) CreateChecklistFromTemplate(_ context.Context, employmentID, templateID uuid.UUID, actorID *uuid.UUID, usersResolver func(context.Context, string) []uuid.UUID) (LifecycleChecklistResponse, error) {
	if m.checklists == nil {
		m.checklists = make(map[uuid.UUID]LifecycleChecklistResponse)
	}
	var templateItems []LifecycleChecklistTemplateItem
	for _, item := range m.items {
		if item.TemplateID == templateID {
			templateItems = append(templateItems, item)
		}
	}
	status, progress := "in_progress", 0
	if len(templateItems) == 0 {
		status, progress = "completed", 100
	}
	checklistID := uuid.New()
	checklist := LifecycleChecklistResponse{ID: checklistID, EmploymentID: employmentID, TemplateID: &templateID, ChecklistType: "onboarding", Status: status, ProgressPercent: progress, Items: make([]LifecycleChecklistItem, 0, len(templateItems))}
	for _, templateItem := range templateItems {
		checklist.Items = append(checklist.Items, LifecycleChecklistItem{ID: uuid.New(), ChecklistID: checklistID, TemplateItemID: &templateItem.ID, Title: templateItem.Title, AssigneeUserID: templateItem.AssigneeUserID, Status: "pending"})
	}
	m.checklists[checklistID] = checklist
	return checklist, nil
}

func (m *mockLifecycleRepo) GetChecklist(_ context.Context, checklistID uuid.UUID) (LifecycleChecklistResponse, error) {
	checklist, ok := m.checklists[checklistID]
	if !ok {
		return LifecycleChecklistResponse{}, ErrChecklistNotFound
	}
	return checklist, nil
}

func (m *mockLifecycleRepo) ListChecklistsForEmployment(_ context.Context, employmentID uuid.UUID) ([]LifecycleChecklistResponse, error) {
	var list []LifecycleChecklistResponse
	for _, checklist := range m.checklists {
		if checklist.EmploymentID == employmentID {
			list = append(list, checklist)
		}
	}
	return list, nil
}

func (m *mockLifecycleRepo) ListChecklists(_ context.Context) ([]LifecycleInstanceSummary, error) {
	var list []LifecycleInstanceSummary
	for _, checklist := range m.checklists {
		templateName, departmentName := "Standard Onboarding", "Engineering"
		list = append(list, LifecycleInstanceSummary{ID: checklist.ID, EmploymentID: checklist.EmploymentID, EmployeeID: uuid.New(), EmployeeName: "Nguyen Van A", EmployeeCode: "EMP-001", DepartmentName: &departmentName, TemplateID: checklist.TemplateID, TemplateName: &templateName, ChecklistType: checklist.ChecklistType, Status: checklist.Status, ProgressPercent: checklist.ProgressPercent, TotalItems: len(checklist.Items), StartedAt: checklist.StartedAt, CreatedAt: checklist.CreatedAt, UpdatedAt: checklist.UpdatedAt})
	}
	return list, nil
}

func (m *mockLifecycleRepo) UpdateChecklistItemStatus(_ context.Context, itemID uuid.UUID, status string, notes *string, completedBy *uuid.UUID) (LifecycleChecklistResponse, error) {
	for checklistID, checklist := range m.checklists {
		for i := range checklist.Items {
			if checklist.Items[i].ID != itemID {
				continue
			}
			checklist.Items[i].Status = status
			checklist.Items[i].Notes = notes
			done := 0
			for _, item := range checklist.Items {
				if item.Status == "completed" || item.Status == "skipped" {
					done++
				}
			}
			checklist.ProgressPercent = int(float64(done) / float64(len(checklist.Items)) * 100.0)
			if checklist.ProgressPercent == 100 {
				checklist.Status = "completed"
			} else {
				checklist.Status = "in_progress"
			}
			m.checklists[checklistID] = checklist
			return checklist, nil
		}
	}
	return LifecycleChecklistResponse{}, ErrChecklistItemNotFound
}

func (m *mockLifecycleRepo) GetItemAssigneeAndChecklist(_ context.Context, itemID uuid.UUID) (*uuid.UUID, uuid.UUID, error) {
	for checklistID, checklist := range m.checklists {
		for _, item := range checklist.Items {
			if item.ID == itemID {
				return item.AssigneeUserID, checklistID, nil
			}
		}
	}
	return nil, uuid.Nil, ErrChecklistItemNotFound
}

func (m *mockLifecycleRepo) GetPendingTasksForUser(_ context.Context, _ uuid.UUID) ([]PendingTaskResponse, error) {
	return m.myTasks, nil
}

type mockLifecycleAuth struct {
	allowed map[string]bool
}

func (m mockLifecycleAuth) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return m.allowed[permission]
}
func (mockLifecycleAuth) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (mockLifecycleAuth) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (mockLifecycleAuth) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (mockLifecycleAuth) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (mockLifecycleAuth) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (mockLifecycleAuth) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func lifecycleRequest(method, path string, body []byte, userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewReader(body))
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestTemplateDefaultReplacement(t *testing.T) {
	userID := uuid.New()
	repo := &mockLifecycleRepo{}
	handler := LifecycleHandler{Repository: repo, Authorizer: mockLifecycleAuth{allowed: map[string]bool{"organization:settings:write": true}}}
	template, _ := repo.CreateTemplate(context.Background(), CreateLifecycleChecklistTemplateRequest{Name: "Template 1", ChecklistType: "onboarding", IsDefault: true})
	if !repo.templates[template.ID].IsDefault {
		t.Fatal("expected initial template to be default")
	}
	payload, _ := json.Marshal(CreateLifecycleChecklistTemplateRequest{Name: "Template 2", ChecklistType: "onboarding", IsDefault: true})
	req := lifecycleRequest(http.MethodPost, "/api/v1/lifecycle-templates", payload, userID)
	rec := httptest.NewRecorder()
	handler.CreateTemplate(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.templates[template.ID].IsDefault {
		t.Fatal("expected previous default flag cleared")
	}
}

func TestTemplateItemNestedNullPatch(t *testing.T) {
	userID, assigneeID := uuid.New(), uuid.New()
	repo := &mockLifecycleRepo{}
	handler := LifecycleHandler{Repository: repo, Authorizer: mockLifecycleAuth{allowed: map[string]bool{"organization:settings:write": true}}}
	item, _ := repo.CreateTemplateItem(context.Background(), uuid.New(), CreateLifecycleChecklistTemplateItemRequest{Title: "Setup Laptop", Category: "it_setup", AssigneeType: "specific_user", AssigneeUserID: &assigneeID, IsRequired: true})
	if item.AssigneeUserID == nil || *item.AssigneeUserID != assigneeID {
		t.Fatal("initial assignee id not set")
	}
	req := lifecycleRequest(http.MethodPut, "/api/v1/lifecycle-templates/"+item.TemplateID.String()+"/items/"+item.ID.String(), []byte(`{"assignee_user_id": null}`), userID)
	req.SetPathValue("tid", item.TemplateID.String())
	req.SetPathValue("iid", item.ID.String())
	rec := httptest.NewRecorder()
	handler.UpdateTemplateItem(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.items[item.ID].AssigneeUserID != nil {
		t.Fatal("expected assignee_user_id cleared to nil")
	}
}

func TestChecklistProgressAndAssigneeAuth(t *testing.T) {
	employmentID, assigneeUser, nonAssigneeUser := uuid.New(), uuid.New(), uuid.New()
	repo := &mockLifecycleRepo{}
	handler := LifecycleHandler{Repository: repo, Authorizer: mockLifecycleAuth{allowed: map[string]bool{"employees:update": false}}}
	templateID := uuid.New()
	_, _ = repo.CreateTemplateItem(context.Background(), templateID, CreateLifecycleChecklistTemplateItemRequest{Title: "Sign Contract", Category: "documentation", AssigneeType: "specific_user", AssigneeUserID: &assigneeUser, IsRequired: true})
	checklist, _ := repo.CreateChecklistFromTemplate(context.Background(), employmentID, templateID, nil, nil)
	itemID := checklist.Items[0].ID
	request := func(userID uuid.UUID) *http.Request {
		req := lifecycleRequest(http.MethodPatch, "/api/v1/lifecycle/"+checklist.ID.String()+"/items/"+itemID.String(), []byte(`{"status":"completed"}`), userID)
		req.SetPathValue("id", checklist.ID.String())
		req.SetPathValue("iid", itemID.String())
		return req
	}
	recForbidden := httptest.NewRecorder()
	handler.UpdateItemStatus(recForbidden, request(nonAssigneeUser))
	if recForbidden.Code != http.StatusForbidden {
		t.Fatalf("non-assignee without permission should be 403, got %d", recForbidden.Code)
	}
	recAllowed := httptest.NewRecorder()
	handler.UpdateItemStatus(recAllowed, request(assigneeUser))
	if recAllowed.Code != http.StatusOK {
		t.Fatalf("assignee should be allowed, got %d: %s", recAllowed.Code, recAllowed.Body.String())
	}
	var updated LifecycleChecklistResponse
	_ = json.Unmarshal(recAllowed.Body.Bytes(), &updated)
	if updated.ProgressPercent != 100 || updated.Status != "completed" {
		t.Fatalf("expected completed checklist at 100%%, got %d%% %s", updated.ProgressPercent, updated.Status)
	}
}

func TestListChecklists(t *testing.T) {
	employmentID := uuid.New()
	repo := &mockLifecycleRepo{}
	handler := LifecycleHandler{Repository: repo, Authorizer: mockLifecycleAuth{allowed: map[string]bool{"employees:read": true}}}
	_, _ = repo.CreateChecklistFromTemplate(context.Background(), employmentID, uuid.New(), nil, nil)
	req := lifecycleRequest(http.MethodGet, "/api/v1/lifecycle/instances", nil, uuid.New())
	rec := httptest.NewRecorder()
	handler.ListChecklists(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d: %s", rec.Code, rec.Body.String())
	}
	var list []LifecycleInstanceSummary
	if err := json.Unmarshal(rec.Body.Bytes(), &list); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}
	if len(list) != 1 || list[0].EmployeeName != "Nguyen Van A" {
		t.Fatalf("unexpected checklist summaries: %+v", list)
	}
}
