package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

// BenchmarkAuthUserContextResolution measures context-based user ID extraction overhead.
func BenchmarkAuthUserContextResolution(b *testing.B) {
	ctx := httpinfra.WithUserIDForTest(context.Background(), uuid.New())
	b.ResetTimer()
	for b.Loop() {
		uid, ok := httpinfra.UserIDFromContext(ctx)
		if !ok || uid == uuid.Nil {
			b.Fatal("failed to resolve user ID")
		}
	}
}

// BenchmarkRBACPermissionCheck measures in-memory single-organization permission evaluation latency.
func BenchmarkRBACPermissionCheck(b *testing.B) {
	authz := singleOrgTestAuthorizer{
		permissions: map[string]bool{
			"departments:read":   true,
			"departments:create": true,
			"departments:delete": true,
			"employees:read":     true,
			"employees:write":    true,
		},
	}
	ctx := context.Background()
	user := uuid.New()

	b.ResetTimer()
	for b.Loop() {
		if !authz.HasPermission(ctx, user, "departments:read") {
			b.Fatal("unexpected permission denial")
		}
	}
}

// BenchmarkDepartmentListHandler measures execution latency for read-only department listing.
func BenchmarkDepartmentListHandler(b *testing.B) {
	userID := uuid.New()
	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: singleOrgTestAuthorizer{permissions: map[string]bool{"departments:read": true}},
	}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/departments", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))

	b.ResetTimer()
	for b.Loop() {
		rec := httptest.NewRecorder()
		h.List(rec, req)
		if rec.Code != http.StatusOK {
			b.Fatalf("expected 200, got %d", rec.Code)
		}
	}
}

// BenchmarkPayrollDecimalPrecision measures high-frequency decimal calculations
// matching payroll evaluation rules without float rounding errors.
func BenchmarkPayrollDecimalPrecision(b *testing.B) {
	rate := decimal.NewFromFloat(45.50)
	hours := decimal.NewFromFloat(168.0)
	taxRate := decimal.NewFromFloat(0.15)

	b.ResetTimer()
	for b.Loop() {
		gross := rate.Mul(hours)
		tax := gross.Mul(taxRate)
		net := gross.Sub(tax)
		if net.IsNegative() {
			b.Fatal("unexpected negative net pay")
		}
	}
}
