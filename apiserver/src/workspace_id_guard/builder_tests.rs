use super::helpers::{get_rs_files, is_allowlisted, strip_comments};
use super::tables::TENANT_TABLES;
use std::fs;
use std::path::PathBuf;

#[test]
#[allow(clippy::disallowed_methods)]
fn all_tenant_query_builders_must_include_workspace_id() {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let rs_files = get_rs_files(&src_dir);
    let mut violations = Vec::new();

    fn is_import_or_decl(content: &str, pos: usize) -> bool {
        let before = &content[..pos];
        if let Some(last_newline) = before.rfind('\n') {
            let line = &before[last_newline + 1..].trim_start();
            line.starts_with("use ") || line.starts_with("pub use ")
        } else {
            before.trim_start().starts_with("use ")
        }
    }

    fn get_statement_context(content: &str, pos: usize) -> &str {
        let before = &content[..pos];
        let start_pos = before
            .rfind(';')
            .or_else(|| before.rfind('{'))
            .or_else(|| before.rfind('}'))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        &content[start_pos..]
    }

    for file_path in &rs_files {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "workspace_id_guard.rs"
            || file_name == "schema.rs"
            || file_name == "query_tests.rs"
            || file_name == "handler_tests.rs"
            || file_name == "builder_tests.rs"
        {
            continue;
        }

        let content = fs::read_to_string(file_path).expect("Failed to read file");
        let stripped = strip_comments(&content);

        for &table in TENANT_TABLES {
            let pattern1 = format!("{table}::table");
            let pattern2 = format!("schema::{table}::table");

            let mut search_idx = 0;
            while let Some(pos) = stripped[search_idx..]
                .find(&pattern1)
                .or_else(|| stripped[search_idx..].find(&pattern2))
            {
                let abs_pos = search_idx + pos;
                if is_import_or_decl(&stripped, abs_pos) {
                    search_idx = abs_pos + 1;
                    continue;
                }

                let remaining = &stripped[abs_pos..];
                let end_pos = remaining.find(';').unwrap_or(remaining.len());
                let statement = &remaining[..end_pos];

                let context = get_statement_context(&stripped, abs_pos);
                let is_safe = statement.contains("workspace_id")
                    || statement.contains("workspace_members")
                    || statement.contains("ws_id")
                    || context.contains("insert_into")
                    || is_allowlisted(file_name, table);

                if !is_safe {
                    let line_no = content[..abs_pos].lines().count();
                    violations.push(format!(
                        "  {file_name}:{line_no} — table `{table}` query builder without workspace_id\n    Statement: {}",
                        statement.trim().lines().next().unwrap_or("").trim()
                    ));
                }

                search_idx = abs_pos + 1;
                if search_idx >= stripped.len() {
                    break;
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔴 DIESEL QUERY BUILDER GUARD FAILURE\n\
             ════════════════════════════════════════════════════════════\n\
             {} query builder(s) touch tenant-scoped tables WITHOUT workspace_id:\n\n{}\n\n\
             FIX: Chain `.filter(workspace_id.eq(...))` to the query builder,\n\
             or add an entry to ALLOWLIST in workspace_id_guard/allowlist.rs with a reason.\n\
             ════════════════════════════════════════════════════════════\n",
        violations.len(),
        violations.join("\n")
    );
}
