package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerLifecycleRoutes(
	router chi.Router,
	conditionTypes *api.ConditionTypeHandler,
	phases *api.EmploymentPhaseHandler,
	lifecycle *api.LifecycleHandler,
) {
	registerRoute(router, "GET /api/v1/condition-types", conditionTypes.List)
	registerRoute(router, "GET /api/v1/condition-types/{id}", conditionTypes.Get)
	registerRoute(router, "POST /api/v1/condition-types", conditionTypes.Create)
	registerRoute(router, "PATCH /api/v1/condition-types/{id}", conditionTypes.Update)
	registerRoute(router, "DELETE /api/v1/condition-types/{id}", conditionTypes.Delete)
	registerRoute(router, "POST /api/v1/employment-phases", phases.CreateDefinition)
	registerRoute(router, "GET /api/v1/employment-phases", phases.ListDefinitions)
	registerRoute(router, "GET /api/v1/employment-phase-assignments", phases.ListOrganization)
	registerRoute(router, "POST /api/v1/employees/{user_id}/phases", phases.AssignEmployee)
	registerRoute(router, "GET /api/v1/employees/{user_id}/phases", phases.ListEmployee)
	registerRoute(router, "PUT /api/v1/employees/{user_id}/phases/{phase_id}/status", phases.UpdateEmployeeStatus)
	registerRoute(router, "POST /api/v1/employments/{employment_id}/phases", phases.AssignEmployment)
	registerRoute(router, "GET /api/v1/employments/{employment_id}/phases", phases.ListEmployment)
	registerRoute(router, "PUT /api/v1/employments/{employment_id}/phases/{phase_id}/status", phases.UpdateEmploymentStatus)
	registerRoute(router, "GET /api/v1/lifecycle/my-tasks", lifecycle.GetMyTasks)
	registerRoute(router, "GET /api/v1/lifecycle/instances", lifecycle.ListChecklists)
	registerRoute(router, "POST /api/v1/lifecycle-templates", lifecycle.CreateTemplate)
	registerRoute(router, "GET /api/v1/lifecycle-templates", lifecycle.ListTemplates)
	registerRoute(router, "GET /api/v1/lifecycle-templates/{id}", lifecycle.GetTemplate)
	registerRoute(router, "PATCH /api/v1/lifecycle-templates/{id}", lifecycle.UpdateTemplate)
	registerRoute(router, "DELETE /api/v1/lifecycle-templates/{id}", lifecycle.DeleteTemplate)
	registerRoute(router, "POST /api/v1/lifecycle-templates/{id}/items", lifecycle.CreateTemplateItem)
	registerRoute(router, "GET /api/v1/lifecycle-templates/{id}/items", lifecycle.ListTemplateItems)
	registerRoute(router, "GET /api/v1/lifecycle-templates/{tid}/items/{iid}", lifecycle.GetTemplateItem)
	registerRoute(router, "PUT /api/v1/lifecycle-templates/{tid}/items/{iid}", lifecycle.UpdateTemplateItem)
	registerRoute(router, "DELETE /api/v1/lifecycle-templates/{tid}/items/{iid}", lifecycle.DeleteTemplateItem)
	registerRoute(router, "POST /api/v1/employments/{emp_id}/lifecycle", lifecycle.CreateChecklist)
	registerRoute(router, "GET /api/v1/employments/{emp_id}/lifecycle", lifecycle.ListEmploymentChecklists)
	registerRoute(router, "GET /api/v1/lifecycle/{id}", lifecycle.GetChecklist)
	registerRoute(router, "PATCH /api/v1/lifecycle/{id}/items/{iid}", lifecycle.UpdateItemStatus)
}
