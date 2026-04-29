mod cli;

use agent::hybrid_database::HybridDatabase;
use agent::orchestrator::Orchestrator;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Check for CLI commands
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let config = agent::config::Config::load();
        let hybrid_db = HybridDatabase::new(&config.database_url, 1000, 90)
            .expect("Failed to initialize database");

        match args[1].as_str() {
            "failed-logs" | "failed" => {
                return cli::handle_failed_logs_command(hybrid_db.store(), &args);
            }
            "dlq" | "dead-letter-queue" => {
                return cli::handle_dlq_command(hybrid_db.store(), &args);
            }
            "dlq-stats" | "stats" => {
                return cli::handle_dlq_stats_command(hybrid_db.store());
            }
            "config" => {
                return cli::handle_config_command(hybrid_db.store(), &args);
            }
            "status" => {
                return cli::handle_status_command(hybrid_db.store());
            }
            "--help" | "-h" | "help" => {
                cli::print_help();
                return Ok(());
            }
            "--version" | "-v" => {
                cli::print_version();
                return Ok(());
            }
            _ => {
                eprintln!("❌ Unknown command: {}", args[1]);
                eprintln!();
                cli::print_help();
                return Ok(());
            }
        }
    }

    info!("🚀 Starting Multi-Brand Rust Agent Orchestrator...");

    let config = agent::config::Config::load();
    let orchestrator = Orchestrator::new(config)?;

    orchestrator.run().await
}
