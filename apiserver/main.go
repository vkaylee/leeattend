package main

import (
	"context"
	"crypto/ed25519"
	"encoding/base64"
	"fmt"
	"github.com/google/uuid"
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
	"log/slog"
	"net"
	"net/http"
	"os"
	"strconv"
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

	dbCfg := dbpool.DefaultConfig()
	dbCfg.PrimaryURL = string(cfg.DBPrimaryURL)
	dbCfg.PoolerTxURL = string(cfg.DBPoolerTxURL)
	dbCfg.PoolerSessURL = string(cfg.DBPoolerSessURL)
	dbCfg.ReaderURL = string(cfg.DBReaderURL)
	dbCfg.WorkspaceID = cfg.StandaloneWorkspaceID
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
		if err := standalonebootstrap.Bootstrap(context.Background(), sysPool, cfg.StandaloneWorkspaceID, cfg.StandaloneWorkspaceName, cfg.StandaloneAdminEmail, cfg.StandaloneAdminPassword.Expose()); err != nil {
			fmt.Fprintf(os.Stderr, "CONFIGURATION FAILED: standalone bootstrap failed\n")
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

	workspaceRepo := repository.WorkspaceRepository{
		SystemDB: func() repository.DBTX {
			pool, err := poolMgr.SystemPool()
			if err != nil {
				return nil
			}
			return pool
		}(),
		TenantDB: func(ctx context.Context, workspaceID uuid.UUID) (repository.DBTX, error) {
			return poolMgr.GetTenantTxPool(ctx, workspaceID)
		},
	}
	workspaceAuthorizer := repository.WorkspaceAuthorizer{DB: workspaceRepo.SystemDB, TenantDB: workspaceRepo.TenantDB}
	workspaceHandler := api.WorkspaceHandler{Repository: workspaceRepo, Authorizer: workspaceAuthorizer}
	var cryptoService *crypto.Service
	if encJSON := cfg.EncryptionKeysJSON.Expose(); encJSON != "" {
		if cs, err := crypto.FromJSON(encJSON); err == nil {
			cryptoService = cs
		} else {
			logger.Warn("crypto service initialization failed", "error", err)
		}
	}
	rbacRepo := repository.WorkspaceRBACRepository{SystemDB: workspaceRepo.SystemDB}
	rbacHandler := api.WorkspaceRBACHandler{
		Repository:    rbacRepo,
		Authorizer:    workspaceAuthorizer,
		Crypto:        cryptoService,
		StorageTester: api.NewS3StorageTester(cfg.S3EndpointURL),
	}
	authHandler.Authorizer = workspaceAuthorizer

	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}", workspaceHandler.Patch)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags", workspaceHandler.ListFeatureFlags)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/members", rbacHandler.ListMembers)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/members", rbacHandler.AddMember)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/members/{user_id}", rbacHandler.UpdateMemberRole)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/members/{user_id}", rbacHandler.RemoveMember)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/members/{user_id}/roles", rbacHandler.GetMemberRoles)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/members/{user_id}/roles", rbacHandler.AssignExtraRole)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/members/{user_id}/roles/{role_id}", rbacHandler.RevokeExtraRole)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/members/{user_id}/permissions", rbacHandler.GetMemberPermissions)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/members/{user_id}/permissions", rbacHandler.AssignExtraPermission)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/members/{user_id}/permissions/{perm_id}", rbacHandler.RevokeExtraPermission)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/roles", rbacHandler.ListRoles)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/roles", rbacHandler.CreateRole)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/roles/{role_id}", rbacHandler.UpdateRole)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/roles/{role_id}", rbacHandler.DeleteRole)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/roles/{role_id}/permissions", rbacHandler.GetRolePermissions)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/roles/{role_id}/permissions", rbacHandler.SetRolePermissions)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/permissions/catalog", rbacHandler.PermissionCatalog)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/permissions/available", rbacHandler.AvailablePermissions)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/settings/storage", rbacHandler.GetStorage)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/settings/storage", rbacHandler.PutStorage)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/settings/storage", rbacHandler.DeleteStorage)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/settings/storage/test", rbacHandler.TestStorage)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/feature-flags", workspaceHandler.CreateFeatureFlag)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/feature-flags", workspaceHandler.UpsertFeatureFlag)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags/{id}", workspaceHandler.GetFeatureFlagByID)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/feature-flags/{id}", workspaceHandler.UpdateFeatureFlag)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/feature-flags/{id}", workspaceHandler.DeleteFeatureFlag)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags/check/{key}", workspaceHandler.CheckFeatureFlag)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/configs", workspaceHandler.PutConfig)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/configs", workspaceHandler.ListConfigs)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/configs/{key}", workspaceHandler.GetConfig)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/configs/{key}", workspaceHandler.DeleteConfig)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}", workspaceHandler.Get)
	healthHandler := health.NewHandler(Version, nil, nil)
	healthHandler.SetLive(true)
	departmentRepo := repository.DepartmentRepository{TenantDB: workspaceRepo.TenantDB}
	departmentHandler := api.DepartmentHandler{Repository: departmentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/departments", departmentHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/departments", departmentHandler.Create)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/departments/{id}", departmentHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/departments/{id}", departmentHandler.Delete)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/departments/{id}/payroll-config", departmentHandler.GetPayrollConfig)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/departments/{id}/payroll-config", departmentHandler.SetPayrollConfig)
	locationRepo := repository.LocationRepository{TenantDB: workspaceRepo.TenantDB}
	locationHandler := api.LocationHandler{Repository: locationRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/locations", locationHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/locations", locationHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/locations/{id}", locationHandler.Get)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/locations/{id}", locationHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/locations/{id}", locationHandler.Delete)
	employerRepo := repository.EmployerRepository{TenantDB: workspaceRepo.TenantDB}
	employerHandler := api.EmployerHandler{Repository: employerRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employers", employerHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employers", employerHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employers/{id}", employerHandler.Get)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employers/{id}", employerHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employers/{id}", employerHandler.Delete)
	recruitmentRepo := repository.RecruitmentSourceRepository{TenantDB: workspaceRepo.TenantDB}
	recruitmentHandler := api.RecruitmentSourceHandler{Repository: recruitmentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/recruitment-sources", recruitmentHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/recruitment-sources", recruitmentHandler.Create)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/recruitment-sources/{id}", recruitmentHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/recruitment-sources/{id}", recruitmentHandler.Delete)
	mux.HandleFunc("GET /api/v1/dashboard/recruitment-sources", recruitmentHandler.Dashboard)
	employeeRepo := repository.EmployeeRepository{TenantDB: workspaceRepo.TenantDB}
	employeeHandler := api.EmployeeHandler{Repository: employeeRepo, Authorizer: workspaceAuthorizer}
	employeeImportRepo := repository.EmployeeImportRepository{TenantDB: workspaceRepo.TenantDB}
	employeeImportHandler := api.EmployeeImportHandler{
		Repository: employeeImportRepo,
		Queue: repository.EmployeeImportQueue{
			RedisClient: redisClient,
			Repo:        employeeImportRepo,
		},
		Authorizer: workspaceAuthorizer,
		StorageDir: cfg.StorageDir,
	}
	employeeConditionRepo := repository.EmployeeConditionRepository{TenantDB: workspaceRepo.TenantDB}
	employeeConditionHandler := api.EmployeeConditionHandler{Repository: employeeConditionRepo, Authorizer: workspaceAuthorizer}
	employeeDepartmentRepo := repository.EmployeeDepartmentRepository{TenantDB: workspaceRepo.TenantDB}
	employeeDepartmentHandler := api.EmployeeDepartmentHandler{Repository: employeeDepartmentRepo, Authorizer: workspaceAuthorizer}
	compensationHistoryRepo := repository.CompensationHistoryRepository{TenantDB: workspaceRepo.TenantDB}
	compensationHistoryHandler := api.CompensationHistoryHandler{Repository: compensationHistoryRepo, Authorizer: workspaceAuthorizer}
	attendanceRepo := repository.AttendanceRepository{TenantDB: workspaceRepo.TenantDB}
	attendanceHandler := api.AttendanceHandler{Repository: attendanceRepo, Authorizer: workspaceAuthorizer}
	attendanceLogsHandler := api.AttendanceLogsHandler{Repository: attendanceRepo, Authorizer: workspaceAuthorizer, StorageDir: cfg.StorageDir}
	attendancePolicyRepo := repository.AttendancePolicyRepository{TenantDB: workspaceRepo.TenantDB}
	attendancePolicyHandler := api.AttendancePolicyHandler{PolicyRepository: attendancePolicyRepo, BehalfRepository: attendancePolicyRepo, Authorizer: workspaceAuthorizer}
	overtimeRepo := repository.OvertimeRepository{TenantDB: workspaceRepo.TenantDB}
	overtimeHandler := api.OvertimeHandler{Repository: overtimeRepo, BehalfRepository: attendancePolicyRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/policies", overtimeHandler.ListPolicies)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/policies", overtimeHandler.CreatePolicy)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}", overtimeHandler.GetPolicy)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}/rules", overtimeHandler.AddRule)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/policies/assign", overtimeHandler.AssignPolicy)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests", overtimeHandler.ListRequests)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests", overtimeHandler.CreateRequest)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests/{id}", overtimeHandler.GetRequest)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests/{id}/submit", overtimeHandler.SubmitRequest)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}", overtimeHandler.ConsentRequest)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments", overtimeHandler.ListRequestBehalf)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments", overtimeHandler.CreateRequestBehalf)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/my-targets", overtimeHandler.RequestBehalfTargets)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/{id}", overtimeHandler.DeleteRequestBehalf)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments", overtimeHandler.ListClaimBehalf)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments", overtimeHandler.CreateClaimBehalf)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{id}", overtimeHandler.DeleteClaimBehalf)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/claim-ot/{employee_id}", overtimeHandler.ClaimOT)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/approve-ot/{employee_id}", overtimeHandler.ApproveOT)
	shiftRepo := repository.ShiftRepository{TenantDB: workspaceRepo.TenantDB}
	shiftHandler := api.ShiftHandler{Repository: shiftRepo, Authorizer: workspaceAuthorizer}
	employeeAssignmentRepo := repository.EmployeeAssignmentRepository{TenantDB: workspaceRepo.TenantDB}
	assignmentRuleRepo := repository.AssignmentRuleRepository{TenantDB: workspaceRepo.TenantDB}
	assignmentHistoryRepo := repository.AssignmentHistoryRepository{TenantDB: workspaceRepo.TenantDB}
	assignmentHandler := api.AssignmentHandler{Assignments: employeeAssignmentRepo, Rules: assignmentRuleRepo, History: assignmentHistoryRepo, Employees: employeeRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/assignment-history", assignmentHandler.AssignmentHistory)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/assignment-rules", assignmentHandler.ListAssignmentRules)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/assignment-rules", assignmentHandler.CreateAssignmentRule)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", assignmentHandler.UpdateAssignmentRule)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", assignmentHandler.DeleteAssignmentRule)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/time/punch-in", attendanceHandler.PunchIn)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/time/punch-out/{employee_id}", attendanceHandler.PunchOut)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/time/history/{employee_id}", attendanceHandler.TimeHistory)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/adjustments", attendanceHandler.CreateAdjustment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/adjustments", attendanceHandler.ListAdjustments)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", attendanceHandler.ApproveAdjustment)
	mux.HandleFunc("GET /api/v1/employees/{id}/attendance/adjustments", attendanceHandler.ListEmployeeAdjustments)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/summaries", attendanceHandler.Summaries)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/reprocess-all", attendanceHandler.ReprocessAll)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/reprocess/{employee_id}", attendanceHandler.ReprocessEmployee)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", attendanceHandler.ApproveAdjustment)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/time/entries/{id}", attendanceHandler.UpdateTimeEntry)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/time/entries/{id}", attendanceHandler.DeleteTimeEntry)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/upload", attendanceLogsHandler.Upload)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/logs/upload", attendanceLogsHandler.UploadLogs)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/bulk-upload-paper-ot", attendanceLogsHandler.BulkPaperOT)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/bulk", attendanceLogsHandler.BulkPaperOT)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/enter-paper-ot/{employee_id}", attendanceLogsHandler.EnterPaperOT)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/{employee_id}", attendanceLogsHandler.EnterPaperOT)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/latest", attendanceLogsHandler.Latest)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}", attendanceLogsHandler.Job)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}/error-file", attendanceLogsHandler.ErrorFile)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/logs", attendanceLogsHandler.Logs)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/raw-logs", attendanceLogsHandler.RawLogs)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/staging-logs", attendanceLogsHandler.Staging)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}/resolve", attendanceLogsHandler.Resolve)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}", attendanceLogsHandler.Dismiss)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/rules", attendancePolicyHandler.GetRules)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/attendance/rules", attendancePolicyHandler.UpdateRules)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/rules/versions", attendancePolicyHandler.ListRuleVersions)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/policies/assignments", attendancePolicyHandler.ListAssignments)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/policies/assignments", attendancePolicyHandler.UpsertAssignment)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}", attendancePolicyHandler.DeleteAssignment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments", attendancePolicyHandler.ListBehalfAssignments)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments", attendancePolicyHandler.CreateBehalfAssignment)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/{id}", attendancePolicyHandler.DeleteBehalfAssignment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/my-targets", attendancePolicyHandler.MyBehalfTargets)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/shifts", shiftHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shifts", shiftHandler.List)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/shifts/{shift_id}", shiftHandler.Update)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shifts/{shift_id}/versions", shiftHandler.Versions)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/shifts/assign", shiftHandler.Assign)
	patternRepo := repository.WorkingPatternRepository{TenantDB: workspaceRepo.TenantDB}
	patternHandler := api.WorkingPatternHandler{Repository: patternRepo, Authorizer: workspaceAuthorizer, Compliance: patternRepo}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/working-patterns", patternHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/working-patterns", patternHandler.List)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Details)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Delete)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{employment_id}/working-patterns", patternHandler.Assign)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/working-patterns/bulk-assign", patternHandler.BulkAssign)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{employment_id}/working-patterns", patternHandler.History)
	rosterRepo := repository.RosterRepository{TenantDB: workspaceRepo.TenantDB}
	rosterHandler := api.RosterHandler{Repository: rosterRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/rosters", rosterHandler.ListEmployee)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rosters", rosterHandler.Upsert)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rosters/bulk", rosterHandler.BulkUpsert)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}/rosters/{date}", rosterHandler.Delete)
	scheduleRepo := repository.ScheduleRepository{TenantDB: workspaceRepo.TenantDB}
	scheduleHandler := api.ScheduleHandler{Repository: scheduleRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/schedules", scheduleHandler.List)
	swapRepo := repository.ShiftSwapRepository{TenantDB: workspaceRepo.TenantDB}
	swapHandler := api.ShiftSwapHandler{Repository: swapRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/shift-swaps", swapHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shift-swaps", swapHandler.List)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/shift-swaps/{id}/approve", swapHandler.Approve)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shift-swaps/stats", swapHandler.Stats)
	segmentRepo := repository.WorkSegmentRepository{TenantDB: workspaceRepo.TenantDB}
	segmentHandler := api.WorkSegmentHandler{Repository: segmentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/work-segments", segmentHandler.List)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/work-segments/{segment_id}", segmentHandler.Get)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/work-segments", segmentHandler.Create)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/work-segments/{segment_id}", segmentHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/work-segments/{segment_id}", segmentHandler.Delete)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/work-segments/bulk", segmentHandler.BulkCreate)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees", employeeHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees", employeeHandler.List)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Get)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Delete)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rehire", employeeHandler.Rehire)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/upload", employeeImportHandler.Upload)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/import-termination/preview", employeeImportHandler.PreviewTermination)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/import-termination/confirm", employeeImportHandler.ConfirmTermination)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/import-termination/template", employeeImportHandler.TerminationTemplate)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/conditions", employeeConditionHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/conditions", employeeConditionHandler.Create)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}/conditions/{cond_id}", employeeConditionHandler.Delete)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/departments", employeeDepartmentHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/departments", employeeDepartmentHandler.Transfer)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/compensation-history", compensationHistoryHandler.Get)
	customFieldRepo := repository.CustomFieldRepository{TenantDB: workspaceRepo.TenantDB}
	customFieldHandler := api.CustomFieldHandler{Repository: customFieldRepo, Authorizer: workspaceAuthorizer}
	contractRepo := repository.ContractRepository{TenantDB: workspaceRepo.TenantDB}
	employmentRepo := repository.EmploymentRepository{TenantDB: workspaceRepo.TenantDB}
	contractEmploymentHandler := api.ContractEmploymentHandler{Contracts: contractRepo, Employments: employmentRepo, CustomFields: customFieldRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/contracts", contractEmploymentHandler.ListWorkspaceContracts)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts", contractEmploymentHandler.ListEmployeeContracts)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts", contractEmploymentHandler.CreateContract)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.GetContract)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.UpdateContract)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.DeleteContract)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments", contractEmploymentHandler.ListEmployments)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments", contractEmploymentHandler.CreateEmployment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.GetEmployment)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.UpdateEmployment)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.DeleteEmployment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments", assignmentHandler.ListEmployeeAssignments)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments", assignmentHandler.CreateEmployeeAssignment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.GetEmployeeAssignment)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.UpdateEmployeeAssignment)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.DeleteEmployeeAssignment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/definitions", customFieldHandler.ListDefinitions)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/custom-fields/definitions", customFieldHandler.CreateDefinition)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.GetDefinition)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.UpdateDefinition)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.DeleteDefinition)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.GetValues)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.SetValues)
	leaveRepo := repository.LeaveRepository{TenantDB: workspaceRepo.TenantDB}
	leaveHandler := api.LeaveHandler{
		Repository:       leaveRepo,
		BehalfRepository: attendancePolicyRepo,
		Authorizer:       workspaceAuthorizer,
		StorageDir:       cfg.StorageDir,
	}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/types", leaveHandler.ListTypes)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/types", leaveHandler.CreateType)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/leave/types/{id}/config", leaveHandler.UpdateTypeConfig)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/types/{id}", leaveHandler.DeleteType)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/types/{id}/restore", leaveHandler.RestoreType)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/submit", leaveHandler.Submit)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/requests", leaveHandler.ListRequests)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/requests/{id}/approve", leaveHandler.Approve)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/leave/balances", leaveHandler.Balances)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/balances", leaveHandler.WorkspaceBalances)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/leave/history", leaveHandler.EmployeeHistory)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/leave/adjust", leaveHandler.AdjustBalance)
	mux.HandleFunc("GET /api/v1/employees/{id}/leave/requests", leaveHandler.MyRequests)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/policies", leaveHandler.ListPolicies)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/policies", leaveHandler.CreatePolicy)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/leave/policies/{policy_id}", leaveHandler.UpdatePolicy)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/policies/{policy_id}", leaveHandler.DeletePolicy)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/policies/assign", leaveHandler.AssignPolicy)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/policies/assignments", leaveHandler.ListPolicyAssignments)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/policies/assignments/{assignment_id}", leaveHandler.DeletePolicyAssignment)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/accrual/run", leaveHandler.TriggerAccrual)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/jobs/{job_id}/status", leaveHandler.JobStatus)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/upload-excel", leaveHandler.UploadExcel)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/import/latest", leaveHandler.LatestImport)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/import/{job_id}/errors", leaveHandler.ImportErrors)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/upload-template", leaveHandler.UploadTemplate)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/behalf-assignments", leaveHandler.ListBehalfAssignments)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/behalf-assignments", leaveHandler.CreateBehalfAssignment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/behalf-assignments/my-targets", leaveHandler.GetMyBehalfTargets)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/behalf-assignments/{id}", leaveHandler.DeleteBehalfAssignment)
	holidayRepo := repository.HolidayRepository{TenantDB: workspaceRepo.TenantDB}
	holidayHandler := api.HolidayHandler{Repository: holidayRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/holidays", holidayHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/holidays", holidayHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Get)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Update)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Delete)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/holidays/{id}/process-leave", holidayHandler.ProcessLeave)
	payrollRepo := repository.PayrollRepository{TenantDB: workspaceRepo.TenantDB, SystemDB: workspaceRepo.SystemDB}
	essRepo := repository.ESSRepository{TenantDB: workspaceRepo.TenantDB}
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
	workflowRepo := repository.ApprovalWorkflowRepository{TenantDB: workspaceRepo.TenantDB}
	workflowHandler := api.ApprovalWorkflowHandler{Repository: workflowRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows", workflowHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/approval-workflows", workflowHandler.Create)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/approval-workflows/approve", workflowHandler.Process)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows/history/{type}/{id}", workflowHandler.History)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows/{id}", workflowHandler.Get)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/approval-workflows/{id}", workflowHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/approval-workflows/{id}", workflowHandler.Delete)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows/pending-tasks", workflowHandler.PendingTasks)
	notifRepo := repository.NotificationRepository{SystemDB: workspaceRepo.SystemDB}
	notifHandler := api.NotificationHandler{Repository: notifRepo}
	mux.HandleFunc("GET /v1/notifications", notifHandler.List)
	mux.HandleFunc("GET /v1/notifications/unread-count", notifHandler.UnreadCount)
	mux.HandleFunc("PATCH /v1/notifications/{id}/read", notifHandler.MarkRead)
	mux.HandleFunc("POST /v1/notifications/read-all", notifHandler.MarkAllRead)
	mux.HandleFunc("GET /api/v1/notifications", notifHandler.List)
	mux.HandleFunc("GET /api/v1/notifications/unread-count", notifHandler.UnreadCount)
	mux.HandleFunc("PATCH /api/v1/notifications/{id}/read", notifHandler.MarkRead)
	mux.HandleFunc("POST /api/v1/notifications/read-all", notifHandler.MarkAllRead)
	payrollHandler := api.PayrollHandler{Repository: payrollRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll/payslips", payrollHandler.ListWorkspace)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/payslips", payrollHandler.ListEmployee)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/salary-components", payrollHandler.ListSalaryComponents)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/salary-components", payrollHandler.CreateSalaryComponent)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{id}/salary-components", payrollHandler.ListEmploymentSalaryComponents)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{id}/salary-components", payrollHandler.AssignEmploymentSalaryComponent)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/salary-bases", payrollHandler.ListSalaryBases)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/salary-bases", payrollHandler.CreateSalaryBase)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members", payrollHandler.AssignBaseMember)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}", payrollHandler.UnassignBaseMember)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-settings", payrollHandler.GetPayrollSettings)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/payroll-settings", payrollHandler.UpdatePayrollSettings)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-settings/history", payrollHandler.GetPayrollSettingsHistory)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/payroll-config", payrollHandler.GetEmployeePayrollConfig)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/payroll-config", payrollHandler.SetEmployeePayrollConfig)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll/incentive-policies", payrollHandler.GetIncentivePolicies)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll/incentive-policies", payrollHandler.CreateIncentivePolicy)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}", payrollHandler.DeleteIncentivePolicy)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone", payrollHandler.CloneIncentivePolicy)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-rules", payrollHandler.ListPayrollRules)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll-rules", payrollHandler.CreatePayrollRule)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers", payrollHandler.ListPayrollRuleTiers)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers", payrollHandler.CreatePayrollRuleTier)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers/{tier_id}", payrollHandler.DeletePayrollRuleTier)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments", payrollHandler.ListPayrollRuleAssignments)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments", payrollHandler.AssignPayrollRule)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}", payrollHandler.DeletePayrollRuleAssignment)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/calculate-payroll", payrollHandler.CalculatePayroll)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll/calculate-batch", payrollHandler.CalculateBatchPayroll)
	reportRepo := repository.ReportRepository{TenantDB: workspaceRepo.TenantDB}
	reportQueue := &repository.RedisReportQueue{Client: redisClient, TenantDB: workspaceRepo.TenantDB}
	reportHandler := api.ReportHandler{Repository: reportRepo, Queue: reportQueue, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/attendance", reportHandler.AttendanceReport)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/no-show", reportHandler.NoShowReport)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/reports/export", reportHandler.CreateExport)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/jobs/{job_id}", reportHandler.Status)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/reports/jobs/{job_id}", reportHandler.Cancel)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/jobs/{job_id}/download", reportHandler.Download)
	fileRepo := repository.FileRepository{TenantDB: workspaceRepo.TenantDB}
	var fileStorage api.FileStorage
	if s3Client, err := external.NewS3Client(cfg.S3EndpointURL, cfg.S3BucketName, cfg.S3AccessKey.Expose(), cfg.S3SecretKey.Expose(), cfg.S3Region, cfg.S3UsePathStyle, nil); err == nil {
		fileStorage = s3Client
	}
	fileHandler := api.FileHandler{Repository: fileRepo, Storage: fileStorage, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/files/upload", fileHandler.UploadFile)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/files/upload", fileHandler.UploadFile)
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("PUT /v1/workspaces/{ws_id}/files/{id}", fileHandler.UpdateFile)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.UpdateFile)
	mux.HandleFunc("DELETE /v1/workspaces/{ws_id}/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}/download", fileHandler.DownloadFile)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}/download", fileHandler.DownloadFile)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}/versions", fileHandler.ListVersions)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}/versions", fileHandler.ListVersions)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}", fileHandler.ListByOwner)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}", fileHandler.ListByOwner)

	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("GET /v1/ess/workspaces/{ws_id}/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("GET /api/v1/ess/workspaces/{ws_id}/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("POST /v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign", fileHandler.SignDocument)
	mux.HandleFunc("POST /api/v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign", fileHandler.SignDocument)

	documentRepo := repository.DocumentRepository{TenantDB: workspaceRepo.TenantDB, SystemDB: workspaceRepo.SystemDB}
	documentHandler := api.DocumentHandler{Repository: documentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/documents/folders", documentHandler.ListFolders)
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/folders", documentHandler.CreateFolder)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.GetFolder)
	mux.HandleFunc("PUT /v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.RenameFolder)
	mux.HandleFunc("DELETE /v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.DeleteFolder)
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/distributions", documentHandler.Distribute)
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/templates/preview", documentHandler.PreviewTemplate)
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/templates/{id}/generate", documentHandler.GenerateTemplate)

	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/documents/folders", documentHandler.ListFolders)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/folders", documentHandler.CreateFolder)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.GetFolder)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.RenameFolder)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.DeleteFolder)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/distributions", documentHandler.Distribute)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/templates/preview", documentHandler.PreviewTemplate)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/templates/{id}/generate", documentHandler.GenerateTemplate)
	conditionRepo := repository.ConditionTypeRepository{TenantDB: workspaceRepo.TenantDB}
	conditionHandler := api.ConditionTypeHandler{Repository: conditionRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/condition-types", conditionHandler.List)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Get)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/condition-types", conditionHandler.Create)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Delete)
	phaseRepo := repository.EmploymentPhaseRepository{TenantDB: workspaceRepo.TenantDB}
	phaseHandler := api.EmploymentPhaseHandler{Repository: phaseRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employment-phases", phaseHandler.CreateDefinition)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employment-phases", phaseHandler.ListDefinitions)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{user_id}/phases", phaseHandler.AssignEmployee)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{user_id}/phases", phaseHandler.ListEmployee)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employees/{user_id}/phases/{phase_id}/status", phaseHandler.UpdateEmployeeStatus)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{employment_id}/phases", phaseHandler.AssignEmployment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{employment_id}/phases", phaseHandler.ListEmployment)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employments/{employment_id}/phases/{phase_id}/status", phaseHandler.UpdateEmploymentStatus)

	lifecycleRepo := repository.LifecycleRepository{TenantDB: workspaceRepo.TenantDB}
	lifecycleHandler := api.LifecycleHandler{
		Repository: lifecycleRepo,
		Authorizer: workspaceAuthorizer,
		UsersWithPermission: func(ctx context.Context, wsID uuid.UUID, permission string) []uuid.UUID {
			users, _ := workspaceAuthorizer.GetUsersWithPermission(ctx, wsID, permission)
			return users
		},
	}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle/my-tasks", lifecycleHandler.GetMyTasks)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/lifecycle-templates", lifecycleHandler.CreateTemplate)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates", lifecycleHandler.ListTemplates)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}", lifecycleHandler.GetTemplate)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}", lifecycleHandler.UpdateTemplate)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}", lifecycleHandler.DeleteTemplate)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}/items", lifecycleHandler.CreateTemplateItem)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}/items", lifecycleHandler.ListTemplateItems)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.GetTemplateItem)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.UpdateTemplateItem)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.DeleteTemplateItem)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{emp_id}/lifecycle", lifecycleHandler.CreateChecklist)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{emp_id}/lifecycle", lifecycleHandler.ListChecklists)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle/{id}", lifecycleHandler.GetChecklist)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/lifecycle/{id}/items/{iid}", lifecycleHandler.UpdateItemStatus)

	mux.HandleFunc("GET /api/v1/health", healthHandler.Liveness)
	mux.HandleFunc("GET /api/v1/health/ready", healthHandler.Readiness)
	systemHealthHandler := api.SystemHealthHandler{Roles: authRepo}
	mux.HandleFunc("GET /api/v1/system/health", systemHealthHandler.Get)

	systemAdminRepo := repository.SystemAdminRepository{DB: workspaceRepo.SystemDB}
	systemAdminHandler := api.SystemAdminHandler{
		RolesRepo:   systemAdminRepo,
		Settings:    systemAdminRepo,
		Roles:       authRepo,
		EmailSender: service.EmailService{Settings: systemAdminRepo},
		Redis:       redisClient,
	}
	mux.HandleFunc("GET /api/v1/system/smtp/config", systemAdminHandler.GetSMTPConfig)
	mux.HandleFunc("PUT /api/v1/system/smtp/config", systemAdminHandler.SetSMTPConfig)
	mux.HandleFunc("POST /api/v1/system/smtp/test", systemAdminHandler.TestSMTP)
	mux.HandleFunc("GET /api/v1/system/roles", systemAdminHandler.ListRoles)
	mux.HandleFunc("GET /api/v1/system/users/{user_id}/roles", systemAdminHandler.GetUserRoles)
	mux.HandleFunc("POST /api/v1/system/users/{user_id}/roles", systemAdminHandler.AssignRole)
	mux.HandleFunc("DELETE /api/v1/system/users/{user_id}/roles/{role_name}", systemAdminHandler.RevokeRole)

	monitorRepo := repository.MonitorRepository(workspaceRepo)
	monitorHandler := api.MonitorHandler{
		Repo:       monitorRepo,
		Authorizer: workspaceAuthorizer,
		RolesRepo:  authRepo,
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
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/audit-logs", monitorHandler.GetAuditLogs)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/jobs/{job_id}", monitorHandler.GetWorkspaceJobStatus)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/monitor/jobs/{job_id}", monitorHandler.GetWorkspaceMonitorJob)

	mux.HandleFunc("POST /api/v1/auth/login", authHandler.Login)
	mux.HandleFunc("POST /api/v1/auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /api/v1/auth/logout", authHandler.Logout)
	mux.HandleFunc("POST /api/v1/auth/forgot-password", authHandler.ForgotPassword)
	mux.HandleFunc("POST /api/v1/auth/reset-password", authHandler.ResetPassword)
	mux.HandleFunc("POST /api/v1/auth/change-password", authHandler.ChangePassword)
	mux.HandleFunc("GET /api/v1/me", authHandler.Me)
	mux.HandleFunc("PATCH /api/v1/me", authHandler.UpdateMe)
	mux.HandleFunc("GET /api/v1/me/permissions", authHandler.MyPermissions)
	mux.HandleFunc("GET /api/v1/users", authHandler.WorkspaceUsers)
	preferencesRepo := repository.DashboardPreferencesRepository{TenantDB: workspaceRepo.TenantDB}
	preferencesHandler := api.DashboardPreferencesHandler{Repository: preferencesRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/dashboard/preferences", preferencesHandler.Get)
	mux.HandleFunc("PUT /api/v1/dashboard/preferences", preferencesHandler.Put)
	dashboardAnalyticsRepo := repository.DashboardAnalyticsRepository{TenantDB: workspaceRepo.TenantDB}
	dashboardAnalyticsHandler := api.DashboardAnalyticsHandler{
		Repository: dashboardAnalyticsRepo,
		Authorizer: workspaceAuthorizer,
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
	mux.HandleFunc("POST /auth/login", authHandler.Login)
	mux.HandleFunc("POST /auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /auth/logout", authHandler.Logout)
	mux.HandleFunc("POST /auth/forgot-password", authHandler.ForgotPassword)
	mux.HandleFunc("POST /auth/reset-password", authHandler.ResetPassword)
	mux.HandleFunc("POST /auth/change-password", authHandler.ChangePassword)
	mux.HandleFunc("GET /me", authHandler.Me)
	mux.HandleFunc("PATCH /me", authHandler.UpdateMe)
	mux.HandleFunc("GET /me/permissions", authHandler.MyPermissions)
	mux.HandleFunc("GET /users", authHandler.WorkspaceUsers)

	publicPrefixes := []string{
		"/health",
		"/health/ready",
		"/api/v1/health",
		"/api/v1/health/ready",
		"/api/v1/auth/login",
		"/api/v1/auth/refresh",
		"/auth/login",
		"/auth/refresh",
		"/api/v1/auth/forgot-password",
		"/api/v1/auth/reset-password",
		"/auth/forgot-password",
		"/auth/reset-password",
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
		httpinfra.BodyLimitMiddleware(cfg.MaxRequestBodyBytes),
		httpinfra.AuthMiddlewareWithRevocation(cfg.JWTSecret.Expose(), publicPrefixes, revocationChecker),
	)(mux)
	handler = httpinfra.StandaloneWorkspaceMiddleware(cfg.StandaloneWorkspaceID)(handler)
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
