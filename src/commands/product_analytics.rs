use anyhow::Result;
use datadog_api_client::datadogV2::api_product_analytics::ProductAnalyticsAPI;
use datadog_api_client::datadogV2::model::{
    ProductAnalyticsAnalyticsRequest, ProductAnalyticsServerSideEventItem,
};

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn events_send(cfg: &Config, file: &str) -> Result<()> {
    let body: ProductAnalyticsServerSideEventItem = util::read_json_file(file)?;
    let api = crate::make_api!(ProductAnalyticsAPI, cfg);
    let resp = api
        .submit_product_analytics_event(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to send product analytics event: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---- Query ----

fn make_api(cfg: &Config) -> ProductAnalyticsAPI {
    crate::make_api!(ProductAnalyticsAPI, cfg)
}

pub async fn query_scalar(cfg: &Config, file: &str) -> Result<()> {
    let body: ProductAnalyticsAnalyticsRequest = util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .query_product_analytics_scalar(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to query product analytics scalar: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn query_timeseries(cfg: &Config, file: &str) -> Result<()> {
    let body: ProductAnalyticsAnalyticsRequest = util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .query_product_analytics_timeseries(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to query product analytics timeseries: {e:?}"))?;
    formatter::output(cfg, &resp)
}
