use anyhow::Result;
use datadog_api_client::datadogV2::api_scorecards::{
    GetScorecardCampaignOptionalParams, ListScorecardCampaignsOptionalParams,
    ListScorecardOutcomesOptionalParams, ListScorecardRulesOptionalParams,
    ListScorecardsOptionalParams, ScorecardsAPI,
};
use datadog_api_client::datadogV2::model::{CreateCampaignRequest, UpdateCampaignRequest};

use crate::client;
use crate::config::Config;
use crate::formatter;
use crate::util;

fn make_api(cfg: &Config) -> ScorecardsAPI {
    let dd_cfg = client::make_dd_config(cfg);
    match client::make_bearer_client(cfg) {
        Some(c) => ScorecardsAPI::with_client_and_config(dd_cfg, c),
        None => ScorecardsAPI::with_config(dd_cfg),
    }
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
