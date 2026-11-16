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

type closureTestRepository struct {
	mockAttendanceRepo
	closed bool
	opened bool
}

func (r *closureTestRepository) CloseAttendancePeriod(context.Context, uuid.UUID, int, int, int64, string, uuid.UUID) (AttendanceClosureResponse, error) {
	r.closed = true
	return AttendanceClosureResponse{PeriodID: uuid.New(), ClosureID: ptrUUID(uuid.New()), ClosureNumber: 1, Status: "closed", Version: 2, DayCount: 1, SnapshotHash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}, nil
}

func (r *closureTestRepository) ReopenAttendancePeriod(context.Context, uuid.UUID, int, int, int64, string, uuid.UUID) (AttendanceClosureResponse, error) {
	r.opened = true
	return AttendanceClosureResponse{PeriodID: uuid.New(), Status: "reopened", Version: 3}, nil
}

func (r *closureTestRepository) GetAttendancePeriodStatus(context.Context, uuid.UUID, int, int) (AttendancePeriodStatusResponse, error) {
	return AttendancePeriodStatusResponse{PeriodID: ptrUUID(uuid.New()), Status: "closed", Version: 2, ClosureNumber: 1, SnapshotHash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", PayrollPeriodFound: true, PayrollStatus: "open"}, nil
}

func ptrUUID(value uuid.UUID) *uuid.UUID { return &value }

type closureTestAuthorizer struct {
	allowed bool
	global  bool
}

func (a closureTestAuthorizer) HasPermission(context.Context, uuid.UUID, string) bool {
	return a.allowed
}
func (a closureTestAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return a.global }
func (closureTestAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (closureTestAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (closureTestAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (closureTestAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (closureTestAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func closureRequest(method, path, body string, userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	req.SetPathValue("year", "2026")
	req.SetPathValue("month", "1")
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestGetAttendancePeriodStatusChecksPermissionBeforeLookup(t *testing.T) {
	repo := &closureTestRepository{}
	handler := AttendanceHandler{Repository: repo, Authorizer: closureTestAuthorizer{allowed: false, global: false}}
	rec := httptest.NewRecorder()
	handler.GetAttendancePeriodStatus(rec, closureRequest(http.MethodGet, "/api/v1/attendance/periods/2026/1?employer_id=bad", "", uuid.New()))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected permission denial before path validation, got %d", rec.Code)
	}
}

func TestGetAttendancePeriodStatusContract(t *testing.T) {
	handler := AttendanceHandler{Repository: &closureTestRepository{}, Authorizer: closureTestAuthorizer{allowed: true, global: true}}
	rec := httptest.NewRecorder()
	employerID := uuid.New()
	handler.GetAttendancePeriodStatus(rec, closureRequest(http.MethodGet, "/api/v1/attendance/periods/2026/1?employer_id="+employerID.String(), "", uuid.New()))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestCloseAttendancePeriodChecksPermissionBeforePayload(t *testing.T) {
	repo := &closureTestRepository{}
	handler := AttendanceHandler{Repository: repo, Authorizer: closureTestAuthorizer{allowed: false, global: false}}
	rec := httptest.NewRecorder()
	handler.CloseAttendancePeriod(rec, closureRequest(http.MethodPost, "/api/v1/attendance/periods/2026/1/close?employer_id=bad", "not-json", uuid.New()))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected permission denial before payload parsing, got %d", rec.Code)
	}
	if repo.closed {
		t.Fatal("forbidden request reached repository")
	}
}

func TestCloseAttendancePeriodRequiresGlobalScope(t *testing.T) {
	repo := &closureTestRepository{}
	handler := AttendanceHandler{Repository: repo, Authorizer: closureTestAuthorizer{allowed: true, global: false}}
	rec := httptest.NewRecorder()
	handler.CloseAttendancePeriod(rec, closureRequest(http.MethodPost, "/api/v1/attendance/periods/2026/1/close?employer_id="+uuid.NewString(), `{"version":1,"reason":"monthly close"}`, uuid.New()))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected scoped actor denial, got %d", rec.Code)
	}
	if repo.closed {
		t.Fatal("scoped actor reached repository without employer coverage proof")
	}
}

func TestCloseAndReopenAttendancePeriodContract(t *testing.T) {
	repo := &closureTestRepository{}
	handler := AttendanceHandler{Repository: repo, Authorizer: closureTestAuthorizer{allowed: true, global: true}}
	userID, employerID := uuid.New(), uuid.New()
	path := "/api/v1/attendance/periods/2026/1/close?employer_id=" + employerID.String()
	rec := httptest.NewRecorder()
	handler.CloseAttendancePeriod(rec, closureRequest(http.MethodPost, path, `{"version":1,"reason":"monthly close"}`, userID))
	if rec.Code != http.StatusCreated || !repo.closed {
		t.Fatalf("expected 201 and repository close call, got %d", rec.Code)
	}

	reopenPath := "/api/v1/attendance/periods/2026/1/reopen?employer_id=" + employerID.String()
	rec = httptest.NewRecorder()
	handler.ReopenAttendancePeriod(rec, closureRequest(http.MethodPost, reopenPath, `{"version":2,"reason":"approved correction"}`, userID))
	if rec.Code != http.StatusOK || !repo.opened {
		t.Fatalf("expected 200 and repository reopen call, got %d", rec.Code)
	}
}
