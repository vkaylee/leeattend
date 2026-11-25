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

func (m *mockWorkSegmentRepository) ListSegments(_ context.Context, _ WorkSegmentFilter, _, _ int) (WorkSegmentListResponse, error) {
	return WorkSegmentListResponse{Segments: []WorkSegment{}, TotalCount: 0}, nil
}

func (m *mockWorkSegmentRepository) GetSegment(_ context.Context, segmentID uuid.UUID) (WorkSegment, error) {
	return WorkSegment{ID: segmentID}, nil
}

func (m *mockWorkSegmentRepository) CreateSegment(_ context.Context, req CreateWorkSegmentRequest) (WorkSegment, error) {
	if m.createErr != nil {
		return WorkSegment{}, m.createErr
	}
	return WorkSegment{ID: uuid.New(), EmploymentID: req.EmploymentID, TemplateID: req.TemplateID, Date: req.Date, Source: req.Source}, nil
}

func (m *mockWorkSegmentRepository) UpdateSegment(_ context.Context, segmentID uuid.UUID, _ UpdateWorkSegmentRequest) (WorkSegment, error) {
	return WorkSegment{ID: segmentID}, nil
}

func (m *mockWorkSegmentRepository) DeleteSegment(_ context.Context, _ uuid.UUID) error {
	return nil
}

func (m *mockWorkSegmentRepository) BulkCreateSegments(_ context.Context, req BulkCreateWorkSegmentsRequest) ([]WorkSegment, error) {
	return make([]WorkSegment, len(req.Segments)), nil
}

func validSegmentJSON() []byte {
	return []byte(`{"employment_id":"` + uuid.New().String() + `","template_id":"` + uuid.New().String() + `","date":"2026-10-20","scheduled_start":"08:00:00","scheduled_end":"17:00:00","compensation_rate":"1.00"}`)
}

func TestWorkSegmentHandler_Create_RequiresPermission(t *testing.T) {
	handler := WorkSegmentHandler{Repository: &mockWorkSegmentRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/work-segments", bytes.NewReader(validSegmentJSON()))

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestWorkSegmentHandler_Create_ForcesManualSource(t *testing.T) {
	handler := WorkSegmentHandler{Repository: &mockWorkSegmentRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := append(validSegmentJSON()[:len(validSegmentJSON())-1], []byte(`,"source":"pattern"}`)...)
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/work-segments", bytes.NewReader(body))

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestValidateCreateSegment_RejectsInvalidClock(t *testing.T) {
	one := decimal.NewFromInt(1)
	req := CreateWorkSegmentRequest{
		EmploymentID:     uuid.New(),
		TemplateID:       uuid.New(),
		Date:             "2026-10-20",
		ScheduledStart:   "bad",
		ScheduledEnd:     "17:00:00",
		CompensationRate: &one,
	}
	if validateCreateSegment(req) == nil {
		t.Fatal("expected invalid clock to fail")
	}
}

func TestWorkSegmentHandler_Create_DistinguishesExplicitZeroAndOmittedCompensationRate(t *testing.T) {
	var captured []CreateWorkSegmentRequest
	repo := &capturingWorkSegmentRepository{onCreate: func(req CreateWorkSegmentRequest) {
		captured = append(captured, req)
	}}
	handler := WorkSegmentHandler{Repository: repo, Authorizer: mockAttendanceAuthorizer{allowed: true}}

	zeroBody := []byte(`{"employment_id":"` + uuid.New().String() + `","template_id":"` + uuid.New().String() + `","date":"2026-11-20","scheduled_start":"08:00:00","scheduled_end":"17:00:00","segment_type":"on_call","compensation_rate":"0"}`)
	reqZero := httptest.NewRequest(http.MethodPost, "/api/v1/work-segments", bytes.NewReader(zeroBody))
	reqZero = reqZero.WithContext(httpinfra.WithUserIDForTest(reqZero.Context(), uuid.New()))
	recZero := httptest.NewRecorder()
	handler.Create(recZero, reqZero)
	if recZero.Code != http.StatusCreated {
		t.Fatalf("expected 201 for explicit zero rate, got %d: %s", recZero.Code, recZero.Body.String())
	}

	omittedBody := []byte(`{"employment_id":"` + uuid.New().String() + `","template_id":"` + uuid.New().String() + `","date":"2026-11-20","scheduled_start":"08:00:00","scheduled_end":"17:00:00"}`)
	reqOmitted := httptest.NewRequest(http.MethodPost, "/api/v1/work-segments", bytes.NewReader(omittedBody))
	reqOmitted = reqOmitted.WithContext(httpinfra.WithUserIDForTest(reqOmitted.Context(), uuid.New()))
	recOmitted := httptest.NewRecorder()
	handler.Create(recOmitted, reqOmitted)
	if recOmitted.Code != http.StatusCreated {
		t.Fatalf("expected 201 for omitted rate, got %d: %s", recOmitted.Code, recOmitted.Body.String())
	}

	if len(captured) != 2 {
		t.Fatalf("expected 2 captured creates, got %d", len(captured))
	}
	if captured[0].CompensationRate == nil || !captured[0].CompensationRate.Equal(decimal.Zero) {
		t.Fatalf("explicit zero compensation_rate = %v, want non-nil 0", captured[0].CompensationRate)
	}
	if captured[1].CompensationRate != nil {
		t.Fatalf("omitted compensation_rate = %v, want nil", captured[1].CompensationRate)
	}
}

type capturingWorkSegmentRepository struct {
	mockWorkSegmentRepository
	onCreate func(CreateWorkSegmentRequest)
}

func (c *capturingWorkSegmentRepository) CreateSegment(_ context.Context, req CreateWorkSegmentRequest) (WorkSegment, error) {
	if c.onCreate != nil {
		c.onCreate(req)
	}
	rate := decimal.NewFromInt(1)
	if req.CompensationRate != nil {
		rate = *req.CompensationRate
	}
	return WorkSegment{ID: uuid.New(), EmploymentID: req.EmploymentID, TemplateID: req.TemplateID, Date: req.Date, CompensationRate: rate, Source: req.Source}, nil
}

func TestWorkSegmentHandler_BulkRejectsOversizedBatch(t *testing.T) {
	handler := WorkSegmentHandler{Repository: &mockWorkSegmentRepository{}, Authorizer: mockAttendanceAuthorizer{allowed: true}}
	body := []byte(`{"segments":[]}`)
	req := httptest.NewRequest(http.MethodPost, "/api/v1"+"/work-segments/bulk", bytes.NewReader(body))

	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.BulkCreate(rec, req)
	if rec.Code != http.StatusBadRequest && rec.Code != http.StatusUnprocessableEntity {
		t.Fatalf("expected validation status, got %d", rec.Code)
	}
}
