use anyhow::Result;
use datadog_api_client::datadogV2::api_scorecards::{
    GetScorecardCampaignOptionalParams, ListScorecardCampaignsOptionalParams,
    ListScorecardOutcomesOptionalParams, ListScorecardRulesOptionalParams,
    ListScorecardsOptionalParams, ScorecardsAPI,
};
use datadog_api_client::datadogV2::model::{CreateCampaignRequest, UpdateCampaignRequest};

use crate::config::Config;
use crate::formatter;
use crate::util;

fn make_api(cfg: &Config) -> ScorecardsAPI {
    crate::make_api!(ScorecardsAPI, cfg)
}

pub async fn rules_list(cfg: &Config) -> Result<()> {
    let api = make_api(cfg);
    let resp = api
        .list_scorecard_rules(ListScorecardRulesOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list scorecard rules: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn outcomes_list(cfg: &Config) -> Result<()> {
    let api = make_api(cfg);
    let resp = api
        .list_scorecard_outcomes(ListScorecardOutcomesOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list scorecard outcomes: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn rules_create(cfg: &Config, file: &str) -> Result<()> {
    let body = util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .create_scorecard_rule(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create scorecard rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn rules_update(cfg: &Config, rule_id: &str, file: &str) -> Result<()> {
    let body = util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .update_scorecard_rule(rule_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update scorecard rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn rules_delete(cfg: &Config, rule_id: &str) -> Result<()> {
    let api = make_api(cfg);
    api.delete_scorecard_rule(rule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete scorecard rule: {e:?}"))?;
    println!("Scorecard rule '{rule_id}' deleted successfully.");
    Ok(())
}

pub async fn outcomes_batch_create(cfg: &Config, file: &str) -> Result<()> {
    let body = util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .create_scorecard_outcomes_batch(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create scorecard outcomes batch: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn list_scorecards(cfg: &Config) -> Result<()> {
    let api = make_api(cfg);
    let resp = api
        .list_scorecards(ListScorecardsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list scorecards: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn campaigns_list(cfg: &Config) -> Result<()> {
    let api = make_api(cfg);
    let resp = api
        .list_scorecard_campaigns(ListScorecardCampaignsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list scorecard campaigns: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn campaigns_get(cfg: &Config, campaign_id: &str) -> Result<()> {
    let api = make_api(cfg);
    let resp = api
        .get_scorecard_campaign(
            campaign_id.to_string(),
            GetScorecardCampaignOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get scorecard campaign '{campaign_id}': {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn campaigns_create(cfg: &Config, file: &str) -> Result<()> {
    let body: CreateCampaignRequest = util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .create_scorecard_campaign(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create scorecard campaign: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn campaigns_update(cfg: &Config, campaign_id: &str, file: &str) -> Result<()> {
    let body: UpdateCampaignRequest = util::read_json_file(file)?;
    let api = make_api(cfg);
    let resp = api
        .update_scorecard_campaign(campaign_id.to_string(), body)
        .await
        .map_err(|e| {
            anyhow::anyhow!("failed to update scorecard campaign '{campaign_id}': {e:?}")
        })?;
    formatter::output(cfg, &resp)
}

pub async fn campaigns_delete(cfg: &Config, campaign_id: &str) -> Result<()> {
    let api = make_api(cfg);
    api.delete_scorecard_campaign(campaign_id.to_string())
        .await
        .map_err(|e| {
            anyhow::anyhow!("failed to delete scorecard campaign '{campaign_id}': {e:?}")
        })?;
    println!("Scorecard campaign '{campaign_id}' deleted successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_scorecard_rules_create_missing_file() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let result = super::rules_create(&cfg, "/nonexistent/file.json").await;
        assert!(result.is_err(), "rules_create should fail for missing file");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_scorecard_outcomes_batch_create_missing_file() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let result = super::outcomes_batch_create(&cfg, "/nonexistent/file.json").await;
        assert!(
            result.is_err(),
            "outcomes_batch_create should fail for missing file"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_scorecard_rules_create_api_error() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(422)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["unprocessable entity"]}"#)
            .create_async()
            .await;
        let tmp = std::env::temp_dir().join("test_scorecard_rule.json");
        std::fs::write(&tmp, r#"{"data":{}}"#).unwrap();
        let result = super::rules_create(&cfg, tmp.to_str().unwrap()).await;
        let _ = std::fs::remove_file(&tmp);
        assert!(result.is_err(), "rules_create should fail on 422 response");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
