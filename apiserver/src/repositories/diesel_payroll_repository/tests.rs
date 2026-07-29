use super::*;
use crate::diesel_pool::DieselPools;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[tokio::test]
async fn test_repo_initialization() {
    let test_db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());

    let pools = DieselPools::for_test(&test_db_url);
    let _repo = DieselPayrollRepository::new(pools);
    // DI connection test passes if it compiles
}

#[tokio::test]
async fn test_create_salary_component_definition() {
    let test_db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());

    let pools = DieselPools::for_test(&test_db_url);
    let repo = DieselPayrollRepository::new(pools.clone());
    let ws_id = Uuid::new_v4();
    let token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

    let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
        c
    } else {
        println!("Database not available, skipping integration assertions");
        return;
    };

    // Insert Workspace to satisfy FK constraints
    let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
        id: ws_id,
        name: "Test Workspace".to_string(),
        created_at: Some(chrono::Utc::now()),
        date_format: "YYYY-MM-DD".to_string(),
        time_format: "HH:mm".to_string(),
        timezone: "UTC".to_string(),
        theme_primary_color: None,
        default_language: "en".to_string(),
        logo_url: None,
        tagline: None,
        custom_db_url_encrypted: None,
    };
    diesel::insert_into(crate::schema::workspaces::table)
        .values(&workspace_entity)
        .execute(&mut *conn)
        .await
        .expect("insert workspace");

    // Insert a base group
    let group_id1 = Uuid::new_v4();
    let group_id2 = Uuid::new_v4();

    diesel::sql_query("INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind::<diesel::sql_types::Uuid, _>(group_id1)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(ws_id))
        .bind::<diesel::sql_types::Text, _>("Test Group 1")
        .bind::<diesel::sql_types::Text, _>("test-group-1")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut *conn)
        .await
        .expect("insert group 1");

    diesel::sql_query("INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind::<diesel::sql_types::Uuid, _>(group_id2)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(ws_id))
        .bind::<diesel::sql_types::Text, _>("Test Group 2")
        .bind::<diesel::sql_types::Text, _>("test-group-2")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut *conn)
        .await
        .expect("insert group 2");

    drop(conn);

    // 1. Success Path: create definition with valid groups
    let res = repo
        .create_salary_component_definition(
            ws_id,
            "Base Salary".to_string(),
            "BASE".to_string(),
            Some(vec![group_id1, group_id2]),
        )
        .await;
    assert!(res.is_ok());
    let def = res.unwrap();
    assert_eq!(def.name, "Base Salary");
    assert_eq!(def.code, "BASE");

    // Verify members are created in DB
    let mut conn = pools.tenant_transaction_conn(&token).await.unwrap();
    let count: i64 = crate::schema::salary_component_base_members::table
        .filter(crate::schema::salary_component_base_members::component_definition_id.eq(def.id))
        .count()
        .get_result(&mut *conn)
        .await
        .unwrap();
    assert_eq!(count, 2);

    // 2. Failure Path: invalid base group (Forbidden error)
    let bad_group = Uuid::new_v4();
    let res_err = repo
        .create_salary_component_definition(
            ws_id,
            "Allowance".to_string(),
            "ALLOW".to_string(),
            Some(vec![bad_group]),
        )
        .await;
    assert!(res_err.is_err());
    assert!(matches!(res_err.unwrap_err(), AppError::Forbidden(_)));
}

