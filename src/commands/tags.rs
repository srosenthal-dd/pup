use anyhow::Result;
use datadog_api_client::datadogV1::api_tags::{
    CreateHostTagsOptionalParams, DeleteHostTagsOptionalParams, GetHostTagsOptionalParams,
    ListHostTagsOptionalParams, TagsAPI, UpdateHostTagsOptionalParams,
};
use datadog_api_client::datadogV1::model::HostTags;

use crate::config::Config;
use crate::formatter;

pub async fn list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(TagsAPI, cfg);
    let resp = api
        .list_host_tags(ListHostTagsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list tags: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, hostname: &str) -> Result<()> {
    let api = crate::make_api!(TagsAPI, cfg);
    let resp = api
        .get_host_tags(hostname.to_string(), GetHostTagsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get tags: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn add(cfg: &Config, hostname: &str, tags: Vec<String>) -> Result<()> {
    let api = crate::make_api!(TagsAPI, cfg);
    let body = HostTags::new().tags(tags);
    let resp = api
        .create_host_tags(
            hostname.to_string(),
            body,
            CreateHostTagsOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to add tags: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn update(cfg: &Config, hostname: &str, tags: Vec<String>) -> Result<()> {
    let api = crate::make_api!(TagsAPI, cfg);
    let body = HostTags::new().tags(tags);
    let resp = api
        .update_host_tags(
            hostname.to_string(),
            body,
            UpdateHostTagsOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to update tags: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn delete(cfg: &Config, hostname: &str) -> Result<()> {
    let api = crate::make_api!(TagsAPI, cfg);
    api.delete_host_tags(
        hostname.to_string(),
        DeleteHostTagsOptionalParams::default(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("failed to delete tags: {e:?}"))?;
    println!("Successfully deleted all tags from host {hostname}");
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_tags_list() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"tags": {}}"#).await;

        let result = super::list(&cfg).await;
        assert!(result.is_ok(), "tags list failed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_tags_get() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "GET",
            r#"{"host": "myhost", "tags": ["env:prod", "service:web"]}"#,
        )
        .await;

        let result = super::get(&cfg, "myhost").await;
        assert!(result.is_ok(), "tags get failed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_tags_add() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "POST",
            r#"{"host": "myhost", "tags": ["env:prod"]}"#,
        )
        .await;

        let result = super::add(&cfg, "myhost", vec!["env:prod".into()]).await;
        assert!(result.is_ok(), "tags add failed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_tags_update() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "PUT",
            r#"{"host": "myhost", "tags": ["env:staging"]}"#,
        )
        .await;

        let result = super::update(&cfg, "myhost", vec!["env:staging".into()]).await;
        assert!(result.is_ok(), "tags update failed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_tags_delete() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        // Delete returns 204 No Content
        let _mock = server
            .mock("DELETE", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(204)
            .create_async()
            .await;

        let result = super::delete(&cfg, "myhost").await;
        assert!(result.is_ok(), "tags delete failed: {:?}", result.err());
        cleanup_env();
    }
}
