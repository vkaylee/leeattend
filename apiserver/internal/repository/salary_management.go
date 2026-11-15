package repository

import (
	"context"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
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
	var employerID, departmentID *uuid.UUID
	if query.EmployerID != nil {
		value := uuid.UUID(*query.EmployerID)
		employerID = &value
	}
	if query.DepartmentID != nil {
		value := uuid.UUID(*query.DepartmentID)
		departmentID = &value
	}
	args := []any{asOfDate, query.Search, employerID, departmentID, query.EmploymentStatus}
	where := `WHERE e.deleted_at IS NULL
	  AND em.deleted_at IS NULL
	  AND er.deleted_at IS NULL
	  AND em.start_date <= $1::date
	  AND (em.end_date IS NULL OR em.end_date >= $1::date)
	  AND ($2 = '' OR e.full_name ILIKE '%' || $2 || '%' OR COALESCE(tenure.employee_number, e.employee_id_number) ILIKE '%' || $2 || '%')
	  AND ($3::uuid IS NULL OR em.employer_id = $3)
	  AND ($4::uuid IS NULL OR em.department_id = $4)
	  AND ($5 = '' OR em.status = $5)`
	componentJoin := `JOIN LATERAL (
		SELECT DISTINCT ON (esc.component_definition_id)
			esc.id, esc.component_definition_id, esc.amount::text AS amount, esc.effective_date::text AS effective_date
		FROM employee_salary_components esc
		WHERE esc.employment_id = em.id AND esc.effective_date <= $1::date
		ORDER BY esc.component_definition_id, esc.effective_date DESC, esc.created_at DESC, esc.id DESC
	) configured_component ON true
	JOIN salary_component_definitions scd ON scd.id = configured_component.component_definition_id`

	var total int64
	countSQL := `SELECT count(*)
FROM employments em
JOIN employees e ON e.id = em.employee_id
JOIN employers er ON er.id = em.employer_id
LEFT JOIN LATERAL (SELECT t.employee_number FROM employee_tenures t WHERE t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= $1::date AND (t.end_date IS NULL OR t.end_date >= $1::date) ORDER BY t.start_date DESC, t.id DESC LIMIT 1) tenure ON TRUE
` + componentJoin + "\n" + where
	if err := db.QueryRow(ctx, countSQL, args...).Scan(&total); err != nil {
		return api.SalaryManagementResponse{}, err
	}

	rowsSQL := `SELECT em.id, e.id, COALESCE(tenure.employee_number, e.employee_id_number), e.full_name,
	er.id, er.name, em.department_id, d.name, em.job_title, em.status,
	em.employment_type, em.start_date::text, em.end_date::text,
	configured_component.id, configured_component.component_definition_id,
	scd.name, scd.code, configured_component.amount, configured_component.effective_date
FROM employments em
JOIN employees e ON e.id = em.employee_id
JOIN employers er ON er.id = em.employer_id
LEFT JOIN departments d ON d.id = em.department_id
LEFT JOIN LATERAL (SELECT t.employee_number FROM employee_tenures t WHERE t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= $1::date AND (t.end_date IS NULL OR t.end_date >= $1::date) ORDER BY t.start_date DESC, t.id DESC LIMIT 1) tenure ON TRUE
` + componentJoin + "\n" + where + `
ORDER BY COALESCE(tenure.employee_number, e.employee_id_number) ASC, em.id ASC, configured_component.effective_date DESC, scd.code ASC, configured_component.id ASC
LIMIT $6 OFFSET $7`
	args = append(args, query.Limit, offset)
	rows, err := db.Query(ctx, rowsSQL, args...)
	if err != nil {
		return api.SalaryManagementResponse{}, err
	}
	defer rows.Close()

	result := api.SalaryManagementResponse{Data: make([]api.SalaryManagementRow, 0), Total: total, Page: query.Page, Limit: query.Limit, AsOf: asOfDate}
	for rows.Next() {
		var row api.SalaryManagementRow
		var amount string
		if err := rows.Scan(
			&row.EmploymentID, &row.EmployeeID, &row.EmployeeIDNumber, &row.EmployeeName,
			&row.EmployerID, &row.EmployerName, &row.DepartmentID, &row.DepartmentName,
			&row.JobTitle, &row.EmploymentStatus, &row.EmploymentType, &row.EmploymentStartDate,
			&row.EmploymentEndDate, &row.ComponentID, &row.ComponentDefinitionID,
			&row.ComponentName, &row.ComponentCode, &amount, &row.EffectiveDate,
		); err != nil {
			return api.SalaryManagementResponse{}, err
		}
		row.Amount, err = decimal.NewFromString(amount)
		if err != nil {
			return api.SalaryManagementResponse{}, err
		}
		row.SalaryStatus = "current"
		result.Data = append(result.Data, row)
	}
	if err := rows.Err(); err != nil {
		return api.SalaryManagementResponse{}, err
	}
	return result, nil
}
