use agent::database::Database;

pub fn print_version() {
    println!("Agent Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Build: {}", env!("CARGO_PKG_NAME"));
}

pub fn print_help() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    LeeAttend Agent - Command Line Tool                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("USAGE:");
    println!("    agent [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!();
    println!("  Sync Queue Management:");
    println!("    failed-logs [limit]      Display logs that failed to sync (retrying)");
    println!("    failed [limit]           Alias for failed-logs (default limit: 50)");
    println!();
    println!("  Dead Letter Queue (DLQ):");
    println!("    dlq [limit]              Display permanently failed logs in DLQ");
    println!("    dead-letter-queue [N]    Same as dlq (default limit: 50)");
    println!("    dlq-stats                Show DLQ statistics and failure analysis");
    println!("    stats                    Alias for dlq-stats");
    println!();
    println!("  Configuration & Status:");
    println!("    config [key] [value]     View or set agent configuration");
    println!("    status                   Show agent status and queue metrics");
    println!();
    println!("  General:");
    println!("    --help, -h, help         Display this help message");
    println!("    --version, -v            Display agent version");
    println!();
    println!("EXAMPLES:");
    println!();
    println!("  Start the agent:");
    println!("    agent                    Run agent orchestrator (main mode)");
    println!();
    println!("  Query failed sync logs (retrying):");
    println!("    agent failed-logs        Show 50 logs that failed but will retry");
    println!("    agent failed 100         Show 100 failed logs");
    println!();
    println!("  Query dead letter queue (permanently failed):");
    println!("    agent dlq                Show 50 permanently failed logs");
    println!("    agent dlq 200            Show 200 DLQ entries");
    println!("    agent dlq-stats          View failure statistics and patterns");
    println!();
    println!("  Configuration:");
    println!("    agent config             Show all configuration values");
    println!("    agent config max_retry_count 10");
    println!("                             Set max retry attempts to 10");
    println!("    agent config retry_ttl_minutes 60");
    println!("                             Set retry TTL to 60 minutes");
    println!();
    println!("  Status & Metrics:");
    println!("    agent status             Show queue sizes, retry rates, DLQ growth");
    println!();
    println!("CONFIGURATION KEYS:");
    println!("  max_retry_count          Maximum retry attempts before DLQ (default: 10)");
    println!("  retry_ttl_minutes        Retry timeout in minutes (default: 60)");
    println!("  max_concurrent_workers   Max parallel device connections (server-managed)");
    println!();
    println!("UNDERSTANDING THE SYSTEM:");
    println!();
    println!("  Sync Flow:");
    println!("    Device → Agent Queue → Central Server");
    println!("                ↓ (on failure)");
    println!("              Retry (up to max_retry_count or retry_ttl_minutes)");
    println!("                ↓ (on retry exhaustion)");
    println!("           Dead Letter Queue (DLQ)");
    println!();
    println!("  Log States:");
    println!("    • Unsynced (new)       - Logs fetched from device, not yet sent");
    println!("    • In-flight            - Logs currently being synced to server");
    println!("    • Failed (retrying)    - Sync failed, will retry (use 'failed-logs')");
    println!("    • Synced               - Successfully sent to central server");
    println!("    • Dead Letter Queue    - Retry exhausted, investigate (use 'dlq')");
    println!();
    println!("  Why logs end up in DLQ:");
    println!("    1. Max retries exceeded (e.g., invalid data format)");
    println!("    2. TTL timeout (e.g., central server down for >60 min)");
    println!();
    println!("DATABASE QUERIES (Direct Access):");
    println!();
    println!("  Query failed logs (retrying):");
    println!("    SELECT * FROM zkt_raw_attendance");
    println!("    WHERE is_synced = 0 AND updated_at IS NOT NULL;");
    println!();
    println!("  Query dead letter queue:");
    println!("    SELECT * FROM zkt_failed_logs");
    println!("    ORDER BY failed_at DESC LIMIT 50;");
    println!();
    println!("  DLQ statistics by device:");
    println!("    SELECT device_id, COUNT(*) as failures");
    println!("    FROM zkt_failed_logs");
    println!("    GROUP BY device_id ORDER BY failures DESC;");
    println!();
    println!("  DLQ statistics by failure reason:");
    println!("    SELECT failure_reason, COUNT(*) as count");
    println!("    FROM zkt_failed_logs");
    println!("    GROUP BY failure_reason;");
    println!();
    println!("For more information, see documentation at:");
    println!("  ./docs/system-architecture.md");
    println!("  ./plans/reports/implementation-*-retry-limit-*.md");
    println!();
}

