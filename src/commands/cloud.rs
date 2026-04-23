use anyhow::Result;
use datadog_api_client::datadogV1::api_aws_integration::{
    AWSIntegrationAPI, ListAWSAccountsOptionalParams,
};
use datadog_api_client::datadogV1::api_azure_integration::AzureIntegrationAPI;
use datadog_api_client::datadogV1::api_gcp_integration::GCPIntegrationAPI;
use datadog_api_client::datadogV2::api_oci_integration::OCIIntegrationAPI;
use datadog_api_client::datadogV2::model::{
    CreateTenancyConfigRequest, UpdateTenancyConfigRequest,
};

use crate::config::Config;
use crate::formatter;

pub async fn aws_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(AWSIntegrationAPI, cfg);
    let resp = api
        .list_aws_accounts(ListAWSAccountsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list AWS accounts: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn gcp_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(GCPIntegrationAPI, cfg);
    let resp = api
        .list_gcp_integration()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list GCP integrations: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn azure_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(AzureIntegrationAPI, cfg);
    let resp = api
        .list_azure_integration()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list Azure integrations: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---------------------------------------------------------------------------
// OCI tenancy management
// ---------------------------------------------------------------------------

fn make_oci_api(cfg: &Config) -> OCIIntegrationAPI {
    crate::make_api!(OCIIntegrationAPI, cfg)
}

pub async fn oci_tenancies_list(cfg: &Config) -> Result<()> {
    let api = make_oci_api(cfg);
    let resp = api
        .get_tenancy_configs()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list OCI tenancies: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn oci_tenancies_get(cfg: &Config, tenancy_id: &str) -> Result<()> {
    let api = make_oci_api(cfg);
    let resp = api
        .get_tenancy_config(tenancy_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get OCI tenancy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn oci_tenancies_create(cfg: &Config, file: &str) -> Result<()> {
    let api = make_oci_api(cfg);
    let body: CreateTenancyConfigRequest = crate::util::read_json_file(file)?;
    let resp = api
        .create_tenancy_config(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create OCI tenancy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn oci_tenancies_update(cfg: &Config, tenancy_id: &str, file: &str) -> Result<()> {
    let api = make_oci_api(cfg);
    let body: UpdateTenancyConfigRequest = crate::util::read_json_file(file)?;
    let resp = api
        .update_tenancy_config(tenancy_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update OCI tenancy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn oci_tenancies_delete(cfg: &Config, tenancy_id: &str) -> Result<()> {
    let api = make_oci_api(cfg);
    api.delete_tenancy_config(tenancy_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete OCI tenancy: {e:?}"))?;
    println!("OCI tenancy '{tenancy_id}' deleted.");
    Ok(())
}

pub async fn oci_products_list(cfg: &Config, product_keys: &str) -> Result<()> {
    let api = make_oci_api(cfg);
    let resp = api
        .list_tenancy_products(product_keys.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list OCI products: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_cloud_aws_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::aws_list(&cfg).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_cloud_gcp_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::gcp_list(&cfg).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_cloud_azure_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::azure_list(&cfg).await;
        cleanup_env();
    }
}
