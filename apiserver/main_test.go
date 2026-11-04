package main

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/health"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type routeTestRow struct {
	err error
}

func (r routeTestRow) Scan(dest ...any) error {
	if r.err != nil {
		return r.err
	}
	*(dest[0].(*uuid.UUID)) = uuid.MustParse("018d0000-0000-7000-8000-000000000001")
	return nil
}

type routeTestDB struct {
	row pgx.Row
}

func (db routeTestDB) QueryRow(context.Context, string, ...any) pgx.Row     { return db.row }
func (routeTestDB) Query(context.Context, string, ...any) (pgx.Rows, error) { return nil, nil }
func (routeTestDB) Exec(context.Context, string, ...any) (pgconn.CommandTag, error) {
	return pgconn.CommandTag{}, nil
}

func TestResolveOrganizationID(t *testing.T) {
	organizationID := uuid.MustParse("018d0000-0000-7000-8000-000000000001")
	got, err := resolveOrganizationID(context.Background(), routeTestDB{row: routeTestRow{}})
	if err != nil || got != organizationID {
		t.Fatalf("expected organization %s, got %s (err=%v)", organizationID, got, err)
	}
}

func TestSingleOrganizationHandlerInjectsOrganizationContext(t *testing.T) {
	called := false
	organizationID := uuid.MustParse("018d0000-0000-7000-8000-000000000001")
	handler := singleOrganizationHandler(organizationID, func(w http.ResponseWriter, r *http.Request) {
		called = true
		if got, ok := httpinfra.WorkspaceIDFromContext(r.Context()); !ok || got != organizationID {
			t.Errorf("expected organization context %s, got %s (ok=%v)", organizationID, got, ok)
		}
		if got := r.PathValue("ws_id"); got != "" {
			t.Errorf("expected no synthetic ws_id path value, got %q", got)
		}
		w.WriteHeader(http.StatusNoContent)
	})

	rec := httptest.NewRecorder()
	handler(rec, httptest.NewRequest(http.MethodGet, "/api/v1/employees", nil))
	if !called || rec.Code != http.StatusNoContent {
		t.Fatalf("expected wrapped handler to run with 204, called=%t status=%d", called, rec.Code)
	}
}

func TestSingleOrganizationHandlerRejectsUnavailableOrganization(t *testing.T) {
	rec := httptest.NewRecorder()
	singleOrganizationHandler(uuid.Nil, func(http.ResponseWriter, *http.Request) {
		t.Fatal("wrapped handler must not run when organization is unavailable")
	})(rec, httptest.NewRequest(http.MethodGet, "/api/v1/employees", nil))
	if rec.Code != http.StatusServiceUnavailable {
		t.Fatalf("expected 503, got %d", rec.Code)
	}
}

func TestHealthRoutesMounted(t *testing.T) {
	mux := http.NewServeMux()
	h := health.NewHandler(Version, nil, nil)
	h.SetLive(true)
	mux.HandleFunc("GET /health", h.Liveness)
	mux.HandleFunc("GET /health/ready", h.Readiness)
	mux.HandleFunc("GET /api/v1/health", h.Liveness)

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	rec := httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	rec = httptest.NewRecorder()
	mux.ServeHTTP(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestAppVersionConstant(t *testing.T) {
	if Version != "0.1.0" {
		t.Fatalf("unexpected version: %s", Version)
	}
}
