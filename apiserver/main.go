package main

import (
	"context"
	"crypto/ed25519"
	"encoding/base64"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"os"
	"strconv"

	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/app"
	"github.com/vkaylee/leeattend/apiserver/internal/config"
	dbpool "github.com/vkaylee/leeattend/apiserver/internal/db/pool"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/crypto"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/external"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/health"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
	"github.com/vkaylee/leeattend/apiserver/internal/license"
	"github.com/vkaylee/leeattend/apiserver/internal/observability"
	"github.com/vkaylee/leeattend/apiserver/internal/repository"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	standalonebootstrap "github.com/vkaylee/leeattend/apiserver/internal/standalone"
)

// Version is the current apiserver Go component version.
const Version = "0.1.0"


func main() {
	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "CONFIGURATION FAILED: %v\n", err)
		os.Exit(1)
	}
	logger := observability.NewLogger(os.Stderr, slog.LevelInfo)
	logger.Info("Starting LeeAttend Go API Server", "version", Version, "env", cfg.AppEnv)

	if cfg.IsDevelopment() {
		logger.Info("Development license verification skipped")
	} else {
		key, keyErr := base64.RawURLEncoding.DecodeString(cfg.LicensePublicKey)
		if keyErr != nil {
			fmt.Fprintf(os.Stderr, "CONFIGURATION FAILED: invalid LICENSE_PUBLIC_KEY\n")
			os.Exit(1)
		}
		verifier, verifierErr := license.NewVerifier(ed25519.PublicKey(key), license.Product)
		if verifierErr != nil {
			fmt.Fprintf(os.Stderr, "CONFIGURATION FAILED: invalid standalone license verifier\n")
			os.Exit(1)
		}
		status, verifyErr := verifier.VerifyFile(cfg.LicensePath)
		if verifyErr != nil || !status.Valid {
			fmt.Fprintf(os.Stderr, "CONFIGURATION FAILED: standalone license verification failed\n")
			os.Exit(1)
		}
		logger.Info("Standalone license verified", "edition", status.Edition, "entitlements", len(status.Entitlements))
	}

	dbCfg := dbpool.DefaultConfig()
	dbCfg.PrimaryURL = string(cfg.DBPrimaryURL)
	dbCfg.PoolerTxURL = string(cfg.DBPoolerTxURL)
	dbCfg.PoolerSessURL = string(cfg.DBPoolerSessURL)
	dbCfg.ReaderURL = string(cfg.DBReaderURL)
	if cfg.DBAPIMaxConn > 0 {
		dbCfg.APIMaxConn = int32(cfg.DBAPIMaxConn)
	}

	poolMgr := dbpool.NewManager(dbCfg, nil)
	defer poolMgr.Close()
	var authRepo repository.AuthRepository

	if err := poolMgr.InitSystemPools(context.Background()); err != nil {
		logger.Warn("database connection deferred or offline", "error", err)
	} else if sysPool, err := poolMgr.SystemPool(); err == nil {
		authRepo = repository.AuthRepository{DB: sysPool}
		if err := standalonebootstrap.Bootstrap(context.Background(), sysPool, cfg.OrganizationName, cfg.StandaloneAdminEmail, cfg.StandaloneAdminPassword.Expose()); err != nil {
			fmt.Fprintf(os.Stderr, "CONFIGURATION FAILED: standalone bootstrap failed: %v\n", err)
			os.Exit(1)
		}

	}

	var redisClient *redisinfra.Client
	if rClient, err := redisinfra.NewClient(redisinfra.Config{URL: string(cfg.RedisURL)}); err == nil {
		redisClient = rClient
		defer func() { _ = redisClient.Close() }()
	} else {
		logger.Warn("redis connection deferred or offline", "error", err)
	}

	var loginLimiter service.LoginLimiter
	if redisClient != nil {
		loginLimiter = redisClient
	}
	authService := service.AuthService{
		Repository: authRepo,
		Limiter:    loginLimiter,
		TokenStore: redisClient,
		JWTSecret:  cfg.JWTSecret.Expose(),
	}
	authHandler := api.AuthHandler{
		Service:      authService,
		CookieDomain: cfg.CookieDomain,
	}

	mux := http.NewServeMux()

	db := func() repository.DBTX {
		pool, err := poolMgr.SystemPool()
		if err != nil {
			return nil
		}
		return pool
	}()

	organizationAuthorizer := repository.OrganizationAuthorizer{DB: db}
	organizationRepo := repository.OrganizationRepository{DB: db}
	organizationHandler := api.OrganizationHandler{Repository: organizationRepo, Authorizer: organizationAuthorizer}
	var cryptoService *crypto.Service
	if encJSON := cfg.EncryptionKeysJSON.Expose(); encJSON != "" {
		if cs, err := crypto.FromJSON(encJSON); err == nil {
			cryptoService = cs
		} else {
			logger.Warn("crypto service initialization failed", "error", err)
		}
	}
	rbacRepo := repository.OrganizationRBACRepository{SystemDB: db}
	rbacHandler := api.OrganizationRBACHandler{
		Repository:    rbacRepo,
		Authorizer:    organizationAuthorizer,
		Crypto:        cryptoService,
		StorageTester: api.NewS3StorageTester(cfg.S3EndpointURL),
	}
	authHandler.Authorizer = organizationAuthorizer

	mux.HandleFunc("PATCH /api/v1", organizationHandler.Patch)
	mux.HandleFunc("GET /api/v1/feature-flags", organizationHandler.ListFeatureFlags)
	mux.HandleFunc("GET /api/v1/members", rbacHandler.ListMembers)
	mux.HandleFunc("POST /api/v1/members", rbacHandler.AddMember)
	mux.HandleFunc("PATCH /api/v1/members/{user_id}", rbacHandler.UpdateMemberRole)
	mux.HandleFunc("DELETE /api/v1/members/{user_id}", rbacHandler.RemoveMember)
	mux.HandleFunc("GET /api/v1/members/{user_id}/roles", rbacHandler.GetMemberRoles)
	mux.HandleFunc("POST /api/v1/members/{user_id}/roles", rbacHandler.AssignExtraRole)
	mux.HandleFunc("DELETE /api/v1/members/{user_id}/roles/{role_id}", rbacHandler.RevokeExtraRole)
	mux.HandleFunc("GET /api/v1/members/{user_id}/permissions", rbacHandler.GetMemberPermissions)
	mux.HandleFunc("POST /api/v1/members/{user_id}/permissions", rbacHandler.AssignExtraPermission)
	mux.HandleFunc("DELETE /api/v1/members/{user_id}/permissions/{perm_id}", rbacHandler.RevokeExtraPermission)
	mux.HandleFunc("GET /api/v1/roles", rbacHandler.ListRoles)
	mux.HandleFunc("POST /api/v1/roles", rbacHandler.CreateRole)
	mux.HandleFunc("PUT /api/v1/roles/{role_id}", rbacHandler.UpdateRole)
	mux.HandleFunc("DELETE /api/v1/roles/{role_id}", rbacHandler.DeleteRole)
	mux.HandleFunc("GET /api/v1/roles/{role_id}/permissions", rbacHandler.GetRolePermissions)
	mux.HandleFunc("PUT /api/v1/roles/{role_id}/permissions", rbacHandler.SetRolePermissions)
	mux.HandleFunc("GET /api/v1/permissions/catalog", rbacHandler.PermissionCatalog)
	mux.HandleFunc("GET /api/v1/permissions/available", rbacHandler.AvailablePermissions)
	mux.HandleFunc("GET /api/v1/settings/storage", rbacHandler.GetStorage)
	mux.HandleFunc("PUT /api/v1/settings/storage", rbacHandler.PutStorage)
	mux.HandleFunc("DELETE /api/v1/settings/storage", rbacHandler.DeleteStorage)
	mux.HandleFunc("POST /api/v1/settings/storage/test", rbacHandler.TestStorage)
	mux.HandleFunc("POST /api/v1/feature-flags", organizationHandler.CreateFeatureFlag)
	mux.HandleFunc("PUT /api/v1/feature-flags", organizationHandler.UpsertFeatureFlag)
	mux.HandleFunc("GET /api/v1/feature-flags/{id}", organizationHandler.GetFeatureFlagByID)
	mux.HandleFunc("PATCH /api/v1/feature-flags/{id}", organizationHandler.UpdateFeatureFlag)
	mux.HandleFunc("DELETE /api/v1/feature-flags/{id}", organizationHandler.DeleteFeatureFlag)
	mux.HandleFunc("GET /api/v1/feature-flags/check/{key}", organizationHandler.CheckFeatureFlag)
	mux.HandleFunc("PUT /api/v1/configs", organizationHandler.PutConfig)
	mux.HandleFunc("GET /api/v1/configs", organizationHandler.ListConfigs)
	mux.HandleFunc("GET /api/v1/configs/{key}", organizationHandler.GetConfig)
	mux.HandleFunc("DELETE /api/v1/configs/{key}", organizationHandler.DeleteConfig)
	mux.HandleFunc("GET /api/v1", organizationHandler.Get)
	healthHandler := health.NewHandler(Version, nil, nil)
	healthHandler.SetLive(true)
	departmentRepo := repository.DepartmentRepository{DB: db}
	departmentHandler := api.DepartmentHandler{Repository: departmentRepo, Authorizer: organizationAuthorizer}

	mux.HandleFunc("GET /api/v1/departments", departmentHandler.List)
	mux.HandleFunc("POST /api/v1/departments", departmentHandler.Create)
	mux.HandleFunc("PATCH /api/v1/departments/{id}", departmentHandler.Update)
	mux.HandleFunc("DELETE /api/v1/departments/{id}", departmentHandler.Delete)
	mux.HandleFunc("GET /api/v1/departments/{id}/payroll-config", departmentHandler.GetPayrollConfig)
	mux.HandleFunc("POST /api/v1/departments/{id}/payroll-config", departmentHandler.SetPayrollConfig)
	locationRepo := repository.LocationRepository{DB: db}
	locationHandler := api.LocationHandler{Repository: locationRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/locations", locationHandler.List)
	mux.HandleFunc("POST /api/v1/locations", locationHandler.Create)
	mux.HandleFunc("GET /api/v1/locations/{id}", locationHandler.Get)
	mux.HandleFunc("PUT /api/v1/locations/{id}", locationHandler.Update)
	mux.HandleFunc("DELETE /api/v1/locations/{id}", locationHandler.Delete)
	employerRepo := repository.EmployerRepository{DB: db}
	employerHandler := api.EmployerHandler{Repository: employerRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/employers", employerHandler.List)
	mux.HandleFunc("POST /api/v1/employers", employerHandler.Create)
	mux.HandleFunc("GET /api/v1/employers/{id}", employerHandler.Get)
	mux.HandleFunc("PUT /api/v1/employers/{id}", employerHandler.Update)
	mux.HandleFunc("DELETE /api/v1/employers/{id}", employerHandler.Delete)
	recruitmentRepo := repository.RecruitmentSourceRepository{DB: db}
	recruitmentHandler := api.RecruitmentSourceHandler{Repository: recruitmentRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/recruitment-sources", recruitmentHandler.List)
	mux.HandleFunc("POST /api/v1/recruitment-sources", recruitmentHandler.Create)
	mux.HandleFunc("PUT /api/v1/recruitment-sources/{id}", recruitmentHandler.Update)
	mux.HandleFunc("DELETE /api/v1/recruitment-sources/{id}", recruitmentHandler.Delete)
	mux.HandleFunc("GET /api/v1/dashboard/recruitment-sources", recruitmentHandler.Dashboard)
	employeeRepo := repository.EmployeeRepository{DB: db}
	employeeHandler := api.EmployeeHandler{Repository: employeeRepo, Authorizer: organizationAuthorizer}
	employeeImportRepo := repository.EmployeeImportRepository{DB: db}
	employeeImportHandler := api.EmployeeImportHandler{
		Repository: employeeImportRepo,
		Queue: repository.EmployeeImportQueue{
			RedisClient: redisClient,
			Repo:        employeeImportRepo,
		},
		Authorizer: organizationAuthorizer,
		StorageDir: cfg.StorageDir,
	}
	employeeConditionRepo := repository.EmployeeConditionRepository{DB: db}
	employeeConditionHandler := api.EmployeeConditionHandler{Repository: employeeConditionRepo, Authorizer: organizationAuthorizer}
	employeeDepartmentRepo := repository.EmployeeDepartmentRepository{DB: db}
	employeeDepartmentHandler := api.EmployeeDepartmentHandler{Repository: employeeDepartmentRepo, Authorizer: organizationAuthorizer}
	compensationHistoryRepo := repository.CompensationHistoryRepository{DB: db}
	compensationHistoryHandler := api.CompensationHistoryHandler{Repository: compensationHistoryRepo, Authorizer: organizationAuthorizer}
	attendanceRepo := repository.AttendanceRepository{DB: db}
	attendanceHandler := api.AttendanceHandler{Repository: attendanceRepo, Authorizer: organizationAuthorizer}
	attendanceLogsHandler := api.AttendanceLogsHandler{Repository: attendanceRepo, Authorizer: organizationAuthorizer, StorageDir: cfg.StorageDir}
	attendancePolicyRepo := repository.AttendancePolicyRepository{DB: db, SystemDB: db}
	attendancePolicyHandler := api.AttendancePolicyHandler{PolicyRepository: attendancePolicyRepo, BehalfRepository: attendancePolicyRepo, Authorizer: organizationAuthorizer}
	overtimeRepo := repository.OvertimeRepository{DB: db}
	overtimeHandler := api.OvertimeHandler{Repository: overtimeRepo, BehalfRepository: attendancePolicyRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/overtime/policies", overtimeHandler.ListPolicies)
	mux.HandleFunc("POST /api/v1/overtime/policies", overtimeHandler.CreatePolicy)
	mux.HandleFunc("GET /api/v1/overtime/policies/{policy_id}", overtimeHandler.GetPolicy)
	mux.HandleFunc("POST /api/v1/overtime/policies/{policy_id}/rules", overtimeHandler.AddRule)
	mux.HandleFunc("POST /api/v1/overtime/policies/assign", overtimeHandler.AssignPolicy)
	mux.HandleFunc("GET /api/v1/overtime/requests", overtimeHandler.ListRequests)
	mux.HandleFunc("POST /api/v1/overtime/requests", overtimeHandler.CreateRequest)
	mux.HandleFunc("GET /api/v1/overtime/requests/{id}", overtimeHandler.GetRequest)
	mux.HandleFunc("POST /api/v1/overtime/requests/{id}/submit", overtimeHandler.SubmitRequest)
	mux.HandleFunc("POST /api/v1/overtime/requests/{id}/consent/{emp_id}", overtimeHandler.ConsentRequest)
	mux.HandleFunc("GET /api/v1/overtime/requests/behalf-assignments", overtimeHandler.ListRequestBehalf)
	mux.HandleFunc("POST /api/v1/overtime/requests/behalf-assignments", overtimeHandler.CreateRequestBehalf)
	mux.HandleFunc("GET /api/v1/overtime/requests/behalf-assignments/my-targets", overtimeHandler.RequestBehalfTargets)
	mux.HandleFunc("DELETE /api/v1/overtime/requests/behalf-assignments/{id}", overtimeHandler.DeleteRequestBehalf)
	mux.HandleFunc("GET /api/v1/attendance/claim-ot/behalf-assignments", overtimeHandler.ListClaimBehalf)
	mux.HandleFunc("POST /api/v1/attendance/claim-ot/behalf-assignments", overtimeHandler.CreateClaimBehalf)
	mux.HandleFunc("DELETE /api/v1/attendance/claim-ot/behalf-assignments/{id}", overtimeHandler.DeleteClaimBehalf)
	mux.HandleFunc("POST /api/v1/attendance/claim-ot/{employee_id}", overtimeHandler.ClaimOT)
	mux.HandleFunc("POST /api/v1/attendance/approve-ot/{employee_id}", overtimeHandler.ApproveOT)
	shiftRepo := repository.ShiftRepository{DB: db}
	shiftHandler := api.ShiftHandler{Repository: shiftRepo, Authorizer: organizationAuthorizer}
	employeeAssignmentRepo := repository.EmployeeAssignmentRepository{DB: db}
	assignmentRuleRepo := repository.AssignmentRuleRepository{DB: db}
	assignmentHistoryRepo := repository.AssignmentHistoryRepository{DB: db}
	assignmentHandler := api.AssignmentHandler{Assignments: employeeAssignmentRepo, Rules: assignmentRuleRepo, History: assignmentHistoryRepo, Employees: employeeRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/assignment-history", assignmentHandler.AssignmentHistory)
	mux.HandleFunc("GET /api/v1/assignment-rules", assignmentHandler.ListAssignmentRules)
	mux.HandleFunc("POST /api/v1/assignment-rules", assignmentHandler.CreateAssignmentRule)
	mux.HandleFunc("PATCH /api/v1/assignment-rules/{rule_id}", assignmentHandler.UpdateAssignmentRule)
	mux.HandleFunc("DELETE /api/v1/assignment-rules/{rule_id}", assignmentHandler.DeleteAssignmentRule)
	mux.HandleFunc("POST /api/v1/time/punch-in", attendanceHandler.PunchIn)
	mux.HandleFunc("POST /api/v1/time/punch-out/{employee_id}", attendanceHandler.PunchOut)
	mux.HandleFunc("GET /api/v1/time/history/{employee_id}", attendanceHandler.TimeHistory)
	mux.HandleFunc("POST /api/v1/attendance/adjustments", attendanceHandler.CreateAdjustment)
	mux.HandleFunc("GET /api/v1/attendance/adjustments", attendanceHandler.ListAdjustments)
	mux.HandleFunc("PUT /api/v1/attendance/adjustments/{id}/approve", attendanceHandler.ApproveAdjustment)
	mux.HandleFunc("GET /api/v1/employees/{id}/attendance/adjustments", attendanceHandler.ListEmployeeAdjustments)
	mux.HandleFunc("GET /api/v1/attendance/summaries", attendanceHandler.Summaries)
	mux.HandleFunc("POST /api/v1/attendance/reprocess-all", attendanceHandler.ReprocessAll)
	mux.HandleFunc("POST /api/v1/attendance/reprocess/{employee_id}", attendanceHandler.ReprocessEmployee)
	mux.HandleFunc("POST /api/v1/attendance/adjustments/{id}/approve", attendanceHandler.ApproveAdjustment)
	mux.HandleFunc("PUT /api/v1/time/entries/{id}", attendanceHandler.UpdateTimeEntry)
	mux.HandleFunc("DELETE /api/v1/time/entries/{id}", attendanceHandler.DeleteTimeEntry)
	mux.HandleFunc("POST /api/v1/attendance/upload", attendanceLogsHandler.Upload)
	mux.HandleFunc("POST /api/v1/attendance/logs/upload", attendanceLogsHandler.UploadLogs)
	mux.HandleFunc("POST /api/v1/attendance/bulk-upload-paper-ot", attendanceLogsHandler.BulkPaperOT)
	mux.HandleFunc("POST /api/v1/attendance/paper-forms/overtime/bulk", attendanceLogsHandler.BulkPaperOT)
	mux.HandleFunc("POST /api/v1/attendance/enter-paper-ot/{employee_id}", attendanceLogsHandler.EnterPaperOT)
	mux.HandleFunc("POST /api/v1/attendance/paper-forms/overtime/{employee_id}", attendanceLogsHandler.EnterPaperOT)
	mux.HandleFunc("GET /api/v1/attendance/import-jobs/latest", attendanceLogsHandler.Latest)
	mux.HandleFunc("GET /api/v1/attendance/import-jobs/{id}", attendanceLogsHandler.Job)
	mux.HandleFunc("GET /api/v1/attendance/import-jobs/{id}/error-file", attendanceLogsHandler.ErrorFile)
	mux.HandleFunc("GET /api/v1/attendance/logs", attendanceLogsHandler.Logs)
	mux.HandleFunc("GET /api/v1/attendance/raw-logs", attendanceLogsHandler.RawLogs)
	mux.HandleFunc("GET /api/v1/attendance/staging-logs", attendanceLogsHandler.Staging)
	mux.HandleFunc("POST /api/v1/attendance/staging-logs/{id}/resolve", attendanceLogsHandler.Resolve)
	mux.HandleFunc("DELETE /api/v1/attendance/staging-logs/{id}", attendanceLogsHandler.Dismiss)
	mux.HandleFunc("GET /api/v1/attendance/rules", attendancePolicyHandler.GetRules)
	mux.HandleFunc("PATCH /api/v1/attendance/rules", attendancePolicyHandler.UpdateRules)
	mux.HandleFunc("GET /api/v1/attendance/rules/versions", attendancePolicyHandler.ListRuleVersions)
	mux.HandleFunc("GET /api/v1/attendance/policies/assignments", attendancePolicyHandler.ListAssignments)
	mux.HandleFunc("POST /api/v1/attendance/policies/assignments", attendancePolicyHandler.UpsertAssignment)
	mux.HandleFunc("DELETE /api/v1/attendance/policies/assignments/{assignment_id}", attendancePolicyHandler.DeleteAssignment)
	mux.HandleFunc("GET /api/v1/attendance/adjustments/behalf-assignments", attendancePolicyHandler.ListBehalfAssignments)
	mux.HandleFunc("POST /api/v1/attendance/adjustments/behalf-assignments", attendancePolicyHandler.CreateBehalfAssignment)
	mux.HandleFunc("DELETE /api/v1/attendance/adjustments/behalf-assignments/{id}", attendancePolicyHandler.DeleteBehalfAssignment)
	mux.HandleFunc("GET /api/v1/attendance/adjustments/behalf-assignments/my-targets", attendancePolicyHandler.MyBehalfTargets)
	mux.HandleFunc("POST /api/v1/shifts", shiftHandler.Create)
	mux.HandleFunc("GET /api/v1/shifts", shiftHandler.List)
	mux.HandleFunc("PUT /api/v1/shifts/{shift_id}", shiftHandler.Update)
	mux.HandleFunc("GET /api/v1/shifts/{shift_id}/versions", shiftHandler.Versions)
	mux.HandleFunc("POST /api/v1/shifts/assign", shiftHandler.Assign)
	patternRepo := repository.WorkingPatternRepository{DB: db}
	patternHandler := api.WorkingPatternHandler{Repository: patternRepo, Authorizer: organizationAuthorizer, Compliance: patternRepo}
	mux.HandleFunc("POST /api/v1/working-patterns", patternHandler.Create)
	mux.HandleFunc("GET /api/v1/working-patterns", patternHandler.List)
	mux.HandleFunc("GET /api/v1/working-patterns/{pattern_id}", patternHandler.Details)
	mux.HandleFunc("PUT /api/v1/working-patterns/{pattern_id}", patternHandler.Update)
	mux.HandleFunc("DELETE /api/v1/working-patterns/{pattern_id}", patternHandler.Delete)
	mux.HandleFunc("POST /api/v1/employments/{employment_id}/working-patterns", patternHandler.Assign)
	mux.HandleFunc("POST /api/v1/working-patterns/bulk-assign", patternHandler.BulkAssign)
	mux.HandleFunc("GET /api/v1/employments/{employment_id}/working-patterns", patternHandler.History)
	rosterRepo := repository.RosterRepository{DB: db}
	rosterHandler := api.RosterHandler{Repository: rosterRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/employees/{id}/rosters", rosterHandler.ListEmployee)
	mux.HandleFunc("POST /api/v1/employees/{id}/rosters", rosterHandler.Upsert)
	mux.HandleFunc("POST /api/v1/employees/{id}/rosters/bulk", rosterHandler.BulkUpsert)
	mux.HandleFunc("DELETE /api/v1/employees/{id}/rosters/{date}", rosterHandler.Delete)
	scheduleRepo := repository.ScheduleRepository{DB: db}
	scheduleHandler := api.ScheduleHandler{Repository: scheduleRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/schedules", scheduleHandler.List)
	swapRepo := repository.ShiftSwapRepository{DB: db}
	swapHandler := api.ShiftSwapHandler{Repository: swapRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("POST /api/v1/shift-swaps", swapHandler.Create)
	mux.HandleFunc("GET /api/v1/shift-swaps", swapHandler.List)
	mux.HandleFunc("PUT /api/v1/shift-swaps/{id}/approve", swapHandler.Approve)
	mux.HandleFunc("GET /api/v1/shift-swaps/stats", swapHandler.Stats)
	segmentRepo := repository.WorkSegmentRepository{DB: db}
	segmentHandler := api.WorkSegmentHandler{Repository: segmentRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/work-segments", segmentHandler.List)
	mux.HandleFunc("GET /api/v1/work-segments/{segment_id}", segmentHandler.Get)
	mux.HandleFunc("POST /api/v1/work-segments", segmentHandler.Create)
	mux.HandleFunc("PUT /api/v1/work-segments/{segment_id}", segmentHandler.Update)
	mux.HandleFunc("DELETE /api/v1/work-segments/{segment_id}", segmentHandler.Delete)
	mux.HandleFunc("POST /api/v1/work-segments/bulk", segmentHandler.BulkCreate)
	mux.HandleFunc("POST /api/v1/employees", employeeHandler.Create)
	mux.HandleFunc("GET /api/v1/employees", employeeHandler.List)
	mux.HandleFunc("GET /api/v1/employees/{id}", employeeHandler.Get)
	mux.HandleFunc("PUT /api/v1/employees/{id}", employeeHandler.Update)
	mux.HandleFunc("DELETE /api/v1/employees/{id}", employeeHandler.Delete)
	mux.HandleFunc("POST /api/v1/employees/{id}/rehire", employeeHandler.Rehire)
	mux.HandleFunc("POST /api/v1/employees/upload", employeeImportHandler.Upload)
	mux.HandleFunc("POST /api/v1/employees/import-termination/preview", employeeImportHandler.PreviewTermination)
	mux.HandleFunc("POST /api/v1/employees/import-termination/confirm", employeeImportHandler.ConfirmTermination)
	mux.HandleFunc("GET /api/v1/employees/import-termination/template", employeeImportHandler.TerminationTemplate)
	mux.HandleFunc("GET /api/v1/working-conditions", employeeConditionHandler.ListConditions)
	mux.HandleFunc("GET /api/v1/employees/{id}/conditions", employeeConditionHandler.List)
	mux.HandleFunc("POST /api/v1/employees/{id}/conditions", employeeConditionHandler.Create)
	mux.HandleFunc("DELETE /api/v1/employees/{id}/conditions/{cond_id}", employeeConditionHandler.Delete)
	mux.HandleFunc("GET /api/v1/employees/{id}/departments", employeeDepartmentHandler.List)
	mux.HandleFunc("POST /api/v1/employees/{id}/departments", employeeDepartmentHandler.Transfer)
	mux.HandleFunc("GET /api/v1/employees/{id}/compensation-history", compensationHistoryHandler.Get)
	customFieldRepo := repository.CustomFieldRepository{DB: db}
	customFieldHandler := api.CustomFieldHandler{Repository: customFieldRepo, Authorizer: organizationAuthorizer}
	contractRepo := repository.ContractRepository{DB: db}
	employmentRepo := repository.EmploymentRepository{DB: db}
	contractEmploymentHandler := api.ContractEmploymentHandler{Contracts: contractRepo, Employments: employmentRepo, CustomFields: customFieldRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/contracts", contractEmploymentHandler.ListContracts)
	mux.HandleFunc("POST /api/v1/employees/{employee_id}/contracts", contractEmploymentHandler.CreateContract)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.GetContract)
	mux.HandleFunc("PATCH /api/v1/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.UpdateContract)
	mux.HandleFunc("DELETE /api/v1/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.DeleteContract)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/employments", contractEmploymentHandler.ListEmployments)
	mux.HandleFunc("POST /api/v1/employees/{employee_id}/employments", contractEmploymentHandler.CreateEmployment)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.GetEmployment)
	mux.HandleFunc("PATCH /api/v1/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.UpdateEmployment)
	mux.HandleFunc("DELETE /api/v1/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.DeleteEmployment)
	mux.HandleFunc("GET /api/v1/assignments", assignmentHandler.ListAssignments)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/assignments", assignmentHandler.ListEmployeeAssignments)
	mux.HandleFunc("POST /api/v1/employees/{employee_id}/assignments", assignmentHandler.CreateEmployeeAssignment)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.GetEmployeeAssignment)
	mux.HandleFunc("PATCH /api/v1/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.UpdateEmployeeAssignment)
	mux.HandleFunc("DELETE /api/v1/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.DeleteEmployeeAssignment)
	mux.HandleFunc("GET /api/v1/custom-fields/definitions", customFieldHandler.ListDefinitions)
	mux.HandleFunc("POST /api/v1/custom-fields/definitions", customFieldHandler.CreateDefinition)
	mux.HandleFunc("GET /api/v1/custom-fields/definitions/{id}", customFieldHandler.GetDefinition)
	mux.HandleFunc("PATCH /api/v1/custom-fields/definitions/{id}", customFieldHandler.UpdateDefinition)
	mux.HandleFunc("DELETE /api/v1/custom-fields/definitions/{id}", customFieldHandler.DeleteDefinition)
	mux.HandleFunc("GET /api/v1/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.GetValues)
	mux.HandleFunc("PUT /api/v1/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.SetValues)
	leaveRepo := repository.LeaveRepository{DB: db}
	leaveHandler := api.LeaveHandler{
		Repository:       leaveRepo,
		BehalfRepository: attendancePolicyRepo,
		Authorizer:       organizationAuthorizer,
		StorageDir:       cfg.StorageDir,
	}
	mux.HandleFunc("GET /api/v1/leave/types", leaveHandler.ListTypes)
	mux.HandleFunc("POST /api/v1/leave/types", leaveHandler.CreateType)
	mux.HandleFunc("PUT /api/v1/leave/types/{id}/config", leaveHandler.UpdateTypeConfig)
	mux.HandleFunc("DELETE /api/v1/leave/types/{id}", leaveHandler.DeleteType)
	mux.HandleFunc("POST /api/v1/leave/types/{id}/restore", leaveHandler.RestoreType)
	mux.HandleFunc("POST /api/v1/leave/submit", leaveHandler.Submit)
	mux.HandleFunc("GET /api/v1/leave/requests", leaveHandler.ListRequests)
	mux.HandleFunc("POST /api/v1/leave/requests/{id}/approve", leaveHandler.Approve)
	mux.HandleFunc("GET /api/v1/employees/{id}/leave/balances", leaveHandler.Balances)
	mux.HandleFunc("GET /api/v1/leave/balances", leaveHandler.OrganizationBalances)
	mux.HandleFunc("GET /api/v1/employees/{id}/leave/history", leaveHandler.EmployeeHistory)
	mux.HandleFunc("POST /api/v1/employees/{id}/leave/adjust", leaveHandler.AdjustBalance)
	mux.HandleFunc("GET /api/v1/employees/{id}/leave/requests", leaveHandler.MyRequests)
	mux.HandleFunc("GET /api/v1/leave/policies", leaveHandler.ListPolicies)
	mux.HandleFunc("POST /api/v1/leave/policies", leaveHandler.CreatePolicy)
	mux.HandleFunc("PATCH /api/v1/leave/policies/{policy_id}", leaveHandler.UpdatePolicy)
	mux.HandleFunc("DELETE /api/v1/leave/policies/{policy_id}", leaveHandler.DeletePolicy)
	mux.HandleFunc("POST /api/v1/leave/policies/assign", leaveHandler.AssignPolicy)
	mux.HandleFunc("GET /api/v1/leave/policies/assignments", leaveHandler.ListPolicyAssignments)
	mux.HandleFunc("DELETE /api/v1/leave/policies/assignments/{assignment_id}", leaveHandler.DeletePolicyAssignment)
	mux.HandleFunc("POST /api/v1/leave/accrual/run", leaveHandler.TriggerAccrual)
	mux.HandleFunc("GET /api/v1/jobs/{job_id}/status", leaveHandler.JobStatus)
	mux.HandleFunc("POST /api/v1/leave/upload-excel", leaveHandler.UploadExcel)
	mux.HandleFunc("GET /api/v1/leave/import/latest", leaveHandler.LatestImport)
	mux.HandleFunc("GET /api/v1/leave/import/{job_id}/errors", leaveHandler.ImportErrors)
	mux.HandleFunc("GET /api/v1/leave/upload-template", leaveHandler.UploadTemplate)
	mux.HandleFunc("GET /api/v1/leave/behalf-assignments", leaveHandler.ListBehalfAssignments)
	mux.HandleFunc("POST /api/v1/leave/behalf-assignments", leaveHandler.CreateBehalfAssignment)
	mux.HandleFunc("GET /api/v1/leave/behalf-assignments/my-targets", leaveHandler.GetMyBehalfTargets)
	mux.HandleFunc("DELETE /api/v1/leave/behalf-assignments/{id}", leaveHandler.DeleteBehalfAssignment)
	holidayRepo := repository.HolidayRepository{DB: db}
	holidayHandler := api.HolidayHandler{Repository: holidayRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/holidays", holidayHandler.List)
	mux.HandleFunc("POST /api/v1/holidays", holidayHandler.Create)
	mux.HandleFunc("GET /api/v1/holidays/{id}", holidayHandler.Get)
	mux.HandleFunc("PUT /api/v1/holidays/{id}", holidayHandler.Update)
	mux.HandleFunc("PATCH /api/v1/holidays/{id}", holidayHandler.Update)
	mux.HandleFunc("DELETE /api/v1/holidays/{id}", holidayHandler.Delete)
	mux.HandleFunc("POST /api/v1/holidays/{id}/process-leave", holidayHandler.ProcessLeave)
	payrollRepo := repository.PayrollRepository{DB: db}
	essRepo := repository.ESSRepository{DB: db}
	essHandler := api.ESSHandler{
		Repository: essRepo,
		Attendance: attendanceRepo,
		Payroll:    payrollRepo,
		Leaves:     leaveRepo,
		ShiftSwaps: swapRepo,
	}
	mux.HandleFunc("GET /api/v1/ess/profile", essHandler.GetProfile)
	mux.HandleFunc("PUT /api/v1/ess/profile", essHandler.UpdateProfile)
	mux.HandleFunc("GET /api/v1/ess/emergency-contacts", essHandler.GetContacts)
	mux.HandleFunc("POST /api/v1/ess/emergency-contacts", essHandler.AddContact)
	mux.HandleFunc("DELETE /api/v1/ess/emergency-contacts/{contact_id}", essHandler.DeleteContact)
	mux.HandleFunc("GET /api/v1/ess/documents/pending", essHandler.GetPendingDocuments)
	mux.HandleFunc("POST /api/v1/ess/documents/{id}/acknowledge", essHandler.AcknowledgeDocument)
	mux.HandleFunc("GET /api/v1/ess/documents", essHandler.GetDocuments)
	mux.HandleFunc("GET /api/v1/ess/payslips", essHandler.GetPayslips)
	mux.HandleFunc("GET /api/v1/ess/attendance", essHandler.GetAttendance)
	mux.HandleFunc("POST /api/v1/ess/punch", essHandler.Punch)
	mux.HandleFunc("GET /api/v1/ess/time-entries", essHandler.GetTimeEntries)
	mux.HandleFunc("GET /api/v1/ess/leave/balances", essHandler.GetLeaveBalances)
	mux.HandleFunc("GET /api/v1/ess/leave/requests", essHandler.GetLeaveRequests)
	mux.HandleFunc("GET /api/v1/ess/shift-swaps", essHandler.GetShiftSwaps)
	mux.HandleFunc("POST /api/v1/ess/shift-swaps", essHandler.CreateShiftSwap)
	mux.HandleFunc("PUT /api/v1/ess/shift-swaps/{id}/respond", essHandler.RespondShiftSwap)
	mux.HandleFunc("DELETE /api/v1/ess/shift-swaps/{id}", essHandler.CancelShiftSwap)
	workflowRepo := repository.ApprovalWorkflowRepository{DB: db, SystemDB: db}
	workflowHandler := api.ApprovalWorkflowHandler{Repository: workflowRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/approval-workflows", workflowHandler.List)
	mux.HandleFunc("POST /api/v1/approval-workflows", workflowHandler.Create)
	mux.HandleFunc("POST /api/v1/approval-workflows/approve", workflowHandler.Process)
	mux.HandleFunc("GET /api/v1/approval-workflows/history/{type}/{id}", workflowHandler.History)
	mux.HandleFunc("GET /api/v1/approval-workflows/{id}", workflowHandler.Get)
	mux.HandleFunc("PUT /api/v1/approval-workflows/{id}", workflowHandler.Update)
	mux.HandleFunc("DELETE /api/v1/approval-workflows/{id}", workflowHandler.Delete)
	mux.HandleFunc("GET /api/v1/approval-workflows/pending-tasks", workflowHandler.PendingTasks)
	notifRepo := repository.NotificationRepository{SystemDB: db}
	notifHandler := api.NotificationHandler{Repository: notifRepo}
	mux.HandleFunc("GET /v1/notifications", notifHandler.List)
	mux.HandleFunc("GET /v1/notifications/unread-count", notifHandler.UnreadCount)
	mux.HandleFunc("PATCH /v1/notifications/{id}/read", notifHandler.MarkRead)
	mux.HandleFunc("POST /v1/notifications/read-all", notifHandler.MarkAllRead)
	mux.HandleFunc("GET /api/v1/notifications", notifHandler.List)
	mux.HandleFunc("GET /api/v1/notifications/unread-count", notifHandler.UnreadCount)
	mux.HandleFunc("PATCH /api/v1/notifications/{id}/read", notifHandler.MarkRead)
	mux.HandleFunc("POST /api/v1/notifications/read-all", notifHandler.MarkAllRead)
	payrollHandler := api.PayrollHandler{Repository: payrollRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/payroll/payslips", payrollHandler.ListPayslips)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/payslips", payrollHandler.ListEmployee)
	mux.HandleFunc("GET /api/v1/salary-components", payrollHandler.ListSalaryComponents)
	mux.HandleFunc("POST /api/v1/salary-components", payrollHandler.CreateSalaryComponent)
	mux.HandleFunc("GET /api/v1/employments/{id}/salary-components", payrollHandler.ListEmploymentSalaryComponents)
	mux.HandleFunc("POST /api/v1/employments/{id}/salary-components", payrollHandler.AssignEmploymentSalaryComponent)
	mux.HandleFunc("GET /api/v1/salary-bases", payrollHandler.ListSalaryBases)
	mux.HandleFunc("POST /api/v1/salary-bases", payrollHandler.CreateSalaryBase)
	mux.HandleFunc("POST /api/v1/salary-bases/{base_id}/members", payrollHandler.AssignBaseMember)
	mux.HandleFunc("DELETE /api/v1/salary-bases/{base_id}/members/{component_id}", payrollHandler.UnassignBaseMember)
	mux.HandleFunc("GET /api/v1/payroll-settings", payrollHandler.GetPayrollSettings)
	mux.HandleFunc("PATCH /api/v1/payroll-settings", payrollHandler.UpdatePayrollSettings)
	mux.HandleFunc("GET /api/v1/payroll-settings/history", payrollHandler.GetPayrollSettingsHistory)
	mux.HandleFunc("GET /api/v1/employees/{id}/payroll-config", payrollHandler.GetEmployeePayrollConfig)
	mux.HandleFunc("POST /api/v1/employees/{id}/payroll-config", payrollHandler.SetEmployeePayrollConfig)
	mux.HandleFunc("GET /api/v1/payroll/incentive-policies", payrollHandler.GetIncentivePolicies)
	mux.HandleFunc("POST /api/v1/payroll/incentive-policies", payrollHandler.CreateIncentivePolicy)
	mux.HandleFunc("DELETE /api/v1/payroll/incentive-policies/{policy_id}", payrollHandler.DeleteIncentivePolicy)
	mux.HandleFunc("POST /api/v1/payroll/incentive-policies/{policy_id}/clone", payrollHandler.CloneIncentivePolicy)
	mux.HandleFunc("GET /api/v1/payroll-rules", payrollHandler.ListPayrollRules)
	mux.HandleFunc("POST /api/v1/payroll-rules", payrollHandler.CreatePayrollRule)
	mux.HandleFunc("GET /api/v1/payroll-rules/{rule_id}/tiers", payrollHandler.ListPayrollRuleTiers)
	mux.HandleFunc("POST /api/v1/payroll-rules/{rule_id}/tiers", payrollHandler.CreatePayrollRuleTier)
	mux.HandleFunc("DELETE /api/v1/payroll-rules/{rule_id}/tiers/{tier_id}", payrollHandler.DeletePayrollRuleTier)
	mux.HandleFunc("GET /api/v1/payroll-rules/{rule_id}/assignments", payrollHandler.ListPayrollRuleAssignments)
	mux.HandleFunc("POST /api/v1/payroll-rules/{rule_id}/assignments", payrollHandler.AssignPayrollRule)
	mux.HandleFunc("DELETE /api/v1/payroll-rules/{rule_id}/assignments/{assignment_id}", payrollHandler.DeletePayrollRuleAssignment)
	mux.HandleFunc("POST /api/v1/employees/{id}/calculate-payroll", payrollHandler.CalculatePayroll)
	mux.HandleFunc("POST /api/v1/payroll/calculate-batch", payrollHandler.CalculateBatchPayroll)
	reportRepo := repository.ReportRepository{DB: db}
	reportQueue := &repository.RedisReportQueue{Client: redisClient}
	reportHandler := api.ReportHandler{Repository: reportRepo, Queue: reportQueue, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/reports/attendance", reportHandler.AttendanceReport)
	mux.HandleFunc("GET /api/v1/reports/no-show", reportHandler.NoShowReport)
	mux.HandleFunc("POST /api/v1/reports/export", reportHandler.CreateExport)
	mux.HandleFunc("GET /api/v1/reports/jobs/{job_id}", reportHandler.Status)
	mux.HandleFunc("DELETE /api/v1/reports/jobs/{job_id}", reportHandler.Cancel)
	mux.HandleFunc("GET /api/v1/reports/jobs/{job_id}/download", reportHandler.Download)
	fileRepo := repository.FileRepository{DB: db}
	var fileStorage api.FileStorage
	if s3Client, err := external.NewS3Client(cfg.S3EndpointURL, cfg.S3BucketName, cfg.S3AccessKey.Expose(), cfg.S3SecretKey.Expose(), cfg.S3Region, cfg.S3UsePathStyle, nil); err == nil {
		fileStorage = s3Client
	}
	fileHandler := api.FileHandler{Repository: fileRepo, Storage: fileStorage, Authorizer: organizationAuthorizer}
	mux.HandleFunc("POST /v1/files/upload", fileHandler.UploadFile)
	mux.HandleFunc("POST /api/v1/files/upload", fileHandler.UploadFile)
	mux.HandleFunc("POST /v1/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("POST /api/v1/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("GET /v1/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("GET /api/v1/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("PUT /v1/files/{id}", fileHandler.UpdateFile)
	mux.HandleFunc("PUT /api/v1/files/{id}", fileHandler.UpdateFile)
	mux.HandleFunc("DELETE /v1/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("DELETE /api/v1/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("GET /v1/files/{id}/download", fileHandler.DownloadFile)
	mux.HandleFunc("GET /api/v1/files/{id}/download", fileHandler.DownloadFile)
	mux.HandleFunc("GET /v1/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("GET /api/v1/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("GET /v1/files/{id}/versions", fileHandler.ListVersions)
	mux.HandleFunc("GET /api/v1/files/{id}/versions", fileHandler.ListVersions)
	mux.HandleFunc("GET /v1/files/by-owner/{owner_type}/{owner_id}", fileHandler.ListByOwner)
	mux.HandleFunc("GET /api/v1/files/by-owner/{owner_type}/{owner_id}", fileHandler.ListByOwner)

	mux.HandleFunc("POST /v1/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("POST /api/v1/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("GET /v1/ess/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("GET /api/v1/ess/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("POST /v1/ess/signatures/{request_id}/sign", fileHandler.SignDocument)
	mux.HandleFunc("POST /api/v1/ess/signatures/{request_id}/sign", fileHandler.SignDocument)

	documentRepo := repository.DocumentRepository{DB: db}
	documentHandler := api.DocumentHandler{Repository: documentRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /v1/documents/folders", documentHandler.ListFolders)
	mux.HandleFunc("POST /v1/documents/folders", documentHandler.CreateFolder)
	mux.HandleFunc("GET /v1/documents/folders/{id}", documentHandler.GetFolder)
	mux.HandleFunc("PUT /v1/documents/folders/{id}", documentHandler.RenameFolder)
	mux.HandleFunc("DELETE /v1/documents/folders/{id}", documentHandler.DeleteFolder)
	mux.HandleFunc("POST /v1/documents/distributions", documentHandler.Distribute)
	mux.HandleFunc("POST /v1/documents/templates/preview", documentHandler.PreviewTemplate)
	mux.HandleFunc("POST /v1/documents/templates/{id}/generate", documentHandler.GenerateTemplate)

	mux.HandleFunc("GET /api/v1/documents/folders", documentHandler.ListFolders)
	mux.HandleFunc("POST /api/v1/documents/folders", documentHandler.CreateFolder)
	mux.HandleFunc("GET /api/v1/documents/folders/{id}", documentHandler.GetFolder)
	mux.HandleFunc("PUT /api/v1/documents/folders/{id}", documentHandler.RenameFolder)
	mux.HandleFunc("DELETE /api/v1/documents/folders/{id}", documentHandler.DeleteFolder)
	mux.HandleFunc("POST /api/v1/documents/distributions", documentHandler.Distribute)
	mux.HandleFunc("POST /api/v1/documents/templates/preview", documentHandler.PreviewTemplate)
	mux.HandleFunc("POST /api/v1/documents/templates/{id}/generate", documentHandler.GenerateTemplate)
	conditionRepo := repository.ConditionTypeRepository{DB: db}
	conditionHandler := api.ConditionTypeHandler{Repository: conditionRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/condition-types", conditionHandler.List)
	mux.HandleFunc("GET /api/v1/condition-types/{id}", conditionHandler.Get)
	mux.HandleFunc("POST /api/v1/condition-types", conditionHandler.Create)
	mux.HandleFunc("PATCH /api/v1/condition-types/{id}", conditionHandler.Update)
	mux.HandleFunc("DELETE /api/v1/condition-types/{id}", conditionHandler.Delete)
	phaseRepo := repository.EmploymentPhaseRepository{DB: db}
	phaseHandler := api.EmploymentPhaseHandler{Repository: phaseRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("POST /api/v1/employment-phases", phaseHandler.CreateDefinition)
	mux.HandleFunc("GET /api/v1/employment-phases", phaseHandler.ListDefinitions)
	mux.HandleFunc("POST /api/v1/employees/{user_id}/phases", phaseHandler.AssignEmployee)
	mux.HandleFunc("GET /api/v1/employees/{user_id}/phases", phaseHandler.ListEmployee)
	mux.HandleFunc("PUT /api/v1/employees/{user_id}/phases/{phase_id}/status", phaseHandler.UpdateEmployeeStatus)
	mux.HandleFunc("POST /api/v1/employments/{employment_id}/phases", phaseHandler.AssignEmployment)
	mux.HandleFunc("GET /api/v1/employments/{employment_id}/phases", phaseHandler.ListEmployment)
	mux.HandleFunc("PUT /api/v1/employments/{employment_id}/phases/{phase_id}/status", phaseHandler.UpdateEmploymentStatus)

	lifecycleRepo := repository.LifecycleRepository{DB: db}
	lifecycleHandler := api.LifecycleHandler{
		Repository: lifecycleRepo,
		Authorizer: organizationAuthorizer,
	}
	mux.HandleFunc("GET /api/v1/lifecycle/my-tasks", lifecycleHandler.GetMyTasks)
	mux.HandleFunc("GET /api/v1/lifecycle/instances", lifecycleHandler.ListChecklists)
	mux.HandleFunc("POST /api/v1/lifecycle-templates", lifecycleHandler.CreateTemplate)
	mux.HandleFunc("GET /api/v1/lifecycle-templates", lifecycleHandler.ListTemplates)
	mux.HandleFunc("GET /api/v1/lifecycle-templates/{id}", lifecycleHandler.GetTemplate)
	mux.HandleFunc("PATCH /api/v1/lifecycle-templates/{id}", lifecycleHandler.UpdateTemplate)
	mux.HandleFunc("DELETE /api/v1/lifecycle-templates/{id}", lifecycleHandler.DeleteTemplate)
	mux.HandleFunc("POST /api/v1/lifecycle-templates/{id}/items", lifecycleHandler.CreateTemplateItem)
	mux.HandleFunc("GET /api/v1/lifecycle-templates/{id}/items", lifecycleHandler.ListTemplateItems)
	mux.HandleFunc("GET /api/v1/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.GetTemplateItem)
	mux.HandleFunc("PUT /api/v1/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.UpdateTemplateItem)
	mux.HandleFunc("DELETE /api/v1/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.DeleteTemplateItem)
	mux.HandleFunc("POST /api/v1/employments/{emp_id}/lifecycle", lifecycleHandler.CreateChecklist)
	mux.HandleFunc("GET /api/v1/employments/{emp_id}/lifecycle", lifecycleHandler.ListEmploymentChecklists)
	mux.HandleFunc("GET /api/v1/lifecycle/{id}", lifecycleHandler.GetChecklist)
	mux.HandleFunc("PATCH /api/v1/lifecycle/{id}/items/{iid}", lifecycleHandler.UpdateItemStatus)

	mux.HandleFunc("GET /api/v1/health", healthHandler.Liveness)
	mux.HandleFunc("GET /api/v1/health/ready", healthHandler.Readiness)
	systemHealthHandler := api.SystemHealthHandler{Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/system/health", systemHealthHandler.Get)

	systemSettingsRepo := repository.SystemSettingsRepository{DB: db}
	systemAdminHandler := api.SystemAdminHandler{
		Authorizer:  organizationAuthorizer,
		Settings:    systemSettingsRepo,
		EmailSender: service.EmailService{Settings: systemSettingsRepo},
		Redis:       redisClient,
	}
	mux.HandleFunc("GET /api/v1/system/smtp/config", systemAdminHandler.GetSMTPConfig)
	mux.HandleFunc("PUT /api/v1/system/smtp/config", systemAdminHandler.SetSMTPConfig)
	mux.HandleFunc("POST /api/v1/system/smtp/test", systemAdminHandler.TestSMTP)

	monitorRepo := repository.MonitorRepository{DB: db, SystemDB: db}
	monitorHandler := api.MonitorHandler{
		Repo:       monitorRepo,
		Authorizer: organizationAuthorizer,
		Redis:      redisClient,
		JWTSecret:  cfg.JWTSecret.Expose(),
	}
	mux.HandleFunc("GET /api/v1/monitor/overview", monitorHandler.GetOverview)
	mux.HandleFunc("GET /v1/monitor/overview", monitorHandler.GetOverview)
	mux.HandleFunc("GET /monitor/overview", monitorHandler.GetOverview)
	mux.HandleFunc("GET /api/v1/monitor/jobs", monitorHandler.GetMonitorJobs)
	mux.HandleFunc("GET /v1/monitor/jobs", monitorHandler.GetMonitorJobs)
	mux.HandleFunc("GET /monitor/jobs", monitorHandler.GetMonitorJobs)
	mux.HandleFunc("GET /api/v1/monitor/correlations", monitorHandler.GetMonitorCorrelations)
	mux.HandleFunc("GET /v1/monitor/correlations", monitorHandler.GetMonitorCorrelations)
	mux.HandleFunc("GET /monitor/correlations", monitorHandler.GetMonitorCorrelations)
	mux.HandleFunc("GET /api/v1/monitor/stream", monitorHandler.Stream)
	mux.HandleFunc("GET /v1/monitor/stream", monitorHandler.Stream)
	mux.HandleFunc("GET /monitor/stream", monitorHandler.Stream)
	mux.HandleFunc("GET /api/v1/monitor/alerts", monitorHandler.ListAlerts)
	mux.HandleFunc("GET /v1/monitor/alerts", monitorHandler.ListAlerts)
	mux.HandleFunc("GET /monitor/alerts", monitorHandler.ListAlerts)
	mux.HandleFunc("GET /api/v1/monitor/alerts/{alert_id}", monitorHandler.GetAlert)
	mux.HandleFunc("GET /v1/monitor/alerts/{alert_id}", monitorHandler.GetAlert)
	mux.HandleFunc("GET /monitor/alerts/{alert_id}", monitorHandler.GetAlert)
	mux.HandleFunc("POST /api/v1/monitor/alerts/{alert_id}/acknowledge", monitorHandler.AcknowledgeAlert)
	mux.HandleFunc("POST /v1/monitor/alerts/{alert_id}/acknowledge", monitorHandler.AcknowledgeAlert)
	mux.HandleFunc("POST /monitor/alerts/{alert_id}/acknowledge", monitorHandler.AcknowledgeAlert)
	mux.HandleFunc("POST /api/v1/monitor/alerts/{alert_id}/resolve", monitorHandler.ResolveAlert)
	mux.HandleFunc("POST /v1/monitor/alerts/{alert_id}/resolve", monitorHandler.ResolveAlert)
	mux.HandleFunc("POST /monitor/alerts/{alert_id}/resolve", monitorHandler.ResolveAlert)
	mux.HandleFunc("GET /api/v1/ws/reports/{job_id}", monitorHandler.ReportStreamWS)
	mux.HandleFunc("GET /v1/ws/reports/{job_id}", monitorHandler.ReportStreamWS)
	mux.HandleFunc("GET /ws/reports/{job_id}", monitorHandler.ReportStreamWS)
	mux.HandleFunc("GET /api/v1/audit-logs", monitorHandler.GetAuditLogs)
	mux.HandleFunc("GET /api/v1/jobs/{job_id}", monitorHandler.GetJobStatus)
	mux.HandleFunc("GET /api/v1/monitor/jobs/{job_id}", monitorHandler.GetMonitorJob)

	mux.HandleFunc("POST /api/v1/auth/login", authHandler.Login)
	mux.HandleFunc("POST /api/v1/auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /api/v1/auth/logout", authHandler.Logout)
	mux.HandleFunc("POST /api/v1/auth/forgot-password", authHandler.ForgotPassword)
	mux.HandleFunc("POST /api/v1/auth/reset-password", authHandler.ResetPassword)
	mux.HandleFunc("POST /api/v1/auth/change-password", authHandler.ChangePassword)
	mux.HandleFunc("GET /api/v1/me", authHandler.Me)
	mux.HandleFunc("PATCH /api/v1/me", authHandler.UpdateMe)
	mux.HandleFunc("GET /api/v1/me/permissions", authHandler.MyPermissions)
	mux.HandleFunc("GET /api/v1/users", authHandler.OrganizationUsers)
	preferencesRepo := repository.DashboardPreferencesRepository{DB: db}
	preferencesHandler := api.DashboardPreferencesHandler{Repository: preferencesRepo, Authorizer: organizationAuthorizer}
	mux.HandleFunc("GET /api/v1/dashboard/preferences", preferencesHandler.Get)
	mux.HandleFunc("PUT /api/v1/dashboard/preferences", preferencesHandler.Put)
	dashboardAnalyticsRepo := repository.DashboardAnalyticsRepository{DB: db}
	dashboardAnalyticsHandler := api.DashboardAnalyticsHandler{
		Repository: dashboardAnalyticsRepo,
		Authorizer: organizationAuthorizer,
	}
	mux.HandleFunc("GET /api/v1/dashboard/executive-overview", dashboardAnalyticsHandler.ExecutiveOverview)
	mux.HandleFunc("GET /api/v1/dashboard/overtime-breakdown", dashboardAnalyticsHandler.OvertimeBreakdown)
	mux.HandleFunc("GET /api/v1/dashboard/workspace-stats", dashboardAnalyticsHandler.WorkspaceStats)
	mux.HandleFunc("GET /api/v1/dashboard/attendance-trend", dashboardAnalyticsHandler.AttendanceTrend)
	mux.HandleFunc("GET /api/v1/dashboard/department-attendance", dashboardAnalyticsHandler.DepartmentAttendance)
	mux.HandleFunc("GET /api/v1/dashboard/unexcused-absences", dashboardAnalyticsHandler.UnexcusedAbsences)
	mux.HandleFunc("GET /api/v1/dashboard/overtime-trend", dashboardAnalyticsHandler.OvertimeTrend)
	mux.HandleFunc("GET /api/v1/dashboard/peak-times", dashboardAnalyticsHandler.PeakTimes)
	mux.HandleFunc("GET /api/v1/dashboard/leave-types", dashboardAnalyticsHandler.LeaveTypes)
	mux.HandleFunc("GET /api/v1/dashboard/lateness-by-day", dashboardAnalyticsHandler.LatenessByDay)
	mux.HandleFunc("GET /api/v1/dashboard/turnover-analytics", dashboardAnalyticsHandler.TurnoverAnalytics)
	mux.HandleFunc("GET /api/v1/dashboard/recruitment-effectiveness", dashboardAnalyticsHandler.RecruitmentEffectiveness)

	publicPrefixes := []string{
		"/health",
		"/health/ready",
		"/api/v1/health",
		"/api/v1/health/ready",
		"/api/v1/auth/login",
		"/api/v1/auth/refresh",
		"/api/v1/auth/forgot-password",
		"/api/v1/auth/reset-password",
		"/api/v1/ws/reports/",
		"/v1/ws/reports/",
		"/ws/reports/",
		"/api/v1/ws",
		"/ws",
	}
	var revocationChecker httpinfra.TokenRevocationChecker
	var rateLimiter httpinfra.RateLimiterBackend
	if redisClient != nil {
		revocationChecker = redisClient
		rateLimiter = redisClient
	}
	rateLimitConfig := httpinfra.RateLimitConfig{
		MaxReadRequests:  uint64(cfg.RateLimitMaxRead),
		MaxWriteRequests: uint64(cfg.RateLimitMaxWrite),
		WindowSecs:       uint64(cfg.RateLimitWindowSecs),
	}
	handler := httpinfra.Chain(
		httpinfra.RequestIDMiddleware(),
		httpinfra.RateLimitMiddleware(rateLimitConfig, cfg.JWTSecret.Expose(), rateLimiter),
		httpinfra.LocaleMiddleware(),
		httpinfra.CORSMiddleware(cfg.AllowedOrigins),
		httpinfra.RejectLegacyWorkspaceRoutes(),
		httpinfra.AuthMiddlewareWithRevocation(cfg.JWTSecret.Expose(), publicPrefixes, revocationChecker),
	)(mux)

	addr := net.JoinHostPort(cfg.AppHost, strconv.Itoa(cfg.AppPort))
	server := &http.Server{
		Addr:    addr,
		Handler: handler,
	}

	var hooks []func(context.Context) error
	application := app.New(server, hooks...)
	ctx, cancel := app.NotifySignals(context.Background())
	defer cancel()

	if err := application.Run(ctx); err != nil {
		logger.Error("server fatal", "error", err)
		os.Exit(1)
	}
}
