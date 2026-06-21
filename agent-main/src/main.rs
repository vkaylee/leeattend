// Multi-brand Attendance Agent Orchestrator executable crate
#![allow(
    missing_docs,
    clippy::disallowed_macros,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_field_names,
    clippy::option_if_let_else,
    clippy::items_after_statements,
    clippy::clone_on_ref_ptr,
    clippy::unused_async,
    clippy::uninlined_format_args,
    clippy::manual_let_else,
    clippy::single_match_else,
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::or_fun_call,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::ignored_unit_patterns,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::map_unwrap_or,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]
// Allow unwrap and expect in test code
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::ref_option_ref,
        clippy::unimplemented,
        clippy::manual_strip,
        clippy::if_same_then_else,
        clippy::float_cmp,
        clippy::module_inception,
        clippy::used_underscore_binding,
        clippy::default_trait_access
    )
)]

mod cli;

use agent::hybrid_database::HybridDatabase;
use agent::orchestrator::Orchestrator;
use log::info;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Explicitly install ring as the default CryptoProvider for rustls
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Check for CLI commands
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let config = agent::config::Config::load();
        #[allow(clippy::expect_used)]
        let hybrid_db = HybridDatabase::new(config.database_url(), 1000, 90)
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
