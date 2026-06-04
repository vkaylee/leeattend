import sys
from pathlib import Path
from unittest.mock import MagicMock, mock_open, patch

import pytest

# Add scripts to path
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.append(str(Path(__file__).resolve().parent.parent))

import lint_tenant_isolation  # noqa: E402


def test_extract_methods() -> None:
    # 1. Normal successful extraction
    rust_code_normal = """
    impl Repo {
        async fn create_item(
            &self,
            auth_token: &TenantAuthToken,
            data: Data,
        ) -> Result<Item, Error> {
            let workspace_id = auth_token.workspace_id();
            let query = items::table.filter(items::workspace_id.eq(workspace_id));
            Ok(query.first()?)
        }

        fn get_item(&self, id: Uuid) -> Option<Item> {
            None
        }
    }
    """
    methods = lint_tenant_isolation.extract_methods(rust_code_normal)
    assert len(methods) >= 2

    create_method = next(m for m in methods if m["name"] == "create_item")
    assert "TenantAuthToken" in create_method["signature"]
    assert "workspace_id" in create_method["body"]

    get_method = next(m for m in methods if m["name"] == "get_item")
    assert "TenantAuthToken" not in get_method["signature"]

    # 2. Edge case: name match fails
    # Let's test a case where fn match is found but name_match is None.
    # Re matches: r'\b(async\s+)?fn\s+(\w+)'
    # If we have "fn  ", it might not match \w+. Let's test by mock or raw text.
    # Note: re.finditer matches \b(async\s+)?fn\s+(\w+), so name_match always matches \w+ if re found it.
    # But let's mock or pass a weird string to test.
    # 3. Edge case: open_brace_idx == -1 (no open brace)
    rust_code_no_brace = "async fn get_something(auth_token: &TenantAuthToken)"
    assert len(lint_tenant_isolation.extract_methods(rust_code_no_brace)) == 0

    # 4. Edge case: brace count mismatch (no matching end brace)
    rust_code_no_end_brace = "async fn get_something() { let x = 1;"
    assert len(lint_tenant_isolation.extract_methods(rust_code_no_end_brace)) == 0


@patch("lint_tenant_isolation.Path")
@patch("sys.exit")
def test_main_success(mock_exit: MagicMock, mock_path_cls: MagicMock) -> None:
    # Set up mocks for repo_dir
    mock_repo_dir = MagicMock()
    mock_repo_dir.exists.return_value = True

    # We have one mock diesel repo file and one system repo file to skip
    mock_file = MagicMock()
    mock_file.is_file.return_value = True
    mock_file.name = "diesel_some_repository.rs"

    mock_sys_file = MagicMock()
    mock_sys_file.is_file.return_value = True
    mock_sys_file.name = "diesel_system_repository.rs"

    mock_repo_dir.iterdir.return_value = [mock_file, mock_sys_file]
    mock_path_cls.return_value = mock_repo_dir

    # Mock file reading
    repo_file_content = """
    impl SomeRepo {
        async fn test_method(
            &self,
            auth_token: &TenantAuthToken,
        ) -> Result<(), Error> {
            if true {
                let workspace_id = auth_token.workspace_id();
                let q = employees::table.filter(employees::workspace_id.eq(workspace_id));
            }
            Ok(())
        }
    }
    """

    mock_opened_file = mock_open(read_data=repo_file_content)
    mock_filepath = MagicMock()
    mock_filepath.open = mock_opened_file
    mock_repo_dir.__truediv__.return_value = mock_filepath

    lint_tenant_isolation.main()

    mock_exit.assert_called_once_with(0)


@patch("lint_tenant_isolation.Path")
@patch("sys.exit")
def test_main_failure(mock_exit: MagicMock, mock_path_cls: MagicMock) -> None:
    # Set up mocks for repo_dir
    mock_repo_dir = MagicMock()
    mock_repo_dir.exists.return_value = True

    mock_file = MagicMock()
    mock_file.is_file.return_value = True
    mock_file.name = "diesel_some_repository.rs"

    mock_repo_dir.iterdir.return_value = [mock_file]
    mock_path_cls.return_value = mock_repo_dir

    # Query is made on table but workspace_id is not checked in the query
    repo_file_content = """
    impl SomeRepo {
        async fn test_method(
            &self,
            auth_token: &TenantAuthToken,
        ) -> Result<(), Error> {
            let q = employees::table.load(&mut conn);
            Ok(())
        }
    }
    """

    mock_opened_file = mock_open(read_data=repo_file_content)
    mock_filepath = MagicMock()
    mock_filepath.open = mock_opened_file
    mock_repo_dir.__truediv__.return_value = mock_filepath

    lint_tenant_isolation.main()

    mock_exit.assert_called_once_with(1)


@patch("lint_tenant_isolation.Path")
@patch("sys.exit")
def test_main_warning(mock_exit: MagicMock, mock_path_cls: MagicMock) -> None:
    # Set up mocks for repo_dir
    mock_repo_dir = MagicMock()
    mock_repo_dir.exists.return_value = True

    mock_file = MagicMock()
    mock_file.is_file.return_value = True
    mock_file.name = "diesel_some_repository.rs"

    mock_repo_dir.iterdir.return_value = [mock_file]
    mock_path_cls.return_value = mock_repo_dir

    # TenantAuthToken passed but no queries exist and no workspace reference
    repo_file_content = """
    impl SomeRepo {
        async fn test_method(
            &self,
            auth_token: &TenantAuthToken,
        ) -> Result<(), Error> {
            Ok(())
        }
    }
    """

    mock_opened_file = mock_open(read_data=repo_file_content)
    mock_filepath = MagicMock()
    mock_filepath.open = mock_opened_file
    mock_repo_dir.__truediv__.return_value = mock_filepath

    lint_tenant_isolation.main()

    mock_exit.assert_called_once_with(0)


@patch("lint_tenant_isolation.Path")
@patch("sys.exit")
def test_main_query_no_workspace_ref(mock_exit: MagicMock, mock_path_cls: MagicMock) -> None:
    # Set up mocks for repo_dir
    mock_repo_dir = MagicMock()
    mock_repo_dir.exists.return_value = True

    mock_file = MagicMock()
    mock_file.is_file.return_value = True
    mock_file.name = "diesel_some_repository.rs"

    mock_repo_dir.iterdir.return_value = [mock_file]
    mock_path_cls.return_value = mock_repo_dir

    # Query is made on table but has_workspace_ref is False (no workspace_id or auth_token in body)
    repo_file_content = """
    impl SomeRepo {
        async fn test_method(
            &self,
            auth_token: &TenantAuthToken,
        ) -> Result<(), Error> {
            let x = employees::table;
            Ok(())
        }
    }
    """

    mock_opened_file = mock_open(read_data=repo_file_content)
    mock_filepath = MagicMock()
    mock_filepath.open = mock_opened_file
    mock_repo_dir.__truediv__.return_value = mock_filepath

    lint_tenant_isolation.main()

    mock_exit.assert_called_once_with(1)


@patch("lint_tenant_isolation.Path")
@patch("sys.exit")
def test_main_dir_missing(mock_exit: MagicMock, mock_path_cls: MagicMock) -> None:
    mock_repo_dir = MagicMock()
    mock_repo_dir.exists.return_value = False
    mock_path_cls.return_value = mock_repo_dir

    lint_tenant_isolation.main()

    mock_exit.assert_called_once_with(1)
