use apiserver::db_pool::{DbPoolConfig, DbPools};
use apiserver::openapi::ApiDoc;
use apiserver::queue::RedisPool;
use apiserver::services::{AttendanceLogService, ReportCoordinator};
use apiserver::worker::{JobWorker, WorkerConfig};
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use std::sync::Arc;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;

struct PortFileGuard {
    filename: String,
}

impl Drop for PortFileGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.filename) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Failed to remove {} file on shutdown: {}", self.filename, e);
            }
        } else {
            eprintln!("{} file cleaned up successfully.", self.filename);
        }
    }
}

#[tokio::main]
async fn main() {
    // Check for openapi generation flag before everything else
    if std::env::args().any(|arg| arg == "gen-openapi") {
        let openapi_json = ApiDoc::openapi()
            .to_pretty_json()
            .expect("Failed to generate OpenAPI JSON");
        println!("{openapi_json}");
        return;
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apiserver=debug,tower_http=debug,sqlx=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize configurable database pools (API + Worker + optional Reader)
    let db_config = DbPoolConfig::from_env();
    let db_pools = DbPools::connect(&db_config).await;
    db_pools.run_migrations().await;

    // ╔═══════════════════════════════════════════════════════════════╗
    // ║  POOL ASSIGNMENT RULES                                       ║
    // ║                                                               ║
    // ║  api pool    → HTTP handlers, CRUD, auth, repos in AppState   ║
    // ║  worker pool → background jobs, import, accrual, log flush    ║
    // ║  reader pool → dashboard analytics, ESS read-only queries     ║
    // ║                                                               ║
    // ║  ⚠  DO NOT pass worker_pool to API services                   ║
    // ║  ⚠  DO NOT use state.infra.db.reader() for mutations (INSERT/etc)   ║
    // ╚═══════════════════════════════════════════════════════════════╝

    // Use API pool for application-level initialization (HTTP handlers, CRUD)
    let pool = db_pools.api().clone();
    // Use Worker pool for background services (patient timeout, won't starve API)
    let worker_pool = db_pools.worker().clone();

    // resume_all_pending_jobs moved to after import_service construction below

    #[cfg(debug_assertions)]
    {
        apiserver::dev_initialization::ensure_dev_data(&pool)
            .await
            .expect("Failed to seed dev data");
    }

    // Initialize Redis for distributed job processing and presence tracking (optional)
    tracing::info!("Initializing Redis connection...");
    let redis_pool = init_redis().await;
    tracing::info!("Redis initialization complete");

    let agent_manager = AgentManager::new(
        redis_pool
            .clone()
            .map(|p| Arc::new(p) as Arc<dyn apiserver::agent_manager::PresenceStore>),
    );
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(pool.clone()));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository::new(pool.clone()));
    let audit_repo =
        Arc::new(apiserver::repositories::audit_log_repository::SqlxAuditLogRepository::new(pool.clone()));
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));
    let employee_repo =
        Arc::new(apiserver::repositories::SqlxEmployeeRepository::new(pool.clone()));
    let employee_service = Arc::new(apiserver::services::EmployeeService::new(
        employee_repo.clone(),
        pool.clone(),
    ));
    let time_entry_repo =
        Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone()));
    let roster_repo: Arc<dyn apiserver::repositories::roster_repository::RosterRepository> =
        Arc::new(apiserver::repositories::roster_repository::SqlxRosterRepository::new(pool.clone()));
    let time_tracking_service = Arc::new(apiserver::services::TimeTrackingService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let attendance_summary_service = Arc::new(
        apiserver::services::AttendanceSummaryService::new(
            time_entry_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let report_service = Arc::new(apiserver::services::ReportService::new(
        time_entry_repo.clone(),
        roster_repo.clone(),
        pool.clone(),
    ));
    let pairing_service = Arc::new(
        apiserver::services::pairing_service::PairingService::new(
            time_entry_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    let adj_repo = Arc::new(
        apiserver::repositories::attendance_adjustment_repository::SqlxAttendanceAdjustmentRepository::new(pool.clone()),
    );
    let adjustment_service = Arc::new(
        apiserver::services::attendance_adjustment_service::AttendanceAdjustmentService::new(
            adj_repo.clone(),
            time_entry_repo.clone(),
            pool.clone(),
        ),
    );
    let import_repo = Arc::new(
        apiserver::repositories::import_job_repository::SqlxImportJobRepository::new(pool.clone()),
    );
    let import_service = Arc::new(
        apiserver::services::attendance_import_service::AttendanceImportService::new(
            import_repo.clone(),
            adj_repo.clone(),
            employee_repo.clone(),
            time_entry_repo.clone(),
            worker_pool.clone(),
        ),
    );
    import_service.resume_all_pending_jobs().await;

    let auth_service = Arc::new(
        apiserver::services::auth_service::AuthService::new(pool.clone()),
    );
    let shift_service = Arc::new(
        apiserver::services::shift_service::ShiftService::new(pool.clone()),
    );
    let workspace_service = Arc::new(
        apiserver::services::workspace_service::WorkspaceService::new(pool.clone()),
    );
    let leave_balance_repo = Arc::new(
        apiserver::repositories::leave_balance_repository::SqlxLeaveBalanceRepository::new(pool.clone()),
    );
    let leave_request_repo = Arc::new(
        apiserver::repositories::leave_request_repository::SqlxLeaveRequestRepository::new(pool.clone()),
    );
    let leave_service = Arc::new(
        apiserver::services::leave_service::LeaveService::new(
            leave_balance_repo.clone(),
            leave_request_repo.clone(),
            employee_repo.clone(),
            roster_repo.clone(),
            pool.clone(),
        ),
    );
    // roster_repo already created earlier (after time_entry_repo)
    let shift_swap_repo = Arc::new(
        apiserver::repositories::shift_swap_repository::SqlxShiftSwapRepository::new(pool.clone()),
    );
    let shift_swap_service = Arc::new(
        apiserver::services::shift_swap_service::ShiftSwapService::new(shift_swap_repo.clone(), roster_repo.clone(), pool.clone()),
    );
    let holiday_leave_service = Arc::new(
        apiserver::services::holiday_leave_service::HolidayLeaveService::new(
            employee_repo.clone(), leave_balance_repo.clone(), leave_request_repo.clone(), roster_repo.clone(), pool.clone(),
        ),
    );
    let department_repo: Arc<dyn apiserver::repositories::department_repository::DepartmentRepository> =
        Arc::new(apiserver::repositories::department_repository::SqlxDepartmentRepository::new(pool.clone()));
    let leave_policy_repo: Arc<dyn apiserver::repositories::leave_policy_repository::LeavePolicyRepository> =
        Arc::new(apiserver::repositories::leave_policy_repository::SqlxLeavePolicyRepository::new(pool.clone()));
    let leave_accrual_service = Arc::new(
        apiserver::services::leave_accrual_service::LeaveAccrualService::new(
            employee_repo.clone(), department_repo.clone(), leave_policy_repo.clone(), worker_pool.clone(),
        ),
    );
    let payroll_repo: Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
        Arc::new(apiserver::repositories::payroll_repository::SqlxPayrollRepository::new(pool.clone()));
    let holiday_repo: Arc<dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait> =
        Arc::new(apiserver::repositories::holiday_repository::SqlxHolidayRepository::new(pool.clone()));
    let payroll_service = Arc::new(
        apiserver::services::payroll_service::PayrollService::new(
            payroll_repo.clone(), department_repo.clone(), time_entry_repo.clone(),
            workspace_repo.clone(), holiday_repo.clone(), roster_repo.clone(), pool.clone(),
        ),
    );
    let jwt_secret = if cfg!(debug_assertions) {
        std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!("⚠️ JWT_SECRET not set — using insecure dev fallback. DO NOT use in production!");
            "dev_secret_key_leeattend_2026".to_string()
        })
    } else {
        std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in production")
    };

    let buffer_capacity: usize = std::env::var("AGENT_LOG_BUFFER_CAPACITY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);
    let batch_size: usize = std::env::var("AGENT_LOG_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let flush_interval_ms: u64 = std::env::var("AGENT_LOG_FLUSH_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    let raw_log_repo: Arc<dyn apiserver::repositories::raw_attendance_log_repository::RawAttendanceLogRepository> =
        Arc::new(apiserver::repositories::raw_attendance_log_repository::SqlxRawAttendanceLogRepository::new(pool.clone()));
    let staging_repo: Arc<dyn apiserver::repositories::staging_log_repository::StagingLogRepository> =
        Arc::new(apiserver::repositories::staging_log_repository::SqlxStagingLogRepository::new(pool.clone()));
    let error_repo: Arc<dyn apiserver::repositories::attendance_log_error_repository::AttendanceLogErrorRepository> =
        Arc::new(apiserver::repositories::attendance_log_error_repository::SqlxAttendanceLogErrorRepository::new(pool.clone()));

    let (attendance_service, log_worker_handle) =
        AttendanceLogService::new(worker_pool.clone(), pairing_service.clone(), buffer_capacity, batch_size, flush_interval_ms,
            employee_repo.clone(), raw_log_repo.clone(), staging_repo.clone(), error_repo.clone());
    let attendance_service = Arc::new(attendance_service);

    // Roster cache (requires Redis; shared between app state and worker)
    let roster_cache = redis_pool
        .as_ref()
        .map(|rpool| Arc::new(apiserver::cache::RosterCache::new(rpool.clone())));

    // Initialize job worker if Redis is available
    let worker_cancel_token = CancellationToken::new();
    let (coordinator, worker_handle) = if let Some(ref redis) = redis_pool {
        let worker_config = WorkerConfig::from_env();
        tracing::info!(
            max_concurrent = worker_config.max_concurrent_jobs,
            poll_timeout = worker_config.poll_timeout_secs,
            "Initializing job worker"
        );

        let coordinator = Arc::new(ReportCoordinator::new(redis.clone(), pool.clone()));

        let worker = JobWorker::new(
            redis.clone(),
            db_pools.worker().clone(),
            worker_config,
            coordinator.clone(),
            roster_cache.clone(),
        );
        (
            Some(coordinator),
            Some(worker.start(worker_cancel_token.clone())),
        )
    } else {
        (None, None)
    };

    let user_repo: Arc<dyn apiserver::repositories::user_repository::UserRepository> =
        Arc::new(apiserver::repositories::user_repository::SqlxUserRepository::new(pool.clone()));

    // roster_repo already created earlier (before shift_swap_service / payroll_service)

    let state = Arc::new(AppState {
        employees: apiserver::EmployeeState {
            service: employee_service,
            repo: employee_repo,
        },
        attendance: apiserver::AttendanceState {
            time_tracking_service,
            summary_service: attendance_summary_service,
            log_service: attendance_service,
            pairing_service,
            adjustment_service,
            import_service,
            time_entry_repo,
            staging_repo,
            raw_log_repo,
            import_repo,
        },
        shifts: apiserver::ShiftState {
            service: shift_service,
            swap_service: shift_swap_service,
            roster_repo,
            swap_repo: shift_swap_repo,
        },
        leaves: apiserver::LeaveState {
            service: leave_service,
            holiday_leave_service,
            accrual_service: leave_accrual_service,
            request_repo: leave_request_repo,
            policy_repo: leave_policy_repo,
            holiday_repo,
        },
        payroll: apiserver::PayrollState {
            service: payroll_service,
            repo: payroll_repo,
            department_repo,
        },
        reports: apiserver::ReportState {
            service: report_service,
            coordinator,
        },
        core: apiserver::CoreState {
            auth_service,
            workspace_service,
            workspace_repo,
            audit_repo,
            user_repo,
            jwt_secret,
        },
        agents: apiserver::AgentState {
            manager: agent_manager,
            service: agent_service,
            repo: agent_repo,
        },
        infra: apiserver::InfraState {
            db: db_pools,
            redis: redis_pool,
            roster_cache,
        },
    });

    let app = create_app(state.clone());

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let host = host.trim().to_string();
    let preferred_port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let port_file_name =
        std::env::var("PORT_FILE_NAME").unwrap_or_else(|_| ".api_port".to_string());
    let _ = std::fs::remove_file(&port_file_name);

    let listener = {
        let mut listener = None;
        let max_port = preferred_port + 10;

        for port in preferred_port..max_port {
            let addr_str = format!("{host}:{port}");
            match tokio::net::TcpListener::bind(&addr_str).await {
                Ok(l) => {
                    listener = Some(l);
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    tracing::warn!("Port {} is in use, trying next port", port);
                    continue;
                }
                Err(e) => panic!("Failed to bind to {addr_str}: {e}"),
            }
        }
        listener.expect("Could not find an available port")
    };

    let actual_addr = listener
        .local_addr()
        .expect("Failed to get listener local address");
    if std::fs::write(&port_file_name, actual_addr.port().to_string()).is_ok() {
        tracing::info!("Actual listening on {}", actual_addr);
    }

    let _guard = PortFileGuard {
        filename: port_file_name,
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server exited with error");

    // Graceful shutdown sequence
    tracing::info!("Starting graceful shutdown...");

    // Cancel worker tokens first
    worker_cancel_token.cancel();
    if let Some(handle) = worker_handle {
        tracing::info!("Waiting for job worker to complete...");
        if let Err(e) = handle.await {
            tracing::error!("Job worker task panicked: {:?}", e);
        }
    }

    state.agents.manager.shutdown().await;

    drop(state);

    tracing::info!("Waiting for log worker to drain...");
    if let Err(e) = log_worker_handle.await {
        tracing::error!("Log worker task panicked: {:?}", e);
    }

    tracing::info!("Server stopped.");
}

async fn init_redis() -> Option<RedisPool> {
    let url = match std::env::var("REDIS_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::info!("REDIS_URL not set, distributed processing and presence tracking disabled");
            return None;
        }
    };

    tracing::info!("Connecting to Redis at {}", url);

    let pool = match timeout(std::time::Duration::from_secs(5), RedisPool::new(&url)).await {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            tracing::warn!("Redis connection failed: {}", e);
            return None;
        }
        Err(_) => {
            tracing::warn!("Redis connection timed out");
            return None;
        }
    };

    tracing::info!("Redis pool created, checking health...");
    match timeout(std::time::Duration::from_secs(5), pool.health_check()).await {
        Ok(Ok(())) => {
            tracing::info!("Redis connected for distributed processing and presence");
            Some(pool)
        }
        Ok(Err(e)) => {
            tracing::warn!("Redis health check failed: {}", e);
            None
        }
        Err(_) => {
            tracing::warn!("Redis health check timed out");
            None
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
