package repository

import (
	"context"
	"time"

	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func (r PayrollRepository) ListSalaryManagementRows(ctx context.Context, query api.SalaryManagementQuery) (api.SalaryManagementResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.SalaryManagementResponse{}, err
	}

	var asOf time.Time
	if query.AsOf != nil {
		asOf = *query.AsOf
	} else {
		asOf = time.Now().In(organizationLocation(ctx, db))
	}
	asOfDate := asOf.Format("2006-01-02")
	offset := (query.Page - 1) * query.Limit
	args := []any{asOfDate, query.Search, query.Unconfigured, query.EmployerID, query.DepartmentID, query.EmploymentStatus}
	where := `WHERE e.deleted_at IS NULL
	  AND em.deleted_at IS NULL
	  AND er.deleted_at IS NULL
	  AND em.start_date <= $1::date
	  AND (em.end_date IS NULL OR em.end_date >= $1::date)
	  AND ($2 = '' OR e.full_name ILIKE '%' || $2 || '%' OR e.employee_id_number ILIKE '%' || $2 || '%')
	  AND ($4::uuid IS NULL OR em.employer_id = $4)
	  AND ($5::uuid IS NULL OR em.department_id = $5)
	  AND ($6 = '' OR em.status = $6)
	  AND ($3::boolean IS NULL OR (($3 = true AND current_components.component_count = 0) OR ($3 = false AND current_components.component_count > 0)))`

	var total int64
	countSQL := `SELECT count(*)
FROM employments em
JOIN employees e ON e.id = em.employee_id
JOIN employers er ON er.id = em.employer_id
LEFT JOIN LATERAL (
	SELECT count(*)::int AS component_count
	FROM (
		SELECT DISTINCT ON (esc.component_definition_id) esc.component_definition_id
		FROM employee_salary_components esc
		WHERE esc.employment_id = em.id AND esc.effective_date <= $1::date
		ORDER BY esc.component_definition_id, esc.effective_date DESC, esc.created_at DESC, esc.id DESC
	) configured_components
) current_components ON true
` + where
	if err := db.QueryRow(ctx, countSQL, args...).Scan(&total); err != nil {
		return api.SalaryManagementResponse{}, err
	}

	rowsSQL := `SELECT em.id, e.id, e.employee_id_number, e.full_name,
	er.id, er.name, em.department_id, d.name, em.job_title, em.status,
	em.employment_type, em.start_date::text, em.end_date::text,
	current_components.component_count, current_components.latest_effective_date,
	CASE
		WHEN current_components.component_count > 0 THEN 'current'
		WHEN future_components.component_count > 0 THEN 'future'
		ELSE 'unconfigured'
	END AS salary_status
FROM employments em
JOIN employees e ON e.id = em.employee_id
JOIN employers er ON er.id = em.employer_id
LEFT JOIN departments d ON d.id = em.department_id
LEFT JOIN LATERAL (
	SELECT count(*)::int AS component_count, max(configured.effective_date)::text AS latest_effective_date
	FROM (
		SELECT DISTINCT ON (esc.component_definition_id) esc.effective_date
		FROM employee_salary_components esc
		WHERE esc.employment_id = em.id AND esc.effective_date <= $1::date
		ORDER BY esc.component_definition_id, esc.effective_date DESC, esc.created_at DESC, esc.id DESC
	) configured
) current_components ON true
LEFT JOIN LATERAL (
	SELECT count(DISTINCT esc.component_definition_id)::int AS component_count
	FROM employee_salary_components esc
	WHERE esc.employment_id = em.id AND esc.effective_date > $1::date
) future_components ON true
` + where + `
ORDER BY e.employee_id_number ASC, em.id ASC
LIMIT $7 OFFSET $8`
	args = append(args, query.Limit, offset)
	rows, err := db.Query(ctx, rowsSQL, args...)
	if err != nil {
		return api.SalaryManagementResponse{}, err
	}
	defer rows.Close()

	result := api.SalaryManagementResponse{Data: make([]api.SalaryManagementRow, 0), Total: total, Page: query.Page, Limit: query.Limit, AsOf: asOfDate}
	for rows.Next() {
		var row api.SalaryManagementRow
		var currentCount int
		var latestEffectiveDate *string
		if err := rows.Scan(
			&row.EmploymentID, &row.EmployeeID, &row.EmployeeIDNumber, &row.EmployeeName,
			&row.EmployerID, &row.EmployerName, &row.DepartmentID, &row.DepartmentName,
			&row.JobTitle, &row.EmploymentStatus, &row.EmploymentType, &row.EmploymentStartDate,
			&row.EmploymentEndDate, &currentCount, &latestEffectiveDate, &row.SalaryStatus,
		); err != nil {
			return api.SalaryManagementResponse{}, err
		}
		row.ConfiguredComponentCount = currentCount
		row.LatestEffectiveDate = latestEffectiveDate
		result.Data = append(result.Data, row)
	}
	if err := rows.Err(); err != nil {
		return api.SalaryManagementResponse{}, err
	}
	return result, nil
}
