package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockHolidayRepository struct {
	listFn         func(ctx context.Context, wsID uuid.UUID, year *int32, locID *uuid.UUID, page, limit int64) (PaginatedHolidays, error)
	getFn          func(ctx context.Context, wsID, id uuid.UUID) (HolidayRecord, error)
	createFn       func(ctx context.Context, wsID uuid.UUID, req CreateHolidayRequest) (HolidayRecord, error)
	updateFn       func(ctx context.Context, wsID, id uuid.UUID, req UpdateHolidayRequest) (HolidayRecord, error)
	deleteFn       func(ctx context.Context, wsID, id uuid.UUID) error
	processLeaveFn func(ctx context.Context, wsID, id uuid.UUID) ([]HolidayLeaveResult, error)
}

func (m *mockHolidayRepository) ListHolidays(ctx context.Context, wsID uuid.UUID, year *int32, locID *uuid.UUID, page, limit int64) (PaginatedHolidays, error) {
	if m.listFn != nil {
		return m.listFn(ctx, wsID, year, locID, page, limit)
	}
	return PaginatedHolidays{Data: []HolidayRecord{}, Total: 0, Page: page, Limit: limit}, nil
}

func (m *mockHolidayRepository) GetHolidayByID(ctx context.Context, wsID, id uuid.UUID) (HolidayRecord, error) {
	if m.getFn != nil {
		return m.getFn(ctx, wsID, id)
	}
	return HolidayRecord{ID: id, WorkspaceID: wsID, Name: "Tet", Date: "2026-02-17", HolidayType: "national", IsPaid: true}, nil
}

func (m *mockHolidayRepository) CreateHoliday(ctx context.Context, wsID uuid.UUID, req CreateHolidayRequest) (HolidayRecord, error) {
	if m.createFn != nil {
		return m.createFn(ctx, wsID, req)
	}
	return HolidayRecord{ID: uuid.New(), WorkspaceID: wsID, Name: req.Name, Date: req.Date, HolidayType: "national", IsPaid: true}, nil
}

func (m *mockHolidayRepository) UpdateHoliday(ctx context.Context, wsID, id uuid.UUID, req UpdateHolidayRequest) (HolidayRecord, error) {
	if m.updateFn != nil {
		return m.updateFn(ctx, wsID, id, req)
	}
	return HolidayRecord{ID: id, WorkspaceID: wsID, Name: "Updated", Date: "2026-02-17", HolidayType: "national", IsPaid: true}, nil
}

func (m *mockHolidayRepository) DeleteHoliday(ctx context.Context, wsID, id uuid.UUID) error {
	if m.deleteFn != nil {
		return m.deleteFn(ctx, wsID, id)
	}
	return nil
}

func (m *mockHolidayRepository) ProcessHolidayLeave(ctx context.Context, wsID, id uuid.UUID) ([]HolidayLeaveResult, error) {
	if m.processLeaveFn != nil {
		return m.processLeaveFn(ctx, wsID, id)
	}
	return []HolidayLeaveResult{{Type: "Paid"}}, nil
}

type mockHolidayAuthorizer struct {
	isMember    bool
	permissions map[string]bool
}

func (a mockHolidayAuthorizer) IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error) {
	return a.isMember, nil
}

