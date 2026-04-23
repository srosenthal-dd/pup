use anyhow::Result;
use datadog_api_client::datadogV2::api_bits_ai::{BitsAIAPI, ListInvestigationsOptionalParams};

use crate::config::Config;
use crate::formatter;
use crate::util;

fn make_api(cfg: &Config) -> BitsAIAPI {
    crate::make_api!(BitsAIAPI, cfg)
}

pub async fn list(cfg: &Config, page_limit: i64, page_offset: i64) -> Result<()> {
    let api = make_api(cfg);
    let params = ListInvestigationsOptionalParams::default()
        .page_limit(page_limit)
        .page_offset(page_offset);
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
