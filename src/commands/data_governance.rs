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

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_data_governance_scanner_rules_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::scanner_rules_list(&cfg).await;
        cleanup_env();
    }
}
