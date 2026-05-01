use anyhow::Result;
use datadog_api_client::datadogV2::api_processes::{ListProcessesOptionalParams, ProcessesAPI};

use crate::config::Config;
use crate::formatter;

pub async fn list(
    cfg: &Config,
    search: Option<String>,
    tags: Option<String>,
    page_limit: Option<i32>,
) -> Result<()> {
    let api = crate::make_api!(ProcessesAPI, cfg);
    let mut params = ListProcessesOptionalParams::default();
    if let Some(s) = search {
        params = params.search(s);
    }
    if let Some(t) = tags {
        params = params.tags(t);
    }
    if let Some(pl) = page_limit {
        params = params.page_limit(pl);
    }
    let resp = api
        .list_processes(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list processes: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_processes_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "GET",
            r#"{"data":[],"meta":{"page":{"after":""}}}"#,
        )
        .await;
        let result = super::list(&cfg, None, None, None).await;
        assert!(result.is_ok(), "processes list failed: {:?}", result.err());
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_processes_list_with_search() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "GET",
            r#"{"data":[],"meta":{"page":{"after":""}}}"#,
        )
        .await;
        let result = super::list(&cfg, Some("nginx".into()), None, Some(10)).await;
        assert!(
            result.is_ok(),
            "processes list with search failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
