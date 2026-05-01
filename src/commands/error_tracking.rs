use anyhow::Result;
use chrono::Utc;
use datadog_api_client::datadogV2::api_error_tracking::{
    ErrorTrackingAPI, GetIssueOptionalParams, SearchIssuesOptionalParams,
};
use datadog_api_client::datadogV2::model::{
    IssuesSearchRequest, IssuesSearchRequestData, IssuesSearchRequestDataAttributes,
    IssuesSearchRequestDataAttributesPersona, IssuesSearchRequestDataAttributesTrack,
    IssuesSearchRequestDataType,
};

use crate::config::Config;
use crate::formatter;

pub async fn issues_search(
    cfg: &Config,
    query: Option<String>,
    _limit: i32,
    track: Option<String>,
    persona: Option<String>,
) -> Result<()> {
    let api = crate::make_api!(ErrorTrackingAPI, cfg);

    let now = Utc::now().timestamp_millis();
    let one_day_ago = now - 86_400_000; // 24 hours in millis

    let query_str = query.unwrap_or_else(|| "*".to_string());
    let mut attrs = IssuesSearchRequestDataAttributes::new(one_day_ago, query_str, now);
    if let Some(ref t) = track {
        let track_value = match t.to_lowercase().as_str() {
            "trace" => IssuesSearchRequestDataAttributesTrack::TRACE,
            "logs" => IssuesSearchRequestDataAttributesTrack::LOGS,
            "rum" => IssuesSearchRequestDataAttributesTrack::RUM,
            other => anyhow::bail!(
                "invalid track value '{}': must be trace, logs, or rum",
                other
            ),
        };
        attrs = attrs.track(track_value);
    }
    if let Some(ref p) = persona {
        let persona_value = match p.to_uppercase().as_str() {
            "ALL" => IssuesSearchRequestDataAttributesPersona::ALL,
            "BROWSER" => IssuesSearchRequestDataAttributesPersona::BROWSER,
            "MOBILE" => IssuesSearchRequestDataAttributesPersona::MOBILE,
            "BACKEND" => IssuesSearchRequestDataAttributesPersona::BACKEND,
            other => anyhow::bail!(
                "invalid persona value '{}': must be ALL, BROWSER, MOBILE, or BACKEND",
                other
            ),
        };
        attrs = attrs.persona(persona_value);
    }
    let data = IssuesSearchRequestData::new(attrs, IssuesSearchRequestDataType::SEARCH_REQUEST);
    let body = IssuesSearchRequest::new(data);
    let params = SearchIssuesOptionalParams::default();

    let resp = api
        .search_issues(body, params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to search issues: {e:?}"))?;
    let val = serde_json::to_value(&resp)?;
    if let Some(data) = val.get("data") {
        if data.as_array().is_some_and(|a| a.is_empty()) {
            println!("No error tracking issues found matching the specified criteria.");
            return Ok(());
        }
    }
    formatter::output(cfg, &resp)
}

pub async fn issues_get(cfg: &Config, issue_id: &str) -> Result<()> {
    let api = crate::make_api!(ErrorTrackingAPI, cfg);
    let resp = api
        .get_issue(issue_id.to_string(), GetIssueOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get issue: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;
    use clap::CommandFactory;

    #[tokio::test]
    async fn test_error_tracking_issues_search() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::issues_search(&cfg, None, 10, Some("trace".into()), None).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_error_tracking_issues_search_persona() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::issues_search(&cfg, None, 10, None, Some("BROWSER".into())).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_error_tracking_issues_search_track_case_insensitive() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::issues_search(&cfg, None, 10, Some("RUM".into()), None).await;
        cleanup_env();
    }

    #[test]
    fn test_error_tracking_clap_mutual_exclusivity() {
        let result = crate::Cli::command().try_get_matches_from([
            "pup",
            "error-tracking",
            "issues",
            "search",
            "--track",
            "trace",
            "--persona",
            "ALL",
        ]);
        assert!(
            result.is_err(),
            "expected error when both --track and --persona are provided"
        );
    }

    #[test]
    fn test_error_tracking_clap_neither_provided() {
        let result = crate::Cli::command().try_get_matches_from([
            "pup",
            "error-tracking",
            "issues",
            "search",
        ]);
        assert!(
            result.is_err(),
            "expected error when neither --track nor --persona is provided"
        );
    }
}
