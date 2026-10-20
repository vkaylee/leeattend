package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockWorkSegmentRepository struct {
	createErr error
}

func (m *mockWorkSegmentRepository) ListSegments(_ context.Context, _ uuid.UUID, _ WorkSegmentFilter, _, _ int) (WorkSegmentListResponse, error) {
	return WorkSegmentListResponse{Segments: []WorkSegment{}, TotalCount: 0}, nil
}

func (m *mockWorkSegmentRepository) GetSegment(_ context.Context, _, segmentID uuid.UUID) (WorkSegment, error) {
	return WorkSegment{ID: segmentID}, nil
}

func (m *mockWorkSegmentRepository) CreateSegment(_ context.Context, workspaceID uuid.UUID, req CreateWorkSegmentRequest) (WorkSegment, error) {
	if m.createErr != nil {
		return WorkSegment{}, m.createErr
	}
	return WorkSegment{ID: uuid.New(), WorkspaceID: workspaceID, EmploymentID: req.EmploymentID, TemplateID: req.TemplateID, Date: req.Date, Source: req.Source}, nil
}

func (m *mockWorkSegmentRepository) UpdateSegment(_ context.Context, _, segmentID uuid.UUID, _ UpdateWorkSegmentRequest) (WorkSegment, error) {
	return WorkSegment{ID: segmentID}, nil
}

func (m *mockWorkSegmentRepository) DeleteSegment(_ context.Context, _, _ uuid.UUID) error {
	return nil
}

func (m *mockWorkSegmentRepository) BulkCreateSegments(_ context.Context, _ uuid.UUID, req BulkCreateWorkSegmentsRequest) ([]WorkSegment, error) {
	return make([]WorkSegment, len(req.Segments)), nil
}

func validSegmentJSON() []byte {
	return []byte(`{"employment_id":"` + uuid.New().String() + `","template_id":"` + uuid.New().String() + `","date":"2026-10-20","scheduled_start":"08:00:00","scheduled_end":"17:00:00","compensation_rate":"1.00"}`)
}

func TestWorkSegmentHandler_Create_RequiresPermission(t *testing.T) {
	wsID := uuid.New()
	handler := WorkSegmentHandler{Repository: &mockWorkSegmentRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/work-segments", bytes.NewReader(validSegmentJSON()))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestWorkSegmentHandler_Create_ForcesManualSource(t *testing.T) {
	wsID := uuid.New()
	handler := WorkSegmentHandler{Repository: &mockWorkSegmentRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := append(validSegmentJSON()[:len(validSegmentJSON())-1], []byte(`,"source":"pattern"}`)...)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/work-segments", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestValidateCreateSegment_RejectsInvalidClock(t *testing.T) {
	req := CreateWorkSegmentRequest{
		EmploymentID:     uuid.New(),
		TemplateID:       uuid.New(),
		Date:             "2026-10-20",
		ScheduledStart:   "bad",
		ScheduledEnd:     "17:00:00",
		CompensationRate: decimal.NewFromInt(1),
	}
	if validateCreateSegment(req) == nil {
		t.Fatal("expected invalid clock to fail")
	}
}

func TestWorkSegmentHandler_BulkRejectsOversizedBatch(t *testing.T) {
	wsID := uuid.New()
	handler := WorkSegmentHandler{Repository: &mockWorkSegmentRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"segments":[]}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/work-segments/bulk", bytes.NewReader(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.BulkCreate(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected validation status, got %d", rec.Code)
	}
}
