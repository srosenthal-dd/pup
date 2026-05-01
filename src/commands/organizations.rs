use anyhow::Result;
use datadog_api_client::datadogV1::api_organizations::OrganizationsAPI;
use datadog_api_client::datadogV2::api_org_groups::{
    ListOrgGroupPoliciesOptionalParams, ListOrgGroupPolicyOverridesOptionalParams, OrgGroupsAPI,
};
use datadog_api_client::datadogV2::model::{
    OrgGroupPolicyCreateRequest, OrgGroupPolicyOverrideCreateRequest,
    OrgGroupPolicyOverrideSortOption, OrgGroupPolicyOverrideUpdateRequest,
    OrgGroupPolicySortOption, OrgGroupPolicyUpdateRequest,
};

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(OrganizationsAPI, cfg);
    let resp = api
        .list_orgs()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list orgs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(OrganizationsAPI, cfg);
    let resp = api
        .get_org("current".to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get org: {e:?}"))?;
    formatter::output(cfg, &resp)
}

fn parse_uuid(label: &str, value: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(value).map_err(|e| anyhow::anyhow!("invalid {label} '{value}': {e}"))
}

fn parse_policy_sort(s: &str) -> Result<OrgGroupPolicySortOption> {
    Ok(match s {
        "id" => OrgGroupPolicySortOption::ID,
        "-id" => OrgGroupPolicySortOption::MINUS_ID,
        "name" => OrgGroupPolicySortOption::NAME,
        "-name" => OrgGroupPolicySortOption::MINUS_NAME,
        _ => anyhow::bail!("invalid sort '{s}' — use one of: id, -id, name, -name"),
    })
}

fn parse_override_sort(s: &str) -> Result<OrgGroupPolicyOverrideSortOption> {
    Ok(match s {
        "id" => OrgGroupPolicyOverrideSortOption::ID,
        "-id" => OrgGroupPolicyOverrideSortOption::MINUS_ID,
        "org_uuid" => OrgGroupPolicyOverrideSortOption::ORG_UUID,
        "-org_uuid" => OrgGroupPolicyOverrideSortOption::MINUS_ORG_UUID,
        _ => anyhow::bail!("invalid sort '{s}' — use one of: id, -id, org_uuid, -org_uuid"),
    })
}

// ---- Policies ----

