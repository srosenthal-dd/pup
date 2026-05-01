use anyhow::Result;
use datadog_api_client::datadogV2::api_key_management::{
    GetAPIKeyOptionalParams, KeyManagementAPI, ListAPIKeysOptionalParams,
};

use crate::config::Config;
use crate::formatter;

pub async fn list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(KeyManagementAPI, cfg);
    let resp = api
        .list_api_keys(ListAPIKeysOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list API keys: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, key_id: &str) -> Result<()> {
    let api = crate::make_api!(KeyManagementAPI, cfg);
    let resp = api
        .get_api_key(key_id.to_string(), GetAPIKeyOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get API key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn create(cfg: &Config, name: &str) -> Result<()> {
    use datadog_api_client::datadogV2::model::{
        APIKeyCreateAttributes, APIKeyCreateData, APIKeyCreateRequest, APIKeysType,
    };
    let body = APIKeyCreateRequest::new(APIKeyCreateData::new(
        APIKeyCreateAttributes::new(name.to_string()),
        APIKeysType::API_KEYS,
    ));
    let api = crate::make_api!(KeyManagementAPI, cfg);
    let resp = api
        .create_api_key(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create API key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn delete(cfg: &Config, key_id: &str) -> Result<()> {
    let api = crate::make_api!(KeyManagementAPI, cfg);
    api.delete_api_key(key_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete API key: {e:?}"))?;
    println!("Successfully deleted API key {key_id}");
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    // -----------------------------------------------------------------------
    // list()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_api_keys_list_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let result = super::list(&cfg).await;
        assert!(result.is_ok(), "list should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_api_keys_list_error_403() {
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
        let result = super::list(&cfg).await;
        assert!(result.is_err(), "expected error on 403");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("failed to list API keys"),
            "error should mention failed listing: {err_msg}"
        );
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // get()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_api_keys_get_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let result = super::get(&cfg, "key-id-123").await;
        assert!(result.is_ok(), "get should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_api_keys_get_error_404() {
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
        let result = super::get(&cfg, "missing").await;
        assert!(result.is_err(), "expected error on 404");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("failed to get API key"),
            "error should mention failed get: {err_msg}"
        );
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // create()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_api_keys_create_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {"id":"k1","type":"api_keys"}}"#).await;
        let result = super::create(&cfg, "my-new-key").await;
        assert!(result.is_ok(), "create should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_api_keys_create_error_400() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Invalid name"]}"#)
            .create_async()
            .await;
        let result = super::create(&cfg, "").await;
        assert!(result.is_err(), "expected error on 400");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("failed to create API key"),
            "error should mention failed create: {err_msg}"
        );
        cleanup_env();
    }

    // -----------------------------------------------------------------------
    // delete()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_api_keys_delete_success() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{}"#).await;
        let result = super::delete(&cfg, "k1").await;
        assert!(result.is_ok(), "delete should succeed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_api_keys_delete_error_404() {
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
        let result = super::delete(&cfg, "nope").await;
        assert!(result.is_err(), "expected error on 404");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("failed to delete API key"),
            "error should mention failed delete: {err_msg}"
        );
        cleanup_env();
    }
}
