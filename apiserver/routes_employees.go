package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerEmployeeRoutes(
	router chi.Router,
	employee *api.EmployeeHandler,
	employeeImport *api.EmployeeImportHandler,
	conditions *api.EmployeeConditionHandler,
	departments *api.EmployeeDepartmentHandler,
	compensation *api.CompensationHistoryHandler,
) {
	registerRoute(router, "POST /api/v1/employees", employee.Create)
	registerRoute(router, "GET /api/v1/employees", employee.List)
	registerRoute(router, "GET /api/v1/employees/{id}", employee.Get)
	registerRoute(router, "PUT /api/v1/employees/{id}", employee.Update)
	registerRoute(router, "DELETE /api/v1/employees/{id}", employee.Delete)
	registerRoute(router, "POST /api/v1/employees/{id}/rehire", employee.Rehire)
	registerRoute(router, "POST /api/v1/employees/upload", employeeImport.Upload)
	registerRoute(router, "POST /api/v1/employees/import-termination/preview", employeeImport.PreviewTermination)
	registerRoute(router, "POST /api/v1/employees/import-termination/confirm", employeeImport.ConfirmTermination)
	registerRoute(router, "GET /api/v1/employees/import-termination/template", employeeImport.TerminationTemplate)
	registerRoute(router, "GET /api/v1/working-conditions", conditions.ListConditions)
	registerRoute(router, "GET /api/v1/employees/{id}/conditions", conditions.List)
	registerRoute(router, "POST /api/v1/employees/{id}/conditions", conditions.Create)
	registerRoute(router, "DELETE /api/v1/employees/{id}/conditions/{cond_id}", conditions.Delete)
	registerRoute(router, "GET /api/v1/employees/{id}/departments", departments.List)
	registerRoute(router, "POST /api/v1/employees/{id}/departments", departments.Transfer)
	registerRoute(router, "GET /api/v1/employees/{id}/compensation-history", compensation.Get)
}
