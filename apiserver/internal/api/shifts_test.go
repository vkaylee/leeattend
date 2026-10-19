package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockShiftRepository struct {
	createErr  error
	listErr    error
	updateErr  error
	versionErr error
	assignErr  error
}

func (m *mockShiftRepository) CreateShift(_ context.Context, workspaceID, _ uuid.UUID, req CreateShiftRequest) (ShiftRecord, error) {
	if m.createErr != nil {
		return ShiftRecord{}, m.createErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return ShiftRecord{ID: uuid.New(), WorkspaceID: workspaceID, Name: req.Name, ReportSymbol: req.ReportSymbol, IsActive: true, CreatedAt: now, UpdatedAt: now}, nil
}

func (m *mockShiftRepository) ListShifts(_ context.Context, _ uuid.UUID, _, _ *uuid.UUID, page, limit int) (PaginatedShifts, error) {
	if m.listErr != nil {
		return PaginatedShifts{}, m.listErr
	}
	return PaginatedShifts{Data: []ShiftRecord{}, Page: page, Limit: limit}, nil
}

func (m *mockShiftRepository) UpdateShift(_ context.Context, workspaceID, templateID, _ uuid.UUID, req UpdateShiftRequest) (ShiftRecord, error) {
	if m.updateErr != nil {
		return ShiftRecord{}, m.updateErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return ShiftRecord{ID: templateID, WorkspaceID: workspaceID, Name: req.Name, ReportSymbol: req.ReportSymbol, IsActive: true, CreatedAt: now, UpdatedAt: now}, nil
}

func (m *mockShiftRepository) ListShiftVersions(_ context.Context, _ uuid.UUID, templateID uuid.UUID) ([]ShiftVersionRecord, error) {
	if m.versionErr != nil {
		return nil, m.versionErr
	}
	now := time.Now().UTC().Format(time.RFC3339)
	return []ShiftVersionRecord{
		{
			ID:            uuid.New(),
			TemplateID:    templateID,
			VersionNumber: 1,
			Name:          "Day",
			EffectiveFrom: "2026-10-19",
			ChangeType:    "create",
			CreatedBy:     uuid.New(),
			CreatedAt:     now,
			Intervals:     []ShiftIntervalRecord{},
		},
	}, nil
}

func (m *mockShiftRepository) AssignShift(_ context.Context, _ uuid.UUID, _ uuid.UUID, _ AssignShiftRequest) error {
	return m.assignErr
}
func TestCreateShift_RequiresPermission(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+workspaceID.String()+"/shifts", bytes.NewBufferString(`{"name":"Day","intervals":[{"start_time":"08:00:00","end_time":"17:00:00"}]}`))
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestCreateShift_RejectsInvalidInterval(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+workspaceID.String()+"/shifts", bytes.NewBufferString(`{"name":"Day","intervals":[{"start_time":"bad","end_time":"17:00:00"}]}`))
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d", rec.Code)
	}
}

func TestCreateShift_Success(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"name":"Day","report_symbol":"D","intervals":[{"start_time":"08:00:00","end_time":"17:00:00","break_duration_minutes":30}]}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+workspaceID.String()+"/shifts", bytes.NewReader(body))
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	var result ShiftRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &result); err != nil {
		t.Fatal(err)
	}
	if result.Name != "Day" || result.WorkspaceID != workspaceID {
		t.Fatalf("unexpected result: %+v", result)
	}
}

func TestListShifts_UsesPagination(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+workspaceID.String()+"/shifts?page=2&limit=10", nil)
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var result PaginatedShifts
	if err := json.Unmarshal(rec.Body.Bytes(), &result); err != nil {
		t.Fatal(err)
	}
	if result.Page != 2 || result.Limit != 10 {
		t.Fatalf("unexpected pagination: %+v", result)
	}
}

func TestUpdateShift_RequiresPermission(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/"+uuid.New().String(), bytes.NewBufferString(`{"name":"Day","intervals":[{"start_time":"08:00:00","end_time":"17:00:00"}]}`))
	req.SetPathValue("ws_id", workspaceID.String())
	req.SetPathValue("shift_id", uuid.New().String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Update(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestUpdateShift_RejectsInvalidEffectiveDate(t *testing.T) {
	workspaceID := uuid.New()
	templateID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/"+templateID.String(), bytes.NewBufferString(`{"name":"Day","effective_date":"19-10-2026","intervals":[{"start_time":"08:00:00","end_time":"17:00:00"}]}`))
	req.SetPathValue("ws_id", workspaceID.String())
	req.SetPathValue("shift_id", templateID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Update(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestUpdateShift_ReturnsNewVersion(t *testing.T) {
	workspaceID := uuid.New()
	templateID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"name":"Evening","effective_date":"2026-11-01","change_reason":"seasonal","intervals":[{"start_time":"14:00:00","end_time":"22:00:00"}]}`)
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/"+templateID.String(), bytes.NewReader(body))
	req.SetPathValue("ws_id", workspaceID.String())
	req.SetPathValue("shift_id", templateID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var result ShiftRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &result); err != nil {
		t.Fatal(err)
	}
	if result.ID != templateID || result.Name != "Evening" {
		t.Fatalf("unexpected result: %+v", result)
	}
}

func TestListShiftVersions_ReturnsHistory(t *testing.T) {
	workspaceID := uuid.New()
	templateID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/"+templateID.String()+"/versions", nil)
	req.SetPathValue("ws_id", workspaceID.String())
	req.SetPathValue("shift_id", templateID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Versions(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var versions []ShiftVersionRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &versions); err != nil {
		t.Fatal(err)
	}
	if len(versions) != 1 || versions[0].TemplateID != templateID || versions[0].VersionNumber != 1 {
		t.Fatalf("unexpected versions: %+v", versions)
	}
}

func TestAssignShift_RequiresPermission(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	body := []byte(`{"employment_id":"` + uuid.New().String() + `","shift_id":"` + uuid.New().String() + `","start_date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/assign", bytes.NewReader(body))
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Assign(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestAssignShift_RejectsInvertedDateRange(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"employment_id":"` + uuid.New().String() + `","shift_id":"` + uuid.New().String() + `","start_date":"2026-10-20","end_date":"2026-10-19"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/assign", bytes.NewReader(body))
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Assign(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected 400/422, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAssignShift_Success(t *testing.T) {
	workspaceID := uuid.New()
	handler := ShiftHandler{Repository: &mockShiftRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"employment_id":"` + uuid.New().String() + `","shift_id":"` + uuid.New().String() + `","start_date":"2026-10-20","end_date":"2026-10-25","allow_warnings":true,"override_reason":"coverage"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/assign", bytes.NewReader(body))
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Assign(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestAssignShift_MapsComplianceConflict(t *testing.T) {
	workspaceID := uuid.New()
	repo := &mockShiftRepository{assignErr: &ShiftComplianceError{Hard: false}}
	handler := ShiftHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"employment_id":"` + uuid.New().String() + `","shift_id":"` + uuid.New().String() + `","start_date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+workspaceID.String()+"/shifts/assign", bytes.NewReader(body))
	req.SetPathValue("ws_id", workspaceID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Assign(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409, got %d: %s", rec.Code, rec.Body.String())
	}
}