pub async fn handle_failed_logs_command(db: &Database, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let limit = if args.len() > 2 {
        args[2].parse::<i64>().unwrap_or(50)
    } else {
        50
    };

    let total_count = db.count_failed_sync_logs().await?;
    let failed_logs = db.get_failed_sync_logs(limit).await?;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                          FAILED SYNC LOGS REPORT                             ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total failed logs: {} (showing top {})", total_count, failed_logs.len());
    println!();

    if failed_logs.is_empty() {
        println!("✅ No failed sync logs found. All logs either synced or never attempted.");
        println!();
        return Ok(());
    }

    println!("┌─────────┬────────────┬────────────┬─────────────────────┬────────┬───────┐");
    println!("│ ID      │ Device     │ User ID    │ Timestamp           │ Status │ Punch │");
    println!("├─────────┼────────────┼────────────┼─────────────────────┼────────┼───────┤");

    for log in &failed_logs {
        let timestamp_str = chrono::DateTime::from_timestamp(log.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| format!("invalid_{}", log.timestamp));
        let status_str = log.status.map(|s| s.to_string()).unwrap_or_else(|| "NULL".to_string());
        let punch_str = log.punch.map(|p| p.to_string()).unwrap_or_else(|| "NULL".to_string());

        println!(
            "│ {:7} │ {:10} │ {:10} │ {:19} │ {:6} │ {:5} │",
            log.id,
            truncate(&log.device_id, 10),
            truncate(&log.user_id, 10),
            truncate(&timestamp_str, 19),
            truncate(&status_str, 6),
            truncate(&punch_str, 5)
        );
    }

    println!("└─────────┴────────────┴────────────┴─────────────────────┴────────┴───────┘");
    println!();
    println!("Legend:");
    println!("  - These logs were attempted to sync but failed");
    println!("  - They will be retried in the next sync cycle");
    println!("  - Check central server logs for error details");
    println!();

    Ok(())
}

