use anyhow::Result;
use datadog_api_client::datadogV2::api_fleet_automation::{
    FleetAutomationAPI, GetFleetDeploymentOptionalParams, ListFleetAgentTracersOptionalParams,
    ListFleetAgentsOptionalParams, ListFleetClustersOptionalParams,
    ListFleetDeploymentsOptionalParams, ListFleetTracersOptionalParams,
};

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn agents_list(
    cfg: &Config,
    page_size: Option<i64>,
    filter: Option<String>,
) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let mut params = ListFleetAgentsOptionalParams::default();
    if let Some(ps) = page_size {
        params = params.page_size(ps);
    }
    if let Some(f) = filter {
        params = params.filter(f);
    }
    let resp = api
        .list_fleet_agents(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list fleet agents: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agents_get(cfg: &Config, agent_key: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let resp = api
        .get_fleet_agent_info(agent_key.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get fleet agent: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agents_versions(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let resp = api
        .list_fleet_agent_versions()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list agent versions: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn deployments_list(cfg: &Config, page_size: Option<i64>) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let mut params = ListFleetDeploymentsOptionalParams::default();
    if let Some(ps) = page_size {
        params = params.page_size(ps);
    }
    let resp = api
        .list_fleet_deployments(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list deployments: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn deployments_get(cfg: &Config, deployment_id: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let resp = api
        .get_fleet_deployment(
            deployment_id.to_string(),
            GetFleetDeploymentOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get deployment: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let resp = api
        .list_fleet_schedules()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list schedules: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_get(cfg: &Config, schedule_id: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let resp = api
        .get_fleet_schedule(schedule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get schedule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_update(cfg: &Config, schedule_id: &str, file: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .update_fleet_schedule(schedule_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update schedule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_delete(cfg: &Config, schedule_id: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    api.delete_fleet_schedule(schedule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete schedule: {e:?}"))?;
    println!("Schedule '{schedule_id}' deleted successfully.");
    Ok(())
}

pub async fn deployments_cancel(cfg: &Config, deployment_id: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    api.cancel_fleet_deployment(deployment_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to cancel deployment: {e:?}"))?;
    println!("Fleet deployment {deployment_id} cancelled.");
    Ok(())
}

pub async fn deployments_configure(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_fleet_deployment_configure(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to configure deployment: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn deployments_upgrade(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_fleet_deployment_upgrade(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to upgrade deployment: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_create(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_fleet_schedule(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create schedule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_trigger(cfg: &Config, schedule_id: &str) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    api.trigger_fleet_schedule(schedule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to trigger schedule: {e:?}"))?;
    println!("Schedule {schedule_id} triggered.");
    Ok(())
}

pub async fn tracers_list(
    cfg: &Config,
    filter: Option<String>,
    page_size: Option<i64>,
    page_number: Option<i64>,
    sort_attribute: Option<String>,
    sort_descending: bool,
) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let mut params = ListFleetTracersOptionalParams::default();
    if let Some(f) = filter {
        params = params.filter(f);
    }
    if let Some(ps) = page_size {
        params = params.page_size(ps);
    }
    if let Some(pn) = page_number {
        params = params.page_number(pn);
    }
    if let Some(sa) = sort_attribute {
        params = params.sort_attribute(sa);
    }
    if sort_descending {
        params = params.sort_descending(true);
    }
    let resp = api
        .list_fleet_tracers(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list fleet tracers: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn agents_tracers_list(
    cfg: &Config,
    agent_key: String,
    page_size: Option<i64>,
    page_number: Option<i64>,
    sort_attribute: Option<String>,
    sort_descending: bool,
) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let mut params = ListFleetAgentTracersOptionalParams::default();
    if let Some(ps) = page_size {
        params = params.page_size(ps);
    }
    if let Some(pn) = page_number {
        params = params.page_number(pn);
    }
    if let Some(sa) = sort_attribute {
        params = params.sort_attribute(sa);
    }
    if sort_descending {
        params = params.sort_descending(true);
    }
    let resp = api
        .list_fleet_agent_tracers(agent_key, params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list agent tracers: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn clusters_list(
    cfg: &Config,
    filter: Option<String>,
    page_size: Option<i64>,
    page_number: Option<i64>,
    sort_attribute: Option<String>,
    sort_descending: bool,
) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let mut params = ListFleetClustersOptionalParams::default();
    if let Some(f) = filter {
        params = params.filter(f);
    }
    if let Some(ps) = page_size {
        params = params.page_size(ps);
    }
    if let Some(pn) = page_number {
        params = params.page_number(pn);
    }
    if let Some(sa) = sort_attribute {
        params = params.sort_attribute(sa);
    }
    if sort_descending {
        params = params.sort_descending(true);
    }
    let resp = api
        .list_fleet_clusters(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list fleet clusters: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn instrumented_pods_list(cfg: &Config, cluster_name: String) -> Result<()> {
    let api = crate::make_api!(FleetAutomationAPI, cfg);
    let resp = api
        .list_fleet_instrumented_pods(cluster_name)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list instrumented pods: {e:?}"))?;
    formatter::output(cfg, &resp)
}
