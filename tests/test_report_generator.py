from __future__ import annotations

from datetime import datetime
from pathlib import Path

from aigiscode.graph.hardwiring import HardwiringFinding, HardwiringResult
from aigiscode.models import FindingVerdict, GraphAnalysisResult, ReportData, ReviewResult
from aigiscode.report.generator import (
    generate_json_report,
    generate_markdown_report,
    write_reports,
)


def test_report_includes_detector_coverage_warning() -> None:
    report = ReportData(
        project_path="/tmp/project",
        files_indexed=10,
        symbols_extracted=20,
        dependencies_found=5,
        language_breakdown={"php": 5, "python": 5},
        detector_coverage={"dead_code": ["python"], "hardwiring": ["python"]},
        graph_analysis=GraphAnalysisResult(),
    )

    markdown = generate_markdown_report(report)
    payload = generate_json_report(report)

    assert "Detector partial coverage" in markdown
    assert "dead_code | python" in markdown
    assert payload["summary"]["detector_coverage"] == {
        "dead_code": ["python"],
        "hardwiring": ["python"],
    }


def test_report_preserves_graph_analysis_contract() -> None:
    report = ReportData(
        project_path="/tmp/project",
        graph_analysis=GraphAnalysisResult(
            circular_dependencies=[["a.py", "b.py"]],
            strong_circular_dependencies=[["a.py", "b.py"]],
            orphan_files=["unused.py"],
        ),
    )

    payload = generate_json_report(report)

    assert payload["graph_analysis"]["strong_circular_dependencies"] == [
        {"cycle": ["a.py", "b.py"]}
    ]
    assert payload["graph_analysis"]["orphan_files"] == ["unused.py"]
    assert "bottleneck_files" in payload["graph_analysis"]
    assert payload["strong_circular_dependencies"] == [
        {"cycle": ["a.py", "b.py"]}
    ]


def test_report_includes_security_summary_and_archives(tmp_path: Path) -> None:
    generated_at = datetime(2026, 3, 9, 12, 30, 45)
    hardwiring = HardwiringResult(
        hardcoded_network=[
            HardwiringFinding(
                file_path="app/service.py",
                line=12,
                category="hardcoded_ip_url",
                value="https://api.example.com/token",
                context="TOKEN_URL = 'https://api.example.com/token'",
                severity="high",
                confidence="high",
                suggestion="Move endpoint to config.",
            )
        ],
        env_outside_config=[
            HardwiringFinding(
                file_path="app/runtime.py",
                line=5,
                category="env_outside_config",
                value="API_KEY",
                context="API_KEY = os.getenv('API_KEY')",
                severity="medium",
                confidence="high",
                suggestion="Read env in config bootstrap only.",
            )
        ],
    )
    review = ReviewResult(
        total_reviewed=2,
        true_positives=1,
        verdicts=[
            FindingVerdict(
                file_path="app/service.py",
                line=12,
                category="hardcoded_ip_url",
                value="https://api.example.com/token",
                verdict="true_positive",
                reason="Runtime endpoint is hardcoded.",
            )
        ],
    )
    report = ReportData(
        project_path="/tmp/project",
        generated_at=generated_at,
        files_indexed=2,
        symbols_extracted=4,
        dependencies_found=1,
        graph_analysis=GraphAnalysisResult(),
        hardwiring=hardwiring,
        review=review,
    )

    markdown = generate_markdown_report(report)
    payload = generate_json_report(report)
    md_path, json_path = write_reports(report, tmp_path)

    assert "## Security Analysis" in markdown
    assert "Hardcoded network endpoints" in markdown
    assert payload["security"] == {
        "total_findings": 2,
        "hardcoded_network": 1,
        "env_outside_config": 1,
        "high_severity": 1,
        "ai_confirmed": 1,
        "top_findings": [
            {
                "file": "app/service.py",
                "line": 12,
                "category": "hardcoded_ip_url",
                "value": "https://api.example.com/token",
                "severity": "high",
                "confidence": "high",
            },
            {
                "file": "app/runtime.py",
                "line": 5,
                "category": "env_outside_config",
                "value": "API_KEY",
                "severity": "medium",
                "confidence": "high",
            },
        ],
    }
    assert md_path == tmp_path / "aigiscode-report.md"
    assert json_path == tmp_path / "aigiscode-report.json"
    assert (tmp_path / "reports" / "20260309_123045-aigiscode-report.md").exists()
    assert (tmp_path / "reports" / "20260309_123045-aigiscode-report.json").exists()
