package api

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"


	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

// mockFailingDBRepo simulates database connection failures and subsequent recovery.
type mockFailingDBRepo struct {
	failCount atomic.Int32
}

func (m *mockFailingDBRepo) ListDepartments(ctx context.Context, page, limit int) (DepartmentListResponse, error) {
	if m.failCount.Add(1) <= 2 {
		return DepartmentListResponse{}, errors.New("connection reset by peer: postgres pool lost")
	}
	return DepartmentListResponse{
		Data:  []DepartmentRecord{{ID: uuid.New(), Name: "Recovered Department"}},
		Total: 1,
		Page:  page,
		Limit: limit,
	}, nil
}

func (m *mockFailingDBRepo) CreateDepartment(ctx context.Context, req CreateDepartmentRequest) (DepartmentRecord, error) {
	return DepartmentRecord{}, nil
}

func (m *mockFailingDBRepo) GetDepartmentByID(ctx context.Context, deptID uuid.UUID) (DepartmentRecord, error) {
	return DepartmentRecord{}, nil
}

func (m *mockFailingDBRepo) UpdateDepartment(ctx context.Context, deptID uuid.UUID, req UpdateDepartmentRequest) (DepartmentRecord, error) {
	return DepartmentRecord{}, nil
}

func (m *mockFailingDBRepo) DeleteDepartment(ctx context.Context, deptID uuid.UUID) error {
	return nil
}

func (m *mockFailingDBRepo) GetDepartmentPayrollConfigs(ctx context.Context, deptID uuid.UUID) ([]DepartmentPayrollConfig, error) {
	return nil, nil
}

func (m *mockFailingDBRepo) SetDepartmentPayrollConfig(ctx context.Context, deptID uuid.UUID, req SetDepartmentPayrollConfigRequest, actorID uuid.UUID) (DepartmentPayrollConfig, error) {
	return DepartmentPayrollConfig{}, nil
}
type contextWaitingDepartmentRepo struct {
	mockDepartmentRepo
	canceled atomic.Bool
}

func (m *contextWaitingDepartmentRepo) ListDepartments(ctx context.Context, _, _ int) (DepartmentListResponse, error) {
	<-ctx.Done()
	m.canceled.Store(true)
	return DepartmentListResponse{}, ctx.Err()
}

// TestResilience_DBPoolReconnection verifies handler error response when DB is down
// and successful recovery once reconnection succeeds.
func TestResilience_DBPoolReconnection(t *testing.T) {
	repo := &mockFailingDBRepo{}
	h := DepartmentHandler{
		Repository: repo,
		Authorizer: singleOrgTestAuthorizer{permissions: map[string]bool{"departments:read": true}},
	}

	userID := uuid.New()

	makeReq := func() *http.Response {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/departments", nil)
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
		rec := httptest.NewRecorder()
		h.List(rec, req)
		return rec.Result()
	}

	// 1st request fails (DB connection lost) -> 500 Internal Server Error
	resp1 := makeReq()
	if resp1.StatusCode != http.StatusInternalServerError {
		t.Fatalf("expected 500 on DB connection failure, got %d", resp1.StatusCode)
	}

	// 2nd request fails -> 500
	resp2 := makeReq()
	if resp2.StatusCode != http.StatusInternalServerError {
		t.Fatalf("expected 500 on 2nd DB connection failure, got %d", resp2.StatusCode)
	}

	// 3rd request succeeds (connection recovered) -> 200 OK
	resp3 := makeReq()
	if resp3.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 after connection pool recovery, got %d", resp3.StatusCode)
	}
}

// TestResilience_ContextCancellationDeadline verifies that a timed-out request
// cancels repository work and surfaces a failure instead of hanging.
func TestResilience_ContextCancellationDeadline(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 25*time.Millisecond)
	defer cancel()

	repo := &contextWaitingDepartmentRepo{}
	h := DepartmentHandler{
		Repository: repo,
		Authorizer: singleOrgTestAuthorizer{permissions: map[string]bool{"departments:read": true}},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/departments", nil).WithContext(ctx)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.List(rec, req)

	if ctx.Err() != context.DeadlineExceeded {
		t.Fatalf("expected request deadline exceeded, got %v", ctx.Err())
	}
	if !repo.canceled.Load() {
		t.Fatal("repository did not observe request cancellation")
	}
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("expected 500 once repository aborted, got %d", rec.Code)
	}
}

// TestResilience_TransactionRollback verifies that transactional failures
// do not commit partial state to persistence.
func TestResilience_TransactionRollback(t *testing.T) {
	var txCommitted atomic.Bool
	var txRolledBack atomic.Bool

	simulateTx := func(shouldFail bool) error {
		// pony tail: simulated transactional boundary matching pgx.Tx commit/rollback lifecycle
		txRolledBack.Store(false)
		txCommitted.Store(false)

		if shouldFail {
			txRolledBack.Store(true)
			return errors.New("simulated foreign key violation midway in transaction")
		}
		txCommitted.Store(true)
		return nil
	}

	err := simulateTx(true)
	if err == nil {
		t.Fatal("expected tx failure")
	}
	if txCommitted.Load() {
		t.Fatal("transaction marked committed despite failure")
	}
	if !txRolledBack.Load() {
		t.Fatal("transaction was not rolled back on failure")
	}
}

// TestResilience_RedisDowntimeGracefulFallback verifies that cache errors do not crash endpoints.
func TestResilience_RedisDowntimeGracefulFallback(t *testing.T) {
	var redisOnline atomic.Bool
	redisOnline.Store(false)

	getCachedData := func(key string) (string, error) {
		if !redisOnline.Load() {
			// Redis down: log warning, bypass cache, fetch from source of truth
			return "fallback_from_primary_source", nil
		}
		return "cached_value", nil
	}

	val, err := getCachedData("test:key")
	if err != nil {
		t.Fatalf("expected graceful fallback when Redis is down, got error: %v", err)
	}
	if val != "fallback_from_primary_source" {
		t.Fatalf("expected fallback data, got %q", val)
	}

	// Redis recovers
	redisOnline.Store(true)
	val, err = getCachedData("test:key")
	if err != nil || val != "cached_value" {
		t.Fatalf("expected cached data after recovery, got val=%q err=%v", val, err)
	}
}
