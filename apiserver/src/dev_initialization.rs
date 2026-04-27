use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::models::MemberRole;

pub async fn ensure_dev_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Ensuring dev data exists...");

    #[allow(clippy::expect_used)]
    let admin_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("valid UUID constant");
    #[allow(clippy::expect_used)]
    let workspace_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").expect("valid UUID constant");
    #[allow(clippy::expect_used)]
    let device_id_1 = Uuid::parse_str("00000000-0000-0000-0000-000000000011").expect("valid UUID constant");
    #[allow(clippy::expect_used)]
    let device_id_2 = Uuid::parse_str("00000000-0000-0000-0000-000000000012").expect("valid UUID constant");

    // 1. Upsert Admin User
    // Password is 'admin123' hashed with bcrypt
    #[allow(clippy::expect_used)]
    let password_hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST).expect("bcrypt hash should not fail");

    sqlx::query(
        r#"
        INSERT INTO users (id, email, full_name, password_hash)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE SET
            email = EXCLUDED.email,
            full_name = EXCLUDED.full_name,
            password_hash = EXCLUDED.password_hash
        "#,
    )
    .bind(admin_user_id)
    .bind("admin@leeattend.com")
    .bind("System Administrator")
    .bind(password_hash)
    .execute(pool)
    .await?;

    // 2. Upsert Workspace
    sqlx::query(
        r#"
        INSERT INTO workspaces (id, name)
        VALUES ($1, $2)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(workspace_id)
    .bind("Default Workspace")
    .execute(pool)
    .await?;

    // 3. Upsert Membership
    sqlx::query(
        r#"
        INSERT INTO workspace_members (workspace_id, user_id, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (workspace_id, user_id) DO NOTHING
        "#,
    )
    .bind(workspace_id)
    .bind(admin_user_id)
    .bind(MemberRole::Owner)
    .execute(pool)
    .await?;

    // 4. Upsert Authorized Agent (for E2E tests)
    sqlx::query(
        r#"
        INSERT INTO authorized_agents (id, name, token, is_trusted, is_active, workspace_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            is_trusted = EXCLUDED.is_trusted,
            is_active = EXCLUDED.is_active
        "#,
    )
    .bind("dev-agent-001")
    .bind("Development Agent")
    .bind("dev-token-secret")
    .bind(true)
    .bind(true)
    .bind(workspace_id)
    .execute(pool)
    .await?;

    // 5. Upsert Devices (for E2E tests)
    sqlx::query(
        r#"
        INSERT INTO devices (id, agent_id, name, ip, port, is_enabled, brand, workspace_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            ip = EXCLUDED.ip,
            port = EXCLUDED.port,
            is_enabled = EXCLUDED.is_enabled,
            brand = EXCLUDED.brand
        "#,
    )
    .bind(device_id_1)
    .bind("dev-agent-001")
    .bind("Main Entrance Clock")
    .bind("192.168.1.100")
    .bind(4370)
    .bind(true)
    .bind("zkteco")
    .bind(workspace_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO devices (id, agent_id, name, ip, port, is_enabled, brand, workspace_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            ip = EXCLUDED.ip,
            port = EXCLUDED.port,
            is_enabled = EXCLUDED.is_enabled,
            brand = EXCLUDED.brand
        "#,
    )
    .bind(device_id_2)
    .bind("dev-agent-001")
    .bind("Back Door Clock")
    .bind("192.168.1.101")
    .bind(4370)
    .bind(true)
    .bind("zkteco")
    .bind(workspace_id)
    .execute(pool)
    .await?;

    info!("Dev data upserted successfully.");
    Ok(())
}
