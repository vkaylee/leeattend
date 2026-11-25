package main

import (
	"context"
	"log/slog"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/config"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/crypto"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/external"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/health"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
	"github.com/vkaylee/leeattend/apiserver/internal/repository"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

func buildRouter(cfg *config.Config, db repository.DBTX, redisClient *redisinfra.Client, authHandler *api.AuthHandler, logger *slog.Logger) http.Handler {

	router := chi.NewRouter()

	var revocationChecker httpinfra.TokenRevocationChecker
	var rateLimiter httpinfra.RateLimiterBackend
	if redisClient != nil {
		revocationChecker = redisClient
		rateLimiter = redisClient
	}
	protectedRouter := router.With(httpinfra.AuthMiddlewareWithRevocation(cfg.JWTSecret.Expose(), nil, revocationChecker))

	organizationAuthorizer := repository.OrganizationAuthorizer{DB: db}
	organizationRepo := repository.OrganizationRepository{DB: db}
	organizationHandler := &api.OrganizationHandler{
		Repository: organizationRepo,
		Authorizer: organizationAuthorizer,
		Audit:      repository.AuditLogWriter{DB: db},
	}
	var cryptoService *crypto.Service
	if encJSON := cfg.EncryptionKeysJSON.Expose(); encJSON != "" {
		if cs, err := crypto.FromJSON(encJSON); err == nil {
			cryptoService = cs
		} else {
			logger.Warn("crypto service initialization failed", "error", err)
		}
	}
	rbacRepo := repository.OrganizationRBACRepository{SystemDB: db}
	rbacHandler := &api.OrganizationRBACHandler{
		Repository:    rbacRepo,
		Authorizer:    organizationAuthorizer,
		Crypto:        cryptoService,
		StorageTester: api.NewS3StorageTester(cfg.S3EndpointURL),
	}
	authHandler.Authorizer = organizationAuthorizer

	registerOrganizationRoutes(protectedRouter, organizationHandler, rbacHandler)
	healthHandler := health.NewHandler(Version, nil, nil)
	healthHandler.SetLive(true)
	departmentRepo := repository.DepartmentRepository{DB: db}
	departmentHandler := &api.DepartmentHandler{Repository: departmentRepo, Authorizer: organizationAuthorizer}

	locationRepo := repository.LocationRepository{DB: db}
	locationHandler := &api.LocationHandler{Repository: locationRepo, Authorizer: organizationAuthorizer}
	employerRepo := repository.EmployerRepository{DB: db}
	employerHandler := &api.EmployerHandler{Repository: employerRepo, Authorizer: organizationAuthorizer}
	recruitmentRepo := repository.RecruitmentSourceRepository{DB: db}
	recruitmentHandler := &api.RecruitmentSourceHandler{Repository: recruitmentRepo, Authorizer: organizationAuthorizer}
	registerPeopleRoutes(protectedRouter, departmentHandler, locationHandler, employerHandler, recruitmentHandler)
	employeeRepo := repository.EmployeeRepository{DB: db}
	employeeIdentityRepo := repository.EmployeeIdentityRepository{DB: db}
	identityHasher, _ := identity.NewHasher(cfg.IdentityLookupHMACSecret.Expose())
	employeeHandler := &api.EmployeeHandler{
		Repository:      employeeRepo,
		TenureCreator:   employeeRepo,
		IdentityMatcher: employeeIdentityRepo,
		IdentityHasher:  identityHasher,
		Authorizer:      organizationAuthorizer,
		Audit:           repository.AuditLogWriter{DB: db},
	}
	employeeImportRepo := repository.EmployeeImportRepository{DB: db}
	employeeImportHandler := &api.EmployeeImportHandler{
		Repository: employeeImportRepo,
		Queue: repository.EmployeeImportQueue{
			RedisClient: redisClient,
			Repo:        employeeImportRepo,
		},
		Authorizer: organizationAuthorizer,
		StorageDir: cfg.StorageDir,
	}
	employeeConditionRepo := repository.EmployeeConditionRepository{DB: db}
	employeeConditionHandler := &api.EmployeeConditionHandler{Repository: employeeConditionRepo, Authorizer: organizationAuthorizer}
	employeeDepartmentRepo := repository.EmployeeDepartmentRepository{DB: db}
	employeeDepartmentHandler := &api.EmployeeDepartmentHandler{Repository: employeeDepartmentRepo, Authorizer: organizationAuthorizer}
	compensationHistoryRepo := repository.CompensationHistoryRepository{DB: db}
	compensationHistoryHandler := &api.CompensationHistoryHandler{Repository: compensationHistoryRepo, Authorizer: organizationAuthorizer}
	attendanceRepo := repository.AttendanceRepository{DB: db}
	attendanceHandler := &api.AttendanceHandler{Repository: attendanceRepo, Authorizer: organizationAuthorizer}
	attendanceLogsHandler := &api.AttendanceLogsHandler{Repository: attendanceRepo, Authorizer: organizationAuthorizer, StorageDir: cfg.StorageDir}
	attendancePolicyRepo := repository.AttendancePolicyRepository{DB: db, SystemDB: db}
	attendancePolicyHandler := &api.AttendancePolicyHandler{PolicyRepository: attendancePolicyRepo, BehalfRepository: attendancePolicyRepo, Authorizer: organizationAuthorizer}
	overtimeRepo := repository.OvertimeRepository{DB: db}
	overtimeHandler := &api.OvertimeHandler{Repository: overtimeRepo, BehalfRepository: attendancePolicyRepo, Authorizer: organizationAuthorizer}
	registerOvertimeRoutes(protectedRouter, overtimeHandler)
	shiftRepo := repository.ShiftRepository{DB: db}
	shiftHandler := &api.ShiftHandler{Repository: shiftRepo, Authorizer: organizationAuthorizer}
	employeeAssignmentRepo := repository.EmployeeAssignmentRepository{DB: db}
	assignmentRuleRepo := repository.AssignmentRuleRepository{DB: db}
	assignmentHistoryRepo := repository.AssignmentHistoryRepository{DB: db}
	assignmentHandler := &api.AssignmentHandler{Assignments: employeeAssignmentRepo, Rules: assignmentRuleRepo, History: assignmentHistoryRepo, Employees: employeeRepo, Authorizer: organizationAuthorizer}
	registerAttendanceRoutes(protectedRouter, attendanceHandler, attendanceLogsHandler, attendancePolicyHandler)
	patternRepo := repository.WorkingPatternRepository{DB: db}
	patternHandler := &api.WorkingPatternHandler{Repository: patternRepo, Authorizer: organizationAuthorizer, Compliance: patternRepo}
	rosterRepo := repository.RosterRepository{DB: db}
	rosterHandler := &api.RosterHandler{Repository: rosterRepo, Authorizer: organizationAuthorizer}
	scheduleRepo := repository.ScheduleRepository{DB: db}
	scheduleHandler := &api.ScheduleHandler{Repository: scheduleRepo, Lifecycle: scheduleRepo, Shadow: scheduleRepo, Activation: scheduleRepo, Authorizer: organizationAuthorizer}
	swapRepo := repository.ShiftSwapRepository{DB: db}
	swapHandler := &api.ShiftSwapHandler{Repository: swapRepo, Authorizer: organizationAuthorizer}
	segmentRepo := repository.WorkSegmentRepository{DB: db}
	segmentHandler := &api.WorkSegmentHandler{Repository: segmentRepo, Authorizer: organizationAuthorizer}
	registerSchedulingRoutes(protectedRouter, shiftHandler, patternHandler, rosterHandler, scheduleHandler, swapHandler, segmentHandler)
	registerEmployeeRoutes(protectedRouter, employeeHandler, employeeImportHandler, employeeConditionHandler, employeeDepartmentHandler, compensationHistoryHandler)
	customFieldRepo := repository.CustomFieldRepository{DB: db}
	customFieldHandler := &api.CustomFieldHandler{Repository: customFieldRepo, Authorizer: organizationAuthorizer}
	contractRepo := repository.ContractRepository{DB: db}
	employmentRepo := repository.EmploymentRepository{DB: db}
	contractEmploymentHandler := &api.ContractEmploymentHandler{Contracts: contractRepo, Employments: employmentRepo, CustomFields: customFieldRepo, Authorizer: organizationAuthorizer}
	registerEmploymentRoutes(protectedRouter, contractEmploymentHandler, assignmentHandler, customFieldHandler)
	leaveRepo := repository.LeaveRepository{DB: db}
	leaveHandler := &api.LeaveHandler{
		Repository:       leaveRepo,
		BehalfRepository: attendancePolicyRepo,
		Authorizer:       organizationAuthorizer,
		StorageDir:       cfg.StorageDir,
	}
	holidayRepo := repository.HolidayRepository{DB: db}
	holidayHandler := &api.HolidayHandler{Repository: holidayRepo, Authorizer: organizationAuthorizer}
	registerLeaveRoutes(protectedRouter, leaveHandler, holidayHandler)
	payrollRepo := repository.PayrollRepository{DB: db}
	essRepo := repository.ESSRepository{DB: db}
	essHandler := &api.ESSHandler{
		Authorizer: organizationAuthorizer,
		Repository: essRepo,
		Attendance: attendanceRepo,
		Payroll:    payrollRepo,
		Leaves:     leaveRepo,
		ShiftSwaps: swapRepo,
	}
	registerESSRoutes(protectedRouter.With(httpinfra.FeatureFlagMiddleware("ff_ess", func(ctx context.Context, key string) (bool, error) {
		flag, err := organizationRepo.GetFeatureFlag(ctx, key)
		if err != nil {
			return false, err
		}
		return flag.Enabled, nil
	})), essHandler)
	workflowRepo := repository.ApprovalWorkflowRepository{DB: db, SystemDB: db}
	workflowHandler := &api.ApprovalWorkflowHandler{Repository: workflowRepo, Authorizer: organizationAuthorizer}
	registerWorkflowRoutes(protectedRouter, workflowHandler)
	notifRepo := repository.NotificationRepository{SystemDB: db}
	notifHandler := &api.NotificationHandler{Repository: notifRepo}
	registerNotificationRoutes(protectedRouter, notifHandler)
	payrollHandler := &api.PayrollHandler{
		Repository: payrollRepo,
		Authorizer: organizationAuthorizer,
		Audit:      repository.AuditLogWriter{DB: db},
	}
	registerPayrollRoutes(protectedRouter, payrollHandler)
	workPolicyRepo := repository.WorkPolicyRepository{DB: db}
	workPolicyHandler := &api.WorkPolicyGroupHandler{Repository: workPolicyRepo, Authorizer: organizationAuthorizer, Audit: repository.AuditLogWriter{DB: db}}
	registerWorkPolicyRoutes(protectedRouter, workPolicyHandler)
	reportRepo := repository.ReportRepository{DB: db}
	reportHandler := &api.ReportHandler{Repository: reportRepo, Authorizer: organizationAuthorizer}
	registerReportRoutes(protectedRouter, reportHandler)
	fileRepo := repository.FileRepository{DB: db}
	var fileStorage api.FileStorage
	if s3Client, err := external.NewS3Client(cfg.S3EndpointURL, cfg.S3BucketName, cfg.S3AccessKey.Expose(), cfg.S3SecretKey.Expose(), cfg.S3Region, cfg.S3UsePathStyle, nil); err == nil {
		fileStorage = s3Client
	}
	fileHandler := &api.FileHandler{Repository: fileRepo, Storage: fileStorage, Authorizer: organizationAuthorizer}
	registerFileRoutes(protectedRouter, fileHandler)
	documentRepo := repository.DocumentRepository{DB: db}
	documentHandler := &api.DocumentHandler{Repository: documentRepo, Authorizer: organizationAuthorizer}
	registerDocumentRoutes(protectedRouter, documentHandler)
	conditionRepo := repository.ConditionTypeRepository{DB: db}
	conditionHandler := &api.ConditionTypeHandler{Repository: conditionRepo, Authorizer: organizationAuthorizer}
	phaseRepo := repository.EmploymentPhaseRepository{DB: db}
	phaseHandler := &api.EmploymentPhaseHandler{Repository: phaseRepo, Authorizer: organizationAuthorizer}
	lifecycleRepo := repository.LifecycleRepository{DB: db}
	lifecycleHandler := &api.LifecycleHandler{
		Repository: lifecycleRepo,
		Authorizer: organizationAuthorizer,
	}
	registerLifecycleRoutes(protectedRouter, conditionHandler, phaseHandler, lifecycleHandler)

	systemHealthHandler := &api.SystemHealthHandler{Authorizer: organizationAuthorizer}

	systemSettingsRepo := repository.SystemSettingsRepository{DB: db}
	systemAdminHandler := &api.SystemAdminHandler{
		Authorizer:  organizationAuthorizer,
		Settings:    systemSettingsRepo,
		EmailSender: service.EmailService{Settings: systemSettingsRepo},
		Redis:       redisClient,
	}
	registerSystemRoutes(protectedRouter, systemHealthHandler, systemAdminHandler)

	monitorRepo := repository.MonitorRepository{DB: db, SystemDB: db}
	monitorHandler := &api.MonitorHandler{
		Repo:       monitorRepo,
		Authorizer: organizationAuthorizer,
		Redis:      redisClient,
		JWTSecret:  cfg.JWTSecret.Expose(),
	}
	registerMonitorRoutes(protectedRouter, monitorHandler)
	registerPublicRoutes(router, healthHandler)
	registerPublicAuthRoutes(router, authHandler)
	registerAuthRoutes(protectedRouter, authHandler)
	preferencesRepo := repository.DashboardPreferencesRepository{DB: db}
	preferencesHandler := &api.DashboardPreferencesHandler{Repository: preferencesRepo, Authorizer: organizationAuthorizer}
	dashboardAnalyticsRepo := repository.DashboardAnalyticsRepository{DB: db}
	dashboardAnalyticsHandler := &api.DashboardAnalyticsHandler{
		Repository: dashboardAnalyticsRepo,
		Authorizer: organizationAuthorizer,
	}
	registerDashboardRoutes(protectedRouter, preferencesHandler, dashboardAnalyticsHandler)

	rateLimitConfig := httpinfra.RateLimitConfig{
		MaxReadRequests:  uint64(cfg.RateLimitMaxRead),
		MaxWriteRequests: uint64(cfg.RateLimitMaxWrite),
		WindowSecs:       uint64(cfg.RateLimitWindowSecs),
	}
	return httpinfra.Chain(
		httpinfra.RequestIDMiddleware(),
		httpinfra.RateLimitMiddleware(rateLimitConfig, cfg.JWTSecret.Expose(), rateLimiter),
		httpinfra.LocaleMiddleware(),
		httpinfra.CORSMiddleware(cfg.AllowedOrigins),
	)(router)
}
