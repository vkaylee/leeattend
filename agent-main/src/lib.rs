// Multi-brand Attendance Agent Orchestrator library crate
#![allow(
    missing_docs,
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
    clippy::disallowed_types,
    clippy::disallowed_methods
)]
// Allow unwrap and expect in test code
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::ref_option_ref,
        clippy::disallowed_macros,
        clippy::unimplemented,
        clippy::manual_strip,
        clippy::if_same_then_else,
        clippy::float_cmp,
        clippy::module_inception,
        clippy::used_underscore_binding,
        clippy::default_trait_access
    )
)]

pub mod adaptive_polling;
pub mod central_client;
pub mod config;
pub mod crash_recovery;
pub mod hybrid_database;
pub mod models;
pub mod redb_store;
pub mod retry;
pub mod utils;

pub mod mock_worker;
pub mod worker;
pub mod worker_processor;

pub mod orchestrator;
pub mod time_sync;
pub mod zk_worker;
