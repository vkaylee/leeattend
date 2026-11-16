package api

import (
	"net/http"
	"strconv"
	"time"

	"github.com/google/uuid"
)

func employmentIDFromRequest(r *http.Request) (uuid.UUID, bool) {
	if id, ok := pathUUID(r, "id"); ok {
		return id, true
	}
	return pathUUID(r, "employment_id")
}

func payrollPeriodEmployer(r *http.Request) (uuid.UUID, bool) {
	id, err := uuid.Parse(r.URL.Query().Get("employer_id"))
	return id, err == nil
}

func payrollPeriodPath(r *http.Request) (int, int, error) {
	year, err := strconv.Atoi(r.PathValue("year"))
	if err != nil || year < 2000 || year > 2100 {
		return 0, 0, ErrPayrollPeriodValidation
	}
	month, err := strconv.Atoi(r.PathValue("month"))
	if err != nil || month < 1 || month > 12 {
		return 0, 0, ErrPayrollPeriodValidation
	}
	return year, month, nil
}

func parsePayrollPeriod(r *http.Request) (time.Time, time.Time, error) {
	year, err := strconv.Atoi(r.URL.Query().Get("year"))
	if err != nil {
		return time.Time{}, time.Time{}, ErrPayrollValidation
	}
	month, err := strconv.Atoi(r.URL.Query().Get("month"))
	if err != nil || year < 2000 || year > 2100 || month < 1 || month > 12 {
		return time.Time{}, time.Time{}, ErrPayrollValidation
	}
	start := time.Date(year, time.Month(month), 1, 0, 0, 0, 0, time.UTC)
	return start, start.AddDate(0, 1, 0), nil
}
