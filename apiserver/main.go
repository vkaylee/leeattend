package main

import (
	"context"
	"fmt"
	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/app"
	"github.com/vkaylee/leeattend/apiserver/internal/config"
	dbpool "github.com/vkaylee/leeattend/apiserver/internal/db/pool"
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

	mux.HandleFunc("POST /api/v1/workspaces", workspaceHandler.Create)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}", workspaceHandler.Patch)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/roles", workspaceHandler.CreateRole)
	mux.HandleFunc("PATCH /api/v1/workspaces/{ws_id}/feature-flags/{id}", workspaceHandler.UpdateFeatureFlag)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/feature-flags/check/{key}", workspaceHandler.CheckFeatureFlag)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/configs", workspaceHandler.PutConfig)
	healthHandler := health.NewHandler(Version, nil, nil)
	healthHandler.SetLive(true)
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
	patternHandler := api.WorkingPatternHandler{Repository: patternRepo, Authorizer: workspaceAuthorizer}
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/working-patterns", patternHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/working-patterns", patternHandler.List)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Details)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Update)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/working-patterns/{pattern_id}", patternHandler.Delete)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employments/{employment_id}/working-patterns", patternHandler.Assign)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/working-patterns/bulk-assign", patternHandler.BulkAssign)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employments/{employment_id}/working-patterns", patternHandler.History)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees", employeeHandler.Create)
	mux.HandleFunc("GET /api/v1/workspaces/{ws_id}/employees", employeeHandler.List)
	mux.HandleFunc("DELETE /api/v1/workspaces/{ws_id}/employees/{id}", employeeHandler.Delete)
	mux.HandleFunc("POST /api/v1/workspaces/{ws_id}/employees/{id}/rehire", employeeHandler.Rehire)
	mux.HandleFunc("PUT /api/v1/workspaces/{ws_id}/custom-fields/values/employee/{entity_id}", employeeHandler.CustomFields)

	mux.HandleFunc("GET /api/v1/health", healthHandler.Liveness)

	mux.HandleFunc("POST /api/v1/auth/login", authHandler.Login)
	mux.HandleFunc("POST /api/v1/auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /api/v1/auth/logout", authHandler.Logout)
	mux.HandleFunc("GET /api/v1/me", authHandler.Me)
	mux.HandleFunc("POST /auth/login", authHandler.Login)
	mux.HandleFunc("POST /auth/refresh", authHandler.Refresh)
	mux.HandleFunc("POST /auth/logout", authHandler.Logout)
	mux.HandleFunc("GET /me", authHandler.Me)

	publicPrefixes := []string{
		"/health",
		"/api/v1/health",
		"/api/v1/auth/login",
		"/api/v1/auth/refresh",
		"/auth/login",
		"/auth/refresh",
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
