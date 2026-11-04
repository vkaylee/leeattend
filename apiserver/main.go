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
)

// Version is the current apiserver Go component version.
const Version = "0.1.0"

func singleWorkspaceHandler(db repository.DBTX, handler http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if db == nil {
			http.Error(w, "database unavailable", http.StatusServiceUnavailable)
			return
		}
		var organizationID uuid.UUID
		if err := db.QueryRow(r.Context(), `SELECT id FROM workspaces ORDER BY id LIMIT 1`).Scan(&organizationID); err != nil {
			http.Error(w, "organization unavailable", http.StatusServiceUnavailable)
			return
		}
		r = r.WithContext(httpinfra.WithWorkspaceID(r.Context(), organizationID))
		handler(w, r)
	}
}

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
		DB: func() repository.DBTX {
			pool, err := poolMgr.SystemPool()
			if err != nil {
				return nil
			}
			return pool
		}(),
	}
	workspaceAuthorizer := repository.WorkspaceAuthorizer{DB: workspaceRepo.SystemDB}
	workspaceHandler := api.WorkspaceHandler{Repository: workspaceRepo, Authorizer: workspaceAuthorizer}
	singleWorkspace := func(handler http.HandlerFunc) http.HandlerFunc {
		return singleWorkspaceHandler(workspaceRepo.SystemDB, handler)
	}
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
	mux.HandleFunc("PATCH /api/v1", singleWorkspace(workspaceHandler.Patch))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags", workspaceHandler.ListFeatureFlags)
	mux.HandleFunc("GET /api/v1/feature-flags", singleWorkspace(workspaceHandler.ListFeatureFlags))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/members", rbacHandler.ListMembers)
	mux.HandleFunc("GET /api/v1/members", singleWorkspace(rbacHandler.ListMembers))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/members", rbacHandler.AddMember)
	mux.HandleFunc("POST /api/v1/members", singleWorkspace(rbacHandler.AddMember))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/members/{user_id}", rbacHandler.UpdateMemberRole)
	mux.HandleFunc("PATCH /api/v1/members/{user_id}", singleWorkspace(rbacHandler.UpdateMemberRole))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/members/{user_id}", rbacHandler.RemoveMember)
	mux.HandleFunc("DELETE /api/v1/members/{user_id}", singleWorkspace(rbacHandler.RemoveMember))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/members/{user_id}/roles", rbacHandler.GetMemberRoles)
	mux.HandleFunc("GET /api/v1/members/{user_id}/roles", singleWorkspace(rbacHandler.GetMemberRoles))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/members/{user_id}/roles", rbacHandler.AssignExtraRole)
	mux.HandleFunc("POST /api/v1/members/{user_id}/roles", singleWorkspace(rbacHandler.AssignExtraRole))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/members/{user_id}/roles/{role_id}", rbacHandler.RevokeExtraRole)
	mux.HandleFunc("DELETE /api/v1/members/{user_id}/roles/{role_id}", singleWorkspace(rbacHandler.RevokeExtraRole))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/members/{user_id}/permissions", rbacHandler.GetMemberPermissions)
	mux.HandleFunc("GET /api/v1/members/{user_id}/permissions", singleWorkspace(rbacHandler.GetMemberPermissions))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/members/{user_id}/permissions", rbacHandler.AssignExtraPermission)
	mux.HandleFunc("POST /api/v1/members/{user_id}/permissions", singleWorkspace(rbacHandler.AssignExtraPermission))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/members/{user_id}/permissions/{perm_id}", rbacHandler.RevokeExtraPermission)
	mux.HandleFunc("DELETE /api/v1/members/{user_id}/permissions/{perm_id}", singleWorkspace(rbacHandler.RevokeExtraPermission))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/roles", rbacHandler.ListRoles)
	mux.HandleFunc("GET /api/v1/roles", singleWorkspace(rbacHandler.ListRoles))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/roles", rbacHandler.CreateRole)
	mux.HandleFunc("POST /api/v1/roles", singleWorkspace(rbacHandler.CreateRole))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/roles/{role_id}", rbacHandler.UpdateRole)
	mux.HandleFunc("PUT /api/v1/roles/{role_id}", singleWorkspace(rbacHandler.UpdateRole))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/roles/{role_id}", rbacHandler.DeleteRole)
	mux.HandleFunc("DELETE /api/v1/roles/{role_id}", singleWorkspace(rbacHandler.DeleteRole))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/roles/{role_id}/permissions", rbacHandler.GetRolePermissions)
	mux.HandleFunc("GET /api/v1/roles/{role_id}/permissions", singleWorkspace(rbacHandler.GetRolePermissions))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/roles/{role_id}/permissions", rbacHandler.SetRolePermissions)
	mux.HandleFunc("PUT /api/v1/roles/{role_id}/permissions", singleWorkspace(rbacHandler.SetRolePermissions))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/permissions/catalog", rbacHandler.PermissionCatalog)
	mux.HandleFunc("GET /api/v1/permissions/catalog", singleWorkspace(rbacHandler.PermissionCatalog))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/permissions/available", rbacHandler.AvailablePermissions)
	mux.HandleFunc("GET /api/v1/permissions/available", singleWorkspace(rbacHandler.AvailablePermissions))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/settings/storage", rbacHandler.GetStorage)
	mux.HandleFunc("GET /api/v1/settings/storage", singleWorkspace(rbacHandler.GetStorage))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/settings/storage", rbacHandler.PutStorage)
	mux.HandleFunc("PUT /api/v1/settings/storage", singleWorkspace(rbacHandler.PutStorage))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/settings/storage", rbacHandler.DeleteStorage)
	mux.HandleFunc("DELETE /api/v1/settings/storage", singleWorkspace(rbacHandler.DeleteStorage))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/settings/storage/test", rbacHandler.TestStorage)
	mux.HandleFunc("POST /api/v1/settings/storage/test", singleWorkspace(rbacHandler.TestStorage))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/feature-flags", workspaceHandler.CreateFeatureFlag)
	mux.HandleFunc("POST /api/v1/feature-flags", singleWorkspace(workspaceHandler.CreateFeatureFlag))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/feature-flags", workspaceHandler.UpsertFeatureFlag)
	mux.HandleFunc("PUT /api/v1/feature-flags", singleWorkspace(workspaceHandler.UpsertFeatureFlag))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags/{id}", workspaceHandler.GetFeatureFlagByID)
	mux.HandleFunc("GET /api/v1/feature-flags/{id}", singleWorkspace(workspaceHandler.GetFeatureFlagByID))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/feature-flags/{id}", workspaceHandler.UpdateFeatureFlag)
	mux.HandleFunc("PATCH /api/v1/feature-flags/{id}", singleWorkspace(workspaceHandler.UpdateFeatureFlag))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/feature-flags/{id}", workspaceHandler.DeleteFeatureFlag)
	mux.HandleFunc("DELETE /api/v1/feature-flags/{id}", singleWorkspace(workspaceHandler.DeleteFeatureFlag))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags/check/{key}", workspaceHandler.CheckFeatureFlag)
	mux.HandleFunc("GET /api/v1/feature-flags/check/{key}", singleWorkspace(workspaceHandler.CheckFeatureFlag))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/configs", workspaceHandler.PutConfig)
	mux.HandleFunc("PUT /api/v1/configs", singleWorkspace(workspaceHandler.PutConfig))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/configs", workspaceHandler.ListConfigs)
	mux.HandleFunc("GET /api/v1/configs", singleWorkspace(workspaceHandler.ListConfigs))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/configs/{key}", workspaceHandler.GetConfig)
	mux.HandleFunc("GET /api/v1/configs/{key}", singleWorkspace(workspaceHandler.GetConfig))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/configs/{key}", workspaceHandler.DeleteConfig)
	mux.HandleFunc("DELETE /api/v1/configs/{key}", singleWorkspace(workspaceHandler.DeleteConfig))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}", workspaceHandler.Get)
	mux.HandleFunc("GET /api/v1", singleWorkspace(workspaceHandler.Get))
	healthHandler := health.NewHandler(Version, nil, nil)
	healthHandler.SetLive(true)
	departmentRepo := repository.DepartmentRepository{DB: workspaceRepo.DB}
	departmentHandler := api.DepartmentHandler{Repository: departmentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/departments", departmentHandler.List)
	mux.HandleFunc("GET /api/v1/departments", singleWorkspace(departmentHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/departments", departmentHandler.Create)
	mux.HandleFunc("POST /api/v1/departments", singleWorkspace(departmentHandler.Create))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/departments/{id}", departmentHandler.Update)
	mux.HandleFunc("PATCH /api/v1/departments/{id}", singleWorkspace(departmentHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/departments/{id}", departmentHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/departments/{id}", singleWorkspace(departmentHandler.Delete))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/departments/{id}/payroll-config", departmentHandler.GetPayrollConfig)
	mux.HandleFunc("GET /api/v1/departments/{id}/payroll-config", singleWorkspace(departmentHandler.GetPayrollConfig))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/departments/{id}/payroll-config", departmentHandler.SetPayrollConfig)
	mux.HandleFunc("POST /api/v1/departments/{id}/payroll-config", singleWorkspace(departmentHandler.SetPayrollConfig))
	locationRepo := repository.LocationRepository{DB: workspaceRepo.DB}
	locationHandler := api.LocationHandler{Repository: locationRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/locations", locationHandler.List)
	mux.HandleFunc("GET /api/v1/locations", singleWorkspace(locationHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/locations", locationHandler.Create)
	mux.HandleFunc("POST /api/v1/locations", singleWorkspace(locationHandler.Create))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/locations/{id}", locationHandler.Get)
	mux.HandleFunc("GET /api/v1/locations/{id}", singleWorkspace(locationHandler.Get))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/locations/{id}", locationHandler.Update)
	mux.HandleFunc("PUT /api/v1/locations/{id}", singleWorkspace(locationHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/locations/{id}", locationHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/locations/{id}", singleWorkspace(locationHandler.Delete))
	employerRepo := repository.EmployerRepository{DB: workspaceRepo.DB}
	employerHandler := api.EmployerHandler{Repository: employerRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employers", employerHandler.List)
	mux.HandleFunc("GET /api/v1/employers", singleWorkspace(employerHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employers", employerHandler.Create)
	mux.HandleFunc("POST /api/v1/employers", singleWorkspace(employerHandler.Create))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employers/{id}", employerHandler.Get)
	mux.HandleFunc("GET /api/v1/employers/{id}", singleWorkspace(employerHandler.Get))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employers/{id}", employerHandler.Update)
	mux.HandleFunc("PUT /api/v1/employers/{id}", singleWorkspace(employerHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employers/{id}", employerHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/employers/{id}", singleWorkspace(employerHandler.Delete))
	recruitmentRepo := repository.RecruitmentSourceRepository{DB: workspaceRepo.DB}
	recruitmentHandler := api.RecruitmentSourceHandler{Repository: recruitmentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/recruitment-sources", recruitmentHandler.List)
	mux.HandleFunc("GET /api/v1/recruitment-sources", singleWorkspace(recruitmentHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/recruitment-sources", recruitmentHandler.Create)
	mux.HandleFunc("POST /api/v1/recruitment-sources", singleWorkspace(recruitmentHandler.Create))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/recruitment-sources/{id}", recruitmentHandler.Update)
	mux.HandleFunc("PUT /api/v1/recruitment-sources/{id}", singleWorkspace(recruitmentHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/recruitment-sources/{id}", recruitmentHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/recruitment-sources/{id}", singleWorkspace(recruitmentHandler.Delete))
	mux.HandleFunc("GET /api/v1/dashboard/recruitment-sources", singleWorkspace(recruitmentHandler.Dashboard))
	employeeRepo := repository.EmployeeRepository{DB: workspaceRepo.DB}
	employeeHandler := api.EmployeeHandler{Repository: employeeRepo, Authorizer: workspaceAuthorizer}
	employeeImportRepo := repository.EmployeeImportRepository{DB: workspaceRepo.DB}
	employeeImportHandler := api.EmployeeImportHandler{
		Repository: employeeImportRepo,
		Queue: repository.EmployeeImportQueue{
			RedisClient: redisClient,
			Repo:        employeeImportRepo,
		},
		Authorizer: workspaceAuthorizer,
		StorageDir: cfg.StorageDir,
	}
	employeeConditionRepo := repository.EmployeeConditionRepository{DB: workspaceRepo.DB}
	employeeConditionHandler := api.EmployeeConditionHandler{Repository: employeeConditionRepo, Authorizer: workspaceAuthorizer}
	employeeDepartmentRepo := repository.EmployeeDepartmentRepository{DB: workspaceRepo.DB}
	employeeDepartmentHandler := api.EmployeeDepartmentHandler{Repository: employeeDepartmentRepo, Authorizer: workspaceAuthorizer}
	compensationHistoryRepo := repository.CompensationHistoryRepository{DB: workspaceRepo.DB}
	compensationHistoryHandler := api.CompensationHistoryHandler{Repository: compensationHistoryRepo, Authorizer: workspaceAuthorizer}
	attendanceRepo := repository.AttendanceRepository{DB: workspaceRepo.DB}
	attendanceHandler := api.AttendanceHandler{Repository: attendanceRepo, Authorizer: workspaceAuthorizer}
	attendanceLogsHandler := api.AttendanceLogsHandler{Repository: attendanceRepo, Authorizer: workspaceAuthorizer, StorageDir: cfg.StorageDir}
	attendancePolicyRepo := repository.AttendancePolicyRepository{DB: workspaceRepo.DB, SystemDB: workspaceRepo.SystemDB}
	attendancePolicyHandler := api.AttendancePolicyHandler{PolicyRepository: attendancePolicyRepo, BehalfRepository: attendancePolicyRepo, Authorizer: workspaceAuthorizer}
	overtimeRepo := repository.OvertimeRepository{DB: workspaceRepo.DB}
	overtimeHandler := api.OvertimeHandler{Repository: overtimeRepo, BehalfRepository: attendancePolicyRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/policies", overtimeHandler.ListPolicies)
	mux.HandleFunc("GET /api/v1/overtime/policies", singleWorkspace(overtimeHandler.ListPolicies))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/policies", overtimeHandler.CreatePolicy)
	mux.HandleFunc("POST /api/v1/overtime/policies", singleWorkspace(overtimeHandler.CreatePolicy))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}", overtimeHandler.GetPolicy)
	mux.HandleFunc("GET /api/v1/overtime/policies/{policy_id}", singleWorkspace(overtimeHandler.GetPolicy))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/policies/{policy_id}/rules", overtimeHandler.AddRule)
	mux.HandleFunc("POST /api/v1/overtime/policies/{policy_id}/rules", singleWorkspace(overtimeHandler.AddRule))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/policies/assign", overtimeHandler.AssignPolicy)
	mux.HandleFunc("POST /api/v1/overtime/policies/assign", singleWorkspace(overtimeHandler.AssignPolicy))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests", overtimeHandler.ListRequests)
	mux.HandleFunc("GET /api/v1/overtime/requests", singleWorkspace(overtimeHandler.ListRequests))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests", overtimeHandler.CreateRequest)
	mux.HandleFunc("POST /api/v1/overtime/requests", singleWorkspace(overtimeHandler.CreateRequest))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests/{id}", overtimeHandler.GetRequest)
	mux.HandleFunc("GET /api/v1/overtime/requests/{id}", singleWorkspace(overtimeHandler.GetRequest))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests/{id}/submit", overtimeHandler.SubmitRequest)
	mux.HandleFunc("POST /api/v1/overtime/requests/{id}/submit", singleWorkspace(overtimeHandler.SubmitRequest))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests/{id}/consent/{emp_id}", overtimeHandler.ConsentRequest)
	mux.HandleFunc("POST /api/v1/overtime/requests/{id}/consent/{emp_id}", singleWorkspace(overtimeHandler.ConsentRequest))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments", overtimeHandler.ListRequestBehalf)
	mux.HandleFunc("GET /api/v1/overtime/requests/behalf-assignments", singleWorkspace(overtimeHandler.ListRequestBehalf))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments", overtimeHandler.CreateRequestBehalf)
	mux.HandleFunc("POST /api/v1/overtime/requests/behalf-assignments", singleWorkspace(overtimeHandler.CreateRequestBehalf))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/my-targets", overtimeHandler.RequestBehalfTargets)
	mux.HandleFunc("GET /api/v1/overtime/requests/behalf-assignments/my-targets", singleWorkspace(overtimeHandler.RequestBehalfTargets))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/overtime/requests/behalf-assignments/{id}", overtimeHandler.DeleteRequestBehalf)
	mux.HandleFunc("DELETE /api/v1/overtime/requests/behalf-assignments/{id}", singleWorkspace(overtimeHandler.DeleteRequestBehalf))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments", overtimeHandler.ListClaimBehalf)
	mux.HandleFunc("GET /api/v1/attendance/claim-ot/behalf-assignments", singleWorkspace(overtimeHandler.ListClaimBehalf))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments", overtimeHandler.CreateClaimBehalf)
	mux.HandleFunc("POST /api/v1/attendance/claim-ot/behalf-assignments", singleWorkspace(overtimeHandler.CreateClaimBehalf))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/claim-ot/behalf-assignments/{id}", overtimeHandler.DeleteClaimBehalf)
	mux.HandleFunc("DELETE /api/v1/attendance/claim-ot/behalf-assignments/{id}", singleWorkspace(overtimeHandler.DeleteClaimBehalf))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/claim-ot/{employee_id}", overtimeHandler.ClaimOT)
	mux.HandleFunc("POST /api/v1/attendance/claim-ot/{employee_id}", singleWorkspace(overtimeHandler.ClaimOT))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/approve-ot/{employee_id}", overtimeHandler.ApproveOT)
	mux.HandleFunc("POST /api/v1/attendance/approve-ot/{employee_id}", singleWorkspace(overtimeHandler.ApproveOT))
	shiftRepo := repository.ShiftRepository{DB: workspaceRepo.DB}
	shiftHandler := api.ShiftHandler{Repository: shiftRepo, Authorizer: workspaceAuthorizer}
	employeeAssignmentRepo := repository.EmployeeAssignmentRepository{DB: workspaceRepo.DB}
	assignmentRuleRepo := repository.AssignmentRuleRepository{DB: workspaceRepo.DB}
	assignmentHistoryRepo := repository.AssignmentHistoryRepository{DB: workspaceRepo.DB}
	assignmentHandler := api.AssignmentHandler{Assignments: employeeAssignmentRepo, Rules: assignmentRuleRepo, History: assignmentHistoryRepo, Employees: employeeRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/assignment-history", assignmentHandler.AssignmentHistory)
	mux.HandleFunc("GET /api/v1/assignment-history", singleWorkspace(assignmentHandler.AssignmentHistory))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/assignment-rules", assignmentHandler.ListAssignmentRules)
	mux.HandleFunc("GET /api/v1/assignment-rules", singleWorkspace(assignmentHandler.ListAssignmentRules))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/assignment-rules", assignmentHandler.CreateAssignmentRule)
	mux.HandleFunc("POST /api/v1/assignment-rules", singleWorkspace(assignmentHandler.CreateAssignmentRule))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", assignmentHandler.UpdateAssignmentRule)
	mux.HandleFunc("PATCH /api/v1/assignment-rules/{rule_id}", singleWorkspace(assignmentHandler.UpdateAssignmentRule))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/assignment-rules/{rule_id}", assignmentHandler.DeleteAssignmentRule)
	mux.HandleFunc("DELETE /api/v1/assignment-rules/{rule_id}", singleWorkspace(assignmentHandler.DeleteAssignmentRule))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/time/punch-in", attendanceHandler.PunchIn)
	mux.HandleFunc("POST /api/v1/time/punch-in", singleWorkspace(attendanceHandler.PunchIn))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/time/punch-out/{employee_id}", attendanceHandler.PunchOut)
	mux.HandleFunc("POST /api/v1/time/punch-out/{employee_id}", singleWorkspace(attendanceHandler.PunchOut))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/time/history/{employee_id}", attendanceHandler.TimeHistory)
	mux.HandleFunc("GET /api/v1/time/history/{employee_id}", singleWorkspace(attendanceHandler.TimeHistory))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/adjustments", attendanceHandler.CreateAdjustment)
	mux.HandleFunc("POST /api/v1/attendance/adjustments", singleWorkspace(attendanceHandler.CreateAdjustment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/adjustments", attendanceHandler.ListAdjustments)
	mux.HandleFunc("GET /api/v1/attendance/adjustments", singleWorkspace(attendanceHandler.ListAdjustments))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", attendanceHandler.ApproveAdjustment)
	mux.HandleFunc("PUT /api/v1/attendance/adjustments/{id}/approve", singleWorkspace(attendanceHandler.ApproveAdjustment))
	mux.HandleFunc("GET /api/v1/employees/{id}/attendance/adjustments", attendanceHandler.ListEmployeeAdjustments)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/summaries", attendanceHandler.Summaries)
	mux.HandleFunc("GET /api/v1/attendance/summaries", singleWorkspace(attendanceHandler.Summaries))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/reprocess-all", attendanceHandler.ReprocessAll)
	mux.HandleFunc("POST /api/v1/attendance/reprocess-all", singleWorkspace(attendanceHandler.ReprocessAll))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/reprocess/{employee_id}", attendanceHandler.ReprocessEmployee)
	mux.HandleFunc("POST /api/v1/attendance/reprocess/{employee_id}", singleWorkspace(attendanceHandler.ReprocessEmployee))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", attendanceHandler.ApproveAdjustment)
	mux.HandleFunc("POST /api/v1/attendance/adjustments/{id}/approve", singleWorkspace(attendanceHandler.ApproveAdjustment))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/time/entries/{id}", attendanceHandler.UpdateTimeEntry)
	mux.HandleFunc("PUT /api/v1/time/entries/{id}", singleWorkspace(attendanceHandler.UpdateTimeEntry))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/time/entries/{id}", attendanceHandler.DeleteTimeEntry)
	mux.HandleFunc("DELETE /api/v1/time/entries/{id}", singleWorkspace(attendanceHandler.DeleteTimeEntry))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/upload", attendanceLogsHandler.Upload)
	mux.HandleFunc("POST /api/v1/attendance/upload", singleWorkspace(attendanceLogsHandler.Upload))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/logs/upload", attendanceLogsHandler.UploadLogs)
	mux.HandleFunc("POST /api/v1/attendance/logs/upload", singleWorkspace(attendanceLogsHandler.UploadLogs))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/bulk-upload-paper-ot", attendanceLogsHandler.BulkPaperOT)
	mux.HandleFunc("POST /api/v1/attendance/bulk-upload-paper-ot", singleWorkspace(attendanceLogsHandler.BulkPaperOT))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/bulk", attendanceLogsHandler.BulkPaperOT)
	mux.HandleFunc("POST /api/v1/attendance/paper-forms/overtime/bulk", singleWorkspace(attendanceLogsHandler.BulkPaperOT))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/enter-paper-ot/{employee_id}", attendanceLogsHandler.EnterPaperOT)
	mux.HandleFunc("POST /api/v1/attendance/enter-paper-ot/{employee_id}", singleWorkspace(attendanceLogsHandler.EnterPaperOT))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/paper-forms/overtime/{employee_id}", attendanceLogsHandler.EnterPaperOT)
	mux.HandleFunc("POST /api/v1/attendance/paper-forms/overtime/{employee_id}", singleWorkspace(attendanceLogsHandler.EnterPaperOT))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/latest", attendanceLogsHandler.Latest)
	mux.HandleFunc("GET /api/v1/attendance/import-jobs/latest", singleWorkspace(attendanceLogsHandler.Latest))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}", attendanceLogsHandler.Job)
	mux.HandleFunc("GET /api/v1/attendance/import-jobs/{id}", singleWorkspace(attendanceLogsHandler.Job))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/import-jobs/{id}/error-file", attendanceLogsHandler.ErrorFile)
	mux.HandleFunc("GET /api/v1/attendance/import-jobs/{id}/error-file", singleWorkspace(attendanceLogsHandler.ErrorFile))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/logs", attendanceLogsHandler.Logs)
	mux.HandleFunc("GET /api/v1/attendance/logs", singleWorkspace(attendanceLogsHandler.Logs))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/raw-logs", attendanceLogsHandler.RawLogs)
	mux.HandleFunc("GET /api/v1/attendance/raw-logs", singleWorkspace(attendanceLogsHandler.RawLogs))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/staging-logs", attendanceLogsHandler.Staging)
	mux.HandleFunc("GET /api/v1/attendance/staging-logs", singleWorkspace(attendanceLogsHandler.Staging))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}/resolve", attendanceLogsHandler.Resolve)
	mux.HandleFunc("POST /api/v1/attendance/staging-logs/{id}/resolve", singleWorkspace(attendanceLogsHandler.Resolve))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/staging-logs/{id}", attendanceLogsHandler.Dismiss)
	mux.HandleFunc("DELETE /api/v1/attendance/staging-logs/{id}", singleWorkspace(attendanceLogsHandler.Dismiss))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/rules", attendancePolicyHandler.GetRules)
	mux.HandleFunc("GET /api/v1/attendance/rules", singleWorkspace(attendancePolicyHandler.GetRules))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/attendance/rules", attendancePolicyHandler.UpdateRules)
	mux.HandleFunc("PATCH /api/v1/attendance/rules", singleWorkspace(attendancePolicyHandler.UpdateRules))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/rules/versions", attendancePolicyHandler.ListRuleVersions)
	mux.HandleFunc("GET /api/v1/attendance/rules/versions", singleWorkspace(attendancePolicyHandler.ListRuleVersions))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/policies/assignments", attendancePolicyHandler.ListAssignments)
	mux.HandleFunc("GET /api/v1/attendance/policies/assignments", singleWorkspace(attendancePolicyHandler.ListAssignments))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/policies/assignments", attendancePolicyHandler.UpsertAssignment)
	mux.HandleFunc("POST /api/v1/attendance/policies/assignments", singleWorkspace(attendancePolicyHandler.UpsertAssignment))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/policies/assignments/{assignment_id}", attendancePolicyHandler.DeleteAssignment)
	mux.HandleFunc("DELETE /api/v1/attendance/policies/assignments/{assignment_id}", singleWorkspace(attendancePolicyHandler.DeleteAssignment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments", attendancePolicyHandler.ListBehalfAssignments)
	mux.HandleFunc("GET /api/v1/attendance/adjustments/behalf-assignments", singleWorkspace(attendancePolicyHandler.ListBehalfAssignments))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments", attendancePolicyHandler.CreateBehalfAssignment)
	mux.HandleFunc("POST /api/v1/attendance/adjustments/behalf-assignments", singleWorkspace(attendancePolicyHandler.CreateBehalfAssignment))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/{id}", attendancePolicyHandler.DeleteBehalfAssignment)
	mux.HandleFunc("DELETE /api/v1/attendance/adjustments/behalf-assignments/{id}", singleWorkspace(attendancePolicyHandler.DeleteBehalfAssignment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/adjustments/behalf-assignments/my-targets", attendancePolicyHandler.MyBehalfTargets)
	mux.HandleFunc("GET /api/v1/attendance/adjustments/behalf-assignments/my-targets", singleWorkspace(attendancePolicyHandler.MyBehalfTargets))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/shifts", shiftHandler.Create)
	mux.HandleFunc("POST /api/v1/shifts", singleWorkspace(shiftHandler.Create))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shifts", shiftHandler.List)
	mux.HandleFunc("GET /api/v1/shifts", singleWorkspace(shiftHandler.List))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/shifts/{shift_id}", shiftHandler.Update)
	mux.HandleFunc("PUT /api/v1/shifts/{shift_id}", singleWorkspace(shiftHandler.Update))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shifts/{shift_id}/versions", shiftHandler.Versions)
	mux.HandleFunc("GET /api/v1/shifts/{shift_id}/versions", singleWorkspace(shiftHandler.Versions))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/shifts/assign", shiftHandler.Assign)
	mux.HandleFunc("POST /api/v1/shifts/assign", singleWorkspace(shiftHandler.Assign))
	patternRepo := repository.WorkingPatternRepository{DB: workspaceRepo.DB}
	patternHandler := api.WorkingPatternHandler{Repository: patternRepo, Authorizer: workspaceAuthorizer, Compliance: patternRepo}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/working-patterns", patternHandler.Create)
	mux.HandleFunc("POST /api/v1/working-patterns", singleWorkspace(patternHandler.Create))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/working-patterns", patternHandler.List)
	mux.HandleFunc("GET /api/v1/working-patterns", singleWorkspace(patternHandler.List))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Details)
	mux.HandleFunc("GET /api/v1/working-patterns/{pattern_id}", singleWorkspace(patternHandler.Details))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Update)
	mux.HandleFunc("PUT /api/v1/working-patterns/{pattern_id}", singleWorkspace(patternHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/working-patterns/{pattern_id}", singleWorkspace(patternHandler.Delete))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{employment_id}/working-patterns", patternHandler.Assign)
	mux.HandleFunc("POST /api/v1/employments/{employment_id}/working-patterns", singleWorkspace(patternHandler.Assign))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/working-patterns/bulk-assign", patternHandler.BulkAssign)
	mux.HandleFunc("POST /api/v1/working-patterns/bulk-assign", singleWorkspace(patternHandler.BulkAssign))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{employment_id}/working-patterns", patternHandler.History)
	mux.HandleFunc("GET /api/v1/employments/{employment_id}/working-patterns", singleWorkspace(patternHandler.History))
	rosterRepo := repository.RosterRepository{DB: workspaceRepo.DB}
	rosterHandler := api.RosterHandler{Repository: rosterRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/rosters", rosterHandler.ListEmployee)
	mux.HandleFunc("GET /api/v1/employees/{id}/rosters", singleWorkspace(rosterHandler.ListEmployee))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rosters", rosterHandler.Upsert)
	mux.HandleFunc("POST /api/v1/employees/{id}/rosters", singleWorkspace(rosterHandler.Upsert))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rosters/bulk", rosterHandler.BulkUpsert)
	mux.HandleFunc("POST /api/v1/employees/{id}/rosters/bulk", singleWorkspace(rosterHandler.BulkUpsert))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}/rosters/{date}", rosterHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/employees/{id}/rosters/{date}", singleWorkspace(rosterHandler.Delete))
	scheduleRepo := repository.ScheduleRepository{DB: workspaceRepo.DB}
	scheduleHandler := api.ScheduleHandler{Repository: scheduleRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/schedules", scheduleHandler.List)
	mux.HandleFunc("GET /api/v1/schedules", singleWorkspace(scheduleHandler.List))
	swapRepo := repository.ShiftSwapRepository{DB: workspaceRepo.DB}
	swapHandler := api.ShiftSwapHandler{Repository: swapRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/shift-swaps", swapHandler.Create)
	mux.HandleFunc("POST /api/v1/shift-swaps", singleWorkspace(swapHandler.Create))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shift-swaps", swapHandler.List)
	mux.HandleFunc("GET /api/v1/shift-swaps", singleWorkspace(swapHandler.List))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/shift-swaps/{id}/approve", swapHandler.Approve)
	mux.HandleFunc("PUT /api/v1/shift-swaps/{id}/approve", singleWorkspace(swapHandler.Approve))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/shift-swaps/stats", swapHandler.Stats)
	mux.HandleFunc("GET /api/v1/shift-swaps/stats", singleWorkspace(swapHandler.Stats))
	segmentRepo := repository.WorkSegmentRepository{DB: workspaceRepo.DB}
	segmentHandler := api.WorkSegmentHandler{Repository: segmentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/work-segments", segmentHandler.List)
	mux.HandleFunc("GET /api/v1/work-segments", singleWorkspace(segmentHandler.List))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/work-segments/{segment_id}", segmentHandler.Get)
	mux.HandleFunc("GET /api/v1/work-segments/{segment_id}", singleWorkspace(segmentHandler.Get))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/work-segments", segmentHandler.Create)
	mux.HandleFunc("POST /api/v1/work-segments", singleWorkspace(segmentHandler.Create))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/work-segments/{segment_id}", segmentHandler.Update)
	mux.HandleFunc("PUT /api/v1/work-segments/{segment_id}", singleWorkspace(segmentHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/work-segments/{segment_id}", segmentHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/work-segments/{segment_id}", singleWorkspace(segmentHandler.Delete))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/work-segments/bulk", segmentHandler.BulkCreate)
	mux.HandleFunc("POST /api/v1/work-segments/bulk", singleWorkspace(segmentHandler.BulkCreate))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees", employeeHandler.Create)
	mux.HandleFunc("POST /api/v1/employees", singleWorkspace(employeeHandler.Create))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees", employeeHandler.List)
	mux.HandleFunc("GET /api/v1/employees", singleWorkspace(employeeHandler.List))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Get)
	mux.HandleFunc("GET /api/v1/employees/{id}", singleWorkspace(employeeHandler.Get))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Update)
	mux.HandleFunc("PUT /api/v1/employees/{id}", singleWorkspace(employeeHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/employees/{id}", singleWorkspace(employeeHandler.Delete))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rehire", employeeHandler.Rehire)
	mux.HandleFunc("POST /api/v1/employees/{id}/rehire", singleWorkspace(employeeHandler.Rehire))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/upload", employeeImportHandler.Upload)
	mux.HandleFunc("POST /api/v1/employees/upload", singleWorkspace(employeeImportHandler.Upload))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/import-termination/preview", employeeImportHandler.PreviewTermination)
	mux.HandleFunc("POST /api/v1/employees/import-termination/preview", singleWorkspace(employeeImportHandler.PreviewTermination))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/import-termination/confirm", employeeImportHandler.ConfirmTermination)
	mux.HandleFunc("POST /api/v1/employees/import-termination/confirm", singleWorkspace(employeeImportHandler.ConfirmTermination))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/import-termination/template", employeeImportHandler.TerminationTemplate)
	mux.HandleFunc("GET /api/v1/employees/import-termination/template", singleWorkspace(employeeImportHandler.TerminationTemplate))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/conditions", employeeConditionHandler.List)
	mux.HandleFunc("GET /api/v1/employees/{id}/conditions", singleWorkspace(employeeConditionHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/conditions", employeeConditionHandler.Create)
	mux.HandleFunc("POST /api/v1/employees/{id}/conditions", singleWorkspace(employeeConditionHandler.Create))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}/conditions/{cond_id}", employeeConditionHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/employees/{id}/conditions/{cond_id}", singleWorkspace(employeeConditionHandler.Delete))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/departments", employeeDepartmentHandler.List)
	mux.HandleFunc("GET /api/v1/employees/{id}/departments", singleWorkspace(employeeDepartmentHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/departments", employeeDepartmentHandler.Transfer)
	mux.HandleFunc("POST /api/v1/employees/{id}/departments", singleWorkspace(employeeDepartmentHandler.Transfer))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/compensation-history", compensationHistoryHandler.Get)
	mux.HandleFunc("GET /api/v1/employees/{id}/compensation-history", singleWorkspace(compensationHistoryHandler.Get))
	customFieldRepo := repository.CustomFieldRepository{DB: workspaceRepo.DB}
	customFieldHandler := api.CustomFieldHandler{Repository: customFieldRepo, Authorizer: workspaceAuthorizer}
	contractRepo := repository.ContractRepository{DB: workspaceRepo.DB}
	employmentRepo := repository.EmploymentRepository{DB: workspaceRepo.DB}
	contractEmploymentHandler := api.ContractEmploymentHandler{Contracts: contractRepo, Employments: employmentRepo, CustomFields: customFieldRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/contracts", contractEmploymentHandler.ListWorkspaceContracts)
	mux.HandleFunc("GET /api/v1/contracts", singleWorkspace(contractEmploymentHandler.ListWorkspaceContracts))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts", contractEmploymentHandler.ListEmployeeContracts)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/contracts", singleWorkspace(contractEmploymentHandler.ListEmployeeContracts))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts", contractEmploymentHandler.CreateContract)
	mux.HandleFunc("POST /api/v1/employees/{employee_id}/contracts", singleWorkspace(contractEmploymentHandler.CreateContract))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.GetContract)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/contracts/{contract_id}", singleWorkspace(contractEmploymentHandler.GetContract))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.UpdateContract)
	mux.HandleFunc("PATCH /api/v1/employees/{employee_id}/contracts/{contract_id}", singleWorkspace(contractEmploymentHandler.UpdateContract))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}", contractEmploymentHandler.DeleteContract)
	mux.HandleFunc("DELETE /api/v1/employees/{employee_id}/contracts/{contract_id}", singleWorkspace(contractEmploymentHandler.DeleteContract))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments", contractEmploymentHandler.ListEmployments)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/employments", singleWorkspace(contractEmploymentHandler.ListEmployments))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments", contractEmploymentHandler.CreateEmployment)
	mux.HandleFunc("POST /api/v1/employees/{employee_id}/employments", singleWorkspace(contractEmploymentHandler.CreateEmployment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.GetEmployment)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/employments/{employment_id}", singleWorkspace(contractEmploymentHandler.GetEmployment))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.UpdateEmployment)
	mux.HandleFunc("PATCH /api/v1/employees/{employee_id}/employments/{employment_id}", singleWorkspace(contractEmploymentHandler.UpdateEmployment))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}", contractEmploymentHandler.DeleteEmployment)
	mux.HandleFunc("DELETE /api/v1/employees/{employee_id}/employments/{employment_id}", singleWorkspace(contractEmploymentHandler.DeleteEmployment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments", assignmentHandler.ListEmployeeAssignments)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/assignments", singleWorkspace(assignmentHandler.ListEmployeeAssignments))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments", assignmentHandler.CreateEmployeeAssignment)
	mux.HandleFunc("POST /api/v1/employees/{employee_id}/assignments", singleWorkspace(assignmentHandler.CreateEmployeeAssignment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.GetEmployeeAssignment)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/assignments/{assignment_id}", singleWorkspace(assignmentHandler.GetEmployeeAssignment))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.UpdateEmployeeAssignment)
	mux.HandleFunc("PATCH /api/v1/employees/{employee_id}/assignments/{assignment_id}", singleWorkspace(assignmentHandler.UpdateEmployeeAssignment))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}", assignmentHandler.DeleteEmployeeAssignment)
	mux.HandleFunc("DELETE /api/v1/employees/{employee_id}/assignments/{assignment_id}", singleWorkspace(assignmentHandler.DeleteEmployeeAssignment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/definitions", customFieldHandler.ListDefinitions)
	mux.HandleFunc("GET /api/v1/custom-fields/definitions", singleWorkspace(customFieldHandler.ListDefinitions))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/custom-fields/definitions", customFieldHandler.CreateDefinition)
	mux.HandleFunc("POST /api/v1/custom-fields/definitions", singleWorkspace(customFieldHandler.CreateDefinition))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.GetDefinition)
	mux.HandleFunc("GET /api/v1/custom-fields/definitions/{id}", singleWorkspace(customFieldHandler.GetDefinition))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.UpdateDefinition)
	mux.HandleFunc("PATCH /api/v1/custom-fields/definitions/{id}", singleWorkspace(customFieldHandler.UpdateDefinition))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.DeleteDefinition)
	mux.HandleFunc("DELETE /api/v1/custom-fields/definitions/{id}", singleWorkspace(customFieldHandler.DeleteDefinition))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.GetValues)
	mux.HandleFunc("GET /api/v1/custom-fields/values/{entity_type}/{entity_id}", singleWorkspace(customFieldHandler.GetValues))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.SetValues)
	mux.HandleFunc("PUT /api/v1/custom-fields/values/{entity_type}/{entity_id}", singleWorkspace(customFieldHandler.SetValues))
	leaveRepo := repository.LeaveRepository{DB: workspaceRepo.DB}
	leaveHandler := api.LeaveHandler{
		Repository:       leaveRepo,
		BehalfRepository: attendancePolicyRepo,
		Authorizer:       workspaceAuthorizer,
		StorageDir:       cfg.StorageDir,
	}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/types", leaveHandler.ListTypes)
	mux.HandleFunc("GET /api/v1/leave/types", singleWorkspace(leaveHandler.ListTypes))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/types", leaveHandler.CreateType)
	mux.HandleFunc("POST /api/v1/leave/types", singleWorkspace(leaveHandler.CreateType))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/leave/types/{id}/config", leaveHandler.UpdateTypeConfig)
	mux.HandleFunc("PUT /api/v1/leave/types/{id}/config", singleWorkspace(leaveHandler.UpdateTypeConfig))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/types/{id}", leaveHandler.DeleteType)
	mux.HandleFunc("DELETE /api/v1/leave/types/{id}", singleWorkspace(leaveHandler.DeleteType))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/types/{id}/restore", leaveHandler.RestoreType)
	mux.HandleFunc("POST /api/v1/leave/types/{id}/restore", singleWorkspace(leaveHandler.RestoreType))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/submit", leaveHandler.Submit)
	mux.HandleFunc("POST /api/v1/leave/submit", singleWorkspace(leaveHandler.Submit))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/requests", leaveHandler.ListRequests)
	mux.HandleFunc("GET /api/v1/leave/requests", singleWorkspace(leaveHandler.ListRequests))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/requests/{id}/approve", leaveHandler.Approve)
	mux.HandleFunc("POST /api/v1/leave/requests/{id}/approve", singleWorkspace(leaveHandler.Approve))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/leave/balances", leaveHandler.Balances)
	mux.HandleFunc("GET /api/v1/employees/{id}/leave/balances", singleWorkspace(leaveHandler.Balances))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/balances", leaveHandler.WorkspaceBalances)
	mux.HandleFunc("GET /api/v1/leave/balances", singleWorkspace(leaveHandler.WorkspaceBalances))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/leave/history", leaveHandler.EmployeeHistory)
	mux.HandleFunc("GET /api/v1/employees/{id}/leave/history", singleWorkspace(leaveHandler.EmployeeHistory))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/leave/adjust", leaveHandler.AdjustBalance)
	mux.HandleFunc("POST /api/v1/employees/{id}/leave/adjust", singleWorkspace(leaveHandler.AdjustBalance))
	mux.HandleFunc("GET /api/v1/employees/{id}/leave/requests", singleWorkspace(leaveHandler.MyRequests))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/policies", leaveHandler.ListPolicies)
	mux.HandleFunc("GET /api/v1/leave/policies", singleWorkspace(leaveHandler.ListPolicies))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/policies", leaveHandler.CreatePolicy)
	mux.HandleFunc("POST /api/v1/leave/policies", singleWorkspace(leaveHandler.CreatePolicy))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/leave/policies/{policy_id}", leaveHandler.UpdatePolicy)
	mux.HandleFunc("PATCH /api/v1/leave/policies/{policy_id}", singleWorkspace(leaveHandler.UpdatePolicy))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/policies/{policy_id}", leaveHandler.DeletePolicy)
	mux.HandleFunc("DELETE /api/v1/leave/policies/{policy_id}", singleWorkspace(leaveHandler.DeletePolicy))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/policies/assign", leaveHandler.AssignPolicy)
	mux.HandleFunc("POST /api/v1/leave/policies/assign", singleWorkspace(leaveHandler.AssignPolicy))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/policies/assignments", leaveHandler.ListPolicyAssignments)
	mux.HandleFunc("GET /api/v1/leave/policies/assignments", singleWorkspace(leaveHandler.ListPolicyAssignments))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/policies/assignments/{assignment_id}", leaveHandler.DeletePolicyAssignment)
	mux.HandleFunc("DELETE /api/v1/leave/policies/assignments/{assignment_id}", singleWorkspace(leaveHandler.DeletePolicyAssignment))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/accrual/run", leaveHandler.TriggerAccrual)
	mux.HandleFunc("POST /api/v1/leave/accrual/run", singleWorkspace(leaveHandler.TriggerAccrual))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/jobs/{job_id}/status", leaveHandler.JobStatus)
	mux.HandleFunc("GET /api/v1/jobs/{job_id}/status", singleWorkspace(leaveHandler.JobStatus))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/upload-excel", leaveHandler.UploadExcel)
	mux.HandleFunc("POST /api/v1/leave/upload-excel", singleWorkspace(leaveHandler.UploadExcel))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/import/latest", leaveHandler.LatestImport)
	mux.HandleFunc("GET /api/v1/leave/import/latest", singleWorkspace(leaveHandler.LatestImport))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/import/{job_id}/errors", leaveHandler.ImportErrors)
	mux.HandleFunc("GET /api/v1/leave/import/{job_id}/errors", singleWorkspace(leaveHandler.ImportErrors))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/upload-template", leaveHandler.UploadTemplate)
	mux.HandleFunc("GET /api/v1/leave/upload-template", singleWorkspace(leaveHandler.UploadTemplate))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/behalf-assignments", leaveHandler.ListBehalfAssignments)
	mux.HandleFunc("GET /api/v1/leave/behalf-assignments", singleWorkspace(leaveHandler.ListBehalfAssignments))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/behalf-assignments", leaveHandler.CreateBehalfAssignment)
	mux.HandleFunc("POST /api/v1/leave/behalf-assignments", singleWorkspace(leaveHandler.CreateBehalfAssignment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/behalf-assignments/my-targets", leaveHandler.GetMyBehalfTargets)
	mux.HandleFunc("GET /api/v1/leave/behalf-assignments/my-targets", singleWorkspace(leaveHandler.GetMyBehalfTargets))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/leave/behalf-assignments/{id}", leaveHandler.DeleteBehalfAssignment)
	mux.HandleFunc("DELETE /api/v1/leave/behalf-assignments/{id}", singleWorkspace(leaveHandler.DeleteBehalfAssignment))
	holidayRepo := repository.HolidayRepository{DB: workspaceRepo.DB}
	holidayHandler := api.HolidayHandler{Repository: holidayRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/holidays", holidayHandler.List)
	mux.HandleFunc("GET /api/v1/holidays", singleWorkspace(holidayHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/holidays", holidayHandler.Create)
	mux.HandleFunc("POST /api/v1/holidays", singleWorkspace(holidayHandler.Create))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Get)
	mux.HandleFunc("GET /api/v1/holidays/{id}", singleWorkspace(holidayHandler.Get))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Update)
	mux.HandleFunc("PUT /api/v1/holidays/{id}", singleWorkspace(holidayHandler.Update))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Update)
	mux.HandleFunc("PATCH /api/v1/holidays/{id}", singleWorkspace(holidayHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/holidays/{id}", holidayHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/holidays/{id}", singleWorkspace(holidayHandler.Delete))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/holidays/{id}/process-leave", holidayHandler.ProcessLeave)
	mux.HandleFunc("POST /api/v1/holidays/{id}/process-leave", singleWorkspace(holidayHandler.ProcessLeave))
	payrollRepo := repository.PayrollRepository{DB: workspaceRepo.DB, SystemDB: workspaceRepo.SystemDB}
	essRepo := repository.ESSRepository{DB: workspaceRepo.DB}
	essHandler := api.ESSHandler{
		Repository: essRepo,
		Attendance: attendanceRepo,
		Payroll:    payrollRepo,
		Leaves:     leaveRepo,
		ShiftSwaps: swapRepo,
	}
	mux.HandleFunc("GET /api/v1/ess/profile", singleWorkspace(essHandler.GetProfile))
	mux.HandleFunc("PUT /api/v1/ess/profile", singleWorkspace(essHandler.UpdateProfile))
	mux.HandleFunc("GET /api/v1/ess/emergency-contacts", singleWorkspace(essHandler.GetContacts))
	mux.HandleFunc("POST /api/v1/ess/emergency-contacts", singleWorkspace(essHandler.AddContact))
	mux.HandleFunc("DELETE /api/v1/ess/emergency-contacts/{contact_id}", singleWorkspace(essHandler.DeleteContact))
	mux.HandleFunc("GET /api/v1/ess/documents/pending", singleWorkspace(essHandler.GetPendingDocuments))
	mux.HandleFunc("POST /api/v1/ess/documents/{id}/acknowledge", singleWorkspace(essHandler.AcknowledgeDocument))
	mux.HandleFunc("GET /api/v1/ess/documents", singleWorkspace(essHandler.GetDocuments))
	mux.HandleFunc("GET /api/v1/ess/payslips", singleWorkspace(essHandler.GetPayslips))
	mux.HandleFunc("GET /api/v1/ess/attendance", singleWorkspace(essHandler.GetAttendance))
	mux.HandleFunc("POST /api/v1/ess/punch", singleWorkspace(essHandler.Punch))
	mux.HandleFunc("GET /api/v1/ess/time-entries", singleWorkspace(essHandler.GetTimeEntries))
	mux.HandleFunc("GET /api/v1/ess/leave/balances", singleWorkspace(essHandler.GetLeaveBalances))
	mux.HandleFunc("GET /api/v1/ess/leave/requests", singleWorkspace(essHandler.GetLeaveRequests))
	mux.HandleFunc("GET /api/v1/ess/shift-swaps", singleWorkspace(essHandler.GetShiftSwaps))
	mux.HandleFunc("POST /api/v1/ess/shift-swaps", singleWorkspace(essHandler.CreateShiftSwap))
	mux.HandleFunc("PUT /api/v1/ess/shift-swaps/{id}/respond", singleWorkspace(essHandler.RespondShiftSwap))
	mux.HandleFunc("DELETE /api/v1/ess/shift-swaps/{id}", singleWorkspace(essHandler.CancelShiftSwap))
	workflowRepo := repository.ApprovalWorkflowRepository{DB: workspaceRepo.DB, SystemDB: workspaceRepo.SystemDB}
	workflowHandler := api.ApprovalWorkflowHandler{Repository: workflowRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows", workflowHandler.List)
	mux.HandleFunc("GET /api/v1/approval-workflows", singleWorkspace(workflowHandler.List))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/approval-workflows", workflowHandler.Create)
	mux.HandleFunc("POST /api/v1/approval-workflows", singleWorkspace(workflowHandler.Create))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/approval-workflows/approve", workflowHandler.Process)
	mux.HandleFunc("POST /api/v1/approval-workflows/approve", singleWorkspace(workflowHandler.Process))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows/history/{type}/{id}", workflowHandler.History)
	mux.HandleFunc("GET /api/v1/approval-workflows/history/{type}/{id}", singleWorkspace(workflowHandler.History))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows/{id}", workflowHandler.Get)
	mux.HandleFunc("GET /api/v1/approval-workflows/{id}", singleWorkspace(workflowHandler.Get))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/approval-workflows/{id}", workflowHandler.Update)
	mux.HandleFunc("PUT /api/v1/approval-workflows/{id}", singleWorkspace(workflowHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/approval-workflows/{id}", workflowHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/approval-workflows/{id}", singleWorkspace(workflowHandler.Delete))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows/pending-tasks", workflowHandler.PendingTasks)
	mux.HandleFunc("GET /api/v1/approval-workflows/pending-tasks", singleWorkspace(workflowHandler.PendingTasks))
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
	mux.HandleFunc("GET /api/v1/payroll/payslips", singleWorkspace(payrollHandler.ListWorkspace))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/payslips", payrollHandler.ListEmployee)
	mux.HandleFunc("GET /api/v1/employees/{employee_id}/payslips", singleWorkspace(payrollHandler.ListEmployee))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/salary-components", payrollHandler.ListSalaryComponents)
	mux.HandleFunc("GET /api/v1/salary-components", singleWorkspace(payrollHandler.ListSalaryComponents))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/salary-components", payrollHandler.CreateSalaryComponent)
	mux.HandleFunc("POST /api/v1/salary-components", singleWorkspace(payrollHandler.CreateSalaryComponent))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{id}/salary-components", payrollHandler.ListEmploymentSalaryComponents)
	mux.HandleFunc("GET /api/v1/employments/{id}/salary-components", singleWorkspace(payrollHandler.ListEmploymentSalaryComponents))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{id}/salary-components", payrollHandler.AssignEmploymentSalaryComponent)
	mux.HandleFunc("POST /api/v1/employments/{id}/salary-components", singleWorkspace(payrollHandler.AssignEmploymentSalaryComponent))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/salary-bases", payrollHandler.ListSalaryBases)
	mux.HandleFunc("GET /api/v1/salary-bases", singleWorkspace(payrollHandler.ListSalaryBases))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/salary-bases", payrollHandler.CreateSalaryBase)
	mux.HandleFunc("POST /api/v1/salary-bases", singleWorkspace(payrollHandler.CreateSalaryBase))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members", payrollHandler.AssignBaseMember)
	mux.HandleFunc("POST /api/v1/salary-bases/{base_id}/members", singleWorkspace(payrollHandler.AssignBaseMember))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id}", payrollHandler.UnassignBaseMember)
	mux.HandleFunc("DELETE /api/v1/salary-bases/{base_id}/members/{component_id}", singleWorkspace(payrollHandler.UnassignBaseMember))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-settings", payrollHandler.GetPayrollSettings)
	mux.HandleFunc("GET /api/v1/payroll-settings", singleWorkspace(payrollHandler.GetPayrollSettings))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/payroll-settings", payrollHandler.UpdatePayrollSettings)
	mux.HandleFunc("PATCH /api/v1/payroll-settings", singleWorkspace(payrollHandler.UpdatePayrollSettings))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-settings/history", payrollHandler.GetPayrollSettingsHistory)
	mux.HandleFunc("GET /api/v1/payroll-settings/history", singleWorkspace(payrollHandler.GetPayrollSettingsHistory))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/payroll-config", payrollHandler.GetEmployeePayrollConfig)
	mux.HandleFunc("GET /api/v1/employees/{id}/payroll-config", singleWorkspace(payrollHandler.GetEmployeePayrollConfig))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/payroll-config", payrollHandler.SetEmployeePayrollConfig)
	mux.HandleFunc("POST /api/v1/employees/{id}/payroll-config", singleWorkspace(payrollHandler.SetEmployeePayrollConfig))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll/incentive-policies", payrollHandler.GetIncentivePolicies)
	mux.HandleFunc("GET /api/v1/payroll/incentive-policies", singleWorkspace(payrollHandler.GetIncentivePolicies))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll/incentive-policies", payrollHandler.CreateIncentivePolicy)
	mux.HandleFunc("POST /api/v1/payroll/incentive-policies", singleWorkspace(payrollHandler.CreateIncentivePolicy))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}", payrollHandler.DeleteIncentivePolicy)
	mux.HandleFunc("DELETE /api/v1/payroll/incentive-policies/{policy_id}", singleWorkspace(payrollHandler.DeleteIncentivePolicy))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone", payrollHandler.CloneIncentivePolicy)
	mux.HandleFunc("POST /api/v1/payroll/incentive-policies/{policy_id}/clone", singleWorkspace(payrollHandler.CloneIncentivePolicy))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-rules", payrollHandler.ListPayrollRules)
	mux.HandleFunc("GET /api/v1/payroll-rules", singleWorkspace(payrollHandler.ListPayrollRules))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll-rules", payrollHandler.CreatePayrollRule)
	mux.HandleFunc("POST /api/v1/payroll-rules", singleWorkspace(payrollHandler.CreatePayrollRule))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers", payrollHandler.ListPayrollRuleTiers)
	mux.HandleFunc("GET /api/v1/payroll-rules/{rule_id}/tiers", singleWorkspace(payrollHandler.ListPayrollRuleTiers))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers", payrollHandler.CreatePayrollRuleTier)
	mux.HandleFunc("POST /api/v1/payroll-rules/{rule_id}/tiers", singleWorkspace(payrollHandler.CreatePayrollRuleTier))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/tiers/{tier_id}", payrollHandler.DeletePayrollRuleTier)
	mux.HandleFunc("DELETE /api/v1/payroll-rules/{rule_id}/tiers/{tier_id}", singleWorkspace(payrollHandler.DeletePayrollRuleTier))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments", payrollHandler.ListPayrollRuleAssignments)
	mux.HandleFunc("GET /api/v1/payroll-rules/{rule_id}/assignments", singleWorkspace(payrollHandler.ListPayrollRuleAssignments))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments", payrollHandler.AssignPayrollRule)
	mux.HandleFunc("POST /api/v1/payroll-rules/{rule_id}/assignments", singleWorkspace(payrollHandler.AssignPayrollRule))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/payroll-rules/{rule_id}/assignments/{assignment_id}", payrollHandler.DeletePayrollRuleAssignment)
	mux.HandleFunc("DELETE /api/v1/payroll-rules/{rule_id}/assignments/{assignment_id}", singleWorkspace(payrollHandler.DeletePayrollRuleAssignment))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/calculate-payroll", payrollHandler.CalculatePayroll)
	mux.HandleFunc("POST /api/v1/employees/{id}/calculate-payroll", singleWorkspace(payrollHandler.CalculatePayroll))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/payroll/calculate-batch", payrollHandler.CalculateBatchPayroll)
	mux.HandleFunc("POST /api/v1/payroll/calculate-batch", singleWorkspace(payrollHandler.CalculateBatchPayroll))
	reportRepo := repository.ReportRepository{DB: workspaceRepo.DB}
	reportQueue := &repository.RedisReportQueue{Client: redisClient, DB: workspaceRepo.DB}
	reportHandler := api.ReportHandler{Repository: reportRepo, Queue: reportQueue, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/attendance", reportHandler.AttendanceReport)
	mux.HandleFunc("GET /api/v1/reports/attendance", singleWorkspace(reportHandler.AttendanceReport))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/no-show", reportHandler.NoShowReport)
	mux.HandleFunc("GET /api/v1/reports/no-show", singleWorkspace(reportHandler.NoShowReport))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/reports/export", reportHandler.CreateExport)
	mux.HandleFunc("POST /api/v1/reports/export", singleWorkspace(reportHandler.CreateExport))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/jobs/{job_id}", reportHandler.Status)
	mux.HandleFunc("GET /api/v1/reports/jobs/{job_id}", singleWorkspace(reportHandler.Status))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/reports/jobs/{job_id}", reportHandler.Cancel)
	mux.HandleFunc("DELETE /api/v1/reports/jobs/{job_id}", singleWorkspace(reportHandler.Cancel))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/reports/jobs/{job_id}/download", reportHandler.Download)
	mux.HandleFunc("GET /api/v1/reports/jobs/{job_id}/download", singleWorkspace(reportHandler.Download))
	fileRepo := repository.FileRepository{DB: workspaceRepo.DB}
	var fileStorage api.FileStorage
	if s3Client, err := external.NewS3Client(cfg.S3EndpointURL, cfg.S3BucketName, cfg.S3AccessKey.Expose(), cfg.S3SecretKey.Expose(), cfg.S3Region, cfg.S3UsePathStyle, nil); err == nil {
		fileStorage = s3Client
	}
	fileHandler := api.FileHandler{Repository: fileRepo, Storage: fileStorage, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/files/upload", fileHandler.UploadFile)
	mux.HandleFunc("POST /v1/files/upload", singleWorkspace(fileHandler.UploadFile))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/files/upload", fileHandler.UploadFile)
	mux.HandleFunc("POST /api/v1/files/upload", singleWorkspace(fileHandler.UploadFile))
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("POST /v1/files/presigned", singleWorkspace(fileHandler.CreateUploadURL))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("POST /api/v1/files/presigned", singleWorkspace(fileHandler.CreateUploadURL))
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("GET /v1/files/{id}", singleWorkspace(fileHandler.GetMetadata))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("GET /api/v1/files/{id}", singleWorkspace(fileHandler.GetMetadata))
	mux.HandleFunc("PUT /v1/workspaces/{ws_id}/files/{id}", fileHandler.UpdateFile)
	mux.HandleFunc("PUT /v1/files/{id}", singleWorkspace(fileHandler.UpdateFile))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.UpdateFile)
	mux.HandleFunc("PUT /api/v1/files/{id}", singleWorkspace(fileHandler.UpdateFile))
	mux.HandleFunc("DELETE /v1/workspaces/{ws_id}/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("DELETE /v1/files/{id}", singleWorkspace(fileHandler.DeleteFile))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("DELETE /api/v1/files/{id}", singleWorkspace(fileHandler.DeleteFile))
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}/download", fileHandler.DownloadFile)
	mux.HandleFunc("GET /v1/files/{id}/download", singleWorkspace(fileHandler.DownloadFile))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}/download", fileHandler.DownloadFile)
	mux.HandleFunc("GET /api/v1/files/{id}/download", singleWorkspace(fileHandler.DownloadFile))
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("GET /v1/files/{id}/url", singleWorkspace(fileHandler.GetURL))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("GET /api/v1/files/{id}/url", singleWorkspace(fileHandler.GetURL))
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}/versions", fileHandler.ListVersions)
	mux.HandleFunc("GET /v1/files/{id}/versions", singleWorkspace(fileHandler.ListVersions))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}/versions", fileHandler.ListVersions)
	mux.HandleFunc("GET /api/v1/files/{id}/versions", singleWorkspace(fileHandler.ListVersions))
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}", fileHandler.ListByOwner)
	mux.HandleFunc("GET /v1/files/by-owner/{owner_type}/{owner_id}", singleWorkspace(fileHandler.ListByOwner))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/by-owner/{owner_type}/{owner_id}", fileHandler.ListByOwner)
	mux.HandleFunc("GET /api/v1/files/by-owner/{owner_type}/{owner_id}", singleWorkspace(fileHandler.ListByOwner))

	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("POST /v1/documents/{document_id}/signature-requests", singleWorkspace(fileHandler.CreateSignatureRequest))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("POST /api/v1/documents/{document_id}/signature-requests", singleWorkspace(fileHandler.CreateSignatureRequest))
	mux.HandleFunc("GET /v1/ess/workspaces/{ws_id}/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("GET /v1/ess/signatures/pending", singleWorkspace(fileHandler.ListPendingSignatures))
	mux.HandleFunc("GET /api/v1/ess/workspaces/{ws_id}/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("GET /api/v1/ess/signatures/pending", singleWorkspace(fileHandler.ListPendingSignatures))
	mux.HandleFunc("POST /v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign", fileHandler.SignDocument)
	mux.HandleFunc("POST /v1/ess/signatures/{request_id}/sign", singleWorkspace(fileHandler.SignDocument))
	mux.HandleFunc("POST /api/v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign", fileHandler.SignDocument)
	mux.HandleFunc("POST /api/v1/ess/signatures/{request_id}/sign", singleWorkspace(fileHandler.SignDocument))

	documentRepo := repository.DocumentRepository{DB: workspaceRepo.DB, SystemDB: workspaceRepo.SystemDB}
	documentHandler := api.DocumentHandler{Repository: documentRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/documents/folders", documentHandler.ListFolders)
	mux.HandleFunc("GET /v1/documents/folders", singleWorkspace(documentHandler.ListFolders))
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/folders", documentHandler.CreateFolder)
	mux.HandleFunc("POST /v1/documents/folders", singleWorkspace(documentHandler.CreateFolder))
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.GetFolder)
	mux.HandleFunc("GET /v1/documents/folders/{id}", singleWorkspace(documentHandler.GetFolder))
	mux.HandleFunc("PUT /v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.RenameFolder)
	mux.HandleFunc("PUT /v1/documents/folders/{id}", singleWorkspace(documentHandler.RenameFolder))
	mux.HandleFunc("DELETE /v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.DeleteFolder)
	mux.HandleFunc("DELETE /v1/documents/folders/{id}", singleWorkspace(documentHandler.DeleteFolder))
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/distributions", documentHandler.Distribute)
	mux.HandleFunc("POST /v1/documents/distributions", singleWorkspace(documentHandler.Distribute))
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/templates/preview", documentHandler.PreviewTemplate)
	mux.HandleFunc("POST /v1/documents/templates/preview", singleWorkspace(documentHandler.PreviewTemplate))
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/templates/{id}/generate", documentHandler.GenerateTemplate)
	mux.HandleFunc("POST /v1/documents/templates/{id}/generate", singleWorkspace(documentHandler.GenerateTemplate))

	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/documents/folders", documentHandler.ListFolders)
	mux.HandleFunc("GET /api/v1/documents/folders", singleWorkspace(documentHandler.ListFolders))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/folders", documentHandler.CreateFolder)
	mux.HandleFunc("POST /api/v1/documents/folders", singleWorkspace(documentHandler.CreateFolder))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.GetFolder)
	mux.HandleFunc("GET /api/v1/documents/folders/{id}", singleWorkspace(documentHandler.GetFolder))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.RenameFolder)
	mux.HandleFunc("PUT /api/v1/documents/folders/{id}", singleWorkspace(documentHandler.RenameFolder))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/documents/folders/{id}", documentHandler.DeleteFolder)
	mux.HandleFunc("DELETE /api/v1/documents/folders/{id}", singleWorkspace(documentHandler.DeleteFolder))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/distributions", documentHandler.Distribute)
	mux.HandleFunc("POST /api/v1/documents/distributions", singleWorkspace(documentHandler.Distribute))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/templates/preview", documentHandler.PreviewTemplate)
	mux.HandleFunc("POST /api/v1/documents/templates/preview", singleWorkspace(documentHandler.PreviewTemplate))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/templates/{id}/generate", documentHandler.GenerateTemplate)
	mux.HandleFunc("POST /api/v1/documents/templates/{id}/generate", singleWorkspace(documentHandler.GenerateTemplate))
	conditionRepo := repository.ConditionTypeRepository{DB: workspaceRepo.DB}
	conditionHandler := api.ConditionTypeHandler{Repository: conditionRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/condition-types", conditionHandler.List)
	mux.HandleFunc("GET /api/v1/condition-types", singleWorkspace(conditionHandler.List))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Get)
	mux.HandleFunc("GET /api/v1/condition-types/{id}", singleWorkspace(conditionHandler.Get))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/condition-types", conditionHandler.Create)
	mux.HandleFunc("POST /api/v1/condition-types", singleWorkspace(conditionHandler.Create))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Update)
	mux.HandleFunc("PATCH /api/v1/condition-types/{id}", singleWorkspace(conditionHandler.Update))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Delete)
	mux.HandleFunc("DELETE /api/v1/condition-types/{id}", singleWorkspace(conditionHandler.Delete))
	phaseRepo := repository.EmploymentPhaseRepository{DB: workspaceRepo.DB}
	phaseHandler := api.EmploymentPhaseHandler{Repository: phaseRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employment-phases", phaseHandler.CreateDefinition)
	mux.HandleFunc("POST /api/v1/employment-phases", singleWorkspace(phaseHandler.CreateDefinition))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employment-phases", phaseHandler.ListDefinitions)
	mux.HandleFunc("GET /api/v1/employment-phases", singleWorkspace(phaseHandler.ListDefinitions))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{user_id}/phases", phaseHandler.AssignEmployee)
	mux.HandleFunc("POST /api/v1/employees/{user_id}/phases", singleWorkspace(phaseHandler.AssignEmployee))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{user_id}/phases", phaseHandler.ListEmployee)
	mux.HandleFunc("GET /api/v1/employees/{user_id}/phases", singleWorkspace(phaseHandler.ListEmployee))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employees/{user_id}/phases/{phase_id}/status", phaseHandler.UpdateEmployeeStatus)
	mux.HandleFunc("PUT /api/v1/employees/{user_id}/phases/{phase_id}/status", singleWorkspace(phaseHandler.UpdateEmployeeStatus))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{employment_id}/phases", phaseHandler.AssignEmployment)
	mux.HandleFunc("POST /api/v1/employments/{employment_id}/phases", singleWorkspace(phaseHandler.AssignEmployment))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{employment_id}/phases", phaseHandler.ListEmployment)
	mux.HandleFunc("GET /api/v1/employments/{employment_id}/phases", singleWorkspace(phaseHandler.ListEmployment))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/employments/{employment_id}/phases/{phase_id}/status", phaseHandler.UpdateEmploymentStatus)
	mux.HandleFunc("PUT /api/v1/employments/{employment_id}/phases/{phase_id}/status", singleWorkspace(phaseHandler.UpdateEmploymentStatus))

	lifecycleRepo := repository.LifecycleRepository{DB: workspaceRepo.DB}
	lifecycleHandler := api.LifecycleHandler{
		Repository: lifecycleRepo,
		Authorizer: workspaceAuthorizer,
		UsersWithPermission: func(ctx context.Context, wsID uuid.UUID, permission string) []uuid.UUID {
			users, _ := workspaceAuthorizer.GetUsersWithPermission(ctx, wsID, permission)
			return users
		},
	}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle/my-tasks", lifecycleHandler.GetMyTasks)
	mux.HandleFunc("GET /api/v1/lifecycle/my-tasks", singleWorkspace(lifecycleHandler.GetMyTasks))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/lifecycle-templates", lifecycleHandler.CreateTemplate)
	mux.HandleFunc("POST /api/v1/lifecycle-templates", singleWorkspace(lifecycleHandler.CreateTemplate))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates", lifecycleHandler.ListTemplates)
	mux.HandleFunc("GET /api/v1/lifecycle-templates", singleWorkspace(lifecycleHandler.ListTemplates))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}", lifecycleHandler.GetTemplate)
	mux.HandleFunc("GET /api/v1/lifecycle-templates/{id}", singleWorkspace(lifecycleHandler.GetTemplate))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}", lifecycleHandler.UpdateTemplate)
	mux.HandleFunc("PATCH /api/v1/lifecycle-templates/{id}", singleWorkspace(lifecycleHandler.UpdateTemplate))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}", lifecycleHandler.DeleteTemplate)
	mux.HandleFunc("DELETE /api/v1/lifecycle-templates/{id}", singleWorkspace(lifecycleHandler.DeleteTemplate))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}/items", lifecycleHandler.CreateTemplateItem)
	mux.HandleFunc("POST /api/v1/lifecycle-templates/{id}/items", singleWorkspace(lifecycleHandler.CreateTemplateItem))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates/{id}/items", lifecycleHandler.ListTemplateItems)
	mux.HandleFunc("GET /api/v1/lifecycle-templates/{id}/items", singleWorkspace(lifecycleHandler.ListTemplateItems))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.GetTemplateItem)
	mux.HandleFunc("GET /api/v1/lifecycle-templates/{tid}/items/{iid}", singleWorkspace(lifecycleHandler.GetTemplateItem))
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.UpdateTemplateItem)
	mux.HandleFunc("PUT /api/v1/lifecycle-templates/{tid}/items/{iid}", singleWorkspace(lifecycleHandler.UpdateTemplateItem))
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/lifecycle-templates/{tid}/items/{iid}", lifecycleHandler.DeleteTemplateItem)
	mux.HandleFunc("DELETE /api/v1/lifecycle-templates/{tid}/items/{iid}", singleWorkspace(lifecycleHandler.DeleteTemplateItem))
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{emp_id}/lifecycle", lifecycleHandler.CreateChecklist)
	mux.HandleFunc("POST /api/v1/employments/{emp_id}/lifecycle", singleWorkspace(lifecycleHandler.CreateChecklist))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{emp_id}/lifecycle", lifecycleHandler.ListChecklists)
	mux.HandleFunc("GET /api/v1/employments/{emp_id}/lifecycle", singleWorkspace(lifecycleHandler.ListChecklists))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/lifecycle/{id}", lifecycleHandler.GetChecklist)
	mux.HandleFunc("GET /api/v1/lifecycle/{id}", singleWorkspace(lifecycleHandler.GetChecklist))
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/lifecycle/{id}/items/{iid}", lifecycleHandler.UpdateItemStatus)
	mux.HandleFunc("PATCH /api/v1/lifecycle/{id}/items/{iid}", singleWorkspace(lifecycleHandler.UpdateItemStatus))

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
	mux.HandleFunc("GET /api/v1/audit-logs", singleWorkspace(monitorHandler.GetAuditLogs))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/jobs/{job_id}", monitorHandler.GetWorkspaceJobStatus)
	mux.HandleFunc("GET /api/v1/jobs/{job_id}", singleWorkspace(monitorHandler.GetWorkspaceJobStatus))
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/monitor/jobs/{job_id}", monitorHandler.GetWorkspaceMonitorJob)
	mux.HandleFunc("GET /api/v1/monitor/jobs/{job_id}", singleWorkspace(monitorHandler.GetWorkspaceMonitorJob))

	mux.HandleFunc("POST /api/v1/auth/login", authHandler.Login)
	mux.HandleFunc("POST /api/v1/auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /api/v1/auth/logout", authHandler.Logout)
	mux.HandleFunc("POST /api/v1/auth/forgot-password", authHandler.ForgotPassword)
	mux.HandleFunc("POST /api/v1/auth/reset-password", authHandler.ResetPassword)
	mux.HandleFunc("POST /api/v1/auth/change-password", authHandler.ChangePassword)
	mux.HandleFunc("GET /api/v1/me", authHandler.Me)
	mux.HandleFunc("PATCH /api/v1/me", authHandler.UpdateMe)
	mux.HandleFunc("GET /api/v1/me/permissions", singleWorkspace(authHandler.MyPermissions))
	mux.HandleFunc("GET /api/v1/users", singleWorkspace(authHandler.WorkspaceUsers))
	preferencesRepo := repository.DashboardPreferencesRepository{DB: workspaceRepo.DB}
	preferencesHandler := api.DashboardPreferencesHandler{Repository: preferencesRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/dashboard/preferences", singleWorkspace(preferencesHandler.Get))
	mux.HandleFunc("PUT /api/v1/dashboard/preferences", singleWorkspace(preferencesHandler.Put))
	dashboardAnalyticsRepo := repository.DashboardAnalyticsRepository{DB: workspaceRepo.DB}
	dashboardAnalyticsHandler := api.DashboardAnalyticsHandler{
		Repository: dashboardAnalyticsRepo,
		Authorizer: workspaceAuthorizer,
	}
	mux.HandleFunc("GET /api/v1/dashboard/executive-overview", singleWorkspace(dashboardAnalyticsHandler.ExecutiveOverview))
	mux.HandleFunc("GET /api/v1/dashboard/overtime-breakdown", singleWorkspace(dashboardAnalyticsHandler.OvertimeBreakdown))
	mux.HandleFunc("GET /api/v1/dashboard/workspace-stats", singleWorkspace(dashboardAnalyticsHandler.WorkspaceStats))
	mux.HandleFunc("GET /api/v1/dashboard/attendance-trend", singleWorkspace(dashboardAnalyticsHandler.AttendanceTrend))
	mux.HandleFunc("GET /api/v1/dashboard/department-attendance", singleWorkspace(dashboardAnalyticsHandler.DepartmentAttendance))
	mux.HandleFunc("GET /api/v1/dashboard/unexcused-absences", singleWorkspace(dashboardAnalyticsHandler.UnexcusedAbsences))
	mux.HandleFunc("GET /api/v1/dashboard/overtime-trend", singleWorkspace(dashboardAnalyticsHandler.OvertimeTrend))
	mux.HandleFunc("GET /api/v1/dashboard/peak-times", singleWorkspace(dashboardAnalyticsHandler.PeakTimes))
	mux.HandleFunc("GET /api/v1/dashboard/leave-types", singleWorkspace(dashboardAnalyticsHandler.LeaveTypes))
	mux.HandleFunc("GET /api/v1/dashboard/lateness-by-day", singleWorkspace(dashboardAnalyticsHandler.LatenessByDay))
	mux.HandleFunc("GET /api/v1/dashboard/turnover-analytics", singleWorkspace(dashboardAnalyticsHandler.TurnoverAnalytics))
	mux.HandleFunc("GET /api/v1/dashboard/recruitment-effectiveness", singleWorkspace(dashboardAnalyticsHandler.RecruitmentEffectiveness))
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
