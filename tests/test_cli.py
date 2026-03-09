from __future__ import annotations

from pathlib import Path

from typer.testing import CliRunner

from aigiscode.cli import app


runner = CliRunner()


def test_index_and_report_support_custom_output_dir(tmp_path: Path) -> None:
    project_root = tmp_path / "project"
    project_root.mkdir()
    (project_root / "app.py").write_text("print('hello')\n", encoding="utf-8")

    output_dir = tmp_path / "reports" / "aigiscode"

    index_result = runner.invoke(
        app,
        ["index", str(project_root), "--output-dir", str(output_dir)],
    )
    assert index_result.exit_code == 0
    assert (output_dir / "aigiscode.db").exists()

    report_result = runner.invoke(
        app,
        ["report", str(project_root), "--output-dir", str(output_dir)],
    )
    assert report_result.exit_code == 0
    assert (output_dir / "aigiscode-report.json").exists()
    assert any(path.name.endswith("aigiscode-report.json") for path in (output_dir / "reports").iterdir())
