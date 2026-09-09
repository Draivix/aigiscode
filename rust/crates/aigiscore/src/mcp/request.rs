use super::contracts::{ConsistencyMode, Freshness, RepoOverviewParams};
use super::{AigiscodeMcpServer, ReadyState};
use rmcp::{model::Meta, ErrorData as McpError};

const INITIAL_WAIT_MS: u64 = 30_000;
const MAX_WAIT_MS: u64 = 120_000;

impl AigiscodeMcpServer {
    /// One immutable index per protocol request; discovery remains available while
    /// indexing. Failed startup wakes waiters and returns a typed error immediately.
    pub(super) async fn for_request(
        &self,
        params: Option<&RepoOverviewParams>,
    ) -> Result<Self, McpError> {
        let target = params.map_or(0, |params| {
            params.min_revision.unwrap_or_else(|| {
                if params.consistency == ConsistencyMode::WaitUntilIndexed {
                    self.live.observed()
                } else {
                    0
                }
            })
        });
        let pending = self.live.load().snapshot.is_none();
        let allow_stale =
            params.is_some_and(|params| params.consistency == ConsistencyMode::AllowStale);
        let wait_ms = params
            .and_then(|params| params.wait_ms)
            .unwrap_or(if pending { INITIAL_WAIT_MS } else { 0 })
            .min(MAX_WAIT_MS);
        if (pending && !allow_stale)
            || params.is_some_and(|params| params.consistency == ConsistencyMode::WaitUntilIndexed)
        {
            self.live.wait_for_revision(target.max(1), wait_ms).await;
        }
        let published = self.live.load();
        if published.snapshot.is_none() {
            let error = self.live.last_error();
            return Err(McpError::internal_error(
                error.clone().unwrap_or_else(|| {
                    String::from("Initial index is still building; retry when ready")
                }),
                Some(serde_json::json!({
                    "index_state": if error.is_some() { "failed" } else { "indexing" },
                    "retryable": error.is_none(),
                    "freshness": self.live.freshness(false),
                })),
            ));
        }
        let mut view = self.clone();
        view.request_snapshot = Some(ReadyState(published));
        view.request_target = target;
        Ok(view)
    }

    pub(super) fn freshness(&self, satisfied: bool) -> Freshness {
        match &self.request_snapshot {
            Some(state) => self.live.freshness_for(
                &state.0,
                satisfied && state.revision() >= self.request_target,
            ),
            None => self.live.freshness(satisfied),
        }
    }

    pub(super) fn actionable_freshness(&self, satisfied: bool) -> Option<Freshness> {
        if self.request_snapshot.is_none() {
            return self.live.actionable_freshness(satisfied);
        }
        let freshness = self.freshness(satisfied);
        (freshness.is_stale || freshness.rebuilding).then_some(freshness)
    }

    pub(super) fn freshness_meta(&self) -> Result<Meta, McpError> {
        let value = serde_json::to_value(self.freshness(true))
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        let published = self.live.load();
        let snapshot = self.request_snapshot.as_ref().map(|state| state.snapshot())
            .or_else(|| published.snapshot.as_ref());
        let coverage = snapshot.map(|snapshot| &snapshot.repo_overview.overview.input_coverage);
        let secondary = snapshot.map(|snapshot| &snapshot.repo_overview.overview.ast_grep_coverage);
        Ok(Meta(serde_json::Map::from_iter([
            (String::from("aigiscode/freshness"), value),
            (String::from("aigiscode/artifact_generation"), serde_json::to_value(snapshot.map(|snapshot| &snapshot.artifact_generation))
                .map_err(|error| McpError::internal_error(error.to_string(), None))?),
            (String::from("aigiscode/ast_grep_coverage"), serde_json::json!({
                "status": secondary.map_or(crate::scanners::coverage::SecondaryCoverageStatus::Unknown, |coverage| coverage.status),
                "oversized_files": secondary.map(|coverage| coverage.oversized_files),
                "unsupported_files": secondary.map(|coverage| coverage.unsupported_files),
                "other_gap_files": secondary.map(|coverage| coverage.other_gap_files),
            })),
            (String::from("aigiscode/input_coverage"), serde_json::json!({
                "status": coverage.map_or(crate::coverage::InputCoverageStatus::Unknown, |coverage| coverage.status),
                "recovered_source_files": coverage.map(|coverage| coverage.recovered_source_files),
                "unsupported_source_files": coverage.map(|coverage| coverage.unsupported_source_files),
                "scope_limited_files": coverage.map(|coverage| coverage.scope_limited_files),
                "files_without_parse_evidence": coverage.map(|coverage| coverage.files_without_parse_evidence),
            })),
        ])))
    }
}
