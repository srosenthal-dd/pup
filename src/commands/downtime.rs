use anyhow::Result;
use datadog_api_client::datadogV2::api_downtimes::{
    DowntimesAPI, GetDowntimeOptionalParams, ListDowntimesOptionalParams,
};

use crate::config::Config;
use crate::formatter;

pub async fn list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(DowntimesAPI, cfg);
    let resp = api
        .list_downtimes(ListDowntimesOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list downtimes: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(DowntimesAPI, cfg);
    let resp = api
        .get_downtime(id.to_string(), GetDowntimeOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get downtime: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn create(cfg: &Config, file: &str) -> Result<()> {
    let body: datadog_api_client::datadogV2::model::DowntimeCreateRequest =
        crate::util::read_json_file(file)?;
    let api = crate::make_api!(DowntimesAPI, cfg);
    let resp = api
        .create_downtime(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create downtime: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn cancel(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(DowntimesAPI, cfg);
    api.cancel_downtime(id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to cancel downtime: {e:?}"))?;
    println!("Downtime {id} cancelled.");
    Ok(())
}
