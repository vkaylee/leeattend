use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::diesel_employment_phase_repository::DieselEmploymentPhaseRepository;
use apiserver::repositories::EmploymentPhaseRepository;
use chrono::NaiveDate;
use uuid::Uuid;

#[tokio::test]
async fn test_employment_phase_repo_crud() {
    let _test_db = TestDatabase::new().await;
    let factory = TestFactory::new(_test_db.diesel_pools.clone());

    let repo = DieselEmploymentPhaseRepository::new(factory.diesel_pools.clone());

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create employee
    let employee_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id = crate::test_query_scalar_uuid!(
        &_test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    let rule_config = serde_json::json!({ "max_duration": 90 });

    // 1. Create Definition
    let def = repo
        .create_definition(
            ws_id,
            "Probation",
            Some("Probation period".to_string()),
            "probation",
            &rule_config,
            1,
        )
        .await
        .unwrap();

    assert_eq!(def.name, "Probation");
    assert_eq!(def.phase_type, "probation");

    // 2. Try creating duplicate definition (should fail with specific AppError)
    let err = repo
        .create_definition(
            ws_id,
            "Probation", // Duplicate name
            None,
            "probation",
            &rule_config,
            2,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, apiserver::errors::AppError::Validation(msg) if msg.to_string().contains("Phase definition with this name already exists"))
    );

    // 3. Get Definitions
    let defs = repo.get_definitions(ws_id).await.unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].id, def.id);

    // 4. Assign Phase
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
    let override_config = serde_json::json!({ "max_duration": 60 });

    let phase = repo
        .assign_phase(
            ws_id,
            employment_id,
            def.id,
            start_date,
            end_date,
            Some(override_config.clone()),
        )
        .await
        .unwrap();

    assert_eq!(phase.status, "active");
    assert_eq!(phase.override_config, Some(override_config.clone()));

    // Assign another phase that overlaps, which should auto-close the first one
    let def2 = repo
        .create_definition(ws_id, "Permanent", None, "permanent", &rule_config, 2)
        .await
        .unwrap();

    let start_date_2 = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
    let end_date_2 = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();

    let phase2 = repo
        .assign_phase(
            ws_id,
            employment_id,
            def2.id,
            start_date_2,
            end_date_2,
            None,
        )
        .await
        .unwrap();

    assert_eq!(phase2.status, "active");

    // 5. Get Employment Phases
    let phases = repo
        .get_employment_phases(ws_id, employment_id)
        .await
        .unwrap();
    assert_eq!(phases.len(), 2);

    // The first phase should now be "completed"
    let old_phase = phases.iter().find(|p| p.id == phase.id).unwrap();
    assert_eq!(old_phase.status, "completed");

    // 6. Update Phase Status
    let updated = repo
        .update_phase_status(
            ws_id,
            employment_id,
            phase2.id,
            "terminated",
            Some("Terminated early".to_string()),
            admin_id,
        )
        .await
        .unwrap();

    assert_eq!(updated.status, "terminated");
    assert_eq!(
        updated.evaluation_notes,
        Some("Terminated early".to_string())
    );
    assert_eq!(updated.evaluated_by, Some(admin_id));
}
