use anyhow::Result;
use datadog_api_client::datadogV2::api_high_availability_multi_region::HighAvailabilityMultiRegionAPI;
use datadog_api_client::datadogV2::model::HamrOrgConnectionRequest;

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn connections_get(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(HighAvailabilityMultiRegionAPI, cfg);
    let resp = api
        .get_hamr_org_connection()
        .await
        .map_err(|e| anyhow::anyhow!("failed to get HAMR connection: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn connections_create(cfg: &Config, file: &str) -> Result<()> {
    let body: HamrOrgConnectionRequest = util::read_json_file(file)?;
    let api = crate::make_api!(HighAvailabilityMultiRegionAPI, cfg);
    let resp = api
        .create_hamr_org_connection(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create HAMR connection: {e:?}"))?;
    formatter::output(cfg, &resp)
}
