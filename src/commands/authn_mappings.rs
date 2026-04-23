use anyhow::Result;
use datadog_api_client::datadogV2::api_authn_mappings::{
    AuthNMappingsAPI, ListAuthNMappingsOptionalParams,
};
use datadog_api_client::datadogV2::model::{AuthNMappingCreateRequest, AuthNMappingUpdateRequest};

use crate::client;
use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn list(cfg: &Config) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = AuthNMappingsAPI::with_config(dd_cfg);
    let resp = api
        .list_authn_mappings(ListAuthNMappingsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list AuthN mappings: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, mapping_id: &str) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = AuthNMappingsAPI::with_config(dd_cfg);
    let resp = api
        .get_authn_mapping(mapping_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get AuthN mapping: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn create(cfg: &Config, file: &str) -> Result<()> {
    let body: AuthNMappingCreateRequest = util::read_json_file(file)?;
    let dd_cfg = client::make_dd_config(cfg);
    let api = AuthNMappingsAPI::with_config(dd_cfg);
    let resp = api
        .create_authn_mapping(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create AuthN mapping: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn update(cfg: &Config, mapping_id: &str, file: &str) -> Result<()> {
    let body: AuthNMappingUpdateRequest = util::read_json_file(file)?;
    let dd_cfg = client::make_dd_config(cfg);
    let api = AuthNMappingsAPI::with_config(dd_cfg);
    let resp = api
        .update_authn_mapping(mapping_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update AuthN mapping: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn delete(cfg: &Config, mapping_id: &str) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = AuthNMappingsAPI::with_config(dd_cfg);
    api.delete_authn_mapping(mapping_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete AuthN mapping: {e:?}"))?;
    println!("AuthN mapping '{mapping_id}' deleted.");
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_authn_mappings_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::list(&cfg).await;
        assert!(
            result.is_ok(),
            "authn mappings list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_authn_mappings_list_error() {
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
        let result = super::list(&cfg).await;
        assert!(result.is_err(), "expected error for 403 response");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_authn_mappings_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "GET",
            r#"{"data":{"type":"authn_mappings","id":"abc-123","attributes":{}}}"#,
        )
        .await;
        let result = super::get(&cfg, "abc-123").await;
        assert!(
            result.is_ok(),
            "authn mappings get failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
