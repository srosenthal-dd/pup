use anyhow::Result;
use datadog_api_client::datadogV2::api_software_catalog::{
    ListCatalogEntityOptionalParams, ListCatalogKindOptionalParams,
    ListCatalogRelationOptionalParams, SoftwareCatalogAPI,
};
use datadog_api_client::datadogV2::model::{UpsertCatalogEntityRequest, UpsertCatalogKindRequest};

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn entities_list(
    cfg: &Config,
    filter: Option<String>,
    filter_kind: Option<String>,
    filter_owner: Option<String>,
    filter_ref: Option<String>,
) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    let mut params = ListCatalogEntityOptionalParams::default();
    if let Some(v) = &filter_kind {
        params = params.filter_kind(v.clone());
    }
    if let Some(v) = &filter_owner {
        params = params.filter_owner(v.clone());
    }
    if let Some(v) = &filter_ref {
        params = params.filter_ref(v.clone());
    }
    // Client-side filter requires paginating through all results since the
    // API's filter[name] only supports exact match.
    if let Some(pattern) = &filter {
        use futures_util::StreamExt;
        let pattern = pattern.to_lowercase();
        let mut stream = Box::pin(api.list_catalog_entity_with_pagination(params));
        let mut filtered = Vec::new();
        while let Some(item) = stream.next().await {
            let entity =
                item.map_err(|e| anyhow::anyhow!("failed to list catalog entities: {e:?}"))?;
            let matches = entity
                .attributes
                .as_ref()
                .and_then(|a| a.name.as_ref())
                .is_some_and(|n| n.to_lowercase().contains(&pattern));
            if matches {
                filtered.push(entity);
            }
        }
        formatter::output(cfg, &serde_json::json!({"data": filtered}))
    } else {
        let resp = api
            .list_catalog_entity(params)
            .await
            .map_err(|e| anyhow::anyhow!("failed to list catalog entities: {e:?}"))?;
        formatter::output(cfg, &resp)
    }
}

pub async fn entities_upsert(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    let body = util::read_json_file::<UpsertCatalogEntityRequest>(file)?;
    let resp = api
        .upsert_catalog_entity(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to upsert catalog entity: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn entities_delete(cfg: &Config, entity_id: &str) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    api.delete_catalog_entity(entity_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete catalog entity: {e:?}"))?;
    println!("Entity '{entity_id}' deleted successfully.");
    Ok(())
}

pub async fn kinds_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    let resp = api
        .list_catalog_kind(ListCatalogKindOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list catalog kinds: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn kinds_upsert(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    let body = util::read_json_file::<UpsertCatalogKindRequest>(file)?;
    let resp = api
        .upsert_catalog_kind(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to upsert catalog kind: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn kinds_delete(cfg: &Config, kind_id: &str) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    api.delete_catalog_kind(kind_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete catalog kind: {e:?}"))?;
    println!("Kind '{kind_id}' deleted successfully.");
    Ok(())
}

pub async fn relations_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    let resp = api
        .list_catalog_relation(ListCatalogRelationOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list catalog relations: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn entities_preview(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(SoftwareCatalogAPI, cfg);
    let resp = api
        .preview_catalog_entities()
        .await
        .map_err(|e| anyhow::anyhow!("failed to preview catalog entities: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_software_catalog_entities_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::entities_list(&cfg, None, None, None, None).await;
        assert!(
            result.is_ok(),
            "software catalog entities list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_software_catalog_entities_list_with_filters() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::entities_list(
            &cfg,
            Some("shop-".to_string()),
            Some("service".to_string()),
            None,
            None,
        )
        .await;
        assert!(
            result.is_ok(),
            "software catalog entities list with name and kind filter failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_software_catalog_kinds_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::kinds_list(&cfg).await;
        assert!(
            result.is_ok(),
            "software catalog kinds list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_software_catalog_relations_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[]}"#).await;
        let result = super::relations_list(&cfg).await;
        assert!(
            result.is_ok(),
            "software catalog relations list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_software_catalog_entities_list_error() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Internal Server Error"]}"#)
            .create_async()
            .await;
        let result = super::entities_list(&cfg, None, None, None, None).await;
        assert!(
            result.is_err(),
            "expected software catalog entities list to fail on 500"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
