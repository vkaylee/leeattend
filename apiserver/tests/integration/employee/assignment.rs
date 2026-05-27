use apiserver::models::employee_assignment::{AssignmentStatusEnum, EmployeeAssignment, NewEmployeeAssignment};
use apiserver::schema::employee_assignments;
use apiserver::schema::work_locations;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;

async fn setup_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    (test_db, factory)
}

#[tokio::test]
async fn test_employee_assignment_crud_and_status_enum() {
    let (test_db, factory) = setup_db().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let mut tenant_conn = test_db.diesel_pools.tenants_tx.get_connection(&ws_id).await.unwrap();

    let hq_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT primary_location_id AS id FROM employees WHERE id = $1",
        diesel::sql_types::Uuid, emp_id
    );

    #[derive(Insertable)]
    #[diesel(table_name = work_locations)]
    struct NewLoc {
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        timezone: Option<String>,
        is_headquarters: bool,
    }

    let tokyo_loc_id = Uuid::new_v4();
    diesel::insert_into(work_locations::table)
        .values(&NewLoc {
            id: tokyo_loc_id,
            workspace_id: ws_id,
            name: "Tokyo Branch".to_string(),
            timezone: Some("Asia/Tokyo".to_string()),
            is_headquarters: false,
        })
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    // Test inserting with Active status
    let new_assignment = NewEmployeeAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        home_location_id: hq_id,
        host_location_id: tokyo_loc_id,
        start_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
        reason: Some("Secondment to Tokyo".to_string()),
        status: AssignmentStatusEnum::Active,
    };

    let inserted: EmployeeAssignment = diesel::insert_into(employee_assignments::table)
        .values(&new_assignment)
        .get_result(&mut *tenant_conn)
        .await
        .unwrap();

    assert_eq!(inserted.status, AssignmentStatusEnum::Active);
    assert_eq!(inserted.reason.unwrap(), "Secondment to Tokyo");

    // Test updating to Completed status
    let updated: EmployeeAssignment = diesel::update(employee_assignments::table.filter(employee_assignments::id.eq(new_assignment.id)))
        .set(employee_assignments::status.eq(AssignmentStatusEnum::Completed))
        .get_result(&mut *tenant_conn)
        .await
        .unwrap();

    assert_eq!(updated.status, AssignmentStatusEnum::Completed);

    // Test inserting with Cancelled status
    let cancelled_assignment = NewEmployeeAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        home_location_id: hq_id,
        host_location_id: tokyo_loc_id,
        start_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        end_date: None,
        reason: None,
        status: AssignmentStatusEnum::Cancelled,
    };

    let inserted_cancelled: EmployeeAssignment = diesel::insert_into(employee_assignments::table)
        .values(&cancelled_assignment)
        .get_result(&mut *tenant_conn)
        .await
        .unwrap();

    assert_eq!(inserted_cancelled.status, AssignmentStatusEnum::Cancelled);
}
