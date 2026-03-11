from __future__ import annotations

import json
import os
from pathlib import Path

from aigiscode.models import ExternalFinding
from aigiscode.rules.checks import StructuralContext
from aigiscode.rules.engine import Rule, filter_external_findings
from aigiscode.security.external import (
    _parse_cargo_deny_output,
    _parse_composer_audit_payload,
    _parse_npm_audit_payload,
    _refine_findings,
    _sanitize_stderr,
    collect_external_analysis,
)


def test_parse_composer_audit_output_imports_advisories_and_abandoned() -> None:
    payload = """
    {
      "advisories": {
        "firebase/php-jwt": [
          {
            "advisoryId": "PKSA-123",
            "title": "JWT signature validation bypass",
            "cve": "CVE-2026-0001",
            "link": "https://example.test/advisory",
            "severity": "high",
            "affectedVersions": "<7.0.0"
          }
        ]
      },
      "abandoned": {
        "legacy/package": "replacement/package"
      }
    }
    """

    findings = _parse_composer_audit_payload(json.loads(payload))

    assert len(findings) == 2
    assert findings[0].tool == "composer-audit"
    assert findings[0].domain == "security"
    assert findings[0].category == "sca"
    assert findings[0].rule_id == "PKSA-123"
    assert findings[1].category == "abandoned_dependency"


def test_parse_npm_audit_output_imports_vulnerabilities() -> None:
    payload = """
    {
      "vulnerabilities": {
        "axios": {
          "severity": "high",
          "isDirect": true,
          "via": [
            {
              "source": 1098583,
              "name": "axios",
              "title": "SSRF in axios"
            }
          ],
          "nodes": [
            "node_modules/axios"
          ],
          "fixAvailable": true
        }
      }
    }
    """

    findings = _parse_npm_audit_payload(json.loads(payload))

    assert len(findings) == 1
    assert findings[0].tool == "npm-audit"
    assert findings[0].domain == "security"
    assert findings[0].category == "sca"
    assert findings[0].rule_id == "1098583"
    assert findings[0].severity == "high"


def test_parse_cargo_deny_output_imports_advisories_and_license_findings() -> None:
    payload = "\n".join(
        [
            json.dumps(
                {
                    "type": "diagnostic",
                    "fields": {
                        "severity": "error",
                        "message": "ring is vulnerable",
                        "code": "vulnerability",
                        "labels": [
                            {
                                "message": "affected crate",
                                "line": 1,
                                "column": 1,
                                "span": "ring",
                            }
                        ],
                        "advisory": {
                            "id": "RUSTSEC-2026-0001",
                            "title": "ring advisory",
                        },
                    },
                }
            ),
            json.dumps(
                {
                    "type": "diagnostic",
                    "fields": {
                        "severity": "warning",
                        "message": "license was not explicitly accepted",
                        "code": "rejected",
                        "labels": [
                            {
                                "message": "crate license",
                                "line": 3,
                                "column": 5,
                                "span": "GPL-3.0-only",
                            }
                        ],
                    },
                }
            ),
        ]
    )

    findings = _parse_cargo_deny_output(payload)

    assert len(findings) == 2
    assert findings[0].tool == "cargo-deny"
    assert findings[0].domain == "security"
    assert findings[0].category == "sca"
    assert findings[0].rule_id == "RUSTSEC-2026-0001"
    assert findings[1].category == "license"
    assert findings[1].severity == "medium"


def test_refine_findings_drops_ruff_asserts_in_test_paths() -> None:
    findings, filtered_count = _refine_findings(
        [
            ExternalFinding(
                tool="ruff",
                domain="security",
                category="sast",
                rule_id="S101",
                severity="medium",
                confidence="high",
                file_path="tests/test_auth.py",
                line=10,
                message="Use of assert detected",
                fingerprint="a",
            ),
            ExternalFinding(
                tool="ruff",
                domain="security",
                category="sast",
                rule_id="S608",
                severity="high",
                confidence="medium",
                file_path="app/query_builder.py",
                line=20,
                message="Possible SQL injection vector through string-based query construction",
                fingerprint="b",
            ),
        ]
    )

    assert filtered_count == 1
    assert [finding.rule_id for finding in findings] == ["S608"]


def test_sanitize_stderr_filters_composer_deprecation_noise() -> None:
    sanitized, suppressed = _sanitize_stderr(
        "composer-audit",
        "\n".join(
            [
                "Deprecation Notice: Something noisy",
                "Deprecation Notice: Something else noisy",
                "Composer could not find a lock file",
            ]
        ),
    )

    assert sanitized == "Composer could not find a lock file"
    assert suppressed == 2


def test_filter_external_findings_applies_saved_rules() -> None:
    from aigiscode.models import ExternalAnalysisResult, ExternalToolRun

    external_analysis = ExternalAnalysisResult(
        tool_runs=[
            ExternalToolRun(
                tool="gitleaks",
                command=["gitleaks"],
                status="findings",
                summary={"finding_count": 2},
            )
        ],
        findings=[
            ExternalFinding(
                tool="gitleaks",
                domain="security",
                category="secrets",
                rule_id="generic-api-key",
                severity="high",
                confidence="medium",
                file_path="vendor/seeds/demo.php",
                line=4,
                message="Seed secret",
                fingerprint="a",
            ),
            ExternalFinding(
                tool="gitleaks",
                domain="security",
                category="secrets",
                rule_id="generic-api-key",
                severity="high",
                confidence="medium",
                file_path="app/config.php",
                line=9,
                message="Real secret",
                fingerprint="b",
            ),
        ],
    )

    filtered, excluded = filter_external_findings(
        external_analysis,
        [
            Rule(
                id="rule-seed-secrets",
                category="secrets",
                checks=[{"type": "file_glob", "params": {"pattern": "vendor/seeds/*"}}],
                reason="Ignore generated demo fixtures",
            )
        ],
        ctx=StructuralContext(),
    )

    assert excluded == 1
    assert [finding.file_path for finding in filtered.findings] == ["app/config.php"]
    assert filtered.tool_runs[0].summary["finding_count"] == 1
    assert filtered.tool_runs[0].summary["rules_filtered_count"] == 1


