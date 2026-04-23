use anyhow::Result;
use datadog_api_client::datadogV2::api_sensitive_data_scanner::SensitiveDataScannerAPI;

use crate::config::Config;
use crate::formatter;

pub async fn scanner_rules_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(SensitiveDataScannerAPI, cfg);
    let resp = api
        .list_scanning_groups()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list scanner rules: {e:?}"))?;
    formatter::output(cfg, &resp)
}
