#![allow(clippy::disallowed_macros)]

use agent::redb_store::RedbStore;
use std::collections::HashMap;

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
    println!();
    println!("  Status & Metrics:");
    println!("    agent status             Show queue sizes, retry rates, DLQ growth");
    println!();
}

pub fn handle_failed_logs_command(
    store: &RedbStore,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let limit = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(50)
    } else {
        50
    };

    let all_logs = store.get_all_logs()?;
    let failed_logs: Vec<_> = all_logs
        .iter()
        .filter(|l| l.is_synced == 0 && l.updated_at.is_some())
        .take(limit)
        .collect();
    let total_count = all_logs
        .iter()
        .filter(|l| l.is_synced == 0 && l.updated_at.is_some())
        .count();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                          FAILED SYNC LOGS REPORT                             ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "Total failed logs: {} (showing top {})",
        total_count,
        failed_logs.len()
    );
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
        let ts_str = chrono::DateTime::from_timestamp(log.timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| format!("invalid_{}", log.timestamp));
        let status_str = log
            .status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "NULL".into());
        let punch_str = log
            .punch
            .map(|p| p.to_string())
            .unwrap_or_else(|| "NULL".into());

        println!(
            "│ {:7} │ {:10} │ {:10} │ {:19} │ {:6} │ {:5} │",
            log.id,
            truncate(&log.device_id, 10),
            truncate(&log.user_id, 10),
            truncate(&ts_str, 19),
            truncate(&status_str, 6),
            truncate(&punch_str, 5)
        );
    }

    println!("└─────────┴────────────┴────────────┴─────────────────────┴────────┴───────┘");
    println!();
    Ok(())
}

pub fn handle_dlq_command(
    store: &RedbStore,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let limit = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(50)
    } else {
        50
    };

    let failed_logs = store.get_failed_logs()?;
    let dlq_count = failed_logs.len();
    let shown: Vec<_> = failed_logs.into_iter().take(limit).collect();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                      DEAD LETTER QUEUE (DLQ) REPORT                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!(
        "Total DLQ entries: {} (showing top {})",
        dlq_count,
        shown.len()
    );
    println!();

    if shown.is_empty() {
        println!("✅ No logs in dead letter queue. All logs either synced or still retrying.");
        println!();
        return Ok(());
    }

    println!(
        "┌────────┬───────────┬───────────┬──────────────────┬────────┬────────────────────────┐"
    );
    println!(
        "│ ID     │ Device    │ User ID   │ Failed At        │ Retries│ Failure Reason         │"
    );
    println!(
        "├────────┼───────────┼───────────┼──────────────────┼────────┼────────────────────────┤"
    );

    for log in &shown {
        let failed_display = chrono::DateTime::from_timestamp(log.failed_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "unknown".into());

        println!(
            "│ {:6} │ {:9} │ {:9} │ {:16} │ {:6} │ {:22} │",
            log.id,
            truncate(&log.device_id, 9),
            truncate(&log.user_id, 9),
            truncate(&failed_display, 16),
            log.retry_attempts,
            truncate(&log.failure_reason, 22)
        );
    }

    println!(
        "└────────┴───────────┴───────────┴──────────────────┴────────┴────────────────────────┘"
    );
    println!();
    Ok(())
}

