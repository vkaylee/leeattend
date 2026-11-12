package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type salaryManagementRepository struct {
	mockPayrollRepository
	query SalaryManagementQuery
}

func (r *salaryManagementRepository) ListSalaryManagementRows(_ context.Context, query SalaryManagementQuery) (SalaryManagementResponse, error) {
	r.query = query
	return SalaryManagementResponse{Data: []SalaryManagementRow{}, Page: query.Page, Limit: query.Limit, AsOf: "2026-01-01"}, nil
}

func TestListSalaryManagementRejectsMalformedQuery(t *testing.T) {
	repo := &salaryManagementRepository{}
	handler := PayrollHandler{Repository: repo, Authorizer: payrollAuthorizer{}}
	for _, rawQuery := range []string{"as_of=2026/01/01", "page=zero", "limit=3001", "employer_id=bad", "unconfigured=maybe"} {
		t.Run(rawQuery, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodGet, "/api/v1/salary-management?"+rawQuery, nil)
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
			rec := httptest.NewRecorder()
			handler.ListSalaryManagement(rec, req)
			if rec.Code != http.StatusBadRequest {
				t.Fatalf("status = %d, want 400", rec.Code)
			}
		})
	}
}

func TestListSalaryManagementRequiresPayrollReadAndSetsNoStore(t *testing.T) {
	repo := &salaryManagementRepository{}
	for _, test := range []struct {
		name      string
		permitted bool
		want      int
	}{
		{name: "forbidden", permitted: false, want: http.StatusForbidden},
		{name: "allowed", permitted: true, want: http.StatusOK},
	} {
		t.Run(test.name, func(t *testing.T) {
			authorizer := payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": test.permitted}}
			handler := PayrollHandler{Repository: repo, Authorizer: authorizer}
			req := httptest.NewRequest(http.MethodGet, "/api/v1/salary-management?as_of=2026-01-01&page=2&limit=10", nil)
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
			rec := httptest.NewRecorder()
			handler.ListSalaryManagement(rec, req)
			if rec.Code != test.want {
				t.Fatalf("status = %d, want %d", rec.Code, test.want)
			}
			if test.permitted {
				if got := rec.Header().Get("Cache-Control"); got != "no-store" {
					t.Fatalf("Cache-Control = %q, want no-store", got)
				}
				if repo.query.Page != 2 || repo.query.Limit != 10 || repo.query.AsOf == nil || repo.query.AsOf.Format("2006-01-02") != "2026-01-01" {
					t.Fatalf("query not forwarded: %+v", repo.query)
				}
			}
		})
	}
}

func TestParseSalaryManagementQueryDefaultsToOrganizationDate(t *testing.T) {
	query, err := parseSalaryManagementQuery(httptest.NewRequest(http.MethodGet, "/api/v1/salary-management", nil))
	if err != nil {
		t.Fatal(err)
	}
	if query.EmploymentStatus != "active" || query.Page != DefaultPage || query.Limit != DefaultLimit {
		t.Fatalf("unexpected defaults: %+v", query)
	}
	if query.AsOf != nil {
		t.Fatal("expected repository to resolve organization-local current date")
	}
}
