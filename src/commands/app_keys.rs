use anyhow::Result;
use datadog_api_client::datadogV2::api_key_management::{
    KeyManagementAPI, ListApplicationKeysOptionalParams,
    ListCurrentUserApplicationKeysOptionalParams,
};
use datadog_api_client::datadogV2::model::ApplicationKeysSort;

use crate::config::Config;
use crate::formatter;

fn parse_sort(s: &str) -> Result<ApplicationKeysSort> {
    match s {
        "created_at" => Ok(ApplicationKeysSort::CREATED_AT_ASCENDING),
        "-created_at" => Ok(ApplicationKeysSort::CREATED_AT_DESCENDING),
        "last4" => Ok(ApplicationKeysSort::LAST4_ASCENDING),
        "-last4" => Ok(ApplicationKeysSort::LAST4_DESCENDING),
        "name" => Ok(ApplicationKeysSort::NAME_ASCENDING),
        "-name" => Ok(ApplicationKeysSort::NAME_DESCENDING),
        _ => anyhow::bail!(
            "invalid --sort value: {s:?}\nExpected: name, -name, created_at, -created_at, last4, -last4"
        ),
    }
}

// ---------------------------------------------------------------------------
// List application keys (current user)
// ---------------------------------------------------------------------------

pub async fn list(
    cfg: &Config,
    page_size: i64,
    page_number: i64,
    filter: &str,
    sort: &str,
) -> Result<()> {
    let api = crate::make_api!(KeyManagementAPI, cfg);

    let mut params = ListCurrentUserApplicationKeysOptionalParams::default();
    if page_size > 0 {
        params.page_size = Some(page_size);
    }
    if page_number > 0 {
        params.page_number = Some(page_number);
    }
    if !filter.is_empty() {
        params.filter = Some(filter.to_string());
    }
    if !sort.is_empty() {
        params.sort = Some(parse_sort(sort)?);
    }

    let resp = api
        .list_current_user_application_keys(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list application keys: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---------------------------------------------------------------------------
// List all application keys (org-wide, requires API keys)
// ---------------------------------------------------------------------------

pub async fn list_all(
    cfg: &Config,
    page_size: i64,
    page_number: i64,
    filter: &str,
    sort: &str,
) -> Result<()> {
    let api = crate::make_api!(KeyManagementAPI, cfg);

    let mut params = ListApplicationKeysOptionalParams::default();
    if page_size > 0 {
        params.page_size = Some(page_size);
    }
    if page_number > 0 {
        params.page_number = Some(page_number);
    }
    if !filter.is_empty() {
        params.filter = Some(filter.to_string());
    }
    if !sort.is_empty() {
        params.sort = Some(parse_sort(sort)?);
    }

    let resp = api
        .list_application_keys(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list all application keys: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---------------------------------------------------------------------------
// Get application key details (current user)
// ---------------------------------------------------------------------------

pub async fn get(cfg: &Config, key_id: &str) -> Result<()> {
    let api = crate::make_api!(KeyManagementAPI, cfg);
    let resp = api
        .get_current_user_application_key(key_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get application key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---------------------------------------------------------------------------
// Create application key (current user)
// ---------------------------------------------------------------------------

pub async fn create(cfg: &Config, name: &str, scopes: &str) -> Result<()> {
    use datadog_api_client::datadogV2::model::{
        ApplicationKeyCreateAttributes, ApplicationKeyCreateData, ApplicationKeyCreateRequest,
        ApplicationKeysType,
    };

    let mut attrs = ApplicationKeyCreateAttributes::new(name.to_string());
    if !scopes.is_empty() {
        let scope_list: Vec<String> = scopes
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        attrs.scopes = Some(Some(scope_list));
    }

    let body = ApplicationKeyCreateRequest::new(ApplicationKeyCreateData::new(
        attrs,
        ApplicationKeysType::APPLICATION_KEYS,
    ));

    let api = crate::make_api!(KeyManagementAPI, cfg);
    let resp = api
        .create_current_user_application_key(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create application key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---------------------------------------------------------------------------
// Update application key (current user)
// ---------------------------------------------------------------------------

pub async fn update(cfg: &Config, key_id: &str, name: &str, scopes: &str) -> Result<()> {
    use datadog_api_client::datadogV2::model::{
        ApplicationKeyUpdateAttributes, ApplicationKeyUpdateData, ApplicationKeyUpdateRequest,
        ApplicationKeysType,
    };

    let mut attrs = ApplicationKeyUpdateAttributes::new();
    if !name.is_empty() {
        attrs.name = Some(name.to_string());
    }
    if !scopes.is_empty() {
        let scope_list: Vec<String> = scopes
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        attrs.scopes = Some(Some(scope_list));
    }

    let body = ApplicationKeyUpdateRequest::new(ApplicationKeyUpdateData::new(
        attrs,
        key_id.to_string(),
        ApplicationKeysType::APPLICATION_KEYS,
    ));

    let api = crate::make_api!(KeyManagementAPI, cfg);
    let resp = api
        .update_current_user_application_key(key_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update application key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---------------------------------------------------------------------------
// Delete application key (current user)
// ---------------------------------------------------------------------------

pub async fn delete(cfg: &Config, key_id: &str) -> Result<()> {
    let api = crate::make_api!(KeyManagementAPI, cfg);
    api.delete_current_user_application_key(key_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete application key: {e:?}"))?;
    println!("Successfully deleted application key {key_id}");
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_app_keys_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::list(&cfg, 10, 0, "", "").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_list_all() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::list_all(&cfg, 10, 0, "", "").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_get() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let _ = super::get(&cfg, "k1").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_create() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let _ = super::create(&cfg, "test-key", "").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_update() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let _ = super::update(&cfg, "k1", "new-name", "").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_delete() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{}"#).await;
        let _ = super::delete(&cfg, "k1").await;
        cleanup_env();
    }
}
