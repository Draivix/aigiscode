"""Run and normalize external security analyzers."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
from pathlib import Path

from aigiscode.models import ExternalAnalysisResult, ExternalFinding, ExternalToolRun

COMPOSER_AUDIT_TOOL = "composer-audit"
CARGO_DENY_TOOL = "cargo-deny"

CARGO_DENY_LICENSE_CODES = {
    "accepted",
    "rejected",
    "unlicensed",
    "skipped-private-workspace-crate",
    "license-not-encountered",
    "license-exception-not-encountered",
    "missing-clarification-file",
    "parse-error",
    "empty-license-field",
    "no-license-field",
    "gather-failure",
}

CARGO_DENY_SOURCE_CODES = {
    "git-source-underspecified",
    "allowed-source",
    "allowed-by-organization",
    "source-not-allowed",
    "unmatched-source",
    "unmatched-organization",
}

SUPPORTED_SECURITY_TOOLS = (
    "ruff",
    "gitleaks",
    "pip-audit",
    "osv-scanner",
    "phpstan",
    COMPOSER_AUDIT_TOOL,
    "npm-audit",
    CARGO_DENY_TOOL,
    "cargo-clippy",
)

TOOL_TIMEOUT_SECONDS = {
    "ruff": 60,
    "gitleaks": 120,
    "pip-audit": 120,
    "osv-scanner": 120,
    "phpstan": 180,
    COMPOSER_AUDIT_TOOL: 120,
    "npm-audit": 120,
    CARGO_DENY_TOOL: 300,
    "cargo-clippy": 300,
}


def collect_external_analysis(
    *,
    project_path: Path,
    output_dir: Path,
    run_id: str,
    selected_tools: list[str] | None = None,
    run_ruff_security: bool = False,
) -> ExternalAnalysisResult:
    """Run configured external security analyzers and archive raw artifacts."""
    result = ExternalAnalysisResult()
    raw_dir = output_dir / "reports" / run_id / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)

    selected_tools = _normalize_selected_tools(
        selected_tools=selected_tools,
        run_ruff_security=run_ruff_security,
    )
    for tool_name in selected_tools:
        runner = _TOOL_RUNNERS.get(tool_name)
        if runner is None:
            result.tool_runs.append(
                ExternalToolRun(
                    tool=tool_name,
                    command=[],
                    status="failed",
                    summary={"error": f"Unsupported security tool: {tool_name}"},
                )
            )
            continue
        tool_run, findings = runner(project_path, raw_dir)
        result.tool_runs.append(tool_run)
        result.findings.extend(findings)
    return result


def _normalize_selected_tools(
    *,
    selected_tools: list[str] | None,
    run_ruff_security: bool,
) -> list[str]:
    normalized: list[str] = []
    seen: set[str] = set()
    for tool in selected_tools or []:
        name = tool.strip().lower()
        if not name:
            continue
        if name == "all":
            for candidate in SUPPORTED_SECURITY_TOOLS:
                if candidate not in seen:
                    normalized.append(candidate)
                    seen.add(candidate)
            continue
        if name in seen:
            continue
        normalized.append(name)
        seen.add(name)
    if run_ruff_security and "ruff" not in seen:
        normalized.append("ruff")
    return normalized


def _run_command(
    command: list[str],
    *,
    cwd: Path | None = None,
    tool: str,
) -> tuple[subprocess.CompletedProcess[str] | None, str | None]:
    try:
        return (
            subprocess.run(
                command,
                cwd=cwd,
                capture_output=True,
                text=True,
                check=False,
                timeout=TOOL_TIMEOUT_SECONDS.get(tool, 120),
            ),
            None,
        )
    except subprocess.TimeoutExpired:
        return None, f"Timed out after {TOOL_TIMEOUT_SECONDS.get(tool, 120)}s"


def _sanitize_stderr(tool: str, stderr: str) -> tuple[str, int]:
    lines = [line for line in stderr.splitlines() if line.strip()]
    if tool == COMPOSER_AUDIT_TOOL:
        filtered = [line for line in lines if not line.startswith("Deprecation Notice:")]
        return "\n".join(filtered), len(lines) - len(filtered)
    return stderr.strip(), 0


def _stderr_summary(tool: str, stderr: str) -> dict[str, object]:
    sanitized, suppressed = _sanitize_stderr(tool, stderr)
    summary: dict[str, object] = {}
    if sanitized:
        summary["stderr"] = sanitized
    if suppressed:
        summary["suppressed_stderr_lines"] = suppressed
    return summary


def _status_with_findings(
    *,
    tool: str,
    returncode: int,
    findings: list[ExternalFinding],
    passed_exit_codes: set[int] | None = None,
) -> tuple[str, dict[str, object]]:
    accepted = passed_exit_codes or {0}
    if findings:
        return "findings", {}
    if returncode in accepted:
        return "passed", {}
    return (
        "failed",
        {
            "error": (
                f"{tool} exited with code {returncode} and produced no normalized findings"
            )
        },
    )


def _refine_findings(
    findings: list[ExternalFinding],
) -> tuple[list[ExternalFinding], int]:
    refined: list[ExternalFinding] = []
    seen: set[str] = set()
    filtered_count = 0
    for finding in findings:
        candidate = _refine_finding(finding)
        if candidate is None:
            filtered_count += 1
            continue
        if candidate.fingerprint in seen:
            filtered_count += 1
            continue
        seen.add(candidate.fingerprint)
        refined.append(candidate)
    return refined, filtered_count


def _refine_finding(finding: ExternalFinding) -> ExternalFinding | None:
    if (
        finding.tool == "ruff"
        and finding.rule_id == "S101"
        and _is_test_like_path(finding.file_path)
    ):
        return None

    if finding.tool == "ruff" and finding.rule_id == "S105":
        message = finding.message.lower()
        if any(token in message for token in ('_url"', '_uri"', '_endpoint"')):
            candidate = finding.model_copy(deep=True)
            candidate.severity = "low"
            candidate.extras["normalized_reason"] = "url_like_token_name"
            return candidate

    return finding


def _is_test_like_path(file_path: str) -> bool:
    normalized = file_path.lower()
    return any(
        token in normalized
        for token in (
            "/test",
            "/tests/",
            "/spec",
            "/specs/",
            "/fixture",
            "/fixtures/",
            "test_",
            "_test.",
            ".spec.",
        )
    )


def _run_ruff_security(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "ruff-security.json"
    ruff_path = shutil.which("ruff")
    command = [
        ruff_path or "ruff",
        "check",
        "--select",
        "S",
        "--output-format",
        "json",
        "--exit-zero",
        str(project_path),
    ]
    if ruff_path is None:
        return (
            ExternalToolRun(
                tool="ruff",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "ruff executable not found on PATH"},
            ),
            [],
        )

    proc, timeout_error = _run_command(command, tool="ruff")
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="ruff",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )
    stdout = proc.stdout.strip()
    artifact_path.write_text(stdout or "[]", encoding="utf-8")

    if proc.returncode not in (0, 1):
        return (
            ExternalToolRun(
                tool="ruff",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary=_stderr_summary("ruff", proc.stderr),
            ),
            [],
        )

    payload, error = _load_json_artifact(artifact_path, default=[])
    if error:
        return (
            ExternalToolRun(
                tool="ruff",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary={**_stderr_summary("ruff", proc.stderr), "error": error},
            ),
            [],
        )

    findings = [
        ExternalFinding(
            tool="ruff",
            domain="security",
            category="sast",
            rule_id=str(item.get("code", "")),
            severity=_ruff_severity(str(item.get("code", ""))),
            confidence="medium",
            file_path=_relative_path(project_path, str(item.get("filename", ""))),
            line=_location_row(item),
            message=str(item.get("message", "")).strip(),
            fingerprint=_fingerprint(item),
            extras={"documentation_url": item.get("url")},
        )
        for item in payload
        if isinstance(item, dict)
    ]
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    return (
        ExternalToolRun(
            tool="ruff",
            command=command,
            status="findings" if findings else "passed",
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
            },
        ),
        findings,
    )


def _run_gitleaks(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "gitleaks.json"
    gitleaks_path = shutil.which("gitleaks")
    command = [
        gitleaks_path or "gitleaks",
        "detect",
        "--source",
        str(project_path),
        "--report-format",
        "json",
        "--report-path",
        str(artifact_path),
        "--no-banner",
        "--exit-code",
        "0",
        "--redact",
    ]
    if gitleaks_path is None:
        return (
            ExternalToolRun(
                tool="gitleaks",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "gitleaks executable not found on PATH"},
            ),
            [],
        )

    proc, timeout_error = _run_command(command, tool="gitleaks")
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="gitleaks",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )
    payload, error = _load_json_artifact(artifact_path, default=[])
    if error:
        return (
            ExternalToolRun(
                tool="gitleaks",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary={**_stderr_summary("gitleaks", proc.stderr), "error": error},
            ),
            [],
        )

    findings = _parse_gitleaks_payload(project_path, payload)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool="gitleaks",
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool="gitleaks",
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary("gitleaks", proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _run_pip_audit(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "pip-audit.json"
    pip_audit_path = shutil.which("pip-audit")
    command = [
        pip_audit_path or "pip-audit",
        "--format",
        "json",
        "--output",
        str(artifact_path),
        str(project_path),
    ]
    if pip_audit_path is None:
        return (
            ExternalToolRun(
                tool="pip-audit",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "pip-audit executable not found on PATH"},
            ),
            [],
        )

    proc, timeout_error = _run_command(command, tool="pip-audit")
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="pip-audit",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )
    payload, error = _load_json_artifact(artifact_path, default={"dependencies": []})
    if error:
        return (
            ExternalToolRun(
                tool="pip-audit",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary={**_stderr_summary("pip-audit", proc.stderr), "error": error},
            ),
            [],
        )

    findings = _parse_pip_audit_payload(payload)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool="pip-audit",
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool="pip-audit",
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary("pip-audit", proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _run_osv_scanner(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "osv-scanner.json"
    osv_path = shutil.which("osv-scanner")
    command = [
        osv_path or "osv-scanner",
        "scan",
        "source",
        "--recursive",
        str(project_path),
        "--format",
        "json",
        "--output",
        str(artifact_path),
    ]
    if osv_path is None:
        return (
            ExternalToolRun(
                tool="osv-scanner",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "osv-scanner executable not found on PATH"},
            ),
            [],
        )

    proc, timeout_error = _run_command(command, tool="osv-scanner")
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="osv-scanner",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )
    payload, error = _load_json_artifact(artifact_path, default={"results": []})
    if error:
        return (
            ExternalToolRun(
                tool="osv-scanner",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary={**_stderr_summary("osv-scanner", proc.stderr), "error": error},
            ),
            [],
        )

    findings = _parse_osv_scanner_payload(payload)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool="osv-scanner",
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool="osv-scanner",
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary("osv-scanner", proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _run_phpstan(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "phpstan.json"
    phpstan_path = project_path / "vendor" / "bin" / "phpstan"
    if not phpstan_path.exists():
        resolved = shutil.which("phpstan")
        phpstan_path = Path(resolved) if resolved else phpstan_path
    command = [
        str(phpstan_path),
        "analyse",
        "--error-format=json",
        "--no-progress",
    ]
    if not phpstan_path.exists():
        return (
            ExternalToolRun(
                tool="phpstan",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "phpstan executable not found"},
            ),
            [],
        )

    proc, timeout_error = _run_command(command, cwd=project_path, tool="phpstan")
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="phpstan",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )
    stdout = proc.stdout.strip()
    artifact_path.write_text(stdout or "{}", encoding="utf-8")
    payload, error = _load_json_artifact(artifact_path, default={})
    if error:
        return (
            ExternalToolRun(
                tool="phpstan",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary={**_stderr_summary("phpstan", proc.stderr), "error": error},
            ),
            [],
        )

    findings = _parse_phpstan_payload(payload)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool="phpstan",
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool="phpstan",
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary("phpstan", proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _run_composer_audit(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "composer-audit.json"
    if not (project_path / "composer.json").exists():
        return (
            ExternalToolRun(
                tool="composer-audit",
                command=[],
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "composer.json not found"},
            ),
            [],
        )

    composer_path = shutil.which("composer")
    command = [
        composer_path or "composer",
        "audit",
        "--format=json",
        "--no-interaction",
        "--no-ansi",
    ]
    if composer_path is None:
        return (
            ExternalToolRun(
                tool="composer-audit",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "composer executable not found"},
            ),
            [],
        )

    proc, timeout_error = _run_command(
        command,
        cwd=project_path,
        tool="composer-audit",
    )
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="composer-audit",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )
    stdout = proc.stdout.strip()
    artifact_path.write_text(stdout or "{}", encoding="utf-8")
    payload, error = _load_json_artifact(artifact_path, default={})
    if error:
        return (
            ExternalToolRun(
                tool="composer-audit",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary={
                    **_stderr_summary("composer-audit", proc.stderr),
                    "error": error,
                },
            ),
            [],
        )

    findings = _parse_composer_audit_payload(payload)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool="composer-audit",
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool="composer-audit",
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary("composer-audit", proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _run_npm_audit(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "npm-audit.json"
    if not (project_path / "package.json").exists():
        return (
            ExternalToolRun(
                tool="npm-audit",
                command=[],
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "package.json not found"},
            ),
            [],
        )

    npm_path = shutil.which("npm")
    command = [npm_path or "npm", "audit", "--json", "--omit=dev"]
    if npm_path is None:
        return (
            ExternalToolRun(
                tool="npm-audit",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "npm executable not found"},
            ),
            [],
        )

    proc, timeout_error = _run_command(command, cwd=project_path, tool="npm-audit")
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="npm-audit",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )
    stdout = proc.stdout.strip()
    artifact_path.write_text(stdout or "{}", encoding="utf-8")
    payload, error = _load_json_artifact(artifact_path, default={})
    if error:
        return (
            ExternalToolRun(
                tool="npm-audit",
                command=command,
                status="failed",
                exit_code=proc.returncode,
                artifact_path=str(artifact_path),
                summary={**_stderr_summary("npm-audit", proc.stderr), "error": error},
            ),
            [],
        )

    findings = _parse_npm_audit_payload(payload)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool="npm-audit",
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool="npm-audit",
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary("npm-audit", proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _run_cargo_clippy(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "cargo-clippy.jsonl"
    if not (project_path / "Cargo.toml").exists():
        return (
            ExternalToolRun(
                tool="cargo-clippy",
                command=[],
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "Cargo.toml not found"},
            ),
            [],
        )

    cargo_path = shutil.which("cargo")
    command = [
        cargo_path or "cargo",
        "clippy",
        "--message-format=json",
        "--all-targets",
        "--workspace",
        "--",
        "-W",
        "clippy::all",
    ]
    if cargo_path is None:
        return (
            ExternalToolRun(
                tool="cargo-clippy",
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "cargo executable not found"},
            ),
            [],
        )

    proc, timeout_error = _run_command(
        command,
        cwd=project_path,
        tool="cargo-clippy",
    )
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool="cargo-clippy",
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )

    stdout = proc.stdout.strip()
    artifact_path.write_text(stdout, encoding="utf-8")
    findings = _parse_cargo_clippy_output(stdout, project_path)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool="cargo-clippy",
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool="cargo-clippy",
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary("cargo-clippy", proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _run_cargo_deny(
    project_path: Path,
    raw_dir: Path,
) -> tuple[ExternalToolRun, list[ExternalFinding]]:
    artifact_path = raw_dir / "cargo-deny.jsonl"
    if not (project_path / "Cargo.toml").exists():
        return (
            ExternalToolRun(
                tool=CARGO_DENY_TOOL,
                command=[],
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "Cargo.toml not found"},
            ),
            [],
        )

    cargo_deny_path = shutil.which(CARGO_DENY_TOOL)
    command = [
        cargo_deny_path or CARGO_DENY_TOOL,
        "check",
        "advisories",
        "bans",
        "licenses",
        "sources",
        "--format",
        "json",
        "--hide-inclusion-graph",
    ]
    if cargo_deny_path is None:
        return (
            ExternalToolRun(
                tool=CARGO_DENY_TOOL,
                command=command,
                status="unavailable",
                artifact_path=str(artifact_path),
                summary={"message": "cargo-deny executable not found on PATH"},
            ),
            [],
        )

    proc, timeout_error = _run_command(
        command,
        cwd=project_path,
        tool=CARGO_DENY_TOOL,
    )
    if timeout_error or proc is None:
        return (
            ExternalToolRun(
                tool=CARGO_DENY_TOOL,
                command=command,
                status="failed",
                artifact_path=str(artifact_path),
                summary={"error": timeout_error},
            ),
            [],
        )

    stdout = proc.stdout.strip()
    artifact_path.write_text(stdout, encoding="utf-8")
    findings = _parse_cargo_deny_output(stdout)
    raw_count = len(findings)
    findings, filtered_count = _refine_findings(findings)
    status, status_summary = _status_with_findings(
        tool=CARGO_DENY_TOOL,
        returncode=proc.returncode,
        findings=findings,
    )
    return (
        ExternalToolRun(
            tool=CARGO_DENY_TOOL,
            command=command,
            status=status,
            exit_code=proc.returncode,
            artifact_path=str(artifact_path),
            summary={
                "finding_count": len(findings),
                "raw_finding_count": raw_count,
                "filtered_count": filtered_count,
                **_stderr_summary(CARGO_DENY_TOOL, proc.stderr),
                **status_summary,
            },
        ),
        findings,
    )


def _load_json_artifact(
    artifact_path: Path,
    *,
    default,
) -> tuple[object, str | None]:
    if not artifact_path.exists():
        return default, f"Artifact was not created: {artifact_path.name}"
    try:
        return json.loads(artifact_path.read_text(encoding="utf-8") or "null"), None
    except json.JSONDecodeError as exc:
        return default, f"Invalid JSON artifact: {exc}"


def _parse_gitleaks_payload(
    project_path: Path,
    payload: object,
) -> list[ExternalFinding]:
    if not isinstance(payload, list):
        return []
    findings: list[ExternalFinding] = []
    for item in payload:
        if not isinstance(item, dict):
            continue
        file_path = _relative_path(project_path, str(item.get("File", "")))
        line = int(item.get("StartLine") or item.get("Line") or 1)
        rule_id = str(item.get("RuleID", "gitleaks"))
        description = str(item.get("Description", "")).strip()
        findings.append(
            ExternalFinding(
                tool="gitleaks",
                domain="security",
                category="secrets",
                rule_id=rule_id,
                severity="high",
                confidence="medium",
                file_path=file_path,
                line=line,
                message=description or "Potential secret detected by Gitleaks",
                fingerprint=str(
                    item.get("Fingerprint") or _stable_fingerprint("gitleaks", item)
                ),
                extras={
                    "commit": item.get("Commit"),
                    "author": item.get("Author"),
                    "entropy": item.get("Entropy"),
                    "match": item.get("Match"),
                },
            )
        )
    return findings


def _parse_pip_audit_payload(payload: object) -> list[ExternalFinding]:
    dependencies: list[dict] = []
    if isinstance(payload, list):
        dependencies = [item for item in payload if isinstance(item, dict)]
    elif isinstance(payload, dict):
        raw_dependencies = payload.get("dependencies")
        if isinstance(raw_dependencies, list):
            dependencies = [item for item in raw_dependencies if isinstance(item, dict)]

    findings: list[ExternalFinding] = []
    for dependency in dependencies:
        vulns = dependency.get("vulns")
        if not isinstance(vulns, list):
            continue
        package_name = str(dependency.get("name", ""))
        package_version = str(dependency.get("version", ""))
        for vuln in vulns:
            if not isinstance(vuln, dict):
                continue
            vuln_id = str(vuln.get("id", "")) or "vulnerability"
            findings.append(
                ExternalFinding(
                    tool="pip-audit",
                    domain="security",
                    category="sca",
                    rule_id=vuln_id,
                    severity="high",
                    confidence="high",
                    message=f"{package_name} {package_version} is affected by {vuln_id}",
                    fingerprint=_stable_fingerprint(
                        "pip-audit",
                        {
                            "name": package_name,
                            "version": package_version,
                            "id": vuln_id,
                        },
                    ),
                    extras={
                        "aliases": vuln.get("aliases", []),
                        "fix_versions": vuln.get("fix_versions", []),
                        "description": vuln.get("description", ""),
                        "package_name": package_name,
                        "package_version": package_version,
                    },
                )
            )
    return findings


def _parse_osv_scanner_payload(payload: object) -> list[ExternalFinding]:
    if not isinstance(payload, dict):
        return []
    findings: list[ExternalFinding] = []
    for result in payload.get("results", []):
        if not isinstance(result, dict):
            continue
        packages = result.get("packages")
        if not isinstance(packages, list):
            continue
        for package_entry in packages:
            if not isinstance(package_entry, dict):
                continue
            package = package_entry.get("package")
            if not isinstance(package, dict):
                continue
            package_name = str(package.get("name", ""))
            package_version = str(package.get("version", ""))
            ecosystem = str(package.get("ecosystem", ""))
            for vuln in package_entry.get("vulnerabilities", []):
                if not isinstance(vuln, dict):
                    continue
                vuln_id = str(vuln.get("id", "")) or "osv"
                findings.append(
                    ExternalFinding(
                        tool="osv-scanner",
                        domain="security",
                        category="sca",
                        rule_id=vuln_id,
                        severity="high",
                        confidence="high",
                        message=f"{package_name} {package_version} is affected by {vuln_id}",
                        fingerprint=_stable_fingerprint(
                            "osv-scanner",
                            {
                                "name": package_name,
                                "version": package_version,
                                "id": vuln_id,
                            },
                        ),
                        extras={
                            "ecosystem": ecosystem,
                            "summary": vuln.get("summary", ""),
                            "details": vuln.get("details", ""),
                            "aliases": vuln.get("aliases", []),
                            "package_name": package_name,
                            "package_version": package_version,
                        },
                    )
                )
    return findings


def _parse_phpstan_payload(payload: object) -> list[ExternalFinding]:
    if not isinstance(payload, dict):
        return []
    findings: list[ExternalFinding] = []
    files = payload.get("files")
    if not isinstance(files, dict):
        return findings
    for file_path, info in files.items():
        if not isinstance(info, dict):
            continue
        for message in info.get("messages", []):
            if not isinstance(message, dict):
                continue
            line = int(message.get("line") or 1)
            rule_id = str(message.get("identifier", ""))
            text = str(message.get("message", "")).strip()
            findings.append(
                ExternalFinding(
                    tool="phpstan",
                    domain="quality",
                    category="static_analysis",
                    rule_id=rule_id,
                    severity="medium",
                    confidence="high",
                    file_path=str(file_path),
                    line=line,
                    message=text,
                    fingerprint=_stable_fingerprint(
                        "phpstan",
                        {
                            "file": file_path,
                            "line": line,
                            "rule": rule_id,
                            "message": text,
                        },
                    ),
                    extras={
                        "tip": message.get("tip"),
                        "ignorable": message.get("ignorable"),
                    },
                )
            )
    return findings


def _parse_composer_audit_payload(payload: object) -> list[ExternalFinding]:
    if not isinstance(payload, dict):
        return []
    findings: list[ExternalFinding] = []
    advisories = payload.get("advisories")
    if isinstance(advisories, dict):
        for package_name, package_advisories in advisories.items():
            if not isinstance(package_advisories, list):
                continue
            for advisory in package_advisories:
                if not isinstance(advisory, dict):
                    continue
                advisory_id = str(
                    advisory.get("advisoryId")
                    or advisory.get("cve")
                    or package_name
                )
                title = str(advisory.get("title") or advisory.get("link") or package_name)
                findings.append(
                    ExternalFinding(
                        tool="composer-audit",
                        domain="security",
                        category="sca",
                        rule_id=advisory_id,
                        severity=str(advisory.get("severity") or "high").lower(),
                        confidence="high",
                        file_path="composer.lock",
                        line=1,
                        message=f"{package_name}: {title}",
                        fingerprint=_stable_fingerprint(
                            "composer-audit",
                            {
                                "package": package_name,
                                "id": advisory_id,
                            },
                        ),
                        extras={
                            "cve": advisory.get("cve"),
                            "link": advisory.get("link"),
                            "affected_versions": advisory.get("affectedVersions"),
                        },
                    )
                )
    abandoned = payload.get("abandoned")
    if isinstance(abandoned, dict):
        for package_name, replacement in abandoned.items():
            findings.append(
                ExternalFinding(
                    tool="composer-audit",
                    domain="security",
                    category="abandoned_dependency",
                    rule_id="abandoned-package",
                    severity="medium",
                    confidence="high",
                    file_path="composer.lock",
                    line=1,
                    message=f"{package_name}: package is abandoned",
                    fingerprint=_stable_fingerprint(
                        "composer-audit",
                        {"package": package_name, "replacement": replacement},
                    ),
                    extras={"replacement": replacement},
                )
            )
    return findings


def _parse_composer_audit_output(payload: str) -> list[ExternalFinding]:
    """Backward-compatible string entrypoint used by tests and older callers."""
    try:
        decoded = json.loads(payload)
    except json.JSONDecodeError:
        return []
    return _parse_composer_audit_payload(decoded)


def _parse_cargo_clippy_output(
    payload: str,
    project_path: Path,
) -> list[ExternalFinding]:
    findings: list[ExternalFinding] = []
    if not payload.strip():
        return findings

    severity_map = {
        "error": "high",
        "warning": "medium",
        "note": "low",
        "help": "low",
    }
    for line in payload.splitlines():
        if not line.strip():
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(entry, dict) or entry.get("reason") != "compiler-message":
            continue
        message = entry.get("message")
        if not isinstance(message, dict):
            continue
        code = message.get("code")
        if not isinstance(code, dict):
            continue
        rule_id = str(code.get("code", ""))
        if not rule_id.startswith("clippy::"):
            continue
        spans = message.get("spans")
        primary_span = None
        if isinstance(spans, list) and spans:
            primary_span = next(
                (span for span in spans if isinstance(span, dict) and span.get("is_primary")),
                spans[0],
            )
        file_path = ""
        line_no = 1
        if isinstance(primary_span, dict):
            raw_file = str(primary_span.get("file_name", ""))
            if raw_file:
                candidate = Path(raw_file)
                if candidate.is_absolute():
                    try:
                        file_path = str(candidate.relative_to(project_path))
                    except ValueError:
                        file_path = raw_file
                else:
                    file_path = raw_file
            raw_line = primary_span.get("line_start")
            if isinstance(raw_line, int) and raw_line > 0:
                line_no = raw_line
        finding_payload = {
            "rule_id": rule_id,
            "file": file_path,
            "line": line_no,
            "message": str(message.get("message", "")),
        }
        findings.append(
            ExternalFinding(
                tool="cargo-clippy",
                domain="quality",
                category="lint",
                rule_id=rule_id,
                severity=severity_map.get(str(message.get("level", "warning")), "medium"),
                confidence="high",
                file_path=file_path,
                line=line_no,
                message=str(message.get("message", "")),
                fingerprint=_stable_fingerprint("cargo-clippy", finding_payload),
                extras={"rendered": str(message.get("rendered", ""))},
            )
        )
    return findings


def _parse_cargo_deny_output(payload: str) -> list[ExternalFinding]:
    findings: list[ExternalFinding] = []
    if not payload.strip():
        return findings

    severity_map = {
        "error": "high",
        "warning": "medium",
        "note": "low",
        "help": "low",
    }
    for line in payload.splitlines():
        if not line.strip():
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(entry, dict) or entry.get("type") != "diagnostic":
            continue
        fields = entry.get("fields")
        if not isinstance(fields, dict):
            continue

        advisory = fields.get("advisory")
        labels = fields.get("labels")
        primary_label = labels[0] if isinstance(labels, list) and labels else None
        line_no = 1
        file_path = ""
        if isinstance(primary_label, dict):
            raw_line = primary_label.get("line")
            if isinstance(raw_line, int) and raw_line > 0:
                line_no = raw_line
            raw_file = primary_label.get("file")
            if isinstance(raw_file, str):
                file_path = raw_file

        code = str(fields.get("code") or "")
        message = str(fields.get("message") or "").strip()
        category, domain = _cargo_deny_category(code, advisory)
        advisory_id = ""
        if isinstance(advisory, dict):
            advisory_id = str(advisory.get("id") or "")

        finding_payload = {
            "code": code,
            "advisory_id": advisory_id,
            "message": message,
            "line": line_no,
            "file": file_path,
        }
        findings.append(
            ExternalFinding(
                tool=CARGO_DENY_TOOL,
                domain=domain,
                category=category,
                rule_id=advisory_id or code or category,
                severity=severity_map.get(str(fields.get("severity", "warning")), "medium"),
                confidence="high",
                file_path=file_path,
                line=line_no,
                message=message or "cargo-deny emitted a diagnostic",
                fingerprint=_stable_fingerprint(CARGO_DENY_TOOL, finding_payload),
                extras={
                    "labels": labels if isinstance(labels, list) else [],
                    "notes": fields.get("notes", []),
                    "advisory": advisory if isinstance(advisory, dict) else {},
                },
            )
        )
    return findings


def _cargo_deny_category(code: str, advisory: object) -> tuple[str, str]:
    if isinstance(advisory, dict):
        return "sca", "security"
    if code in CARGO_DENY_LICENSE_CODES:
        return "license", "security"
    if code in CARGO_DENY_SOURCE_CODES:
        return "source_policy", "security"
    return "supply_chain_policy", "quality"


def _parse_npm_audit_payload(payload: object) -> list[ExternalFinding]:
    if not isinstance(payload, dict):
        return []
    vulnerabilities = payload.get("vulnerabilities")
    if not isinstance(vulnerabilities, dict):
        return []
    findings: list[ExternalFinding] = []
    for package_name, vulnerability in vulnerabilities.items():
        if not isinstance(vulnerability, dict):
            continue
        severity = str(vulnerability.get("severity") or "medium").lower()
        via = vulnerability.get("via") or []
        rule_id = package_name
        title = package_name
        if isinstance(via, list):
            for entry in via:
                if isinstance(entry, dict):
                    rule_id = str(entry.get("source") or entry.get("name") or package_name)
                    title = str(entry.get("title") or entry.get("url") or package_name)
                    break
        findings.append(
            ExternalFinding(
                tool="npm-audit",
                domain="security",
                category="sca",
                rule_id=rule_id,
                severity=severity,
                confidence="high",
                file_path="package-lock.json",
                line=1,
                message=f"{package_name}: {title}",
                fingerprint=_stable_fingerprint(
                    "npm-audit",
                    {"package": package_name, "rule": rule_id},
                ),
                extras={
                    "is_direct": vulnerability.get("isDirect"),
                    "fix_available": vulnerability.get("fixAvailable"),
                    "nodes": vulnerability.get("nodes"),
                },
            )
        )
    return findings


def _parse_npm_audit_output(payload: str) -> list[ExternalFinding]:
    """Backward-compatible string entrypoint used by tests and older callers."""
    try:
        decoded = json.loads(payload)
    except json.JSONDecodeError:
        return []
    return _parse_npm_audit_payload(decoded)


def _relative_path(project_path: Path, candidate: str) -> str:
    if not candidate:
        return ""
    try:
        return str(Path(candidate).resolve().relative_to(project_path.resolve()))
    except ValueError:
        return candidate


def _location_row(item: dict) -> int:
    location = item.get("location")
    if isinstance(location, dict):
        row = location.get("row")
        if isinstance(row, int):
            return row
    return 1


def _fingerprint(item: dict) -> str:
    filename = str(item.get("filename", ""))
    code = str(item.get("code", ""))
    row = _location_row(item)
    return f"{filename}:{row}:{code}"


def _stable_fingerprint(tool: str, payload: dict) -> str:
    return hashlib.sha256(
        f"{tool}:{json.dumps(payload, sort_keys=True, ensure_ascii=False)}".encode(
            "utf-8"
        )
    ).hexdigest()


def _ruff_severity(rule_id: str) -> str:
    if rule_id in {
        "S105",
        "S106",
        "S107",
        "S324",
        "S501",
        "S602",
        "S603",
        "S607",
        "S608",
    }:
        return "high"
    if rule_id.startswith("S"):
        return "medium"
    return "low"


_TOOL_RUNNERS = {
    "ruff": _run_ruff_security,
    "gitleaks": _run_gitleaks,
    "pip-audit": _run_pip_audit,
    "osv-scanner": _run_osv_scanner,
    "phpstan": _run_phpstan,
    "composer-audit": _run_composer_audit,
    "npm-audit": _run_npm_audit,
    "cargo-deny": _run_cargo_deny,
    "cargo-clippy": _run_cargo_clippy,
}
