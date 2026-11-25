package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerPeopleRoutes(router chi.Router, departments *api.DepartmentHandler, locations *api.LocationHandler, employers *api.EmployerHandler, recruitment *api.RecruitmentSourceHandler) {
	registerRoute(router, "GET /api/v1/departments", departments.List)
	registerRoute(router, "POST /api/v1/departments", departments.Create)
	registerRoute(router, "PATCH /api/v1/departments/{id}", departments.Update)
	registerRoute(router, "DELETE /api/v1/departments/{id}", departments.Delete)
	registerRoute(router, "GET /api/v1/locations", locations.List)
	registerRoute(router, "POST /api/v1/locations", locations.Create)
	registerRoute(router, "GET /api/v1/locations/{id}", locations.Get)
	registerRoute(router, "PUT /api/v1/locations/{id}", locations.Update)
	registerRoute(router, "DELETE /api/v1/locations/{id}", locations.Delete)
	registerRoute(router, "GET /api/v1/employers", employers.List)
	registerRoute(router, "POST /api/v1/employers", employers.Create)
	registerRoute(router, "GET /api/v1/employers/{id}", employers.Get)
	registerRoute(router, "PUT /api/v1/employers/{id}", employers.Update)
	registerRoute(router, "DELETE /api/v1/employers/{id}", employers.Delete)
	registerRoute(router, "GET /api/v1/recruitment-sources", recruitment.List)
	registerRoute(router, "POST /api/v1/recruitment-sources", recruitment.Create)
	registerRoute(router, "PUT /api/v1/recruitment-sources/{id}", recruitment.Update)
	registerRoute(router, "DELETE /api/v1/recruitment-sources/{id}", recruitment.Delete)
	registerRoute(router, "GET /api/v1/dashboard/recruitment-sources", recruitment.Dashboard)
}
