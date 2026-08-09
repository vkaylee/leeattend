use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::{salary_base_groups, salary_component_base_members};
use crate::{
    diesel_pool::{DieselPools, DieselPoolsTenantTxExt, TenantAuthToken},
    errors::AppError,
    models::{SalaryBaseGroup, SalaryBaseGroupMember},
    utils::i18n::I18nKey,
};

// ---------------------------------------------------------------------------
// Diesel-native query structs
// ---------------------------------------------------------------------------

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = salary_base_groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SalaryBaseGroupRow {
    id: Uuid,
    workspace_id: Option<Uuid>,
    name: String,
    slug: String,
    description: Option<String>,
    is_system: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<SalaryBaseGroupRow> for SalaryBaseGroup {
    fn from(r: SalaryBaseGroupRow) -> Self {
        Self {
            id: r.id,
            workspace_id: r.workspace_id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            is_system: r.is_system,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = salary_component_base_members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SalaryBaseGroupMemberRow {
    component_definition_id: Uuid,
    base_group_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<SalaryBaseGroupMemberRow> for SalaryBaseGroupMember {
    fn from(r: SalaryBaseGroupMemberRow) -> Self {
        Self {
            component_definition_id: r.component_definition_id,
            base_group_id: r.base_group_id,
            created_at: r.created_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

pub struct DieselSalaryBaseRepository {
    diesel_pools: DieselPools,
}

impl DieselSalaryBaseRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait::async_trait]
impl crate::repositories::salary_base_repository::SalaryBaseRepository
    for DieselSalaryBaseRepository
{
    async fn get_groups(&self, workspace_id: Uuid) -> Result<Vec<SalaryBaseGroup>, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    salary_base_groups::table
                        .filter(
                            salary_base_groups::workspace_id
                                .eq(workspace_id)
                                .or(salary_base_groups::workspace_id.is_null()),
                        )
                        .order((
                            salary_base_groups::is_system.desc(),
                            salary_base_groups::name.asc(),
                        ))
                        .select(SalaryBaseGroupRow::as_select())
                        .load(conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to fetch salary bases: {e}"))
                        })
                        .map(|rows| rows.into_iter().map(Into::into).collect())
                })
            })
            .await
    }

    async fn create_group(
        &self,
        workspace_id: Uuid,
        name: &str,
        slug: &str,
        description: Option<String>,
    ) -> Result<SalaryBaseGroup, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        let name = name.to_string();
        let slug = slug.to_string();
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let new_group = SalaryBaseGroupRow {
                        id: Uuid::new_v4(),
                        workspace_id: Some(workspace_id),
                        name,
                        slug,
                        description,
                        is_system: false,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };

                    diesel::insert_into(salary_base_groups::table)
                        .values(&new_group)
                        .returning(SalaryBaseGroupRow::as_returning())
                        .get_result(conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to create salary base group: {e}"))
                        })
                        .map(Into::into)
                })
            })
            .await
    }

    async fn get_group_members(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryBaseGroupMember>, AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    salary_component_base_members::table
                        .inner_join(salary_base_groups::table)
                        .filter(
                            salary_base_groups::workspace_id
                                .eq(workspace_id)
                                .or(salary_base_groups::workspace_id.is_null()),
                        )
                        .select(SalaryBaseGroupMemberRow::as_select())
                        .load(conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to fetch salary base members: {e}"))
                        })
                        .map(|rows| rows.into_iter().map(Into::into).collect())
                })
            })
            .await
    }

    async fn assign_component_to_group(
        &self,
        workspace_id: Uuid,
        component_id: Uuid,
        base_group_id: Uuid,
    ) -> Result<(), AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    let valid_group: Option<Uuid> = salary_base_groups::table
                        .filter(salary_base_groups::id.eq(base_group_id))
                        .filter(salary_base_groups::workspace_id.eq(workspace_id))
                        .select(salary_base_groups::id)
                        .first(conn)
                        .await
                        .optional()
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to validate group: {e}"))
                        })?;

                    if valid_group.is_none() {
                        return Err(AppError::forbidden(I18nKey::InvalidBaseGroup));
                    }

                    let new_member = SalaryBaseGroupMemberRow {
                        component_definition_id: component_id,
                        base_group_id,
                        created_at: Utc::now(),
                    };

                    diesel::insert_into(salary_component_base_members::table)
                        .values(&new_member)
                        .on_conflict((
                            salary_component_base_members::component_definition_id,
                            salary_component_base_members::base_group_id,
                        ))
                        .do_nothing()
                        .execute(conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to assign component to base: {e}"))
                        })?;

                    Ok(())
                })
            })
            .await
    }

    async fn unassign_component_from_group(
        &self,
        workspace_id: Uuid,
        component_id: Uuid,
        base_group_id: Uuid,
    ) -> Result<(), AppError> {
        let token = TenantAuthToken::system_override(workspace_id);
        self.diesel_pools
            .with_tenant_tx(&token, |conn| {
                Box::pin(async move {
                    diesel::delete(
                        salary_component_base_members::table
                            .filter(
                                salary_component_base_members::component_definition_id
                                    .eq(component_id),
                            )
                            .filter(salary_component_base_members::base_group_id.eq(base_group_id))
                            .filter(
                                salary_component_base_members::base_group_id.eq_any(
                                    salary_base_groups::table
                                        .filter(salary_base_groups::id.eq(base_group_id))
                                        .filter(salary_base_groups::workspace_id.eq(workspace_id))
                                        .select(salary_base_groups::id),
                                ),
                            ),
                    )
                    .execute(conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Failed to unassign component from base: {e}"))
                    })?;

                    Ok(())
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselSalaryBaseRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
