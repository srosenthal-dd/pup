use anyhow::Result;
use datadog_api_client::datadogV1::api_hosts::{HostsAPI, ListHostsOptionalParams};

use crate::config::Config;
use crate::formatter;

pub async fn hosts_list(
    cfg: &Config,
    filter: Option<String>,
    sort: String,
    count: i64,
) -> Result<()> {
    let api = crate::make_api!(HostsAPI, cfg);
    let mut params = ListHostsOptionalParams::default()
        .count(count)
        .sort_field(sort);
    if let Some(f) = filter {
        params = params.filter(f);
    }
    let resp = api
        .list_hosts(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list hosts: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn hosts_get(cfg: &Config, hostname: &str) -> Result<()> {
    // The V1 HostsAPI does not have a direct get-host method.
    // Use list_hosts with a filter to find the specific host.
    let api = crate::make_api!(HostsAPI, cfg);
    let params = ListHostsOptionalParams::default()
        .filter(hostname.to_string())
        .count(1);
    let resp = api
        .list_hosts(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get host {hostname}: {e:?}"))?;
    formatter::output(cfg, &resp)
}
