use anyhow::Result;
use datadog_api_client::datadogV2::api_bits_ai::{BitsAIAPI, ListInvestigationsOptionalParams};

use crate::config::Config;
use crate::formatter;
use crate::util;

fn make_api(cfg: &Config) -> BitsAIAPI {
    crate::make_api!(BitsAIAPI, cfg)
}

pub async fn list(cfg: &Config, page_limit: i64, page_offset: i64, monitor_id: i64) -> Result<()> {
    let api = make_api(cfg);
    let mut params = ListInvestigationsOptionalParams::default()
        .page_limit(page_limit)
        .page_offset(page_offset);
    if monitor_id != 0 {
        params = params.filter_monitor_id(monitor_id);
    }
    let resp = api
        .list_investigations(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list investigations: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, investigation_id: &str) -> Result<()> {
    let api = make_api(cfg);
    let resp = api
        .get_investigation(investigation_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get investigation: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn trigger(cfg: &Config, file: &str) -> Result<()> {
    let body: datadog_api_client::datadogV2::model::TriggerInvestigationRequest =
        util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .trigger_investigation(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to trigger investigation: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_investigations_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::list(&cfg, 10, 0, 0).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_investigations_get() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let _ = super::get(&cfg, "inv1").await;
        cleanup_env();
    }
}
