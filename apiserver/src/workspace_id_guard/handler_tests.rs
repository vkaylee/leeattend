use super::context_allowlist::CONTEXT_ALLOWLIST;
use super::helpers::get_rs_files;
use std::fs;
use std::path::PathBuf;

#[test]
#[allow(clippy::disallowed_methods)]
fn handlers_must_use_member_context_for_workspace_id() {
    let handlers_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("handlers");
    assert!(
        handlers_dir.exists(),
        "Handlers directory not found: {handlers_dir:?}"
    );

    let rs_files = get_rs_files(&handlers_dir);
    assert!(!rs_files.is_empty(), "No handler .rs files found");

    let mut violations: Vec<String> = Vec::new();
    let mut scanned_handlers = 0u32;

    for file_path in &rs_files {
        let content = fs::read_to_string(file_path).expect("Failed to read file");
        let rel_path = file_path
            .strip_prefix(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("src")
                    .join("handlers"),
            )
            .unwrap_or(file_path.as_path())
            .to_str()
            .unwrap_or("");
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip this guard file itself
        if file_name == "workspace_id_guard.rs" {
            continue;
        }

        // Check if this file is in the context allowlist
        let is_allowed = CONTEXT_ALLOWLIST.iter().any(|(f, _)| rel_path.ends_with(f));
        if is_allowed {
            continue;
        }

        scanned_handlers += 1;

        // Detect unsafe patterns: Path extractors containing ws_id or workspace_id
        for (line_no, line) in content.lines().enumerate() {
            let line_trimmed = line.trim();

            let has_path_ws = line_trimmed.contains("Path(ws_id)")
                || line_trimmed.contains("Path(workspace_id)")
                || line_trimmed.contains("Path((ws_id")
                || line_trimmed.contains("Path((workspace_id");

            if has_path_ws {
                // Ignore if the variable is prefixed with _ (intentionally unused)
                let is_unused =
                    line_trimmed.contains("_ws_id") || line_trimmed.contains("_workspace_id");
                if is_unused {
                    continue;
                }

                violations.push(format!(
                    "  handlers/{}:{} — workspace_id from Path (use MemberContext instead)\n    {}",
                    rel_path,
                    line_no + 1,
                    line_trimmed,
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔴 CONTEXT SOURCE GUARD FAILURE\n\
             ════════════════════════════════════════════════════════════\n\
             {} handler(s) extract workspace_id from Path instead of MemberContext:\n\n{}\n\n\
             FIX: Replace `Path(ws_id): Path<Uuid>` with `ctx: MemberContext`\n\
             and use `ctx.workspace_id`. This ensures the user is a verified\n\
             member of the workspace.\n\n\
             If this is a legitimate exception (e.g., admin endpoint),\n\
             add to CONTEXT_ALLOWLIST in workspace_id_guard/context_allowlist.rs.\n\
             ════════════════════════════════════════════════════════════\n",
        violations.len(),
        violations.join("\n")
    );

    assert!(
        scanned_handlers > 5,
        "Only scanned {scanned_handlers} handler files — directory structure may have changed"
    );

    eprintln!(
        "✅ context source guard: scanned {scanned_handlers} handler files, all use MemberContext"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn handler_functions_must_not_accept_workspace_id_param() {
    let handlers_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("handlers");

    let rs_files = get_rs_files(&handlers_dir);
    let mut violations: Vec<String> = Vec::new();

    for file_path in &rs_files {
        let content = fs::read_to_string(file_path).expect("Failed to read file");
        let rel_path = file_path
            .strip_prefix(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("src")
                    .join("handlers"),
            )
            .unwrap_or(file_path.as_path())
            .to_str()
            .unwrap_or("");
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name == "workspace_id_guard.rs" || file_name == "common.rs" || file_name == "mod.rs"
        {
            continue;
        }

        // Check for Query/Json extractors containing workspace_id
        for (line_no, line) in content.lines().enumerate() {
            let line_trimmed = line.trim();

            let has_query_ws = line_trimmed.contains("Query<")
                && (line_trimmed.contains("workspace_id") || line_trimmed.contains("ws_id"));

            if has_query_ws {
                violations.push(format!(
                    "  handlers/{}:{} — workspace_id in Query params (use MemberContext)\n    {}",
                    rel_path,
                    line_no + 1,
                    line_trimmed,
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔴 QUERY PARAM GUARD FAILURE\n\
             ════════════════════════════════════════════════════════════\n\
             {} handler(s) accept workspace_id via Query params:\n\n{}\n\n\
             FIX: Remove workspace_id from query params. Use MemberContext\n\
             to get the workspace from the authenticated session.\n\
             ════════════════════════════════════════════════════════════\n",
        violations.len(),
        violations.join("\n")
    );
}
