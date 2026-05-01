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

    use super::*;
    use crate::test_support::*;

    // -----------------------------------------------------------------------
    // parse_sort()
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_sort_name_ascending() {
        let sort = parse_sort("name").unwrap();
        assert_eq!(sort, ApplicationKeysSort::NAME_ASCENDING);
    }

    #[test]
    fn test_parse_sort_name_descending() {
        let sort = parse_sort("-name").unwrap();
        assert_eq!(sort, ApplicationKeysSort::NAME_DESCENDING);
    }

    #[test]
    fn test_parse_sort_created_at_ascending() {
        let sort = parse_sort("created_at").unwrap();
        assert_eq!(sort, ApplicationKeysSort::CREATED_AT_ASCENDING);
    }

    #[test]
    fn test_parse_sort_created_at_descending() {
        let sort = parse_sort("-created_at").unwrap();
        assert_eq!(sort, ApplicationKeysSort::CREATED_AT_DESCENDING);
    }

    #[test]
    fn test_parse_sort_last4_ascending() {
        let sort = parse_sort("last4").unwrap();
        assert_eq!(sort, ApplicationKeysSort::LAST4_ASCENDING);
    }

    #[test]
    fn test_parse_sort_last4_descending() {
        let sort = parse_sort("-last4").unwrap();
        assert_eq!(sort, ApplicationKeysSort::LAST4_DESCENDING);
    }

    #[test]
    fn test_parse_sort_invalid_rejects_unknown_value() {
        let result = parse_sort("created");
        assert!(result.is_err(), "bare 'created' should be rejected");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("invalid --sort value"),
            "error should mention invalid sort: {err_msg}"
        );
    }

    #[test]
    fn test_parse_sort_invalid_lists_accepted_values() {
        let err_msg = parse_sort("bogus").unwrap_err().to_string();
        // Error message should list the accepted values so users can self-correct.
        assert!(err_msg.contains("name"), "error should list name");
        assert!(
            err_msg.contains("created_at"),
            "error should list created_at"
        );
        assert!(err_msg.contains("last4"), "error should list last4");
    }

    #[test]
    fn test_parse_sort_empty_is_invalid() {
        // Empty string is not a valid sort token — callers short-circuit before
        // invoking parse_sort, but the parser itself must reject it.
        assert!(parse_sort("").is_err());
    }

    #[test]
    fn test_parse_sort_case_sensitive() {
        // The API enum is case-sensitive; uppercase variants must not be silently accepted.
        assert!(parse_sort("NAME").is_err());
        assert!(parse_sort("Created_At").is_err());
    }

    // -----------------------------------------------------------------------
    // list()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_app_keys_list_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let result = super::list(&cfg, 10, 0, "", "").await;
        assert!(result.is_ok(), "list should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_list_with_filter_and_sort() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let result = super::list(&cfg, 25, 1, "prod", "-created_at").await;
        assert!(
            result.is_ok(),
            "list with filter/sort should succeed: {:?}",
            result.err()
        );
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_list_invalid_sort_rejected_before_network() {
        let _lock = lock_env().await;
        // No mocks registered — if the function tried to make an HTTP call it
        // would hang or error with a connection-refused. Asserting is_err()
        // with the correct error message confirms validation runs first.
        let server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let result = super::list(&cfg, 10, 0, "", "not-a-sort").await;
        assert!(result.is_err(), "invalid sort should be rejected");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid --sort value"));
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_list_error_403() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Forbidden"]}"#)
            .create_async()
            .await;
        let result = super::list(&cfg, 10, 0, "", "").await;
        assert!(result.is_err(), "expected error on 403");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed to list application keys"));
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // list_all()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_app_keys_list_all_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let result = super::list_all(&cfg, 10, 0, "", "").await;
        assert!(
            result.is_ok(),
            "list_all should succeed: {:?}",
            result.err()
        );
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_list_all_invalid_sort_rejected() {
        let _lock = lock_env().await;
        let server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let result = super::list_all(&cfg, 10, 0, "", "garbage").await;
        assert!(result.is_err(), "invalid sort should be rejected");
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_list_all_error_500() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Internal"]}"#)
            .create_async()
            .await;
        let result = super::list_all(&cfg, 10, 0, "", "").await;
        assert!(result.is_err(), "expected error on 500");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed to list all application keys"));
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // get()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_app_keys_get_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let result = super::get(&cfg, "key-id").await;
        assert!(result.is_ok(), "get should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_get_error_404() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Not Found"]}"#)
            .create_async()
            .await;
        let result = super::get(&cfg, "missing-key").await;
        assert!(result.is_err(), "expected error on 404");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed to get application key"));
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // create()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_app_keys_create_success_no_scopes() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let result = super::create(&cfg, "test-key", "").await;
        assert!(
            result.is_ok(),
            "create without scopes should succeed: {:?}",
            result.err()
        );
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_create_success_with_scopes() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        // Comma-separated scopes, with surrounding whitespace that should be trimmed.
        let result = super::create(&cfg, "test-key", "events_read, metrics_read ,logs_read").await;
        assert!(
            result.is_ok(),
            "create with scopes should succeed: {:?}",
            result.err()
        );
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_create_error_400() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Invalid scope"]}"#)
            .create_async()
            .await;
        let result = super::create(&cfg, "test-key", "bogus_scope").await;
        assert!(result.is_err(), "expected error on 400");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed to create application key"));
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // update()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_app_keys_update_success_name_only() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let result = super::update(&cfg, "k1", "renamed", "").await;
        assert!(result.is_ok(), "update should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_update_success_scopes_only() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let result = super::update(&cfg, "k1", "", "events_read,metrics_read").await;
        assert!(
            result.is_ok(),
            "update with scopes should succeed: {:?}",
            result.err()
        );
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_update_error_404() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("PATCH", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Not Found"]}"#)
            .create_async()
            .await;
        let result = super::update(&cfg, "missing", "name", "").await;
        assert!(result.is_err(), "expected error on 404");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed to update application key"));
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // delete()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_app_keys_delete_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{}"#).await;
        let result = super::delete(&cfg, "k1").await;
        assert!(result.is_ok(), "delete should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_app_keys_delete_error_404() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("DELETE", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Not Found"]}"#)
            .create_async()
            .await;
        let result = super::delete(&cfg, "missing").await;
        assert!(result.is_err(), "expected error on 404");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed to delete application key"));
        cleanup_env();
    }
}
