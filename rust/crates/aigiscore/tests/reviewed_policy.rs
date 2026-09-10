use aigiscore::policy::PolicyBundle;
use serde_json::json;
use std::fs;

#[test]
fn policy_loading_rejects_unbound_drift_and_conflicting_adoptions() {
    let root = std::env::temp_dir().join(format!(
        "aigiscode-policy-boundaries-{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(root.join(".aigiscode")).unwrap();
    let mut row = json!({
        "id": "reviewed-entry", "review_id": "0".repeat(32), "claim_index": 0,
        "disposition": "source_confirmed_concern", "concern": "boundary_violation",
        "conclusion": "violation", "action": "investigate", "reason": "Approved configuration boundary requires investigation",
        "finding_fingerprints": ["configuration-boundary"], "anchor_files": ["src/main.rs"],
        "source_scope_id": "1".repeat(32), "observed_change": "New", "comparison_baseline_id": null
    });
    let write = |rows| fs::write(root.join(".aigiscode/policy.json"), serde_json::to_vec(&json!({"reviewed_decisions": rows})).unwrap()).unwrap();
    write(json!([row.clone()]));
    assert!(PolicyBundle::load(&root).unwrap_err().to_string().contains("eligible comparison baseline"));
    row["observed_change"] = json!("NotCompared");
    write(json!([row.clone()]));
    assert_eq!(PolicyBundle::load(&root).unwrap().reviewed_decisions().len(), 1);
    let mut competing = row.clone();
    competing["id"] = json!("another-review");
    write(json!([row.clone(), competing]));
    assert!(PolicyBundle::load(&root).unwrap_err().to_string().contains("multiple reviewed dispositions"));
    row["disposition"] = json!("accepted_pattern");
    write(json!([row.clone()]));
    assert!(PolicyBundle::load(&root).unwrap_err().to_string().contains("explicitly retained"));
    row["disposition"] = json!("source_confirmed_concern");
    row["anchor_files"] = json!(["../outside.php"]);
    write(json!([row]));
    assert!(PolicyBundle::load(&root).unwrap_err().to_string().contains("repository-relative"));
    fs::remove_dir_all(root).unwrap();
}
