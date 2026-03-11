from __future__ import annotations

import json
import os
from pathlib import Path

from typer.testing import CliRunner

from aigiscode.cli import app


runner = CliRunner()


def _write(project_root: Path, relative_path: str, content: str) -> None:
    path = project_root / relative_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _write_executable(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)


def _latest_run_dir(output_dir: Path) -> Path:
    run_dirs = [path for path in (output_dir / "reports").iterdir() if path.is_dir()]
    assert run_dirs
    return max(run_dirs, key=lambda path: path.stat().st_mtime_ns)


def test_analyze_writes_security_findings_and_archived_reports(tmp_path: Path) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()

    _write(
        project_root,
        "app/runtime.py",
        "import os\n"
        "API_KEY = os.getenv('API_KEY')\n",
    )
    _write(
        project_root,
        "app/api.php",
        "<?php\n"
        "$endpoint = 'https://api.acme.local/v1/users';\n",
    )

    result = runner.invoke(
        app,
        [
            "analyze",
            str(project_root),
            "--skip-ai",
            "--skip-review",
            "--skip-synthesis",
        ],
    )

    assert result.exit_code == 0, result.stdout

    output_dir = project_root / ".aigiscode"
    json_path = output_dir / "aigiscode-report.json"
    markdown_path = output_dir / "aigiscode-report.md"
    handoff_json_path = output_dir / "aigiscode-handoff.json"
    handoff_markdown_path = output_dir / "aigiscode-handoff.md"
    archive_dir = output_dir / "reports"

    assert json_path.exists()
    assert markdown_path.exists()
    assert handoff_json_path.exists()
    assert handoff_markdown_path.exists()
    archived_reports = sorted(archive_dir.glob("*-aigiscode-report.json"))
    assert archived_reports
    archived_handoffs = sorted(archive_dir.glob("*-aigiscode-handoff.json"))
    assert archived_handoffs

    payload = json.loads(json_path.read_text(encoding="utf-8"))

    assert payload["security"]["hardcoded_network"] == 1
    assert payload["security"]["env_outside_config"] == 1
    assert payload["hardwiring"]["hardcoded_network"] == 1
    assert payload["hardwiring"]["env_outside_config"] == 1
    assert payload["agent_handoff"]["next_steps"]
    assert "## Security Analysis" in markdown_path.read_text(encoding="utf-8")
    assert "## Agent Handoff Brief" in handoff_markdown_path.read_text(encoding="utf-8")


def test_analyze_runs_ruff_security_and_archives_raw_artifacts(tmp_path: Path) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()

    _write(
        project_root,
        "app/runtime.py",
        "import subprocess\n"
        "subprocess.call('ls', shell=True)\n",
    )

    result = runner.invoke(
        app,
        [
            "analyze",
            str(project_root),
            "--skip-ai",
            "--skip-review",
            "--skip-synthesis",
            "--run-ruff-security",
        ],
    )

    assert result.exit_code == 0, result.stdout

    output_dir = project_root / ".aigiscode"
    payload = json.loads(
        (output_dir / "aigiscode-report.json").read_text(encoding="utf-8")
    )

    assert payload["external_analysis"] is not None
    assert payload["external_analysis"]["tool_runs"][0]["tool"] == "ruff"
    assert payload["external_analysis"]["tool_runs"][0]["status"] == "findings"
    assert any(
        finding["rule_id"] == "S602"
        for finding in payload["external_analysis"]["findings"]
    )

    run_dir = _latest_run_dir(output_dir)
    assert (run_dir / "raw" / "ruff-security.json").exists()
    assert (run_dir / "aigiscode-handoff.json").exists()


