package repository

import (
	"cmp"
	"context"
	"errors"
	"slices"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type CompensationHistoryRepository struct {
	DB DBTX
}

func (r CompensationHistoryRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("compensation repository: tenant database is not configured")
	}
	return r.DB, nil
}

type compEmploymentRow struct {
	id           uuid.UUID
	employerID   uuid.UUID
	departmentID *uuid.UUID
	jobTitle     *string
	startDate    string
}

type compSalaryRow struct {
	component  api.EmployeeSalaryComponent
	definition api.SalaryComponentDefinition
}

type timelineKey struct {
	employmentID  uuid.UUID
	effectiveDate string
}

func (r CompensationHistoryRepository) GetEmployeeCompensationHistory(ctx context.Context, workspaceID, employeeID uuid.UUID, filterEmploymentID *uuid.UUID) (api.CompensationHistoryResponse, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.CompensationHistoryResponse{}, err
	}

	rows, err := db.Query(ctx, `SELECT em.id, em.employer_id, em.department_id, em.job_title, em.start_date::text FROM employments em JOIN employees e ON e.id = em.employee_id WHERE em.employee_id = $1 AND e.workspace_id = $2 AND em.deleted_at IS NULL ORDER BY em.start_date ASC`, employeeID, workspaceID)
	if err != nil {
		return api.CompensationHistoryResponse{}, err
	}
	defer rows.Close()

	employments := make([]compEmploymentRow, 0)
	empMap := make(map[uuid.UUID]compEmploymentRow)
	for rows.Next() {
		var row compEmploymentRow
		if err := rows.Scan(&row.id, &row.employerID, &row.departmentID, &row.jobTitle, &row.startDate); err != nil {
			return api.CompensationHistoryResponse{}, err
		}
		if filterEmploymentID == nil || *filterEmploymentID == row.id {
			employments = append(employments, row)
			empMap[row.id] = row
		}
	}
	if err := rows.Err(); err != nil {
		return api.CompensationHistoryResponse{}, err
	}
	if len(employments) == 0 {
		return api.CompensationHistoryResponse{Timeline: []api.CompensationTimelineEvent{}}, nil
	}

	empIDs := make([]uuid.UUID, 0, len(employments))
	employerIDMap := make(map[uuid.UUID]struct{})
	deptIDMap := make(map[uuid.UUID]struct{})
	for _, e := range employments {
		empIDs = append(empIDs, e.id)
		employerIDMap[e.employerID] = struct{}{}
		if e.departmentID != nil {
			deptIDMap[*e.departmentID] = struct{}{}
		}
	}

	employerIDs := make([]uuid.UUID, 0, len(employerIDMap))
	for id := range employerIDMap {
		employerIDs = append(employerIDs, id)
	}
	deptIDs := make([]uuid.UUID, 0, len(deptIDMap))
	for id := range deptIDMap {
		deptIDs = append(deptIDs, id)
	}

	employerNames := make(map[uuid.UUID]string)
	if len(employerIDs) > 0 {
		empRows, err := db.Query(ctx, `SELECT id, name FROM employers WHERE id = ANY($1) AND workspace_id = $2`, employerIDs, workspaceID)
		if err != nil {
			return api.CompensationHistoryResponse{}, err
		}
		defer empRows.Close()
		for empRows.Next() {
			var id uuid.UUID
			var name string
			if err := empRows.Scan(&id, &name); err == nil {
				employerNames[id] = name
			}
		}
	}

	deptNames := make(map[uuid.UUID]string)
	if len(deptIDs) > 0 {
		deptRows, err := db.Query(ctx, `SELECT id, name FROM departments WHERE id = ANY($1) AND workspace_id = $2`, deptIDs, workspaceID)
		if err != nil {
			return api.CompensationHistoryResponse{}, err
		}
		defer deptRows.Close()
		for deptRows.Next() {
			var id uuid.UUID
			var name string
			if err := deptRows.Scan(&id, &name); err == nil {
				deptNames[id] = name
			}
		}
	}

	compRows, err := db.Query(ctx, `SELECT esc.id, esc.employment_id, esc.component_definition_id, esc.amount::text, esc.effective_date::text, esc.created_at, esc.updated_at, scd.id, scd.workspace_id, scd.name, scd.code, scd.created_at, scd.updated_at FROM employee_salary_components esc JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id WHERE esc.employment_id = ANY($1) AND scd.workspace_id = $2 ORDER BY esc.employment_id ASC, esc.effective_date ASC, esc.created_at ASC`, empIDs, workspaceID)
	if err != nil {
		return api.CompensationHistoryResponse{}, err
	}
	defer compRows.Close()

	componentsHistory := make([]compSalaryRow, 0)
	keyMap := make(map[timelineKey]struct{})
	keys := make([]timelineKey, 0)
	for compRows.Next() {
		var esc api.EmployeeSalaryComponent
		var scd api.SalaryComponentDefinition
		var escAmt string
		var escCreated, escUpdated, scdCreated, scdUpdated time.Time
		if err := compRows.Scan(&esc.ID, &esc.EmploymentID, &esc.ComponentDefinitionID, &escAmt, &esc.EffectiveDate, &escCreated, &escUpdated, &scd.ID, &scd.WorkspaceID, &scd.Name, &scd.Code, &scdCreated, &scdUpdated); err != nil {
			return api.CompensationHistoryResponse{}, err
		}
		esc.Amount, _ = decimal.NewFromString(escAmt)
		esc.CreatedAt = escCreated.UTC().Format(time.RFC3339)
		esc.UpdatedAt = escUpdated.UTC().Format(time.RFC3339)
		scd.CreatedAt = scdCreated.UTC().Format(time.RFC3339)
		scd.UpdatedAt = scdUpdated.UTC().Format(time.RFC3339)
		componentsHistory = append(componentsHistory, compSalaryRow{component: esc, definition: scd})
		k := timelineKey{employmentID: esc.EmploymentID, effectiveDate: esc.EffectiveDate}
		if _, exists := keyMap[k]; !exists {
			keyMap[k] = struct{}{}
			keys = append(keys, k)
		}
	}
	if err := compRows.Err(); err != nil {
		return api.CompensationHistoryResponse{}, err
	}

	slices.SortFunc(keys, func(a, b timelineKey) int {
		if c := cmp.Compare(a.effectiveDate, b.effectiveDate); c != 0 {
			return c
		}
		return cmp.Compare(a.employmentID.String(), b.employmentID.String())
	})

	activeComponents := make(map[uuid.UUID]map[uuid.UUID]api.SalaryComponentWithDefinition)
	timelineEvents := make([]api.CompensationTimelineEvent, 0)
	var previousTotal *decimal.Decimal
	var previousEmploymentID *uuid.UUID

	for _, k := range keys {
		emp, ok := empMap[k.employmentID]
		if !ok {
			continue
		}
		empComps, ok := activeComponents[k.employmentID]
		if !ok {
			empComps = make(map[uuid.UUID]api.SalaryComponentWithDefinition)
			activeComponents[k.employmentID] = empComps
		}
		for _, row := range componentsHistory {
			if row.component.EmploymentID == k.employmentID && row.component.EffectiveDate == k.effectiveDate {
				empComps[row.component.ComponentDefinitionID] = api.SalaryComponentWithDefinition{
					Component:  row.component,
					Definition: row.definition,
				}
			}
		}

		eventComponents := make([]api.SalaryComponentWithDefinition, 0, len(empComps))
		for _, item := range empComps {
			eventComponents = append(eventComponents, item)
		}
		slices.SortFunc(eventComponents, func(a, b api.SalaryComponentWithDefinition) int {
			return cmp.Compare(a.Definition.Code, b.Definition.Code)
		})

		totalCompensation := decimal.Zero
		for _, c := range eventComponents {
			totalCompensation = totalCompensation.Add(c.Component.Amount)
		}

		var changeType string
		if previousTotal == nil {
			changeType = "initial"
		} else if previousEmploymentID == nil || k.employmentID != *previousEmploymentID {
			changeType = "transfer"
		} else {
			changeType = "adjustment"
		}

		var delta decimal.Decimal
		if previousTotal != nil {
			delta = totalCompensation.Sub(*previousTotal)
		} else {
			delta = totalCompensation
		}

		employerName := "Unknown Employer"
		if name, ok := employerNames[emp.employerID]; ok {
			employerName = name
		}
		var deptName *string
		if emp.departmentID != nil {
			if name, ok := deptNames[*emp.departmentID]; ok {
				deptName = &name
			}
		}

		currEmpID := k.employmentID
		event := api.CompensationTimelineEvent{
			EffectiveDate: k.effectiveDate,
			Employment: api.CompensationEmploymentContext{
				ID:             currEmpID,
				EmployerName:   employerName,
				DepartmentName: deptName,
				JobTitle:       emp.jobTitle,
			},
			Components:        eventComponents,
			TotalCompensation: totalCompensation,
			ChangeType:        changeType,
			ChangeDetails: api.CompensationChangeDetails{
				PreviousTotal: previousTotal,
				Delta:         delta,
			},
		}
		timelineEvents = append(timelineEvents, event)
		currTotal := totalCompensation
		previousTotal = &currTotal
		previousEmploymentID = &currEmpID
	}

	// Rust returns newest first
	slices.Reverse(timelineEvents)

	return api.CompensationHistoryResponse{Timeline: timelineEvents}, nil
}
