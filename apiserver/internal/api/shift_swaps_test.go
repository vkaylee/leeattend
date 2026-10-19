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

type mockShiftSwapRepository struct {
	createErr  error
	approveErr error
	lastStatus ShiftSwapStatus
}

func (m mockShiftSwapRepository) CreateSwap(_ context.Context, workspaceID, userID uuid.UUID, req CreateShiftSwapRequest) (ShiftSwapRequest, error) {
	if m.createErr != nil {
		return ShiftSwapRequest{}, m.createErr
	}
	return ShiftSwapRequest{
		ID:                uuid.New(),
		WorkspaceID:       workspaceID,
		RequesterID:       req.RequesterID,
		TargetID:          req.TargetID,
		SwapDate:          req.SwapDate,
		Status:            ShiftSwapPendingAdmin,
		RequestedByUserID: &userID,
	}, nil
}

func (mockShiftSwapRepository) ListSwaps(context.Context, uuid.UUID, *ShiftSwapStatus) ([]ShiftSwapRequestWithNames, error) {
	return []ShiftSwapRequestWithNames{}, nil
}

func (m *mockShiftSwapRepository) ApproveSwap(_ context.Context, _, _, _ uuid.UUID, status ShiftSwapStatus) error {
	m.lastStatus = status
	return m.approveErr
}

func (mockShiftSwapRepository) Stats(context.Context, uuid.UUID) (SwapStats, error) {
	return SwapStats{PendingAdmin: 2, Approved: 5}, nil
}

func TestShiftSwapHandler_Create_RequiresPermission(t *testing.T) {
	wsID := uuid.New()
	handler := ShiftSwapHandler{Repository: &mockShiftSwapRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	body := []byte(`{"requester_id":"` + uuid.New().String() + `","target_id":"` + uuid.New().String() + `","swap_date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestShiftSwapHandler_Create_RejectsSameEmployee(t *testing.T) {
	wsID := uuid.New()
	sameID := uuid.New()
	handler := ShiftSwapHandler{Repository: &mockShiftSwapRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"requester_id":"` + sameID.String() + `","target_id":"` + sameID.String() + `","swap_date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected validation error, got %d", rec.Code)
	}
}

func TestShiftSwapHandler_Create_Success(t *testing.T) {
	wsID := uuid.New()
	handler := ShiftSwapHandler{Repository: &mockShiftSwapRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"requester_id":"` + uuid.New().String() + `","target_id":"` + uuid.New().String() + `","swap_date":"2026-10-20"}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestShiftSwapHandler_Create_MapsConflictAndValidationErrors(t *testing.T) {
	wsID := uuid.New()
	for _, tc := range []struct {
		name         string
		repoErr      error
		expectedCode int
	}{
		{"duplicate active request conflict", ErrShiftSwapConflict, http.StatusConflict},
		{"same shift validation failure", ErrShiftSwapValidation, http.StatusBadRequest},
		{"shift swap not found", ErrShiftSwapNotFound, http.StatusNotFound},
	} {
		t.Run(tc.name, func(t *testing.T) {
			handler := ShiftSwapHandler{Repository: &mockShiftSwapRepository{createErr: tc.repoErr}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
			body := []byte(`{"requester_id":"` + uuid.New().String() + `","target_id":"` + uuid.New().String() + `","swap_date":"2026-10-20"}`)
			req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps", bytes.NewReader(body))
			req.SetPathValue("ws_id", wsID.String())
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
			rec := httptest.NewRecorder()
			handler.Create(rec, req)
			if rec.Code != tc.expectedCode {
				t.Fatalf("expected status %d, got %d: %s", tc.expectedCode, rec.Code, rec.Body.String())
			}
		})
	}
}

func TestShiftSwapHandler_Approve_RejectsInvalidStatus(t *testing.T) {
	wsID, swapID := uuid.New(), uuid.New()
	handler := ShiftSwapHandler{Repository: &mockShiftSwapRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"status":"pending_target"}`)
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps/"+swapID.String()+"/approve", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", swapID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Approve(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected validation error, got %d", rec.Code)
	}
}

func TestShiftSwapHandler_Approve_ConflictOnInvalidStateTransition(t *testing.T) {
	wsID, swapID := uuid.New(), uuid.New()
	handler := ShiftSwapHandler{Repository: &mockShiftSwapRepository{approveErr: ErrShiftSwapConflict}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"status":"approved"}`)
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps/"+swapID.String()+"/approve", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", swapID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Approve(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409 conflict on non-pending_admin transition, got %d", rec.Code)
	}
}

func TestShiftSwapHandler_Approve_Success(t *testing.T) {
	wsID, swapID := uuid.New(), uuid.New()
	repo := &mockShiftSwapRepository{}
	handler := ShiftSwapHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"status":"approved"}`)
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps/"+swapID.String()+"/approve", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", swapID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Approve(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if repo.lastStatus != ShiftSwapApproved {
		t.Fatalf("expected lastStatus approved, got %s", repo.lastStatus)
	}
}

func TestShiftSwapHandler_Stats_Success(t *testing.T) {
	wsID := uuid.New()
	handler := ShiftSwapHandler{Repository: &mockShiftSwapRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/shift-swaps/stats", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Stats(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}