pub async fn handle_dlq_command(db: &Database, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let limit = if args.len() > 2 {
        args[2].parse::<i64>().unwrap_or(50)
    } else {
        50
    };

    // Query DLQ table
    let dlq_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await?;

    type DlqRecord = (i64, String, String, String, Option<i32>, Option<i32>, String, String, i64);
    let dlq_logs: Vec<DlqRecord> = sqlx::query_as(
        "SELECT id, device_id, user_id, timestamp, status, punch, failed_at, failure_reason, retry_attempts
         FROM zkt_failed_logs
         ORDER BY failed_at DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(db.get_pool())
    .await?;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                      DEAD LETTER QUEUE (DLQ) REPORT                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total DLQ entries: {} (showing top {})", dlq_count, dlq_logs.len());
    println!();

    if dlq_logs.is_empty() {
        println!("✅ No logs in dead letter queue. All logs either synced or still retrying.");
        println!();
        return Ok(());
    }

    println!("┌────────┬───────────┬───────────┬──────────────────┬────────┬────────────────────────┐");
    println!("│ ID     │ Device    │ User ID   │ Failed At        │ Retries│ Failure Reason         │");
    println!("├────────┼───────────┼───────────┼──────────────────┼────────┼────────────────────────┤");

    for log in &dlq_logs {
        let (id, device_id, user_id, _timestamp, _status, _punch, failed_at, failure_reason, retry_attempts) = log;

        // Parse failed_at timestamp
        let failed_display = if let Some(pos) = failed_at.find(' ') {
            &failed_at[..std::cmp::min(failed_at.len(), pos + 9)]
        } else {
            &failed_at[..std::cmp::min(failed_at.len(), 16)]
        };

        println!(
            "│ {:6} │ {:9} │ {:9} │ {:16} │ {:6} │ {:22} │",
            id,
            truncate(device_id, 9),
            truncate(user_id, 9),
            truncate(failed_display, 16),
            retry_attempts,
            truncate(failure_reason, 22)
        );
    }

    println!("└────────┴───────────┴───────────┴──────────────────┴────────┴────────────────────────┘");
    println!();
    println!("Legend:");
    println!("  • These logs exceeded retry limits and were moved to DLQ");
    println!("  • They will NOT be retried automatically");
    println!("  • Investigate failure reasons and manually re-queue if needed");
    println!();
    println!("Next steps:");
    println!("  1. Review failure reasons to identify patterns");
    println!("  2. Fix root cause (data format, central server, network)");
    println!("  3. Use 'agent dlq-stats' for detailed analysis");
    println!();

    Ok(())
}

pub async fn handle_dlq_stats_command(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let total_dlq: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await?;

    let by_reason: Vec<(String, i64)> = sqlx::query_as(
        "SELECT failure_reason, COUNT(*) as count
         FROM zkt_failed_logs
         GROUP BY failure_reason
         ORDER BY count DESC"
    )
    .fetch_all(db.get_pool())
    .await?;

    let by_device: Vec<(String, i64)> = sqlx::query_as(
        "SELECT device_id, COUNT(*) as count
         FROM zkt_failed_logs
         GROUP BY device_id
         ORDER BY count DESC
         LIMIT 10"
    )
    .fetch_all(db.get_pool())
    .await?;

    let recent_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_failed_logs WHERE failed_at > datetime('now', '-24 hours')"
    )
    .fetch_one(db.get_pool())
    .await?;

    let recent_1h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_failed_logs WHERE failed_at > datetime('now', '-1 hour')"
    )
    .fetch_one(db.get_pool())
    .await?;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    DEAD LETTER QUEUE STATISTICS                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total DLQ entries: {total_dlq}");
    println!("  - Last hour: {recent_1h}");
    println!("  - Last 24 hours: {recent_24h}");
    println!();

    if total_dlq == 0 {
        println!("✅ No entries in dead letter queue. System healthy!");
        println!();
        return Ok(());
    }

    println!("Failures by Reason:");
    println!("┌─────────────────────────────────────────────┬────────┬─────────┐");
    println!("│ Failure Reason                              │ Count  │ Percent │");
    println!("├─────────────────────────────────────────────┼────────┼─────────┤");

    for (reason, count) in &by_reason {
        let percent = (*count as f64 / total_dlq as f64) * 100.0;
        println!(
            "│ {:43} │ {:6} │ {:6.1}% │",
            truncate(reason, 43),
            count,
            percent
        );
    }
    println!("└─────────────────────────────────────────────┴────────┴─────────┘");
    println!();

    if !by_device.is_empty() {
        println!("Top Failing Devices:");
        println!("┌──────────────────────────┬────────┬─────────┐");
        println!("│ Device ID                │ Count  │ Percent │");
        println!("├──────────────────────────┼────────┼─────────┤");

        for (device, count) in &by_device {
            let percent = (*count as f64 / total_dlq as f64) * 100.0;
            println!(
                "│ {:24} │ {:6} │ {:6.1}% │",
                truncate(device, 24),
                count,
                percent
            );
        }
        println!("└──────────────────────────┴────────┴─────────┘");
        println!();
    }

    // Health assessment
    println!("Health Assessment:");
    if recent_1h > 20 {
        println!("  ⚠️  CRITICAL: {recent_1h} logs moved to DLQ in last hour");
        println!("      Action: Check central server status immediately");
    } else if recent_24h > 100 {
        println!("  ⚠️  WARNING: {recent_24h} logs moved to DLQ in last 24h");
        println!("      Action: Investigate failure reasons and trends");
    } else if total_dlq > 1000 {
        println!("  ℹ️  INFO: {total_dlq} total DLQ entries (consider cleanup)");
        println!("      Action: Archive old entries or investigate patterns");
    } else {
        println!("  ✅ HEALTHY: Low DLQ growth rate");
    }
    println!();

    Ok(())
}