#[tokio::test]
async fn test_get_policy_tiers_batch() {
    let test_db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());

    let pools = DieselPools::for_test(&test_db_url);
    let repo = DieselPayrollRepository::new(pools.clone());
    let ws_id = Uuid::new_v4();
    let token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

    let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
        c
    } else {
        println!("Database not available, skipping integration assertions");
        return;
    };

    // Insert workspace
    let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
        id: ws_id,
        name: "Test Workspace Batch".to_string(),
        created_at: Some(chrono::Utc::now()),
        date_format: "YYYY-MM-DD".to_string(),
        time_format: "HH:mm".to_string(),
        timezone: "UTC".to_string(),
        theme_primary_color: None,
        default_language: "en".to_string(),
        logo_url: None,
        tagline: None,
        custom_db_url_encrypted: None,
    };
    diesel::insert_into(crate::schema::workspaces::table)
        .values(&workspace_entity)
        .execute(&mut *conn)
        .await
        .expect("insert workspace");

    // Insert two policies in this workspace + one in a foreign workspace to prove isolation
    let policy_a = Uuid::new_v4();
    let policy_b = Uuid::new_v4();
    let policy_foreign = Uuid::new_v4();
    let foreign_ws_id = Uuid::new_v4();

    let foreign_ws_entity = crate::models::diesel::core::WorkspaceEntity {
        id: foreign_ws_id,
        name: "Foreign Workspace".to_string(),
        created_at: Some(chrono::Utc::now()),
        date_format: "YYYY-MM-DD".to_string(),
        time_format: "HH:mm".to_string(),
        timezone: "UTC".to_string(),
        theme_primary_color: None,
        default_language: "en".to_string(),
        logo_url: None,
        tagline: None,
        custom_db_url_encrypted: None,
    };
    diesel::insert_into(crate::schema::workspaces::table)
        .values(&foreign_ws_entity)
        .execute(&mut *conn)
        .await
        .expect("insert foreign workspace");

    for (pid, wsid, name) in [
        (policy_a, ws_id, "Policy A"),
        (policy_b, ws_id, "Policy B"),
        (policy_foreign, foreign_ws_id, "Policy Foreign"),
    ] {
        let entity = crate::models::diesel::payroll::PayrollIncentivePolicyEntity {
            id: pid,
            workspace_id: wsid,
            name: name.to_string(),
            base_amount: 100.0,
            payout_type: "fixed".to_string(),
            is_active: true,
            created_at: chrono::Utc::now(),
            effective_from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            effective_to: None,
            deleted_at: None,
        };
        diesel::insert_into(crate::schema::payroll_incentive_policies::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .expect("insert policy");
    }

    // Policy A: 3 tiers with priorities 3, 1, 2 → expect load order 1, 2, 3
    // Policy B: 2 tiers with priorities 2, 1 → expect load order 1, 2
    // Policy Foreign: 1 tier (should not appear even if requested — cross-workspace isolation via connection scoping)
    let tier_data = vec![
        (policy_a, 3, 30.0),
        (policy_a, 1, 10.0),
        (policy_a, 2, 20.0),
        (policy_b, 2, 200.0),
        (policy_b, 1, 100.0),
        (policy_foreign, 1, 999.0),
    ];
    for (pid, priority, amount) in tier_data {
        let entity = crate::models::diesel::payroll::IncentiveTierEntity {
            id: Uuid::new_v4(),
            policy_id: pid,
            priority,
            payout_amount: amount,
            rule_config: serde_json::json!({}),
        };
        diesel::insert_into(crate::schema::payroll_incentive_tiers::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .expect("insert tier");
    }
    drop(conn);

    // 1. Empty input → empty HashMap
    let empty = repo
        .get_policy_tiers_batch(ws_id, &[])
        .await
        .expect("empty batch");
    assert!(empty.is_empty());

    // 2. Batch fetch for policy_a + policy_b + a non-existent id
    let missing = Uuid::new_v4();
    let ids = vec![policy_a, policy_b, missing];
    let map = repo
        .get_policy_tiers_batch(ws_id, &ids)
        .await
        .expect("batch fetch");

    // Pre-seeded: every requested id has an entry
    assert_eq!(map.len(), 3);
    assert!(map.contains_key(&policy_a));
    assert!(map.contains_key(&policy_b));
    assert!(map.contains_key(&missing));

    // Missing policy → empty Vec
    assert!(map.get(&missing).unwrap().is_empty());

    // Policy A ordered by priority asc
    let tiers_a = map.get(&policy_a).unwrap();
    assert_eq!(tiers_a.len(), 3);
    assert_eq!(tiers_a[0].priority(), 1);
    assert_eq!(tiers_a[1].priority(), 2);
    assert_eq!(tiers_a[2].priority(), 3);

    // Policy B ordered by priority asc
    let tiers_b = map.get(&policy_b).unwrap();
    assert_eq!(tiers_b.len(), 2);
    assert_eq!(tiers_b[0].priority(), 1);
    assert_eq!(tiers_b[1].priority(), 2);

    // 3. Cross-workspace isolation — request policy_foreign from ws_id token
    //    tenants_tx.get_connection(&ws_id) scopes to ws_id's DB shard, so foreign
    //    policy_id filters yield no rows even though the id is requested.
    let cross = repo
        .get_policy_tiers_batch(ws_id, &[policy_foreign])
        .await
        .expect("cross-workspace batch");
    // Pre-seeded map has the key, but the Vec must be empty (isolation).
    assert!(cross.get(&policy_foreign).unwrap().is_empty());
}
