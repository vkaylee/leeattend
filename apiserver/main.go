package main

import (
	"context"
	"fmt"
	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/app"
	"github.com/vkaylee/leeattend/apiserver/internal/config"
	dbpool "github.com/vkaylee/leeattend/apiserver/internal/db/pool"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/external"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/health"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
	"github.com/vkaylee/leeattend/apiserver/internal/observability"
	"github.com/vkaylee/leeattend/apiserver/internal/repository"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
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

	dbCfg := dbpool.DefaultConfig()
	dbCfg.PrimaryURL = string(cfg.DBPrimaryURL)
	dbCfg.PoolerTxURL = string(cfg.DBPoolerTxURL)
	dbCfg.PoolerSessURL = string(cfg.DBPoolerSessURL)
	dbCfg.TenantPoolerTxURL = string(cfg.DBTenantPoolerTxURL)
	dbCfg.TenantPoolerSessURL = string(cfg.DBTenantPoolerSessURL)
	dbCfg.ReaderURL = string(cfg.DBReaderURL)
	if cfg.DBAPIMaxConn > 0 {
		dbCfg.APIMaxConn = int32(cfg.DBAPIMaxConn)
	}
	if cfg.DBTenantMaxConn > 0 {
		dbCfg.TenantMaxConn = int32(cfg.DBTenantMaxConn)
	}

	poolMgr := dbpool.NewManager(dbCfg, nil)
	defer poolMgr.Close()
	var authRepo repository.AuthRepository
	if err := poolMgr.InitSystemPools(context.Background()); err != nil {
		logger.Warn("database connection deferred or offline", "error", err)
	} else if sysPool, err := poolMgr.SystemPool(); err == nil {
		authRepo = repository.AuthRepository{DB: sysPool}
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
	workspaceAuthorizer := repository.WorkspaceAuthorizer{DB: workspaceRepo.SystemDB}
	workspaceHandler := api.WorkspaceHandler{Repository: workspaceRepo, Authorizer: workspaceAuthorizer}
	authHandler.Authorizer = workspaceAuthorizer

	mux.HandleFunc("POST /api/v1/workspaces", workspaceHandler.Create)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}", workspaceHandler.Patch)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/roles", workspaceHandler.CreateRole)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags", workspaceHandler.ListFeatureFlags)
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
	mux.HandleFunc("GET /api/v1/workspaces", workspaceHandler.List)
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
	employeeRepo := repository.EmployeeRepository{TenantDB: workspaceRepo.TenantDB}
	employeeHandler := api.EmployeeHandler{Repository: employeeRepo, Authorizer: workspaceAuthorizer}
	attendanceRepo := repository.AttendanceRepository{TenantDB: workspaceRepo.TenantDB}
	attendanceHandler := api.AttendanceHandler{Repository: attendanceRepo, Authorizer: workspaceAuthorizer}
	shiftRepo := repository.ShiftRepository{TenantDB: workspaceRepo.TenantDB}
	shiftHandler := api.ShiftHandler{Repository: shiftRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/time/punch-in", attendanceHandler.PunchIn)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/time/punch-out/{employee_id}", attendanceHandler.PunchOut)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/time/history/{employee_id}", attendanceHandler.TimeHistory)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/attendance/adjustments", attendanceHandler.CreateAdjustment)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/attendance/adjustments", attendanceHandler.ListAdjustments)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/attendance/adjustments/{id}/approve", attendanceHandler.ApproveAdjustment)
	mux.HandleFunc("GET /api/v1/employees/{id}/attendance/adjustments", attendanceHandler.ListEmployeeAdjustments)
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
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Delete)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rehire", employeeHandler.Rehire)
	customFieldRepo := repository.CustomFieldRepository{TenantDB: workspaceRepo.TenantDB}
	customFieldHandler := api.CustomFieldHandler{Repository: customFieldRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/definitions", customFieldHandler.ListDefinitions)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/custom-fields/definitions", customFieldHandler.CreateDefinition)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.GetDefinition)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.UpdateDefinition)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/custom-fields/definitions/{id}", customFieldHandler.DeleteDefinition)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.GetValues)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/custom-fields/values/{entity_type}/{entity_id}", customFieldHandler.SetValues)
	leaveRepo := repository.LeaveRepository{TenantDB: workspaceRepo.TenantDB}
	leaveHandler := api.LeaveHandler{Repository: leaveRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/types", leaveHandler.ListTypes)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/types", leaveHandler.CreateType)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/submit", leaveHandler.Submit)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/leave/requests", leaveHandler.ListRequests)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/leave/requests/{id}/approve", leaveHandler.Approve)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{id}/leave/balances", leaveHandler.Balances)
	essRepo := repository.ESSRepository{TenantDB: workspaceRepo.TenantDB}
	essHandler := api.ESSHandler{Repository: essRepo}
	mux.HandleFunc("GET /api/v1/ess/profile", essHandler.GetProfile)
	mux.HandleFunc("PUT /api/v1/ess/profile", essHandler.UpdateProfile)
	mux.HandleFunc("GET /api/v1/ess/emergency-contacts", essHandler.GetContacts)
	mux.HandleFunc("POST /api/v1/ess/emergency-contacts", essHandler.AddContact)
	mux.HandleFunc("DELETE /api/v1/ess/emergency-contacts/{contact_id}", essHandler.DeleteContact)
	workflowRepo := repository.ApprovalWorkflowRepository{TenantDB: workspaceRepo.TenantDB}
	workflowHandler := api.ApprovalWorkflowHandler{Repository: workflowRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows", workflowHandler.List)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/approval-workflows", workflowHandler.Create)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/approval-workflows/approve", workflowHandler.Process)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/approval-workflows/history/{type}/{id}", workflowHandler.History)
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
	payrollRepo := repository.PayrollRepository{TenantDB: workspaceRepo.TenantDB}
	payrollHandler := api.PayrollHandler{Repository: payrollRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/payroll/payslips", payrollHandler.ListWorkspace)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees/{employee_id}/payslips", payrollHandler.ListEmployee)
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
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("GET /v1/workspaces/{ws_id}/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("DELETE /v1/workspaces/{ws_id}/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/files/presigned", fileHandler.CreateUploadURL)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.GetMetadata)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/files/{id}/url", fileHandler.GetURL)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/files/{id}", fileHandler.DeleteFile)
	mux.HandleFunc("POST /v1/workspaces/{ws_id}/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("GET /v1/ess/workspaces/{ws_id}/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("POST /v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign", fileHandler.SignDocument)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/documents/{document_id}/signature-requests", fileHandler.CreateSignatureRequest)
	mux.HandleFunc("GET /api/v1/ess/workspaces/{ws_id}/signatures/pending", fileHandler.ListPendingSignatures)
	mux.HandleFunc("POST /api/v1/ess/workspaces/{ws_id}/signatures/{request_id}/sign", fileHandler.SignDocument)
	conditionRepo := repository.ConditionTypeRepository{TenantDB: workspaceRepo.TenantDB}
	conditionHandler := api.ConditionTypeHandler{Repository: conditionRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/condition-types", conditionHandler.List)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Get)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/condition-types", conditionHandler.Create)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/condition-types/{id}", conditionHandler.Delete)

	mux.HandleFunc("GET /api/v1/health", healthHandler.Liveness)
	mux.HandleFunc("GET /api/v1/health/ready", healthHandler.Readiness)

	mux.HandleFunc("POST /api/v1/auth/login", authHandler.Login)
	mux.HandleFunc("POST /api/v1/auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /api/v1/auth/logout", authHandler.Logout)
	mux.HandleFunc("POST /api/v1/auth/register", authHandler.Register)
	mux.HandleFunc("POST /api/v1/auth/forgot-password", authHandler.ForgotPassword)
	mux.HandleFunc("POST /api/v1/auth/reset-password", authHandler.ResetPassword)
	mux.HandleFunc("POST /api/v1/auth/change-password", authHandler.ChangePassword)
	mux.HandleFunc("GET /api/v1/me", authHandler.Me)
	mux.HandleFunc("PATCH /api/v1/me", authHandler.UpdateMe)
	mux.HandleFunc("GET /api/v1/me/permissions", authHandler.MyPermissions)
	mux.HandleFunc("GET /api/v1/users", authHandler.WorkspaceUsers)
	mux.HandleFunc("POST /auth/login", authHandler.Login)
	mux.HandleFunc("POST /auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /auth/logout", authHandler.Logout)
	mux.HandleFunc("POST /auth/register", authHandler.Register)
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
		"/api/v1/auth/register",
		"/api/v1/auth/forgot-password",
		"/api/v1/auth/reset-password",
		"/auth/register",
		"/auth/forgot-password",
		"/auth/reset-password",
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

	application := app.New(server)
	ctx, cancel := app.NotifySignals(context.Background())
	defer cancel()

	if err := application.Run(ctx); err != nil {
		logger.Error("server fatal", "error", err)
		os.Exit(1)
	}
}
