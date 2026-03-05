use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use datadog_api_client::datadogV2::api_entity_risk_scores::{
    EntityRiskScoresAPI, ListEntityRiskScoresOptionalParams,
};
#[cfg(not(target_arch = "wasm32"))]
use datadog_api_client::datadogV2::api_security_monitoring::{
    ListFindingsOptionalParams, ListSecurityMonitoringRulesOptionalParams,
    ListSecurityMonitoringSuppressionsOptionalParams,
    SearchSecurityMonitoringSignalsOptionalParams, SecurityMonitoringAPI,
};
#[cfg(not(target_arch = "wasm32"))]
use datadog_api_client::datadogV2::model::{
    SecurityMonitoringRuleBulkExportAttributes, SecurityMonitoringRuleBulkExportData,
    SecurityMonitoringRuleBulkExportDataType, SecurityMonitoringRuleBulkExportPayload,
    SecurityMonitoringRuleSort, SecurityMonitoringSignalListRequest,
    SecurityMonitoringSignalListRequestFilter, SecurityMonitoringSignalListRequestPage,
    SecurityMonitoringSignalsSort, SecurityMonitoringSuppressionCreateRequest,
    SecurityMonitoringSuppressionSort, SecurityMonitoringSuppressionUpdateRequest,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::client;
use crate::config::Config;
use crate::formatter;
use crate::util;

#[cfg(not(target_arch = "wasm32"))]
pub async fn rules_list(cfg: &Config, sort: Option<String>) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let mut params = ListSecurityMonitoringRulesOptionalParams::default();
    if let Some(s) = sort {
        params = params.sort(parse_rule_sort(&s));
    }
    let resp = api
        .list_security_monitoring_rules(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list rules: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn rules_list(cfg: &Config, sort: Option<String>) -> Result<()> {
    let mut q: Vec<(&str, String)> = vec![];
    if let Some(s) = sort {
        q.push(("sort", s));
    }
    let data = crate::api::get(cfg, "/api/v2/security_monitoring/rules", &q).await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_rule_sort(s: &str) -> SecurityMonitoringRuleSort {
    match s {
        "name" => SecurityMonitoringRuleSort::NAME,
        "-name" => SecurityMonitoringRuleSort::NAME_DESCENDING,
        "creation_date" => SecurityMonitoringRuleSort::CREATION_DATE,
        "-creation_date" => SecurityMonitoringRuleSort::CREATION_DATE_DESCENDING,
        "update_date" => SecurityMonitoringRuleSort::UPDATE_DATE,
        "-update_date" => SecurityMonitoringRuleSort::UPDATE_DATE_DESCENDING,
        "enabled" => SecurityMonitoringRuleSort::ENABLED,
        "-enabled" => SecurityMonitoringRuleSort::ENABLED_DESCENDING,
        "type" => SecurityMonitoringRuleSort::TYPE,
        "-type" => SecurityMonitoringRuleSort::TYPE_DESCENDING,
        "highest_severity" => SecurityMonitoringRuleSort::HIGHEST_SEVERITY,
        "-highest_severity" => SecurityMonitoringRuleSort::HIGHEST_SEVERITY_DESCENDING,
        "source" => SecurityMonitoringRuleSort::SOURCE,
        "-source" => SecurityMonitoringRuleSort::SOURCE_DESCENDING,
        _ => SecurityMonitoringRuleSort::NAME,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn rules_get(cfg: &Config, rule_id: &str) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let resp = api
        .get_security_monitoring_rule(rule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn rules_get(cfg: &Config, rule_id: &str) -> Result<()> {
    let data = crate::api::get(
        cfg,
        &format!("/api/v2/security_monitoring/rules/{rule_id}"),
        &[],
    )
    .await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn signals_search(
    cfg: &Config,
    query: String,
    from: String,
    to: String,
    limit: i32,
) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };

    let from_dt =
        chrono::DateTime::from_timestamp_millis(util::parse_time_to_unix_millis(&from)?).unwrap();
    let to_dt =
        chrono::DateTime::from_timestamp_millis(util::parse_time_to_unix_millis(&to)?).unwrap();

    let body = SecurityMonitoringSignalListRequest::new()
        .filter(
            SecurityMonitoringSignalListRequestFilter::new()
                .query(query)
                .from(from_dt)
                .to(to_dt),
        )
        .page(SecurityMonitoringSignalListRequestPage::new().limit(limit))
        .sort(SecurityMonitoringSignalsSort::TIMESTAMP_DESCENDING);

    let params = SearchSecurityMonitoringSignalsOptionalParams::default().body(body);
    let resp = api
        .search_security_monitoring_signals(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to search signals: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn signals_search(
    cfg: &Config,
    query: String,
    from: String,
    to: String,
    limit: i32,
) -> Result<()> {
    let from_ms = util::parse_time_to_unix_millis(&from)?;
    let to_ms = util::parse_time_to_unix_millis(&to)?;
    let body = serde_json::json!({
        "filter": {
            "query": query,
            "from": from_ms,
            "to": to_ms
        },
        "page": {
            "limit": limit
        },
        "sort": "timestamp"
    });
    let data = crate::api::post(cfg, "/api/v2/security_monitoring/signals/search", &body).await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn findings_search(cfg: &Config, query: Option<String>, limit: i64) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let mut params = ListFindingsOptionalParams::default().page_limit(limit);
    if let Some(q) = query {
        params = params.filter_tags(q);
    }
    let resp = api
        .list_findings(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to search findings: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn findings_search(cfg: &Config, query: Option<String>, limit: i64) -> Result<()> {
    let mut q: Vec<(&str, String)> = vec![("page[limit]", limit.to_string())];
    if let Some(tags) = &query {
        q.push(("filter[tags]", tags.clone()));
    }
    let data = crate::api::get(cfg, "/api/v2/posture_management/findings", &q).await?;
    crate::formatter::output(cfg, &data)
}

// ---- Bulk Export ----

#[cfg(not(target_arch = "wasm32"))]
pub async fn rules_bulk_export(cfg: &Config, rule_ids: Vec<String>) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let attrs = SecurityMonitoringRuleBulkExportAttributes::new(rule_ids);
    let data = SecurityMonitoringRuleBulkExportData::new(
        attrs,
        SecurityMonitoringRuleBulkExportDataType::SECURITY_MONITORING_RULES_BULK_EXPORT,
    );
    let body = SecurityMonitoringRuleBulkExportPayload::new(data);
    let resp = api
        .bulk_export_security_monitoring_rules(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to bulk export security rules: {e:?}"))?;
    // resp is Vec<u8> (ZIP data), output as raw bytes to stdout
    let output = String::from_utf8_lossy(&resp);
    println!("{output}");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn rules_bulk_export(cfg: &Config, rule_ids: Vec<String>) -> Result<()> {
    let body = serde_json::json!({
        "data": {
            "attributes": {
                "rule_ids": rule_ids
            },
            "type": "security_monitoring_rules_bulk_export"
        }
    });
    let data =
        crate::api::post(cfg, "/api/v2/security_monitoring/rules/_bulk_export", &body).await?;
    println!("{data}");
    Ok(())
}

// ---- Content Packs ----

#[cfg(not(target_arch = "wasm32"))]
pub async fn content_packs_list(cfg: &Config) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let resp = api
        .get_content_packs_states()
        .await
        .map_err(|e| anyhow::anyhow!("failed to list content packs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn content_packs_list(cfg: &Config) -> Result<()> {
    let data = crate::api::get(cfg, "/api/v2/security_monitoring/content_packs", &[]).await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn content_packs_activate(cfg: &Config, pack_id: &str) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    api.activate_content_pack(pack_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to activate content pack: {e:?}"))?;
    println!("Content pack '{pack_id}' activated successfully.");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn content_packs_activate(cfg: &Config, pack_id: &str) -> Result<()> {
    let body = serde_json::json!({});
    crate::api::post(
        cfg,
        &format!("/api/v2/security_monitoring/content_packs/{pack_id}/activate"),
        &body,
    )
    .await?;
    println!("Content pack '{pack_id}' activated successfully.");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn content_packs_deactivate(cfg: &Config, pack_id: &str) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    api.deactivate_content_pack(pack_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to deactivate content pack: {e:?}"))?;
    println!("Content pack '{pack_id}' deactivated successfully.");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn content_packs_deactivate(cfg: &Config, pack_id: &str) -> Result<()> {
    let body = serde_json::json!({});
    crate::api::post(
        cfg,
        &format!("/api/v2/security_monitoring/content_packs/{pack_id}/deactivate"),
        &body,
    )
    .await?;
    println!("Content pack '{pack_id}' deactivated successfully.");
    Ok(())
}

// ---- Risk Scores ----

#[cfg(not(target_arch = "wasm32"))]
pub async fn risk_scores_list(cfg: &Config, query: Option<String>) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => EntityRiskScoresAPI::with_client_and_config(dd_cfg, c),
        None => EntityRiskScoresAPI::with_config(dd_cfg),
    };
    let mut params = ListEntityRiskScoresOptionalParams::default();
    if let Some(q) = query {
        params = params.filter_query(q);
    }
    let resp = api
        .list_entity_risk_scores(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list entity risk scores: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn risk_scores_list(cfg: &Config, query: Option<String>) -> Result<()> {
    let mut q: Vec<(&str, String)> = vec![];
    if let Some(filter) = &query {
        q.push(("filter[query]", filter.clone()));
    }
    let data = crate::api::get(cfg, "/api/v2/entity_risk_scores", &q).await?;
    crate::formatter::output(cfg, &data)
}

// ---- Suppressions ----

#[cfg(not(target_arch = "wasm32"))]
fn parse_suppression_sort(s: &str) -> SecurityMonitoringSuppressionSort {
    match s {
        "name" => SecurityMonitoringSuppressionSort::NAME,
        "-name" => SecurityMonitoringSuppressionSort::NAME_DESCENDING,
        "start_date" => SecurityMonitoringSuppressionSort::START_DATE,
        "-start_date" => SecurityMonitoringSuppressionSort::START_DATE_DESCENDING,
        "expiration_date" => SecurityMonitoringSuppressionSort::EXPIRATION_DATE,
        "-expiration_date" => SecurityMonitoringSuppressionSort::EXPIRATION_DATE_DESCENDING,
        "update_date" => SecurityMonitoringSuppressionSort::UPDATE_DATE,
        "-update_date" => SecurityMonitoringSuppressionSort::UPDATE_DATE_DESCENDING,
        "-creation_date" => SecurityMonitoringSuppressionSort::CREATION_DATE_DESCENDING,
        "enabled" => SecurityMonitoringSuppressionSort::ENABLED,
        "-enabled" => SecurityMonitoringSuppressionSort::ENABLED_DESCENDING,
        _ => SecurityMonitoringSuppressionSort::NAME,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn suppressions_list(cfg: &Config, sort: Option<String>) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let mut params = ListSecurityMonitoringSuppressionsOptionalParams::default();
    if let Some(s) = sort {
        params = params.sort(parse_suppression_sort(&s));
    }
    let resp = api
        .list_security_monitoring_suppressions(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list suppressions: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn suppressions_list(cfg: &Config, sort: Option<String>) -> Result<()> {
    let mut q: Vec<(&str, String)> = vec![];
    if let Some(s) = &sort {
        q.push(("sort", s.clone()));
    }
    let data = crate::api::get(
        cfg,
        "/api/v2/security_monitoring/configuration/suppressions",
        &q,
    )
    .await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn suppressions_get(cfg: &Config, suppression_id: &str) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let resp = api
        .get_security_monitoring_suppression(suppression_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get suppression: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn suppressions_get(cfg: &Config, suppression_id: &str) -> Result<()> {
    let data = crate::api::get(
        cfg,
        &format!("/api/v2/security_monitoring/configuration/suppressions/{suppression_id}"),
        &[],
    )
    .await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn suppressions_create(cfg: &Config, file: &str) -> Result<()> {
    let body: SecurityMonitoringSuppressionCreateRequest = util::read_json_file(file)?;
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let resp = api
        .create_security_monitoring_suppression(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create suppression: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn suppressions_create(cfg: &Config, file: &str) -> Result<()> {
    let body: serde_json::Value = util::read_json_file(file)?;
    let data = crate::api::post(
        cfg,
        "/api/v2/security_monitoring/configuration/suppressions",
        &body,
    )
    .await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn suppressions_update(cfg: &Config, suppression_id: &str, file: &str) -> Result<()> {
    let body: SecurityMonitoringSuppressionUpdateRequest = util::read_json_file(file)?;
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    let resp = api
        .update_security_monitoring_suppression(suppression_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update suppression: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(target_arch = "wasm32")]
pub async fn suppressions_update(cfg: &Config, suppression_id: &str, file: &str) -> Result<()> {
    let body: serde_json::Value = util::read_json_file(file)?;
    let data = crate::api::patch(
        cfg,
        &format!("/api/v2/security_monitoring/configuration/suppressions/{suppression_id}"),
        &body,
    )
    .await?;
    crate::formatter::output(cfg, &data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn suppressions_delete(cfg: &Config, suppression_id: &str) -> Result<()> {
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    api.delete_security_monitoring_suppression(suppression_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete suppression: {e:?}"))?;
    println!("Suppression '{suppression_id}' deleted.");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn suppressions_delete(cfg: &Config, suppression_id: &str) -> Result<()> {
    crate::api::delete(
        cfg,
        &format!("/api/v2/security_monitoring/configuration/suppressions/{suppression_id}"),
    )
    .await?;
    println!("Suppression '{suppression_id}' deleted.");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn suppressions_validate(cfg: &Config, file: &str) -> Result<()> {
    let body: SecurityMonitoringSuppressionCreateRequest = util::read_json_file(file)?;
    let dd_cfg = client::make_dd_config(cfg);
    let api = match client::make_bearer_client(cfg) {
        Some(c) => SecurityMonitoringAPI::with_client_and_config(dd_cfg, c),
        None => SecurityMonitoringAPI::with_config(dd_cfg),
    };
    api.validate_security_monitoring_suppression(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to validate suppression: {e:?}"))?;
    println!("Suppression is valid.");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn suppressions_validate(cfg: &Config, file: &str) -> Result<()> {
    let body: serde_json::Value = util::read_json_file(file)?;
    crate::api::post(
        cfg,
        "/api/v2/security_monitoring/configuration/suppressions/validation",
        &body,
    )
    .await?;
    println!("Suppression is valid.");
    Ok(())
}