def test_analyze_runs_fake_gitleaks_and_archives_external_artifacts(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    _write(project_root, "app/config.php", "<?php\nreturn [];\n")

    bin_dir = tmp_path / "bin"
    gitleaks_path = bin_dir / "gitleaks"
    _write_executable(
        gitleaks_path,
        """#!/usr/bin/env python3
import json
import sys
from pathlib import Path

report_path = None
for index, arg in enumerate(sys.argv):
    if arg == "--report-path" and index + 1 < len(sys.argv):
        report_path = Path(sys.argv[index + 1])
        break

payload = [
    {
        "RuleID": "generic-api-key",
        "Description": "Potential secret in source",
        "File": "app/config.php",
        "StartLine": 2,
        "Fingerprint": "fake-fingerprint"
    }
]
report_path.write_text(json.dumps(payload), encoding="utf-8")
""",
    )

    env = os.environ.copy()
    env["PATH"] = f"{bin_dir}:{env.get('PATH', '')}"

    result = runner.invoke(
        app,
        [
            "analyze",
            str(project_root),
            "--skip-ai",
            "--skip-review",
            "--skip-synthesis",
            "--external-tool",
            "gitleaks",
        ],
        env=env,
    )

    assert result.exit_code == 0, result.stdout

    output_dir = project_root / ".aigiscode"
    payload = json.loads(
        (output_dir / "aigiscode-report.json").read_text(encoding="utf-8")
    )
    assert payload["external_analysis"] is not None
    assert payload["security"]["external_findings"] == 1
    assert payload["security"]["secrets"] == 1

    run_dir = _latest_run_dir(output_dir)
    assert (run_dir / "raw" / "gitleaks.json").exists()
    assert (run_dir / "external-analysis.json").exists()
    assert (run_dir / "aigiscode-handoff.json").exists()


def test_analyze_runs_external_tool_ruff_and_archives_raw_artifacts(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()

    _write(
        project_root,
        "app/runtime.py",
        "import subprocess\n"
        "subprocess.call('ls', shell=True)\n",
    )

    result = runner.invoke(
        app,
        [
            "analyze",
            str(project_root),
            "--skip-ai",
            "--skip-review",
            "--skip-synthesis",
            "--external-tool",
            "ruff",
        ],
    )

    assert result.exit_code == 0, result.stdout

    output_dir = project_root / ".aigiscode"
    payload = json.loads(
        (output_dir / "aigiscode-report.json").read_text(encoding="utf-8")
    )

    assert payload["external_analysis"] is not None
    assert payload["external_analysis"]["tool_runs"][0]["tool"] == "ruff"
    assert any(
        finding["rule_id"] == "S602"
        for finding in payload["external_analysis"]["findings"]
    )


def test_analyze_runs_fake_cargo_deny_and_archives_raw_artifacts(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    _write(
        project_root,
        "Cargo.toml",
        '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n',
    )
    _write(project_root, "src/main.rs", "fn main() {}\n")

    bin_dir = tmp_path / "bin"
    cargo_deny_path = bin_dir / "cargo-deny"
    _write_executable(
        cargo_deny_path,
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
    )

    env = os.environ.copy()
    env["PATH"] = f"{bin_dir}:{env.get('PATH', '')}"

    result = runner.invoke(
        app,
        [
            "analyze",
            str(project_root),
            "--skip-ai",
            "--skip-review",
            "--skip-synthesis",
            "--external-tool",
            "cargo-deny",
        ],
        env=env,
    )

    assert result.exit_code == 0, result.stdout

    output_dir = project_root / ".aigiscode"
    payload = json.loads(
        (output_dir / "aigiscode-report.json").read_text(encoding="utf-8")
    )

    assert payload["external_analysis"] is not None
    assert payload["external_analysis"]["tool_runs"][0]["tool"] == "cargo-deny"
    assert payload["security"]["external_findings"] == 2
    assert payload["security"]["sca"] == 1
    assert payload["security"]["license"] == 1

    run_dir = _latest_run_dir(output_dir)
    assert (run_dir / "raw" / "cargo-deny.jsonl").exists()
    assert (run_dir / "aigiscode-handoff.json").exists()




def test_report_runs_fake_gitleaks_and_archives_external_artifacts(
    tmp_path: Path,
) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    _write(project_root, "app/config.php", "<?php\nreturn [];\n")

    index_result = runner.invoke(app, ["index", str(project_root)])
    assert index_result.exit_code == 0, index_result.stdout

    bin_dir = tmp_path / "bin"
    gitleaks_path = bin_dir / "gitleaks"
    _write_executable(
        gitleaks_path,
        """#!/usr/bin/env python3
import json
import sys
from pathlib import Path

report_path = None
for index, arg in enumerate(sys.argv):
    if arg == "--report-path" and index + 1 < len(sys.argv):
        report_path = Path(sys.argv[index + 1])
        break

payload = [
    {
        "RuleID": "generic-api-key",
        "Description": "Potential secret in source",
        "File": "app/config.php",
        "StartLine": 2,
        "Fingerprint": "fake-fingerprint"
    }
]
report_path.write_text(json.dumps(payload), encoding="utf-8")
""",
    )

    env = os.environ.copy()
    env["PATH"] = f"{bin_dir}:{env.get('PATH', '')}"

    result = runner.invoke(
        app,
        [
            "report",
            str(project_root),
            "--external-tool",
            "gitleaks",
        ],
        env=env,
    )

    assert result.exit_code == 0, result.stdout

    output_dir = project_root / ".aigiscode"
    payload = json.loads(
        (output_dir / "aigiscode-report.json").read_text(encoding="utf-8")
    )
    assert payload["external_analysis"] is not None
    assert payload["security"]["external_findings"] == 1
    assert payload["security"]["secrets"] == 1

    run_dir = _latest_run_dir(output_dir)
    assert (run_dir / "raw" / "gitleaks.json").exists()
    assert (run_dir / "external-analysis.json").exists()


def test_report_filters_external_findings_with_saved_rules(tmp_path: Path) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    _write(project_root, "app/config.php", "<?php\nreturn [];\n")

    index_result = runner.invoke(app, ["index", str(project_root)])
    assert index_result.exit_code == 0, index_result.stdout

    output_dir = project_root / ".aigiscode"
    (output_dir / "rules.json").write_text(
        json.dumps(
            {
                "version": 2,
                "rules": [
                    {
                        "id": "ignore-config-secret",
                        "category": "secrets",
                        "checks": [
                            {
                                "type": "file_glob",
                                "params": {"pattern": "app/config.php"},
                            }
                        ],
                        "reason": "Fixture secret for test coverage",
                        "created_by": "test",
                        "created_at": "2026-03-09T12:00:00",
                        "status": "active",
                        "hit_count": 0,
                        "last_hit_run": "",
                        "miss_streak": 0,
                    }
                ],
            }
        ),
        encoding="utf-8",
    )

    bin_dir = tmp_path / "bin"
    gitleaks_path = bin_dir / "gitleaks"
    _write_executable(
        gitleaks_path,
        """#!/usr/bin/env python3
import json
import sys
from pathlib import Path

report_path = None
for index, arg in enumerate(sys.argv):
    if arg == "--report-path" and index + 1 < len(sys.argv):
        report_path = Path(sys.argv[index + 1])
        break

payload = [
    {
        "RuleID": "generic-api-key",
        "Description": "Potential secret in source",
        "File": "app/config.php",
        "StartLine": 2,
        "Fingerprint": "fake-fingerprint"
    }
]
report_path.write_text(json.dumps(payload), encoding="utf-8")
""",
    )

    env = os.environ.copy()
    env["PATH"] = f"{bin_dir}:{env.get('PATH', '')}"

    result = runner.invoke(
        app,
        [
            "report",
            str(project_root),
            "--external-tool",
            "gitleaks",
        ],
        env=env,
    )

    assert result.exit_code == 0, result.stdout

    payload = json.loads(
        (output_dir / "aigiscode-report.json").read_text(encoding="utf-8")
    )
    assert payload["external_analysis"] is not None
    assert payload["external_analysis"]["findings"] == []
    assert payload["external_analysis"]["tool_runs"][0]["summary"]["finding_count"] == 0
    assert (
        payload["external_analysis"]["tool_runs"][0]["summary"][
            "rules_filtered_count"
        ]
        == 1
    )
    assert payload["review"]["rules_prefiltered"] == 1
