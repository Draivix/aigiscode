use super::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub(super) struct ParsedReport {
    pub findings: Vec<ExternalFinding>,
    /// Preserve usable findings even when another part of the report is incomplete.
    pub error: Option<String>,
}

pub(super) fn read(tool: &str, root: &Path, path: &Path) -> Result<ParsedReport, String> {
    let reader =
        BufReader::new(File::open(path).map_err(|error| format!("{}: {error}", path.display()))?);
    if matches!(tool, OPENGREP_TOOL | TRIVY_TOOL | GRYPE_TOOL) {
        let sarif: sarif_rust::types::SarifLog =
            serde_json::from_reader(reader).map_err(|error| format!("invalid SARIF: {error}"))?;
        let error = if sarif.version != "2.1.0" || sarif.runs.is_empty() {
            Some(String::from(
                "SARIF must contain an executed run in supported version 2.1.0",
            ))
        } else if sarif.runs.iter().any(|run| {
            run.external_property_file_references.is_some()
                || run.invocations.iter().flatten().any(|invocation| {
                    invocation.execution_successful == Some(false)
                        || invocation.process_start_failure_message.is_some()
                        || invocation
                            .tool_execution_notifications
                            .iter()
                            .flatten()
                            .chain(invocation.tool_configuration_notifications.iter().flatten())
                            .any(|notification| {
                                matches!(
                                    notification.level,
                                    Some(sarif_rust::types::NotificationLevel::Error)
                                )
                            })
                })
        }) {
            Some(String::from(
                "SARIF reports an execution/configuration error or external result fragments",
            ))
        } else {
            None
        };
        let fallback = SarifFallback {
            domain: "security",
            category: if tool == OPENGREP_TOOL { "sast" } else { "sca" },
        };
        let findings = sarif
            .runs
            .iter()
            .flat_map(|run| {
                parse_sarif_run(tool, root, run, &build_sarif_rule_lookup(run), fallback)
            })
            .collect();
        return Ok(ParsedReport { findings, error });
    }
    if matches!(tool, CARGO_DENY_TOOL | CARGO_CLIPPY_TOOL) {
        return read_json_lines(tool, root, reader);
    }
    let payload: Value =
        serde_json::from_reader(reader).map_err(|error| format!("invalid JSON: {error}"))?;
    let error = validate_json(tool, &payload).err();
    let findings = match tool {
        RUFF_TOOL => parse_ruff_payload(root, &payload),
        GITLEAKS_TOOL => parse_gitleaks_payload(root, &payload),
        PIP_AUDIT_TOOL => parse_pip_audit_payload(&payload),
        OSV_SCANNER_TOOL => parse_osv_scanner_payload(&payload),
        COMPOSER_AUDIT_TOOL => parse_composer_audit_payload(&payload),
        NPM_AUDIT_TOOL => parse_npm_audit_payload(&payload),
        _ => return Err(format!("unsupported report format for {tool}")),
    };
    Ok(ParsedReport { findings, error })
}

fn read_json_lines(tool: &str, root: &Path, reader: impl BufRead) -> Result<ParsedReport, String> {
    let mut findings = Vec::new();
    let mut error = None;
    let mut build_finished = false;
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(&line) {
            Ok(Value::Object(entry)) => Value::Object(entry),
            _ => {
                error.get_or_insert_with(|| format!("invalid JSON record at line {}", index + 1));
                continue;
            }
        };
        if tool == CARGO_CLIPPY_TOOL {
            match entry.get("reason").and_then(Value::as_str) {
                Some("build-finished") => {
                    build_finished = entry.get("success").and_then(Value::as_bool) == Some(true);
                    if !build_finished {
                        error.get_or_insert(String::from(
                            "Cargo build did not complete successfully",
                        ));
                    }
                }
                Some("compiler-message" | "compiler-artifact" | "build-script-executed") => {}
                _ => {
                    error.get_or_insert(String::from("unrecognized Cargo JSON record"));
                }
            }
            findings.extend(parse_cargo_clippy_output(root, &line));
        } else {
            // cargo-deny emits both diagnostic records and structured logs to stderr.
            if entry.get("type").and_then(Value::as_str) == Some("diagnostic") {
                if string(entry.pointer("/fields/message"), "diagnostic message").is_err()
                    || string(entry.pointer("/fields/severity"), "diagnostic severity").is_err()
                {
                    error.get_or_insert(String::from("cargo-deny diagnostic has invalid fields"));
                }
                if entry.pointer("/fields/severity").and_then(Value::as_str) == Some("bug") {
                    error.get_or_insert(String::from("cargo-deny reported an internal error"));
                }
            } else if entry
                .pointer("/fields/level")
                .or_else(|| entry.get("level"))
                .and_then(Value::as_str)
                .is_some_and(|level| level.eq_ignore_ascii_case("error"))
            {
                error.get_or_insert(String::from("cargo-deny reported an operational error"));
            } else if !entry
                .pointer("/fields/level")
                .or_else(|| entry.get("level"))
                .is_some_and(Value::is_string)
                && entry.get("type").and_then(Value::as_str) != Some("summary")
            {
                error.get_or_insert(String::from("unrecognized cargo-deny JSON record"));
            }
            findings.extend(parse_cargo_deny_output(&line));
        }
    }
    if tool == CARGO_CLIPPY_TOOL && !build_finished {
        error.get_or_insert(String::from(
            "Cargo report has no successful build-finished record",
        ));
    }
    Ok(ParsedReport { findings, error })
}