pub fn handle_dlq_stats_command(store: &RedbStore) -> Result<(), Box<dyn std::error::Error>> {
    let failed_logs = store.get_failed_logs()?;
    let total_dlq = i64::try_from(failed_logs.len()).unwrap_or(0);

    // Aggregate by reason
    let mut by_reason: HashMap<String, i64> = HashMap::new();
    let mut by_device: HashMap<String, i64> = HashMap::new();
    let now = chrono::Utc::now().timestamp();
    let mut recent_24h: i64 = 0;
    let mut recent_1h: i64 = 0;

    for log in &failed_logs {
        *by_reason.entry(log.failure_reason.clone()).or_insert(0) += 1;
        *by_device.entry(log.device_id.clone()).or_insert(0) += 1;
        if now - log.failed_at < 86400 {
            recent_24h += 1;
        }
        if now - log.failed_at < 3600 {
            recent_1h += 1;
        }
    }

    let mut reason_sorted: Vec<_> = by_reason.into_iter().collect();
    reason_sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
    let mut device_sorted: Vec<_> = by_device.into_iter().collect();
    device_sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
    device_sorted.truncate(10);

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

    for (reason, count) in &reason_sorted {
        #[allow(clippy::cast_precision_loss)]
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

    if !device_sorted.is_empty() {
        println!("Top Failing Devices:");
        println!("┌──────────────────────────┬────────┬─────────┐");
        println!("│ Device ID                │ Count  │ Percent │");
        println!("├──────────────────────────┼────────┼─────────┤");
        for (device, count) in &device_sorted {
            #[allow(clippy::cast_precision_loss)]
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
    } else if recent_24h > 100 {
        println!("  ⚠️  WARNING: {recent_24h} logs moved to DLQ in last 24h");
    } else {
        println!("  ✅ HEALTHY: Low DLQ growth rate");
    }
    println!();
    Ok(())
}

pub fn handle_config_command(
    store: &RedbStore,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() == 2 {
        println!();
        println!(
            "╔══════════════════════════════════════════════════════════════════════════════╗"
        );
        println!(
            "║                          AGENT CONFIGURATION                                 ║"
        );
        println!(
            "╚══════════════════════════════════════════════════════════════════════════════╝"
        );
        println!();

        // Show known config keys
        let keys = &[
            "max_retry_count",
            "retry_ttl_minutes",
            "max_concurrent_workers",
        ];
        let mut found = false;
        for key in keys {
            if let Ok(Some(val)) = store.get_agent_config(key) {
                if !found {
                    println!("Current Configuration:");
                    found = true;
                }
                println!("  {key} = {val}");
            }
        }
        if !found {
            println!("No custom configuration set. Using defaults.");
        }
        println!();
    } else if args.len() == 3 {
        let key = &args[2];
        let value = store
            .get_agent_config(key)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        println!();
        match value {
            Some(v) => println!("{key} = {v}"),
            None => println!("{key} not set (using default)"),
        }
        println!();
    } else if args.len() == 4 {
        let key = &args[2];
        let value = &args[3];
        store
            .upsert_agent_config(key, value)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
        println!();
        println!("✅ Configuration updated: {key} = {value}");
        println!();
    }
    Ok(())
}

pub fn handle_status_command(store: &RedbStore) -> Result<(), Box<dyn std::error::Error>> {
    let all_logs = store.get_all_logs()?;
    let total_logs = all_logs.len();
    let unsynced = all_logs.iter().filter(|l| l.is_synced == 0).count();
    let in_flight = all_logs.iter().filter(|l| l.is_synced == 2).count();
    let new_logs = all_logs
        .iter()
        .filter(|l| l.is_synced == 0 && l.updated_at.is_none())
        .count();
    let retry_logs = unsynced - new_logs;
    #[allow(clippy::cast_precision_loss)]
    let retry_rate = if unsynced > 0 {
        (retry_logs as f64 / unsynced as f64) * 100.0
    } else {
        0.0
    };

    let dlq_total = store.count_failed_logs()?;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                            AGENT STATUS                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Sync Queue Status:");
    println!("  Total active logs:       {total_logs}");
    println!("  ├─ In-flight (⏳):        {in_flight}");
    println!("  └─ Unsynced (📤):         {unsynced}");
    println!("     ├─ New logs:          {new_logs}");
    println!("     └─ Retrying:          {retry_logs} ({retry_rate:.1}%)");
    println!();
    println!("Dead Letter Queue (DLQ):");
    println!("  Total failed:            {dlq_total}");
    println!();

    println!("System Health:");
    if retry_rate > 70.0 {
        println!("  ⚠️  CRITICAL: High retry rate ({retry_rate:.1}%)");
    } else if retry_rate > 30.0 {
        println!("  ⚠️  WARNING: Elevated retry rate ({retry_rate:.1}%)");
    } else {
        println!("  ✅ HEALTHY: System operating normally");
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
