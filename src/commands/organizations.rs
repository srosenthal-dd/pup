use anyhow::Result;
use datadog_api_client::datadogV1::api_organizations::OrganizationsAPI;

use crate::config::Config;
use crate::formatter;

pub async fn list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(OrganizationsAPI, cfg);
    let resp = api
        .list_orgs()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list orgs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(OrganizationsAPI, cfg);
    let resp = api
        .get_org("current".to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get org: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_organizations_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"orgs": []}"#).await;
        let _ = super::list(&cfg).await;
        cleanup_env();
    }
}
