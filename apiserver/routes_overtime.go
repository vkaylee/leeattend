package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerOvertimeRoutes(router chi.Router, handler *api.OvertimeHandler) {
	registerRoute(router, "GET /api/v1/overtime/policies", handler.ListPolicies)
	registerRoute(router, "POST /api/v1/overtime/policies", handler.CreatePolicy)
	registerRoute(router, "GET /api/v1/overtime/policies/{policy_id}", handler.GetPolicy)
	registerRoute(router, "POST /api/v1/overtime/policies/{policy_id}/rules", handler.AddRule)
	registerRoute(router, "POST /api/v1/overtime/policies/assign", handler.AssignPolicy)
	registerRoute(router, "GET /api/v1/overtime/requests", handler.ListRequests)
	registerRoute(router, "POST /api/v1/overtime/requests", handler.CreateRequest)
	registerRoute(router, "GET /api/v1/overtime/requests/{id}", handler.GetRequest)
	registerRoute(router, "POST /api/v1/overtime/requests/{id}/submit", handler.SubmitRequest)
	registerRoute(router, "POST /api/v1/overtime/requests/{id}/consent/{emp_id}", handler.ConsentRequest)
	registerRoute(router, "GET /api/v1/overtime/requests/behalf-assignments", handler.ListRequestBehalf)
	registerRoute(router, "POST /api/v1/overtime/requests/behalf-assignments", handler.CreateRequestBehalf)
	registerRoute(router, "GET /api/v1/overtime/requests/behalf-assignments/my-targets", handler.RequestBehalfTargets)
	registerRoute(router, "DELETE /api/v1/overtime/requests/behalf-assignments/{id}", handler.DeleteRequestBehalf)
	registerRoute(router, "GET /api/v1/attendance/claim-ot/behalf-assignments", handler.ListClaimBehalf)
	registerRoute(router, "POST /api/v1/attendance/claim-ot/behalf-assignments", handler.CreateClaimBehalf)
	registerRoute(router, "DELETE /api/v1/attendance/claim-ot/behalf-assignments/{id}", handler.DeleteClaimBehalf)
	registerRoute(router, "POST /api/v1/attendance/claim-ot/{employee_id}", handler.ClaimOT)
	registerRoute(router, "POST /api/v1/attendance/approve-ot/{employee_id}", handler.ApproveOT)
}
