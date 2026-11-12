package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerPayrollRoutes(router chi.Router, handler api.PayrollHandler) {
	registerRoute(router, "GET /api/v1/payroll/payslips", handler.ListPayslips)
	registerRoute(router, "GET /api/v1/employees/{employee_id}/payslips", handler.ListEmployee)
	registerRoute(router, "GET /api/v1/salary-components", handler.ListSalaryComponents)
	registerRoute(router, "POST /api/v1/salary-components", handler.CreateSalaryComponent)
	registerRoute(router, "GET /api/v1/employments/{id}/salary-components", handler.ListEmploymentSalaryComponents)
	registerRoute(router, "POST /api/v1/employments/{id}/salary-components", handler.AssignEmploymentSalaryComponent)
	registerRoute(router, "GET /api/v1/salary-bases", handler.ListSalaryBases)
	registerRoute(router, "POST /api/v1/salary-bases", handler.CreateSalaryBase)
	registerRoute(router, "POST /api/v1/salary-bases/{base_id}/members", handler.AssignBaseMember)
	registerRoute(router, "DELETE /api/v1/salary-bases/{base_id}/members/{component_id}", handler.UnassignBaseMember)
	registerRoute(router, "GET /api/v1/payroll-settings", handler.GetPayrollSettings)
	registerRoute(router, "PATCH /api/v1/payroll-settings", handler.UpdatePayrollSettings)
	registerRoute(router, "GET /api/v1/payroll-settings/history", handler.GetPayrollSettingsHistory)
	registerRoute(router, "GET /api/v1/employees/{id}/payroll-config", handler.GetEmployeePayrollConfig)
	registerRoute(router, "POST /api/v1/employees/{id}/payroll-config", handler.SetEmployeePayrollConfig)
	registerRoute(router, "GET /api/v1/payroll/incentive-policies", handler.GetIncentivePolicies)
	registerRoute(router, "POST /api/v1/payroll/incentive-policies", handler.CreateIncentivePolicy)
	registerRoute(router, "DELETE /api/v1/payroll/incentive-policies/{policy_id}", handler.DeleteIncentivePolicy)
	registerRoute(router, "POST /api/v1/payroll/incentive-policies/{policy_id}/clone", handler.CloneIncentivePolicy)
	registerRoute(router, "GET /api/v1/payroll-rules", handler.ListPayrollRules)
	registerRoute(router, "POST /api/v1/payroll-rules", handler.CreatePayrollRule)
	registerRoute(router, "GET /api/v1/payroll-rules/{rule_id}/tiers", handler.ListPayrollRuleTiers)
	registerRoute(router, "POST /api/v1/payroll-rules/{rule_id}/tiers", handler.CreatePayrollRuleTier)
	registerRoute(router, "DELETE /api/v1/payroll-rules/{rule_id}/tiers/{tier_id}", handler.DeletePayrollRuleTier)
	registerRoute(router, "GET /api/v1/payroll-rules/{rule_id}/assignments", handler.ListPayrollRuleAssignments)
	registerRoute(router, "POST /api/v1/payroll-rules/{rule_id}/assignments", handler.AssignPayrollRule)
	registerRoute(router, "DELETE /api/v1/payroll-rules/{rule_id}/assignments/{assignment_id}", handler.DeletePayrollRuleAssignment)
	registerRoute(router, "POST /api/v1/employees/{id}/calculate-payroll", handler.CalculatePayroll)
	registerRoute(router, "POST /api/v1/payroll/calculate-batch", handler.CalculateBatchPayroll)
}
