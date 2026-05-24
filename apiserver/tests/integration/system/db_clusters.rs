use crate::common::app::spawn_app_with_factory;
use apiserver::models::system_db_cluster::{NewSystemDbCluster, SystemDbCluster};
use apiserver::repositories::diesel_system_db_cluster_repository::DieselSystemDbClusterRepository;
use apiserver::repositories::system_db_cluster_repository::SystemDbClusterRepository;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_db_cluster_repository_direct() {
    let (_address, _workspace_id, _user_id, factory, _test_db) = spawn_app_with_factory().await;
    let repo = DieselSystemDbClusterRepository::new(factory.diesel_pools.clone());

    // 1. Create a cluster
    let cluster_id = Uuid::new_v4();
    let new_cluster = NewSystemDbCluster {
        id: cluster_id,
        name: "Cluster Direct".to_string(),
        admin_database_url: "postgres://admin:pass@host:5432/db".to_string(),
        region: "ap-southeast-1".to_string(),
        max_tenants: 5,
        active_tenants: 0,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let created = repo.create_cluster(&new_cluster).await.unwrap();
    assert_eq!(created.id, cluster_id);
    assert_eq!(created.name, "Cluster Direct");

    // 2. List clusters
    let list = repo.list_clusters().await.unwrap();
    assert!(list.iter().any(|c| c.id == cluster_id));

    // 3. Least loaded cluster (initially cluster_id because active_tenants is 0)
    let least = repo.get_least_loaded_cluster().await.unwrap();
    assert!(least.is_some());
    let least = least.unwrap();
    assert_eq!(least.id, cluster_id);

    // 4. Increment active tenants
    repo.increment_active_tenants(cluster_id).await.unwrap();
    let updated_least = repo.get_least_loaded_cluster().await.unwrap().unwrap();
    assert_eq!(updated_least.active_tenants, 1);

    // 5. Update cluster status to inactive
    let updated_status = repo.update_cluster_status(cluster_id, false).await.unwrap();
    assert!(!updated_status.is_active);

    // After making it inactive, get_least_loaded_cluster should return None (since it filters for is_active = true)
    let least_none = repo.get_least_loaded_cluster().await.unwrap();
    assert!(least_none.is_none());

    // 6. Delete cluster
    repo.delete_cluster(cluster_id).await.unwrap();
    let final_list = repo.list_clusters().await.unwrap();
    assert!(!final_list.iter().any(|c| c.id == cluster_id));
}

#[tokio::test]
async fn test_db_cluster_api_lifecycle() {
    let (address, _workspace_id, user_id, factory, _test_db) = spawn_app_with_factory().await;

    // Assign PlatformAdmin role
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid,
        user_id
    );

    let client = reqwest::Client::new();

    // 1. Create Db Cluster via API
    let req_payload = serde_json::json!({
        "name": "Cluster API",
        "admin_database_url": "postgres://api:pass@localhost:5432/api_db",
        "region": "us-east-1",
        "max_tenants": 100,
        "is_active": true
    });

    let create_res = client
        .post(&format!("{address}/api/system/db-clusters"))
        .header("X-User-Id", user_id.to_string())
        .json(&req_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(create_res.status(), reqwest::StatusCode::OK);
    let created_cluster: SystemDbCluster = create_res.json().await.unwrap();
    assert_eq!(created_cluster.name, "Cluster API");
    assert_eq!(created_cluster.region, "us-east-1");
    assert_eq!(created_cluster.max_tenants, 100);
    assert!(created_cluster.is_active);

    let cluster_id = created_cluster.id;

    // 2. List Db Clusters via API
    let list_res = client
        .get(&format!("{address}/api/system/db-clusters"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(list_res.status(), reqwest::StatusCode::OK);
    let list: Vec<SystemDbCluster> = list_res.json().await.unwrap();
    assert!(list.iter().any(|c| c.id == cluster_id));

    // 3. Update status via API
    let status_payload = serde_json::json!({
        "is_active": false
    });
    let patch_res = client
        .patch(&format!(
            "{address}/api/system/db-clusters/{cluster_id}/status"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&status_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(patch_res.status(), reqwest::StatusCode::OK);
    let updated_cluster: SystemDbCluster = patch_res.json().await.unwrap();
    assert!(!updated_cluster.is_active);

    // 4. Delete via API
    let delete_res = client
        .delete(&format!("{address}/api/system/db-clusters/{cluster_id}"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(delete_res.status(), reqwest::StatusCode::NO_CONTENT);

    // Verify it is deleted from list
    let list_res2 = client
        .get(&format!("{address}/api/system/db-clusters"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    let list2: Vec<SystemDbCluster> = list_res2.json().await.unwrap();
    assert!(!list2.iter().any(|c| c.id == cluster_id));
}

#[tokio::test]
async fn test_db_cluster_api_authorization() {
    let (address, _workspace_id, user_id, _factory, _test_db) = spawn_app_with_factory().await;

    // Note: user_id is a normal user, NOT a PlatformAdmin.
    let client = reqwest::Client::new();

    // 1. List request should be forbidden
    let list_res = client
        .get(&format!("{address}/api/system/db-clusters"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(list_res.status(), reqwest::StatusCode::FORBIDDEN);

    // 2. Create request should be forbidden
    let req_payload = serde_json::json!({
        "name": "Cluster API Auth",
        "admin_database_url": "postgres://api:pass@localhost:5432/api_db",
        "region": "us-east-1",
        "max_tenants": 100,
        "is_active": true
    });
    let create_res = client
        .post(&format!("{address}/api/system/db-clusters"))
        .header("X-User-Id", user_id.to_string())
        .json(&req_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), reqwest::StatusCode::FORBIDDEN);

    // 3. Update status request should be forbidden
    let status_payload = serde_json::json!({
        "is_active": false
    });
    let patch_res = client
        .patch(&format!(
            "{address}/api/system/db-clusters/{}/status",
            Uuid::new_v4()
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&status_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(patch_res.status(), reqwest::StatusCode::FORBIDDEN);

    // 4. Delete request should be forbidden
    let delete_res = client
        .delete(&format!(
            "{address}/api/system/db-clusters/{}",
            Uuid::new_v4()
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(delete_res.status(), reqwest::StatusCode::FORBIDDEN);
}
