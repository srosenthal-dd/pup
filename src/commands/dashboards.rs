use anyhow::Result;
use datadog_api_client::datadogV1::api_dashboards::{DashboardsAPI, ListDashboardsOptionalParams};
use datadog_api_client::datadogV1::model::Dashboard;
use url::Url;

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(DashboardsAPI, cfg);
    let resp = api
        .list_dashboards(ListDashboardsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list dashboards: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(DashboardsAPI, cfg);
    let resp = api
        .get_dashboard(id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get dashboard: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn create(cfg: &Config, file: &str) -> Result<()> {
    let body: Dashboard = util::read_json_file(file)?;
    let api = crate::make_api!(DashboardsAPI, cfg);
    let resp = api
        .create_dashboard(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create dashboard: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn update(cfg: &Config, id: &str, file: &str) -> Result<()> {
    let body: Dashboard = util::read_json_file(file)?;
    let api = crate::make_api!(DashboardsAPI, cfg);
    let resp = api
        .update_dashboard(id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update dashboard: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn delete(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(DashboardsAPI, cfg);
    let resp = api
        .delete_dashboard(id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete dashboard: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn url(cfg: &Config, id: &str, from: &str, to: &str, live: bool) -> Result<()> {
    let api = crate::make_api!(DashboardsAPI, cfg);
    let dashboard = api
        .get_dashboard(id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get dashboard: {e:?}"))?;
    let base_url = dashboard
        .url
        .ok_or_else(|| anyhow::anyhow!("dashboard response did not include url"))?;
    println!("{}", dashboard_url_with_time(&base_url, from, to, live)?);
    Ok(())
}

fn dashboard_url_with_time(base_url: &str, from: &str, to: &str, live: bool) -> Result<String> {
    let mut url = Url::parse(base_url).map_err(|e| {
        anyhow::anyhow!("dashboard response included invalid url {base_url:?}: {e}")
    })?;
    let mut query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| key != "from_ts" && key != "to_ts" && key != "live")
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    query_pairs.push(("from_ts".to_string(), from.to_string()));
    query_pairs.push(("to_ts".to_string(), to.to_string()));
    query_pairs.push(("live".to_string(), live.to_string()));
    url.query_pairs_mut().clear().extend_pairs(query_pairs);
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_dashboards_list() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"dashboards": []}"#).await;

        let result = super::list(&cfg).await;
        assert!(result.is_ok(), "dashboards list failed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_dashboards_get() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "GET",
            r#"{"id": "abc-123", "title": "Test Dashboard", "layout_type": "ordered", "widgets": []}"#,
        )
        .await;

        let result = super::get(&cfg, "abc-123").await;
        assert!(result.is_ok(), "dashboards get failed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_dashboards_delete() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "DELETE",
            r#"{"deleted_dashboard_id": "abc-123"}"#,
        )
        .await;

        let result = super::delete(&cfg, "abc-123").await;
        assert!(
            result.is_ok(),
            "dashboards delete failed: {:?}",
            result.err()
        );
        cleanup_env();
    }

    #[test]
    fn test_dashboard_url_with_time_adds_live_window() {
        let url = super::dashboard_url_with_time(
            "https://app.datadoghq.com/dashboard/abc-123/test-dashboard?tpl_var_env=prod",
            "now-1w",
            "now",
            true,
        )
        .expect("dashboard URL should be valid");

        assert_eq!(
            url,
            "https://app.datadoghq.com/dashboard/abc-123/test-dashboard?tpl_var_env=prod&from_ts=now-1w&to_ts=now&live=true"
        );
    }

    #[test]
    fn test_dashboard_url_with_time_replaces_existing_time_params() {
        let url = super::dashboard_url_with_time(
            "https://app.datadoghq.com/dashboard/abc-123/test-dashboard?from_ts=old&to_ts=old&live=false",
            "now-1w",
            "now",
            true,
        )
        .expect("dashboard URL should be valid");

        assert_eq!(
            url,
            "https://app.datadoghq.com/dashboard/abc-123/test-dashboard?from_ts=now-1w&to_ts=now&live=true"
        );
    }
}
