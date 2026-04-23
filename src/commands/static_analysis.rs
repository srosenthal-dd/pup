use anyhow::Result;
use datadog_api_client::datadogV2::api_static_analysis::{
    ListCustomRuleRevisionsOptionalParams, StaticAnalysisAPI,
};

use crate::config::Config;
use crate::formatter;
use crate::util;

// ---------------------------------------------------------------------------
// Custom rulesets
// ---------------------------------------------------------------------------

pub async fn custom_rulesets_get(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    let resp = api
        .get_custom_ruleset(id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get custom ruleset: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn custom_rulesets_update(cfg: &Config, ruleset_name: &str, file: &str) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .update_custom_ruleset(ruleset_name.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update custom ruleset: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn custom_rulesets_delete(cfg: &Config, ruleset_name: &str) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    api.delete_custom_ruleset(ruleset_name.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete custom ruleset: {e:?}"))?;
    println!("Custom ruleset '{ruleset_name}' deleted successfully.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Custom rules within a ruleset
// ---------------------------------------------------------------------------

pub async fn custom_rules_get(cfg: &Config, ruleset_name: &str, rule_name: &str) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    let resp = api
        .get_custom_rule(ruleset_name.to_string(), rule_name.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get custom rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn custom_rules_create(cfg: &Config, ruleset_name: &str, file: &str) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_custom_rule(ruleset_name.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create custom rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn custom_rules_delete(cfg: &Config, ruleset_name: &str, rule_name: &str) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    api.delete_custom_rule(ruleset_name.to_string(), rule_name.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete custom rule: {e:?}"))?;
    println!("Custom rule '{rule_name}' in ruleset '{ruleset_name}' deleted successfully.");
    Ok(())
}

pub async fn custom_rule_revisions_list(
    cfg: &Config,
    ruleset_name: &str,
    rule_name: &str,
) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    let resp = api
        .list_custom_rule_revisions(
            ruleset_name.to_string(),
            rule_name.to_string(),
            ListCustomRuleRevisionsOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to list custom rule revisions: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn custom_rule_revision_get(
    cfg: &Config,
    ruleset_name: &str,
    rule_name: &str,
    revision_id: &str,
) -> Result<()> {
    let api = crate::make_api!(StaticAnalysisAPI, cfg);
    let resp = api
        .get_custom_rule_revision(
            ruleset_name.to_string(),
            rule_name.to_string(),
            revision_id.to_string(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get custom rule revision: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_static_analysis_custom_rulesets_get_existing() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(
            &mut s,
            r#"{"data": {"id": "rs", "type": "custom-ruleset"}}"#,
        )
        .await;
        let _ = super::custom_rulesets_get(&cfg, "my-ruleset").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rules_get_existing() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {"id": "rule", "type": "custom-rule"}}"#).await;
        let _ = super::custom_rules_get(&cfg, "my-ruleset", "my-rule").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rulesets_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let body = r#"{"data":{"id":"my-ruleset","type":"custom-ruleset","attributes":{"name":"my-ruleset","created_at":"2024-01-01T00:00:00Z","created_by":"user","description":"","short_description":"","rules":[]}}}"#;
        let _mock = mock_any(&mut server, "GET", body).await;
        let result = super::custom_rulesets_get(&cfg, "my-ruleset").await;
        assert!(
            result.is_ok(),
            "static analysis custom rulesets get failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rulesets_get_error() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["not found"]}"#)
            .create_async()
            .await;
        let result = super::custom_rulesets_get(&cfg, "nonexistent").await;
        assert!(
            result.is_err(),
            "static analysis custom rulesets get should fail on 404"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rules_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        // Complex nested model — test that a 404 error is correctly propagated.
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["not found"]}"#)
            .create_async()
            .await;
        let result = super::custom_rules_get(&cfg, "my-ruleset", "my-rule").await;
        assert!(
            result.is_err(),
            "static analysis custom rules get should fail on 404"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rule_revisions_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::custom_rule_revisions_list(&cfg, "my-ruleset", "my-rule").await;
        assert!(
            result.is_ok(),
            "static analysis custom rule revisions list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rules_delete() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("DELETE", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(204)
            .create_async()
            .await;
        let result = super::custom_rules_delete(&cfg, "my-ruleset", "my-rule").await;
        assert!(
            result.is_ok(),
            "static analysis custom rules delete failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rules_create_missing_file() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let result = super::custom_rules_create(&cfg, "my-ruleset", "/nonexistent/file.json").await;
        assert!(
            result.is_err(),
            "custom_rules_create should fail for missing file"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_static_analysis_custom_rules_list_error() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["forbidden"]}"#)
            .create_async()
            .await;
        let result = super::custom_rules_get(&cfg, "my-ruleset", "my-rule").await;
        assert!(
            result.is_err(),
            "custom_rules_get should fail on 403 response"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