pub(super) fn exit_error(tool: &str, exit: Option<i32>, has_findings: bool) -> Option<String> {
    // These are the exit contracts of the exact invocations in external/mod.rs:
    // Ruff and Gitleaks explicitly request zero on findings; SARIF tools do not
    // request a findings exit threshold. Other audit tools use result exit codes.
    let accepted = match exit {
        Some(0) => true,
        Some(1) if has_findings => matches!(
            tool,
            PIP_AUDIT_TOOL
                | OSV_SCANNER_TOOL
                | NPM_AUDIT_TOOL
                | COMPOSER_AUDIT_TOOL
                | CARGO_DENY_TOOL
        ),
        Some(2 | 3) if has_findings && tool == COMPOSER_AUDIT_TOOL => true, // Composer 2.x legacy bitmask
        _ => false,
    };
    if accepted {
        None
    } else if tool == OSV_SCANNER_TOOL && exit == Some(128) {
        Some(String::from("OSV-Scanner found no packages to audit"))
    } else {
        Some(format!("{tool} did not complete its audit (exit {exit:?})"))
    }
}

fn array<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a [Value], String> {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("expected {label} array"))
}

fn string(value: Option<&Value>, label: &str) -> Result<(), String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(|_| ())
        .ok_or_else(|| format!("expected nonempty {label} string"))
}

fn validate_json(tool: &str, payload: &Value) -> Result<(), String> {
    if payload.get("error").is_some_and(|value| !value.is_null()) {
        return Err(String::from("tool returned an error response"));
    }
    match tool {
        RUFF_TOOL | GITLEAKS_TOOL => {
            for item in array(Some(payload), "findings")? {
                let (rule, file, line) = if tool == RUFF_TOOL {
                    (
                        item.get("code"),
                        item.get("filename"),
                        item.pointer("/location/row"),
                    )
                } else {
                    (
                        item.get("RuleID"),
                        item.get("File"),
                        item.get("StartLine").or_else(|| item.get("Line")),
                    )
                };
                string(rule, "rule")?;
                string(file, "file")?;
                if !line.and_then(Value::as_u64).is_some_and(|line| line > 0) {
                    return Err(String::from("finding has no valid source line"));
                }
            }
        }
        PIP_AUDIT_TOOL => {
            let dependencies = if payload.is_array() {
                Some(payload)
            } else {
                payload.get("dependencies")
            };
            for dependency in array(dependencies, "dependencies")? {
                string(dependency.get("name"), "package name")?;
                string(dependency.get("version"), "package version")?;
                if dependency
                    .get("skip_reason")
                    .is_some_and(|value| !value.is_null())
                {
                    return Err(String::from("pip-audit skipped a dependency"));
                }
                for vuln in array(dependency.get("vulns"), "vulnerabilities")? {
                    string(vuln.get("id"), "vulnerability id")?;
                }
            }
        }
        OSV_SCANNER_TOOL => {
            for result in array(payload.get("results"), "results")? {
                for package in array(result.get("packages"), "packages")? {
                    string(package.pointer("/package/name"), "package name")?;
                    if let Some(vulnerabilities) = package.get("vulnerabilities") {
                        for vuln in array(Some(vulnerabilities), "vulnerabilities")? {
                            string(vuln.get("id"), "vulnerability id")?;
                        }
                    }
                }
            }
        }
        COMPOSER_AUDIT_TOOL => {
            if payload
                .get("unreachable-repositories")
                .is_some_and(|value| {
                    value.as_array().is_some_and(|entries| !entries.is_empty())
                        || value.as_object().is_some_and(|entries| !entries.is_empty())
                })
            {
                return Err("Composer could not reach every advisory repository".into());
            }
            let advisories = payload
                .get("advisories")
                .ok_or("Composer report has no advisories field")?;
            if let Some(packages) = advisories.as_object() {
                for entries in packages.values() {
                    if !entries.is_array() && !entries.is_object() {
                        return Err("invalid Composer advisories collection".into());
                    }
                    for advisory in super::json_collection(entries) {
                        string(
                            advisory.get("advisoryId").or_else(|| advisory.get("cve")),
                            "advisory id",
                        )?;
                    }
                }
            } else if !advisories.as_array().is_some_and(Vec::is_empty) {
                return Err("invalid Composer advisories field".into());
            }
            if let Some(abandoned) = payload.get("abandoned") {
                if !abandoned.is_object() && !abandoned.as_array().is_some_and(Vec::is_empty) {
                    return Err("invalid Composer abandoned field".into());
                }
            }
            if let Some(filtered) = payload.get("filter") {
                if let Some(packages) = filtered.as_object() {
                    for entries in packages.values() {
                        for entry in array(Some(entries), "Composer filter entries")? {
                            string(entry.get("listName"), "dependency policy list name")?;
                        }
                    }
                } else if !filtered.as_array().is_some_and(Vec::is_empty) {
                    return Err("invalid Composer filter field".into());
                }
            }
        }
        NPM_AUDIT_TOOL => {
            let vulnerabilities = payload
                .get("vulnerabilities")
                .and_then(Value::as_object)
                .ok_or("unsupported npm audit report: expected vulnerabilities object")?;
            for entry in vulnerabilities.values() {
                string(entry.get("severity"), "severity")?;
                array(entry.get("via"), "advisory references")?;
            }
        }
        _ => return Err(format!("unsupported JSON report for {tool}")),
    }
    Ok(())
}
