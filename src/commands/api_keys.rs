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

    #[tokio::test]
    async fn test_api_keys_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::list(&cfg).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_api_keys_get() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let _ = super::get(&cfg, "k1").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_api_keys_delete() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{}"#).await;
        let _ = super::delete(&cfg, "k1").await;
        cleanup_env();
    }
}
