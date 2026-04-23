use anyhow::Result;
use datadog_api_client::datadogV2::api_csm_threats::{
    CSMThreatsAPI, DeleteCSMThreatsAgentRuleOptionalParams, GetCSMThreatsAgentRuleOptionalParams,
    ListCSMThreatsAgentRulesOptionalParams, UpdateCSMThreatsAgentRuleOptionalParams,
};
use datadog_api_client::datadogV2::api_security_monitoring::{
    ListSecurityMonitoringRulesOptionalParams, SecurityMonitoringAPI,
};
use datadog_api_client::datadogV2::model::{
    SecurityMonitoringRuleCreatePayload, SecurityMonitoringRuleUpdatePayload,
    SecurityMonitoringRuleValidatePayload,
};

use crate::config::Config;
use crate::formatter;
use crate::util;

fn make_csm_api(cfg: &Config) -> CSMThreatsAPI {
    crate::make_api!(CSMThreatsAPI, cfg)
}

pub async fn agent_policies_list(cfg: &Config) -> Result<()> {
    let api = make_csm_api(cfg);
    let resp = api
        .list_csm_threats_agent_policies()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list CSM threats agent policies: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_policies_get(cfg: &Config, policy_id: &str) -> Result<()> {
    let api = make_csm_api(cfg);
    let resp = api
        .get_csm_threats_agent_policy(policy_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get CSM threats agent policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_policies_create(cfg: &Config, file: &str) -> Result<()> {
    let api = make_csm_api(cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_csm_threats_agent_policy(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create CSM threats agent policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_policies_update(cfg: &Config, policy_id: &str, file: &str) -> Result<()> {
    let api = make_csm_api(cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .update_csm_threats_agent_policy(policy_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update CSM threats agent policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_policies_delete(cfg: &Config, policy_id: &str) -> Result<()> {
    let api = make_csm_api(cfg);
    api.delete_csm_threats_agent_policy(policy_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete CSM threats agent policy: {e:?}"))?;
    println!("Agent policy '{policy_id}' deleted successfully.");
    Ok(())
}

pub async fn agent_rules_list(cfg: &Config, policy_id: Option<String>) -> Result<()> {
    let api = make_csm_api(cfg);
    let mut params = ListCSMThreatsAgentRulesOptionalParams::default();
    if let Some(pid) = policy_id {
        params = params.policy_id(pid);
    }
    let resp = api
        .list_csm_threats_agent_rules(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list CSM threats agent rules: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_rules_get(cfg: &Config, rule_id: &str, policy_id: Option<String>) -> Result<()> {
    let api = make_csm_api(cfg);
    let mut params = GetCSMThreatsAgentRuleOptionalParams::default();
    if let Some(pid) = policy_id {
        params = params.policy_id(pid);
    }
    let resp = api
        .get_csm_threats_agent_rule(rule_id.to_string(), params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get CSM threats agent rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_rules_create(cfg: &Config, file: &str) -> Result<()> {
    let api = make_csm_api(cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_csm_threats_agent_rule(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create CSM threats agent rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_rules_update(
    cfg: &Config,
    rule_id: &str,
    file: &str,
    policy_id: Option<String>,
) -> Result<()> {
    let api = make_csm_api(cfg);
    let body = util::read_json_file(file)?;
    let mut params = UpdateCSMThreatsAgentRuleOptionalParams::default();
    if let Some(pid) = policy_id {
        params = params.policy_id(pid);
    }
    let resp = api
        .update_csm_threats_agent_rule(rule_id.to_string(), body, params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update CSM threats agent rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agent_rules_delete(
    cfg: &Config,
    rule_id: &str,
    policy_id: Option<String>,
) -> Result<()> {
    let api = make_csm_api(cfg);
    let mut params = DeleteCSMThreatsAgentRuleOptionalParams::default();
    if let Some(pid) = policy_id {
        params = params.policy_id(pid);
    }
    api.delete_csm_threats_agent_rule(rule_id.to_string(), params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete CSM threats agent rule: {e:?}"))?;
    println!("Agent rule '{rule_id}' deleted successfully.");
    Ok(())
}

pub async fn policy_download(cfg: &Config) -> Result<()> {
    let api = make_csm_api(cfg);
    let bytes = api
        .download_csm_threats_policy()
        .await
        .map_err(|e| anyhow::anyhow!("failed to download CSM threats policy: {e:?}"))?;
    let content = String::from_utf8_lossy(&bytes);
    println!("{content}");
    Ok(())
}

// ---- Backend Rules (Workload Security detection rules) ----

fn make_sec_api(cfg: &Config) -> SecurityMonitoringAPI {
    crate::make_api!(SecurityMonitoringAPI, cfg)
}

pub async fn backend_rules_list(cfg: &Config, query: Option<String>) -> Result<()> {
    let api = make_sec_api(cfg);
    let filter = match query {
        Some(q) => format!("type:workload_security {q}"),
        None => "type:workload_security".to_string(),
    };
    let params = ListSecurityMonitoringRulesOptionalParams::default()
        .page_size(100)
        .query(filter);
    let resp = api
        .list_security_monitoring_rules(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list backend rules: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn backend_rules_get(cfg: &Config, rule_id: &str) -> Result<()> {
    let api = make_sec_api(cfg);
    let resp = api
        .get_security_monitoring_rule(rule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get backend rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn backend_rules_create(cfg: &Config, file: &str) -> Result<()> {
    let body: SecurityMonitoringRuleCreatePayload = util::read_json_file(file)?;
    let api = make_sec_api(cfg);
    let resp = api
        .create_security_monitoring_rule(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create backend rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn backend_rules_update(cfg: &Config, rule_id: &str, file: &str) -> Result<()> {
    let body: SecurityMonitoringRuleUpdatePayload = util::read_json_file(file)?;
    let api = make_sec_api(cfg);
    let resp = api
        .update_security_monitoring_rule(rule_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update backend rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn backend_rules_delete(cfg: &Config, rule_id: &str) -> Result<()> {
    let api = make_sec_api(cfg);
    api.delete_security_monitoring_rule(rule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete backend rule: {e:?}"))?;
    println!("Backend rule '{rule_id}' deleted successfully.");
    Ok(())
}

pub async fn backend_rules_validate(cfg: &Config, file: &str) -> Result<()> {
    let body: SecurityMonitoringRuleValidatePayload = util::read_json_file(file)?;
    let api = make_sec_api(cfg);
    api.validate_security_monitoring_rule(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to validate backend rule: {e:?}"))?;
    println!("Backend rule is valid.");
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_csm_threats_agent_policies_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::agent_policies_list(&cfg).await;
        assert!(
            result.is_ok(),
            "CSM threats agent policies list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_agent_rules_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::agent_rules_list(&cfg, None).await;
        assert!(
            result.is_ok(),
            "CSM threats agent rules list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_agent_rules_list_with_policy() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::agent_rules_list(&cfg, Some("policy-123".to_string())).await;
        assert!(
            result.is_ok(),
            "CSM threats agent rules list with policy failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_agent_policies_list_error() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Forbidden"]}"#)
            .create_async()
            .await;
        let result = super::agent_policies_list(&cfg).await;
        assert!(
            result.is_err(),
            "CSM threats agent policies list should fail on 403"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_backend_rules_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[],"meta":{}}"#).await;
        let result = super::backend_rules_list(&cfg, None).await;
        assert!(
            result.is_ok(),
            "backend rules list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_backend_rules_list_with_query() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[],"meta":{}}"#).await;
        let result = super::backend_rules_list(&cfg, Some("name:my-rule".into())).await;
        assert!(
            result.is_ok(),
            "backend rules list with query failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_backend_rules_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"id":"rule-123","name":"test"}"#).await;
        let result = super::backend_rules_get(&cfg, "rule-123").await;
        assert!(
            result.is_ok(),
            "backend rules get failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_backend_rules_delete() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("DELETE", mockito::Matcher::Any)
            .with_status(204)
            .create_async()
            .await;
        let result = super::backend_rules_delete(&cfg, "rule-123").await;
        assert!(
            result.is_ok(),
            "backend rules delete failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_csm_threats_backend_rules_list_error() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Forbidden"]}"#)
            .create_async()
            .await;
        let result = super::backend_rules_list(&cfg, None).await;
        assert!(result.is_err(), "backend rules list should fail on 403");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