func (a mockHolidayAuthorizer) EffectivePermissions(context.Context, uuid.UUID, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (a mockHolidayAuthorizer) HasPermission(_ context.Context, _, _ uuid.UUID, perm string) bool {
	return a.permissions[perm]
}

func TestHolidayHandler_Unauthenticated(t *testing.T) {
	wsID := uuid.New()
	handler := HolidayHandler{
		Repository: &mockHolidayRepository{},
		Authorizer: mockHolidayAuthorizer{isMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/holidays", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	rec := httptest.NewRecorder()
	handler.List(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}

func TestHolidayHandler_NonMemberForbidden(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	handler := HolidayHandler{
		Repository: &mockHolidayRepository{},
		Authorizer: mockHolidayAuthorizer{isMember: false},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/holidays", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.List(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestHolidayHandler_ListPaginationAndFilters(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	locID := uuid.New()
	var capturedYear *int32
	var capturedLoc *uuid.UUID
	var capturedPage, capturedLimit int64

	repo := &mockHolidayRepository{
		listFn: func(_ context.Context, _ uuid.UUID, year *int32, loc *uuid.UUID, page, limit int64) (PaginatedHolidays, error) {
			capturedYear = year
			capturedLoc = loc
			capturedPage = page
			capturedLimit = limit
			return PaginatedHolidays{
				Data: []HolidayRecord{
					{
						ID:          uuid.New(),
						WorkspaceID: wsID,
						Name:        "New Year",
						Date:        "2026-01-01",
						HolidayType: "national",
						IsPaid:      true,
					},
				},
				Total: 1,
				Page:  page,
				Limit: limit,
			}, nil
		},
	}
	handler := HolidayHandler{
		Repository: repo,
		Authorizer: mockHolidayAuthorizer{isMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/holidays?year=2026&location_id="+locID.String()+"&page=2&limit=25", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.List(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if capturedYear == nil || *capturedYear != 2026 {
		t.Fatalf("expected year 2026, got %v", capturedYear)
	}
	if capturedLoc == nil || *capturedLoc != locID {
		t.Fatalf("expected locID %v, got %v", locID, capturedLoc)
	}
	if capturedPage != 2 || capturedLimit != 25 {
		t.Fatalf("expected page 2 limit 25, got page %d limit %d", capturedPage, capturedLimit)
	}
}

func TestHolidayHandler_GetNotFound(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	hID := uuid.New()
	repo := &mockHolidayRepository{
		getFn: func(context.Context, uuid.UUID, uuid.UUID) (HolidayRecord, error) {
			return HolidayRecord{}, ErrHolidayNotFound
		},
	}
	handler := HolidayHandler{
		Repository: repo,
		Authorizer: mockHolidayAuthorizer{isMember: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/holidays/"+hID.String(), nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", hID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.Get(rec, req)

	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestHolidayHandler_CreateRequiresPermission(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	handler := HolidayHandler{
		Repository: &mockHolidayRepository{},
		Authorizer: mockHolidayAuthorizer{isMember: true, permissions: map[string]bool{}},
	}

	body := `{"name":"National Day","date":"2026-09-02"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/holidays", bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)

	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestHolidayHandler_CreateConflict(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	repo := &mockHolidayRepository{
		createFn: func(context.Context, uuid.UUID, CreateHolidayRequest) (HolidayRecord, error) {
			return HolidayRecord{}, ErrHolidayAlreadyExists
		},
	}
	handler := HolidayHandler{
		Repository: repo,
		Authorizer: mockHolidayAuthorizer{
			isMember:    true,
			permissions: map[string]bool{"holidays:manage": true},
		},
	}

	body := `{"name":"National Day","date":"2026-09-02"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/holidays", bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)

	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHolidayHandler_CreateSuccess(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	createdID := uuid.New()
	otMult := decimal.NewFromFloat(2.0)
	repo := &mockHolidayRepository{
		createFn: func(_ context.Context, ws uuid.UUID, r CreateHolidayRequest) (HolidayRecord, error) {
			return HolidayRecord{
				ID:           createdID,
				WorkspaceID:  ws,
				Name:         r.Name,
				Date:         r.Date,
				HolidayType:  "national",
				IsPaid:       true,
				OTMultiplier: &otMult,
			}, nil
		},
	}
	handler := HolidayHandler{
		Repository: repo,
		Authorizer: mockHolidayAuthorizer{
			isMember:    true,
			permissions: map[string]bool{"holidays:manage": true},
		},
	}

	body := `{"name":"National Day","date":"2026-09-02","ot_multiplier":2.0}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/holidays", bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHolidayHandler_UpdateAndPatch(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	hID := uuid.New()
	repo := &mockHolidayRepository{
		updateFn: func(_ context.Context, ws, id uuid.UUID, req UpdateHolidayRequest) (HolidayRecord, error) {
			return HolidayRecord{
				ID:          id,
				WorkspaceID: ws,
				Name:        *req.Name,
				Date:        "2026-09-02",
				HolidayType: "company",
				IsPaid:      false,
			}, nil
		},
	}
	handler := HolidayHandler{
		Repository: repo,
		Authorizer: mockHolidayAuthorizer{
			isMember:    true,
			permissions: map[string]bool{"holidays:manage": true},
		},
	}

	body := `{"name":"Company Anniversary","holiday_type":"company","is_paid":false}`
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/holidays/"+hID.String(), bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", hID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.Update(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestHolidayHandler_DeleteSuccessAndNotFound(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	hID := uuid.New()
	deleted := false
	repo := &mockHolidayRepository{
		deleteFn: func(_ context.Context, _, id uuid.UUID) error {
			if id == hID {
				deleted = true
				return nil
			}
			return ErrHolidayNotFound
		},
	}
	handler := HolidayHandler{
		Repository: repo,
		Authorizer: mockHolidayAuthorizer{
			isMember:    true,
			permissions: map[string]bool{"holidays:manage": true},
		},
	}

	req := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/holidays/"+hID.String(), nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", hID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.Delete(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
	if !deleted {
		t.Fatal("expected holiday to be deleted")
	}

	notFoundID := uuid.New()
	req2 := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/holidays/"+notFoundID.String(), nil)
	req2 = req2.WithContext(httpinfra.WithOrganizationIDForTest(req2.Context(), wsID))
	req2.SetPathValue("id", notFoundID.String())
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), userID))
	rec2 := httptest.NewRecorder()
	handler.Delete(rec2, req2)

	if rec2.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec2.Code)
	}
}

func TestHolidayHandler_ProcessLeaveResponses(t *testing.T) {
	wsID := uuid.New()
	userID := uuid.New()
	hID := uuid.New()
	empID := uuid.New()
	reqID := uuid.New()
	leaveTypeName := "Annual Leave"

	tests := []struct {
		name         string
		results      []HolidayLeaveResult
		repoErr      error
		expectedCode int
	}{
		{
			name:         "paid holiday",
			results:      []HolidayLeaveResult{{Type: "Paid"}},
			expectedCode: http.StatusOK,
		},
		{
			name: "leave deducted and unpaid",
			results: []HolidayLeaveResult{
				{Type: "LeaveDeducted", EmployeeID: &empID, LeaveRequestID: &reqID, LeaveTypeName: &leaveTypeName},
				{Type: "Unpaid", EmployeeID: &empID},
			},
			expectedCode: http.StatusOK,
		},
		{
			name:         "leave type not found",
			repoErr:      ErrLeaveTypeNotFound,
			expectedCode: http.StatusNotFound,
		},
		{
			name:         "holiday not found",
			repoErr:      ErrHolidayNotFound,
			expectedCode: http.StatusNotFound,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			repo := &mockHolidayRepository{
				processLeaveFn: func(context.Context, uuid.UUID, uuid.UUID) ([]HolidayLeaveResult, error) {
					if tc.repoErr != nil {
						return nil, tc.repoErr
					}
					return tc.results, nil
				},
			}
			handler := HolidayHandler{
				Repository: repo,
				Authorizer: mockHolidayAuthorizer{
					isMember:    true,
					permissions: map[string]bool{"holidays:manage": true},
				},
			}

			req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/holidays/"+hID.String()+"/process-leave", nil)
			req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
			req.SetPathValue("id", hID.String())
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
			rec := httptest.NewRecorder()
			handler.ProcessLeave(rec, req)

			if rec.Code != tc.expectedCode {
				t.Fatalf("expected status %d, got %d: %s", tc.expectedCode, rec.Code, rec.Body.String())
			}
		})
	}
}

func TestHolidayLeaveResult_Serialization(t *testing.T) {
	paid := HolidayLeaveResult{Type: "Paid"}
	data, err := json.Marshal(paid)
	if err != nil {
		t.Fatal(err)
	}
	if string(data) != `"Paid"` {
		t.Fatalf("expected \"Paid\", got %s", string(data))
	}

	empID := uuid.New()
	reqID := uuid.New()
	name := "Annual Leave"
	deducted := HolidayLeaveResult{
		Type:           "LeaveDeducted",
		EmployeeID:     &empID,
		LeaveRequestID: &reqID,
		LeaveTypeName:  &name,
	}
	data, err = json.Marshal(deducted)
	if err != nil {
		t.Fatal(err)
	}
	var decoded HolidayLeaveResult
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatal(err)
	}
	if decoded.Type != "LeaveDeducted" || *decoded.EmployeeID != empID || *decoded.LeaveRequestID != reqID || *decoded.LeaveTypeName != name {
		t.Fatalf("mismatch in decoded struct: %+v", decoded)
	}
}
