use anyhow::Result;
use datadog_api_client::datadogV1::api_monitors::{
    DeleteMonitorOptionalParams, GetMonitorOptionalParams, ListMonitorsOptionalParams, MonitorsAPI,
    SearchMonitorsOptionalParams,
};
use datadog_api_client::datadogV1::model::Monitor;

use crate::config::Config;
use crate::formatter::{self, Metadata};
use crate::util;

pub async fn list(
    cfg: &Config,
    name: Option<String>,
    tags: Option<String>,
    limit: i32,
) -> Result<()> {
    let api = crate::make_api!(MonitorsAPI, cfg);

    let mut params = ListMonitorsOptionalParams::default();
    if let Some(name) = name {
        params = params.name(name);
    }
    if let Some(tags) = tags {
        params = params.monitor_tags(tags);
    }

    let limit = limit.clamp(1, 1000);
    params = params.page_size(limit).page(0);

    let monitors = api
        .list_monitors(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list monitors: {:?}", e))?;

    if monitors.is_empty() {
        eprintln!("No monitors found matching the specified criteria.");
        return Ok(());
    }

    let monitors: Vec<_> = monitors.into_iter().take(limit as usize).collect();
    let meta = Metadata {
        count: Some(monitors.len()),
        truncated: false,
        command: Some("monitors list".to_string()),
        next_action: None,
    };
    formatter::format_and_print(&monitors, &cfg.output_format, cfg.agent_mode, Some(&meta))?;
    Ok(())
}

pub async fn get(cfg: &Config, monitor_id: i64) -> Result<()> {
    let api = crate::make_api!(MonitorsAPI, cfg);
    let resp = api
        .get_monitor(monitor_id, GetMonitorOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get monitor: {:?}", e))?;
    let meta = Metadata {
        count: None,
        truncated: false,
        command: Some("monitors get".to_string()),
        next_action: None,
    };
    formatter::format_and_print(&resp, &cfg.output_format, cfg.agent_mode, Some(&meta))
}

pub async fn create(cfg: &Config, file: &str) -> Result<()> {
    let body: Monitor = util::read_json_file(file)?;
    let api = crate::make_api!(MonitorsAPI, cfg);
    let resp = api
        .create_monitor(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create monitor: {:?}", e))?;
    formatter::output(cfg, &resp)
}

pub async fn update(cfg: &Config, monitor_id: i64, file: &str) -> Result<()> {
    let body: datadog_api_client::datadogV1::model::MonitorUpdateRequest =
        util::read_json_file(file)?;
    let api = crate::make_api!(MonitorsAPI, cfg);
    let resp = api
        .update_monitor(monitor_id, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update monitor: {:?}", e))?;
    formatter::output(cfg, &resp)
}

pub async fn search(cfg: &Config, query: Option<String>) -> Result<()> {
    let api = crate::make_api!(MonitorsAPI, cfg);

    let mut params = SearchMonitorsOptionalParams::default();
    if let Some(q) = query {
        params = params.query(q);
    }

    let resp = api
        .search_monitors(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to search monitors: {:?}", e))?;
    formatter::output(cfg, &resp)
}

pub async fn delete(cfg: &Config, monitor_id: i64) -> Result<()> {
    let api = crate::make_api!(MonitorsAPI, cfg);
    let resp = api
        .delete_monitor(monitor_id, DeleteMonitorOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete monitor: {:?}", e))?;
    formatter::output(cfg, &resp)
}