pub async fn handle_config_command(db: &Database, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() == 2 {
        // Show all config
        let configs: Vec<(String, String)> = sqlx::query_as(
            "SELECT key, value FROM agent_config ORDER BY key"
        )
        .fetch_all(db.get_pool())
        .await?;

        println!();
        println!("╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                          AGENT CONFIGURATION                                 ║");
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
        println!();

        if configs.is_empty() {
            println!("No custom configuration set. Using defaults:");
            println!("  max_retry_count      = 10");
            println!("  retry_ttl_minutes    = 60");
            println!("  max_concurrent_workers = 2 (server-managed)");
            println!();
        } else {
            println!("Current Configuration:");
            println!("┌────────────────────────────┬─────────────────────────────────┐");
            println!("│ Key                        │ Value                           │");
            println!("├────────────────────────────┼─────────────────────────────────┤");

            for (key, value) in &configs {
                println!(
                    "│ {:26} │ {:31} │",
                    truncate(key, 26),
                    truncate(value, 31)
                );
            }
            println!("└────────────────────────────┴─────────────────────────────────┘");
            println!();
        }
    } else if args.len() == 3 {
        // Get specific key
        let key = &args[2];
        let value = db.get_agent_config(key).await?;

        println!();
        match value {
            Some(v) => println!("{key} = {v}"),
            None => println!("{key} not set (using default)"),
        }
        println!();
    } else if args.len() == 4 {
        // Set key value
        let key = &args[2];
        let value = &args[3];

        db.upsert_agent_config(key, value).await?;
        println!();
        println!("✅ Configuration updated: {key} = {value}");
        println!("   Note: Agent restart may be required for some settings");
        println!();
    } else {
        eprintln!("Usage: agent config [key] [value]");
        return Ok(());
    }

    Ok(())
}

pub async fn handle_status_command(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let total_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance")
        .fetch_one(db.get_pool())
        .await?;

    let unsynced: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0")
        .fetch_one(db.get_pool())
        .await?;

    let in_flight: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2")
        .fetch_one(db.get_pool())
        .await?;

    let synced: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 1")
        .fetch_one(db.get_pool())
        .await?;

    let dlq_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await?;

    let dlq_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_failed_logs WHERE failed_at > datetime('now', '-24 hours')"
    )
    .fetch_one(db.get_pool())
        .await?;

    // Get retry classification
    let new_logs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0 AND updated_at IS NULL"
    )
    .fetch_one(db.get_pool())
    .await?;

    let retry_logs = unsynced - new_logs;
    let retry_rate = if unsynced > 0 {
        (retry_logs as f64 / unsynced as f64) * 100.0
    } else {
        0.0
    };

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                            AGENT STATUS                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    println!("Sync Queue Status:");
    println!("  Total logs in database:  {total_logs}");
    println!("  ├─ Synced (✅):           {synced}");
    println!("  ├─ In-flight (⏳):        {in_flight}");
    println!("  └─ Unsynced (📤):         {unsynced}");
    println!("     ├─ New logs:          {} ({:.1}%)", new_logs, if unsynced > 0 { (new_logs as f64 / unsynced as f64) * 100.0 } else { 0.0 });
    println!("     └─ Retrying:          {retry_logs} ({retry_rate:.1}%)");
    println!();

    println!("Dead Letter Queue (DLQ):");
    println!("  Total failed:            {dlq_total}");
    println!("  Last 24 hours:           {dlq_24h}");
    println!();

    // Health indicator
    println!("System Health:");
    if retry_rate > 70.0 {
        println!("  ⚠️  CRITICAL: High retry rate ({retry_rate:.1}%)");
        println!("      Action: Check central server connectivity");
    } else if retry_rate > 30.0 {
        println!("  ⚠️  WARNING: Elevated retry rate ({retry_rate:.1}%)");
        println!("      Action: Monitor central server");
    } else if dlq_24h > 50 {
        println!("  ⚠️  WARNING: {dlq_24h} logs moved to DLQ in 24h");
        println!("      Action: Review DLQ stats");
    } else {
        println!("  ✅ HEALTHY: System operating normally");
        println!("      Retry rate: {retry_rate:.1}%");
        println!("      DLQ growth (24h): {dlq_24h} entries");
    }
    println!();

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{s:max_len$}")
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