def test_collect_external_analysis_marks_nonzero_npm_without_findings_as_failed(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    (project_root / "package.json").write_text('{"name":"demo","version":"1.0.0"}\n', encoding="utf-8")

    bin_dir = tmp_path / "bin"
    npm_path = bin_dir / "npm"
    npm_path.parent.mkdir(parents=True, exist_ok=True)
    npm_path.write_text(
        """#!/usr/bin/env python3
import json
import sys

print(json.dumps({"error": {"summary": "registry unavailable"}}))
sys.exit(1)
""",
        encoding="utf-8",
    )
    npm_path.chmod(0o755)

    original_path = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{bin_dir}:{original_path}"
    try:
        result = collect_external_analysis(
            project_path=project_root,
            output_dir=tmp_path / ".aigiscode",
            run_id="20260309_150000",
            selected_tools=["npm-audit"],
        )
    finally:
        os.environ["PATH"] = original_path

    assert result.findings == []
    assert result.tool_runs[0].tool == "npm-audit"
    assert result.tool_runs[0].status == "failed"
    assert result.tool_runs[0].summary["error"].startswith("npm-audit exited with code 1")


def test_collect_external_analysis_runs_fake_cargo_clippy(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    (project_root / "Cargo.toml").write_text(
        """[package]
name = "demo"
version = "0.1.0"
edition = "2021"
""",
        encoding="utf-8",
    )

    bin_dir = tmp_path / "bin"
    cargo_path = bin_dir / "cargo"
    cargo_path.parent.mkdir(parents=True, exist_ok=True)
    cargo_path.write_text(
        """#!/usr/bin/env python3
import json

payload = {
    "reason": "compiler-message",
    "message": {
        "level": "warning",
        "message": "this returns a `String` unnecessarily",
        "code": {"code": "clippy::needless_to_string"},
        "spans": [
            {
                "file_name": "src/main.rs",
                "line_start": 7,
                "is_primary": True
            }
        ],
        "rendered": "warning: needless_to_string"
    }
}
print(json.dumps(payload))
""",
        encoding="utf-8",
    )
    cargo_path.chmod(0o755)

    original_path = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{bin_dir}:{original_path}"
    try:
        result = collect_external_analysis(
            project_path=project_root,
            output_dir=tmp_path / ".aigiscode",
            run_id="20260311_130000",
            selected_tools=["cargo-clippy"],
        )
    finally:
        os.environ["PATH"] = original_path

    assert result.tool_runs[0].tool == "cargo-clippy"
    assert result.tool_runs[0].status == "findings"
    assert result.tool_runs[0].summary["finding_count"] == 1
    assert result.findings[0].tool == "cargo-clippy"
    assert result.findings[0].domain == "quality"
    assert result.findings[0].rule_id == "clippy::needless_to_string"
    assert result.findings[0].file_path == "src/main.rs"
    assert result.findings[0].line == 7


def test_collect_external_analysis_runs_fake_cargo_deny(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    (project_root / "Cargo.toml").write_text(
        """[package]
name = "demo"
version = "0.1.0"
edition = "2021"
""",
        encoding="utf-8",
    )

    bin_dir = tmp_path / "bin"
    cargo_deny_path = bin_dir / "cargo-deny"
    cargo_deny_path.parent.mkdir(parents=True, exist_ok=True)
    cargo_deny_path.write_text(
        """#!/usr/bin/env python3
import json

payloads = [
    {
        "type": "diagnostic",
        "fields": {
            "severity": "error",
            "message": "demo crate is vulnerable",
            "code": "vulnerability",
            "labels": [{"line": 1, "column": 1, "span": "demo"}],
            "advisory": {"id": "RUSTSEC-2026-0001", "title": "demo advisory"}
        }
    },
    {
        "type": "diagnostic",
        "fields": {
            "severity": "warning",
            "message": "license was not explicitly accepted",
            "code": "rejected",
            "labels": [{"line": 2, "column": 1, "span": "GPL-3.0-only"}]
        }
    }
]
for payload in payloads:
    print(json.dumps(payload))
""",
        encoding="utf-8",
    )
    cargo_deny_path.chmod(0o755)

    original_path = os.environ.get("PATH", "")
    os.environ["PATH"] = f"{bin_dir}:{original_path}"
    try:
        result = collect_external_analysis(
            project_path=project_root,
            output_dir=tmp_path / ".aigiscode",
            run_id="20260311_140000",
            selected_tools=["cargo-deny"],
        )
    finally:
        os.environ["PATH"] = original_path

    assert result.tool_runs[0].tool == "cargo-deny"
    assert result.tool_runs[0].status == "findings"
    assert result.tool_runs[0].summary["finding_count"] == 2
    assert result.findings[0].tool == "cargo-deny"
    assert result.findings[0].domain == "security"
    assert result.findings[0].rule_id == "RUSTSEC-2026-0001"
    assert result.findings[1].category == "license"
