"""Tests to ensure infrastructure files maintain standardized best practices via Abstract Compliance Interfaces."""

from abc import ABC, abstractmethod
from pathlib import Path


class BaseComplianceChecker(ABC):
    """Abstract Base Class enforcing standard structure and AI-guidance for infrastructure compliance tests."""

    @property
    @abstractmethod
    def target_file(self) -> str:
        """The relative path to the file being checked from project root."""
        pass

    @abstractmethod
    def get_mandatory_rules(self) -> dict[str, str] | list[str]:
        """
        Return either:
        - A list of string rules: The base class will auto-generate default AI guidance.
        - A dictionary: key = rule string, value = custom explicit AI guidance.
        """
        pass

    def execute_compliance_check(self):
        """Core template method to read the file and assert all mandatory rules with structured AI guidance."""
        project_root = Path(__file__).parent.parent.parent
        file_path = project_root / self.target_file

        assert file_path.exists(), f"Target file missing: {self.target_file}"

        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()

        # Strip comments and empty lines
        lines = [
            line.strip()
            for line in content.splitlines()
            if line.strip() and not line.startswith("#")
        ]

        # Normalize rules
        rules_data = self.get_mandatory_rules()
        normalized_rules = {}

        if isinstance(rules_data, list):
            for rule in rules_data:
                normalized_rules[rule] = (
                    f"Use the file editing tool to open '{self.target_file}' at the project root and append '{rule}'."
                )
        elif isinstance(rules_data, dict):
            for rule, guidance in rules_data.items():
                normalized_rules[rule] = (
                    guidance
                    if guidance
                    else f"Use the file editing tool to open '{self.target_file}' at the project root and append '{rule}'."
                )
        else:
            raise TypeError("get_mandatory_rules must return a list[str] or dict[str, str].")

        for rule, ai_guidance in normalized_rules.items():
            error_msg = (
                f"\n\n🚨 [AI_GUIDANCE: ACTION REQUIRED] 🚨\n"
                f"ERROR: The mandatory rule '{rule}' is missing from {self.target_file}.\n"
                f"HOW TO FIX: {ai_guidance}\n"
                f"DO NOT ignore this failure. Fix it immediately to maintain compliance.\n"
            )
            assert rule in lines, error_msg


# ==============================================================================
# Concrete Implementations (Pytest will discover classes starting with 'Test')
# ==============================================================================


class TestDockerIgnoreCompliance(BaseComplianceChecker):
    """Concrete compliance test ensuring .dockerignore prevents security leaks and cache invalidation."""

    @property
    def target_file(self) -> str:
        return ".dockerignore"

    def get_mandatory_rules(self) -> list[str]:
        return [
            # Baseline & Security Rules
            ".env",
            "**/node_modules",
            "**/target/",
            ".pytest_cache",
            # Docker-Specific Cache Optimization Rules
            ".git/",
            ".cursorrules",
            ".gemini/",
            "docker-compose*.yml",
        ]

    def test_dockerignore_rules(self):
        """Pytest entrypoint to trigger the compliance check."""
        self.execute_compliance_check()


class TestGitIgnoreCompliance(BaseComplianceChecker):
    """Concrete compliance test ensuring .gitignore blocks secrets, logs, and heavy build folders."""

    @property
    def target_file(self) -> str:
        return ".gitignore"

    def get_mandatory_rules(self) -> list[str]:
        return [
            # Security
            ".env",
            "*.env",
            # Caches & Environments
            "__pycache__/",
            ".pytest_cache/",
            ".venv/",
            # Build Artifacts
            "**/target/",
            "node_modules/",
            # Logs
            "*.log",
            "logs/",
        ]

    def test_gitignore_rules(self):
        """Pytest entrypoint to trigger the compliance check."""
        self.execute_compliance_check()


class TestEnvExampleCompliance(BaseComplianceChecker):
    """Concrete compliance test ensuring .env.example contains necessary template variables."""

    @property
    def target_file(self) -> str:
        return ".env.example"

    def get_mandatory_rules(self) -> list[str]:
        return [
            "DB_NAME=leeattend",
            "DB_USER=admin",
            "DB_PASSWORD=your_password_here",
            "JWT_SECRET=your_jwt_secret_here",
        ]

    def test_envexample_rules(self):
        """Pytest entrypoint to trigger the compliance check."""
        self.execute_compliance_check()
