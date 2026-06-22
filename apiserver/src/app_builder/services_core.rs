#![allow(clippy::clone_on_ref_ptr)]
use super::repositories::Repositories;
use crate::agent_manager::AgentManager;
use crate::crypto::CryptoService;
use crate::diesel_pool::DieselPools;
use crate::queue::RedisPool;
use std::sync::Arc;

pub struct CoreServices {
    pub lifecycle: Arc<crate::services::LifecycleService>,
    pub employee: Arc<crate::services::EmployeeService>,
    pub outbox: Arc<crate::services::OutboxJobService>,
    pub auth: Arc<crate::services::auth_service::AuthService>,
    pub workspace: Arc<crate::services::workspace_service::WorkspaceService>,
    pub location: Arc<crate::services::LocationService>,
    pub employer: Arc<crate::services::EmployerService>,
    pub custom_field: Arc<crate::services::CustomFieldService>,
    pub recruitment: Arc<crate::services::RecruitmentService>,
    pub storage_resolver: Arc<crate::services::storage::resolver::StorageResolver>,
    pub file: Arc<crate::services::storage::file_service::FileService>,
    pub signature: Arc<crate::services::signature_service::SignatureService>,
    pub scan: Arc<dyn crate::services::storage::VirusScanner>,
    pub agent: Arc<crate::services::AgentService>,
    pub tenant_provisioner: Arc<dyn crate::services::tenant_provisioner::TenantProvisionerTrait>,
    pub shift: Arc<crate::services::shift_service::ShiftService>,
}

pub async fn build_core_services(
    diesel_pools: &DieselPools,
    redis_pool: &Option<RedisPool>,
    _crypto_service: &Option<Arc<CryptoService>>,
    agent_manager: &AgentManager,
    repos: &Repositories,
) -> Result<CoreServices, Box<dyn std::error::Error>> {
    let lifecycle = Arc::new(crate::services::LifecycleService::new(
        Arc::clone(&repos.lifecycle),
        Arc::clone(&repos.employee),
        Arc::clone(&repos.employment),
        Arc::clone(&repos.user),
        Arc::clone(&repos.workspace),
    ));

    let employee = Arc::new(
        crate::services::EmployeeService::new(
            Arc::clone(&repos.employee),
            Arc::clone(&repos.employment),
            Arc::clone(&repos.department),
            Arc::clone(&repos.user),
            Arc::clone(&repos.staging),
            Arc::clone(&repos.leave_balance),
        )
        .with_lifecycle(Arc::clone(&lifecycle))
        .with_payroll(Arc::clone(&repos.payroll))
        .with_employer(Arc::clone(&repos.employer)),
    );

    let outbox = Arc::new(crate::services::OutboxJobService::new(repos.outbox.clone()));

    let auth = Arc::new(crate::services::auth_service::AuthService::new(
        repos.user.clone(),
        repos.session.clone(),
        redis_pool.clone(),
    ));

    let tenant_provisioner = Arc::new(crate::services::tenant_provisioner::TenantMigrator::new(
        diesel_pools.clone(),
    ));

    let workspace = Arc::new(crate::services::workspace_service::WorkspaceService::new(
        repos.workspace.clone(),
        repos.user.clone(),
        tenant_provisioner.clone()
            as Arc<dyn crate::services::tenant_provisioner::TenantProvisionerTrait>,
        repos.system_db_cluster.clone(),
        repos.workspace_config.clone(),
        tenant_provisioner.clone()
            as Arc<dyn crate::services::tenant_provisioner::DatabaseMigratorTrait>,
    ));

    let location = Arc::new(crate::services::LocationService::new(
        repos.location.clone(),
        repos.audit.clone(),
    ));
    let employer = Arc::new(crate::services::EmployerService::new(
        repos.employer.clone(),
        repos.audit.clone(),
    ));
    let custom_field = Arc::new(crate::services::CustomFieldService::new(
        Arc::clone(&repos.custom_field),
        Arc::clone(&repos.audit),
    ));

    let recruitment = Arc::new(crate::services::RecruitmentService::new(Arc::clone(
        &repos.recruitment,
    )));

    let central_storage =
        Arc::new(crate::services::storage::s3_storage::S3Storage::from_env().await?);
    let storage_resolver = Arc::new(crate::services::storage::resolver::StorageResolver::new(
        central_storage,
        repos.storage_config.clone(),
    ));
    let file = Arc::new(crate::services::storage::file_service::FileService::new(
        storage_resolver.clone(),
        repos.file.clone(),
    ));

    let signature = Arc::new(crate::services::signature_service::SignatureService::new(
        repos.signature.clone(),
        file.clone(),
    ));
    let scan = crate::services::storage::ScanProvider::build();

    let agent = Arc::new(crate::services::AgentService::new(
        Arc::clone(&repos.agent),
        agent_manager.clone(),
    ));
    let shift = Arc::new(crate::services::shift_service::ShiftService::new(
        repos.shift.clone(),
    ));

    Ok(CoreServices {
        lifecycle,
        employee,
        outbox,
        auth,
        workspace,
        location,
        employer,
        custom_field,
        recruitment,
        storage_resolver,
        file,
        signature,
        scan,
        agent,
        tenant_provisioner,
        shift,
    })
}
