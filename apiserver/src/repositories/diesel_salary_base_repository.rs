use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools,
    errors::AppError,
    models::diesel::payroll::{SalaryBaseGroupEntity, SalaryBaseGroupMemberEntity},
    models::{SalaryBaseGroup, SalaryBaseGroupMember},
    repositories::salary_base_repository::SalaryBaseRepository,
};

pub struct DieselSalaryBaseRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselSalaryBaseRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl SalaryBaseRepository for DieselSalaryBaseRepository {
    async fn get_groups(&self, workspace_id_val: Uuid) -> Result<Vec<SalaryBaseGroup>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let groups = crate::schema::salary_base_groups::table
            .filter(
                crate::schema::salary_base_groups::workspace_id
                    .eq(workspace_id_val)
                    .or(crate::schema::salary_base_groups::workspace_id.is_null()),
            )
            .order((
                crate::schema::salary_base_groups::is_system.desc(),
                crate::schema::salary_base_groups::name.asc(),
            ))
            .load::<SalaryBaseGroupEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch salary bases: {e}")))?;

        Ok(groups.into_iter().map(Into::into).collect())
    }

    async fn create_group(
        &self,
        workspace_id_val: Uuid,
        name_val: &str,
        slug_val: &str,
        description_val: Option<&str>,
    ) -> Result<SalaryBaseGroup, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let new_group = SalaryBaseGroupEntity {
            id: Uuid::new_v4(),
            workspace_id: Some(workspace_id_val),
            name: name_val.to_string(),
            slug: slug_val.to_string(),
            description: description_val.map(String::from),
            is_system: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let group = diesel::insert_into(crate::schema::salary_base_groups::table)
            .values(&new_group)
            .get_result::<SalaryBaseGroupEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create salary base group: {e}")))?;

        Ok(group.into())
    }

    async fn get_group_members(&self, workspace_id_val: Uuid) -> Result<Vec<SalaryBaseGroupMember>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let members = crate::schema::salary_component_base_members::table
            .inner_join(crate::schema::salary_base_groups::table)
            .filter(
                crate::schema::salary_base_groups::workspace_id
                    .eq(workspace_id_val)
                    .or(crate::schema::salary_base_groups::workspace_id.is_null()),
            )
            .select(SalaryBaseGroupMemberEntity::as_select())
            .load::<SalaryBaseGroupMemberEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch salary base members: {e}")))?;

        Ok(members.into_iter().map(Into::into).collect())
    }

    async fn assign_component_to_group(
        &self,
        component_id_val: Uuid,
        base_group_id_val: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        let new_member = SalaryBaseGroupMemberEntity {
            component_definition_id: component_id_val,
            base_group_id: base_group_id_val,
            created_at: Utc::now(),
        };

        diesel::insert_into(crate::schema::salary_component_base_members::table)
            .values(&new_member)
            .on_conflict((
                crate::schema::salary_component_base_members::component_definition_id,
                crate::schema::salary_component_base_members::base_group_id,
            ))
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to assign component to base: {e}")))?;

        Ok(())
    }

    async fn unassign_component_from_group(
        &self,
        component_id_val: Uuid,
        base_group_id_val: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {e}")))?;

        diesel::delete(
            crate::schema::salary_component_base_members::table
                .filter(crate::schema::salary_component_base_members::component_definition_id.eq(component_id_val))
                .filter(crate::schema::salary_component_base_members::base_group_id.eq(base_group_id_val)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to unassign component from base: {e}")))?;

        Ok(())
    }
}
