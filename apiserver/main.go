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

	db := func() repository.DBTX {
		pool, err := poolMgr.SystemPool()
		if err != nil {
			return nil
		}
		return pool
	}()
	handler := buildRouter(cfg, db, redisClient, authHandler, logger)

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
