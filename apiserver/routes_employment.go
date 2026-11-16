package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerEmploymentRoutes(
	router chi.Router,
	contracts *api.ContractEmploymentHandler,
	assignments *api.AssignmentHandler,
	customFields *api.CustomFieldHandler,
) {
	registerRoute(router, "GET /api/v1/contracts", contracts.ListContracts)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/contracts", contracts.ListEmployeeContracts)
	registerRoute(router, "POST /api/v1/employees/{employee_id}/contracts", contracts.CreateContract)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/contracts/{contract_id}", contracts.GetContract)
	registerRoute(router, "PATCH /api/v1/employees/{employee_id}/contracts/{contract_id}", contracts.UpdateContract)
	registerRoute(router, "DELETE /api/v1/employees/{employee_id}/contracts/{contract_id}", contracts.DeleteContract)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/employment-relationships", contracts.ListEmploymentRelationships)
	registerRoute(router, "POST /api/v1/employees/{employee_id}/employment-relationships", contracts.CreateEmploymentRelationship)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/employment-relationships/{relationship_id}", contracts.GetEmploymentRelationship)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/employments", contracts.ListEmployments)
	registerRoute(router, "POST /api/v1/employees/{employee_id}/employments", contracts.CreateEmployment)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/employments/{employment_id}", contracts.GetEmployment)
	registerRoute(router, "PATCH /api/v1/employees/{employee_id}/employments/{employment_id}", contracts.UpdateEmployment)
	registerRoute(router, "DELETE /api/v1/employees/{employee_id}/employments/{employment_id}", contracts.DeleteEmployment)
	registerRoute(router, "GET /api/v1/assignment-history", assignments.AssignmentHistory)
	registerRoute(router, "GET /api/v1/assignment-rules", assignments.ListAssignmentRules)
	registerRoute(router, "POST /api/v1/assignment-rules", assignments.CreateAssignmentRule)
	registerRoute(router, "PATCH /api/v1/assignment-rules/{rule_id}", assignments.UpdateAssignmentRule)
	registerRoute(router, "DELETE /api/v1/assignment-rules/{rule_id}", assignments.DeleteAssignmentRule)
	registerRoute(router, "GET /api/v1/assignments", assignments.ListAssignments)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/assignments", assignments.ListEmployeeAssignments)
	registerRoute(router, "POST /api/v1/employees/{employee_id}/assignments", assignments.CreateEmployeeAssignment)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/assignments/{assignment_id}", assignments.GetEmployeeAssignment)
	registerRoute(router, "PATCH /api/v1/employees/{employee_id}/assignments/{assignment_id}", assignments.UpdateEmployeeAssignment)
	registerRoute(router, "DELETE /api/v1/employees/{employee_id}/assignments/{assignment_id}", assignments.DeleteEmployeeAssignment)
	registerRoute(router, "GET /api/v1/custom-fields/definitions", customFields.ListDefinitions)
	registerRoute(router, "POST /api/v1/custom-fields/definitions", customFields.CreateDefinition)
	registerRoute(router, "GET /api/v1/custom-fields/definitions/{id}", customFields.GetDefinition)
	registerRoute(router, "PATCH /api/v1/custom-fields/definitions/{id}", customFields.UpdateDefinition)
	registerRoute(router, "DELETE /api/v1/custom-fields/definitions/{id}", customFields.DeleteDefinition)
	registerRoute(router, "GET /api/v1/custom-fields/values/{entity_type}/{entity_id}", customFields.GetValues)
	registerRoute(router, "PUT /api/v1/custom-fields/values/{entity_type}/{entity_id}", customFields.SetValues)
}
