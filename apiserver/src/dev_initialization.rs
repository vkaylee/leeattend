use sqlx::PgPool;
use uuid::Uuid;
use tracing::info;

pub async fn ensure_dev_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Ensuring dev data exists...");

    let admin_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    let workspace_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

    // 1. Upsert Admin User
    // Password is 'admin123' hashed with bcrypt
    let password_hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST).unwrap();
    
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
    .bind("owner")
    .execute(pool)
    .await?;

    info!("Dev data upserted successfully.");
    Ok(())
}
