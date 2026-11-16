package api

import (
	"bytes"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

func TestPayrollPeriodParsingHelpersPreserveBoundaries(t *testing.T) {
	valid := httptest.NewRequest(http.MethodGet, "/?year=2026&month=1", nil)
	start, end, err := parsePayrollPeriod(valid)
	if err != nil || !start.Equal(time.Date(2026, time.January, 1, 0, 0, 0, 0, time.UTC)) || !end.Equal(time.Date(2026, time.February, 1, 0, 0, 0, 0, time.UTC)) {
		t.Fatalf("period = %v, %v, %v", start, end, err)
	}
	for _, query := range []string{"year=1999&month=1", "year=2026&month=0", "year=2026&month=13", "year=bad&month=1"} {
		req := httptest.NewRequest(http.MethodGet, "/?"+query, nil)
		if _, _, err := parsePayrollPeriod(req); err == nil {
			t.Fatalf("query %q accepted invalid payroll period", query)
		}
	}
}

func TestPayrollPeriodHandlersValidateAuthorizationAndPayload(t *testing.T) {
	if _, ok := payrollPeriodEmployer(httptest.NewRequest(http.MethodGet, "/?employer_id="+uuid.NewString(), nil)); !ok {
		t.Fatal("query employer_id must be accepted")
	}
	pathScoped := httptest.NewRequest(http.MethodGet, "/", nil)
	pathScoped.SetPathValue("employer_id", uuid.NewString())
	if _, ok := payrollPeriodEmployer(pathScoped); ok {
		t.Fatal("path employer_id must not be accepted")
	}

	employerID := uuid.New()
	userID := uuid.New()
	base := func(perms map[string]bool) (*httptest.ResponseRecorder, PayrollHandler) {
		rec := httptest.NewRecorder()
		return rec, PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: perms}}
	}

	rec, handler := base(map[string]bool{"payroll:read": true})
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	req.SetPathValue("year", "2026")
	req.SetPathValue("month", "1")
	req.URL.RawQuery = "employer_id=" + employerID.String()
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	handler.GetPayrollPeriod(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("GET period status = %d, want 200: %s", rec.Code, rec.Body.String())
	}

	rec, handler = base(map[string]bool{"payroll:periods:lock": true})
	req = httptest.NewRequest(http.MethodPost, "/", bytes.NewBufferString(`{"version":0}`))
	req.SetPathValue("year", "2026")
	req.SetPathValue("month", "1")
	req.URL.RawQuery = "employer_id=" + employerID.String()
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	handler.LockPayrollPeriod(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("invalid lock version status = %d, want 400", rec.Code)
	}

	rec, handler = base(map[string]bool{})
	req = httptest.NewRequest(http.MethodPost, "/", bytes.NewBufferString(`{"version":1}`))
	req.SetPathValue("year", "2026")
	req.SetPathValue("month", "1")
	req.URL.RawQuery = "employer_id=" + employerID.String()
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	handler.LockPayrollPeriod(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("unauthorized lock status = %d, want 403", rec.Code)
	}

	rec, handler = base(map[string]bool{"payroll:periods:reopen": true})
	req = httptest.NewRequest(http.MethodPost, "/", bytes.NewBufferString(`{"version":1,"reason":""}`))
	req.SetPathValue("year", "2026")
	req.SetPathValue("month", "1")
	req.URL.RawQuery = "employer_id=" + employerID.String()
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	handler.ReopenPayrollPeriod(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("empty reopen reason status = %d, want 400", rec.Code)
	}

}