pub async fn policies_list(
    cfg: &Config,
    group_id: &str,
    name: Option<String>,
    page_number: Option<i64>,
    page_size: Option<i64>,
    sort: Option<String>,
) -> Result<()> {
    let group_uuid = parse_uuid("group-id", group_id)?;
    let mut params = ListOrgGroupPoliciesOptionalParams::default();
    if let Some(n) = name {
        params.filter_policy_name = Some(n);
    }
    if let Some(n) = page_number {
        params.page_number = Some(n);
    }
    if let Some(n) = page_size {
        params.page_size = Some(n);
    }
    if let Some(s) = sort {
        params.sort = Some(parse_policy_sort(&s)?);
    }
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .list_org_group_policies(group_uuid, params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list org group policies: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policies_get(cfg: &Config, policy_id: &str) -> Result<()> {
    let id = parse_uuid("policy-id", policy_id)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .get_org_group_policy(id)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get org group policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policies_create(cfg: &Config, file: &str) -> Result<()> {
    let body: OrgGroupPolicyCreateRequest = util::read_json_file(file)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .create_org_group_policy(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create org group policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policies_update(cfg: &Config, policy_id: &str, file: &str) -> Result<()> {
    let id = parse_uuid("policy-id", policy_id)?;
    let body: OrgGroupPolicyUpdateRequest = util::read_json_file(file)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .update_org_group_policy(id, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update org group policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policies_delete(cfg: &Config, policy_id: &str) -> Result<()> {
    let id = parse_uuid("policy-id", policy_id)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    api.delete_org_group_policy(id)
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete org group policy: {e:?}"))?;
    eprintln!("Org group policy '{policy_id}' deleted.");
    Ok(())
}

// ---- Policy Overrides ----

pub async fn policy_overrides_list(
    cfg: &Config,
    group_id: &str,
    policy_id: Option<String>,
    page_number: Option<i64>,
    page_size: Option<i64>,
    sort: Option<String>,
) -> Result<()> {
    let group_uuid = parse_uuid("group-id", group_id)?;
    let mut params = ListOrgGroupPolicyOverridesOptionalParams::default();
    if let Some(pid) = policy_id {
        params.filter_policy_id = Some(parse_uuid("policy-id", &pid)?);
    }
    if let Some(n) = page_number {
        params.page_number = Some(n);
    }
    if let Some(n) = page_size {
        params.page_size = Some(n);
    }
    if let Some(s) = sort {
        params.sort = Some(parse_override_sort(&s)?);
    }
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .list_org_group_policy_overrides(group_uuid, params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list org group policy overrides: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policy_overrides_get(cfg: &Config, override_id: &str) -> Result<()> {
    let id = parse_uuid("override-id", override_id)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .get_org_group_policy_override(id)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get org group policy override: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policy_overrides_create(cfg: &Config, file: &str) -> Result<()> {
    let body: OrgGroupPolicyOverrideCreateRequest = util::read_json_file(file)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .create_org_group_policy_override(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create org group policy override: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policy_overrides_update(cfg: &Config, override_id: &str, file: &str) -> Result<()> {
    let id = parse_uuid("override-id", override_id)?;
    let body: OrgGroupPolicyOverrideUpdateRequest = util::read_json_file(file)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .update_org_group_policy_override(id, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update org group policy override: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn policy_overrides_delete(cfg: &Config, override_id: &str) -> Result<()> {
    let id = parse_uuid("override-id", override_id)?;
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    api.delete_org_group_policy_override(id)
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete org group policy override: {e:?}"))?;
    eprintln!("Org group policy override '{override_id}' deleted.");
    Ok(())
}

// ---- Policy Configs ----

pub async fn policy_configs_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(OrgGroupsAPI, cfg);
    let resp = api
        .list_org_group_policy_configs()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list org group policy configs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    const GROUP_UUID: &str = "11111111-1111-1111-1111-111111111111";
    const POLICY_UUID: &str = "22222222-2222-2222-2222-222222222222";
    const OVERRIDE_UUID: &str = "33333333-3333-3333-3333-333333333333";

    #[tokio::test]
    async fn test_organizations_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"orgs": []}"#).await;
        let _ = super::list(&cfg).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_policies_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::policies_list(&cfg, GROUP_UUID, None, None, None, None).await;
        assert!(result.is_ok(), "policies_list failed: {:?}", result.err());
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policies_list_with_params() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::policies_list(
            &cfg,
            GROUP_UUID,
            Some("admin".into()),
            Some(1),
            Some(50),
            Some("-name".into()),
        )
        .await;
        assert!(
            result.is_ok(),
            "policies_list with params failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policies_list_bad_group_id() {
        let _lock = lock_env().await;
        let cfg = test_config("http://unused.local");
        let result = super::policies_list(&cfg, "not-a-uuid", None, None, None, None).await;
        assert!(result.is_err(), "expected UUID parse error");
        assert!(result.unwrap_err().to_string().contains("invalid group-id"));
    }

    #[tokio::test]
    async fn test_policies_list_bad_sort() {
        let _lock = lock_env().await;
        let cfg = test_config("http://unused.local");
        let result =
            super::policies_list(&cfg, GROUP_UUID, None, None, None, Some("bogus".into())).await;
        assert!(result.is_err(), "expected sort parse error");
    }

    #[tokio::test]
    async fn test_policies_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let body = format!(
            r#"{{"data":{{"id":"{POLICY_UUID}","type":"org_group_policies","attributes":{{"enforcement_tier":"DEFAULT","modified_at":"2024-01-01T00:00:00Z","policy_name":"test","policy_type":"org_config"}}}}}}"#
        );
        let _mock = mock_any(&mut server, "GET", &body).await;
        let result = super::policies_get(&cfg, POLICY_UUID).await;
        assert!(result.is_ok(), "policies_get failed: {:?}", result.err());
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policies_get_404() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["not found"]}"#)
            .create_async()
            .await;
        let result = super::policies_get(&cfg, POLICY_UUID).await;
        assert!(result.is_err(), "expected 404 error");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policies_create() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let tmp = write_temp_json(
            "pup_test_policies_create.json",
            &format!(
                r#"{{"data":{{"type":"org_group_policies","attributes":{{"content":{{}},"policy_name":"test"}},"relationships":{{"org_group":{{"data":{{"id":"{GROUP_UUID}","type":"org_groups"}}}}}}}}}}"#
            ),
        );
        let body = format!(
            r#"{{"data":{{"id":"{POLICY_UUID}","type":"org_group_policies","attributes":{{"enforcement_tier":"DEFAULT","modified_at":"2024-01-01T00:00:00Z","policy_name":"test","policy_type":"org_config"}}}}}}"#
        );
        let _mock = mock_any(&mut server, "POST", &body).await;
        let result = super::policies_create(&cfg, tmp.to_str().unwrap()).await;
        assert!(result.is_ok(), "policies_create failed: {:?}", result.err());
        let _ = std::fs::remove_file(tmp);
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policies_update() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let tmp = write_temp_json(
            "pup_test_policies_update.json",
            &format!(
                r#"{{"data":{{"id":"{POLICY_UUID}","type":"org_group_policies","attributes":{{}}}}}}"#
            ),
        );
        let body = format!(
            r#"{{"data":{{"id":"{POLICY_UUID}","type":"org_group_policies","attributes":{{"enforcement_tier":"DEFAULT","modified_at":"2024-01-01T00:00:00Z","policy_name":"test","policy_type":"org_config"}}}}}}"#
        );
        let _mock = mock_any(&mut server, "PATCH", &body).await;
        let result = super::policies_update(&cfg, POLICY_UUID, tmp.to_str().unwrap()).await;
        assert!(result.is_ok(), "policies_update failed: {:?}", result.err());
        let _ = std::fs::remove_file(tmp);
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policies_delete() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("DELETE", mockito::Matcher::Any)
            .with_status(204)
            .create_async()
            .await;
        let result = super::policies_delete(&cfg, POLICY_UUID).await;
        assert!(result.is_ok(), "policies_delete failed: {:?}", result.err());
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_overrides_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::policy_overrides_list(&cfg, GROUP_UUID, None, None, None, None).await;
        assert!(
            result.is_ok(),
            "policy_overrides_list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_overrides_list_filter_policy() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::policy_overrides_list(
            &cfg,
            GROUP_UUID,
            Some(POLICY_UUID.into()),
            Some(1),
            Some(25),
            Some("org_uuid".into()),
        )
        .await;
        assert!(
            result.is_ok(),
            "policy_overrides_list filter failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_overrides_list_bad_policy_id() {
        let _lock = lock_env().await;
        let cfg = test_config("http://unused.local");
        let result = super::policy_overrides_list(
            &cfg,
            GROUP_UUID,
            Some("not-a-uuid".into()),
            None,
            None,
            None,
        )
        .await;
        assert!(result.is_err(), "expected UUID parse error");
    }

    #[tokio::test]
    async fn test_policy_overrides_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let body = format!(
            r#"{{"data":{{"id":"{OVERRIDE_UUID}","type":"org_group_policy_overrides","attributes":{{"created_at":"2024-01-01T00:00:00Z","modified_at":"2024-01-01T00:00:00Z","org_site":"datadoghq.com","org_uuid":"{OVERRIDE_UUID}"}}}}}}"#
        );
        let _mock = mock_any(&mut server, "GET", &body).await;
        let result = super::policy_overrides_get(&cfg, OVERRIDE_UUID).await;
        assert!(
            result.is_ok(),
            "policy_overrides_get failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_overrides_create() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let tmp = write_temp_json(
            "pup_test_overrides_create.json",
            &format!(
                r#"{{"data":{{"type":"org_group_policy_overrides","attributes":{{"org_site":"datadoghq.com","org_uuid":"{OVERRIDE_UUID}"}},"relationships":{{"org_group":{{"data":{{"id":"{GROUP_UUID}","type":"org_groups"}}}},"org_group_policy":{{"data":{{"id":"{POLICY_UUID}","type":"org_group_policies"}}}}}}}}}}"#
            ),
        );
        let body = format!(
            r#"{{"data":{{"id":"{OVERRIDE_UUID}","type":"org_group_policy_overrides","attributes":{{"created_at":"2024-01-01T00:00:00Z","modified_at":"2024-01-01T00:00:00Z","org_site":"datadoghq.com","org_uuid":"{OVERRIDE_UUID}"}}}}}}"#
        );
        let _mock = mock_any(&mut server, "POST", &body).await;
        let result = super::policy_overrides_create(&cfg, tmp.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "policy_overrides_create failed: {:?}",
            result.err()
        );
        let _ = std::fs::remove_file(tmp);
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_overrides_update() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let tmp = write_temp_json(
            "pup_test_overrides_update.json",
            &format!(
                r#"{{"data":{{"id":"{OVERRIDE_UUID}","type":"org_group_policy_overrides","attributes":{{"org_site":"datadoghq.com","org_uuid":"{OVERRIDE_UUID}"}}}}}}"#
            ),
        );
        let body = format!(
            r#"{{"data":{{"id":"{OVERRIDE_UUID}","type":"org_group_policy_overrides","attributes":{{"created_at":"2024-01-01T00:00:00Z","modified_at":"2024-01-01T00:00:00Z","org_site":"datadoghq.com","org_uuid":"{OVERRIDE_UUID}"}}}}}}"#
        );
        let _mock = mock_any(&mut server, "PATCH", &body).await;
        let result =
            super::policy_overrides_update(&cfg, OVERRIDE_UUID, tmp.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "policy_overrides_update failed: {:?}",
            result.err()
        );
        let _ = std::fs::remove_file(tmp);
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_overrides_delete() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("DELETE", mockito::Matcher::Any)
            .with_status(204)
            .create_async()
            .await;
        let result = super::policy_overrides_delete(&cfg, OVERRIDE_UUID).await;
        assert!(
            result.is_ok(),
            "policy_overrides_delete failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_configs_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::policy_configs_list(&cfg).await;
        assert!(
            result.is_ok(),
            "policy_configs_list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_policy_configs_list_403() {
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
        let result = super::policy_configs_list(&cfg).await;
        assert!(result.is_err(), "expected 403 error");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
