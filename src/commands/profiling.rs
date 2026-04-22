use anyhow::Result;
use serde_json::json;

use crate::client;
use crate::config::Config;
use crate::formatter;
use crate::util;

fn parse_window(from: &str, to: &str) -> Result<(String, String)> {
    let from_ms = util::parse_time_to_unix_millis(from)
        .map_err(|e| anyhow::anyhow!("invalid --from value: {e}"))?;
    let to_ms = util::parse_time_to_unix_millis(to)
        .map_err(|e| anyhow::anyhow!("invalid --to value: {e}"))?;
    let from_iso = chrono::DateTime::from_timestamp_millis(from_ms)
        .ok_or_else(|| {
            anyhow::anyhow!("--from {from:?} resolved to {from_ms} ms which is outside the representable date range")
        })?
        .to_rfc3339();
    let to_iso = chrono::DateTime::from_timestamp_millis(to_ms)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "--to {to:?} resolved to {to_ms} ms which is outside the representable date range"
            )
        })?
        .to_rfc3339();
    Ok((from_iso, to_iso))
}

fn filter_body(query: &str, from: &str, to: &str) -> Result<serde_json::Value> {
    let (from_iso, to_iso) = parse_window(from, to)?;
    Ok(json!({
        "filter": { "from": from_iso, "to": to_iso, "query": query },
    }))
}

fn split_csv(flag: &str, value: Option<String>) -> Result<Vec<String>> {
    let Some(raw) = value else {
        return Ok(Vec::new());
    };
    let parts: Vec<String> = raw
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        anyhow::bail!("{flag} was provided but contained no non-empty values: {raw:?}");
    }
    Ok(parts)
}

#[allow(clippy::too_many_arguments)]
pub async fn aggregate(
    cfg: &Config,
    query: String,
    profile_type: String,
    from: String,
    to: String,
    limit: u32,
    aggregation_function: String,
) -> Result<()> {
    let (from_iso, to_iso) = parse_window(&from, &to)?;
    // /profiling/api/v1/aggregate expects a flat body — query/from/to are siblings, not wrapped in filter{}.
    let body = json!({
        "profileType": profile_type,
        "query": query,
        "from": from_iso,
        "to": to_iso,
        "limit": limit,
        "aggregationFunction": aggregation_function,
    });
    let resp = client::raw_post(cfg, "/profiling/api/v1/aggregate", body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to aggregate profiles: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn analysis(cfg: &Config, profile_id: &str, event_id: Option<String>) -> Result<()> {
    let path = format!("/profiling/api/v1/profiles/{profile_id}/analysis");
    let query: Vec<(&str, &str)> = match event_id.as_deref() {
        Some(eid) => vec![("eventId", eid)],
        None => vec![],
    };
    let resp = client::raw_get(cfg, &path, &query)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get profile analysis: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[allow(clippy::too_many_arguments)]
pub async fn analytics(
    cfg: &Config,
    query: String,
    from: String,
    to: String,
    group_by: Option<String>,
    compute: Option<String>,
    limit: u32,
) -> Result<()> {
    let mut body = filter_body(&query, &from, &to)?;
    body["limit"] = json!(limit);
    let groups = split_csv("--group-by", group_by)?;
    if !groups.is_empty() {
        body["groupBy"] = json!(groups);
    }
    let computes = split_csv("--compute", compute)?;
    if !computes.is_empty() {
        body["compute"] = json!(computes);
    }
    let resp = client::raw_post(cfg, "/api/unstable/profiles/analytics", body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to run profiling analytics: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn breakdown(
    cfg: &Config,
    profile_id: &str,
    query: Option<String>,
    from: Option<String>,
    to: Option<String>,
) -> Result<()> {
    let mut body = json!({ "profileIds": [profile_id] });
    match (query.as_ref(), from.as_ref(), to.as_ref()) {
        (Some(q), Some(f), Some(t)) => {
            let (from_iso, to_iso) = parse_window(f, t)?;
            body["filter"] = json!({ "from": from_iso, "to": to_iso, "query": q });
        }
        (None, None, None) => {}
        _ => {
            anyhow::bail!("--query, --from, and --to must all be provided together, or all omitted")
        }
    }
    let path = format!("/profiling/api/v1/profiles/{profile_id}/breakdown");
    let resp = client::raw_post(cfg, &path, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to compute profile breakdown: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn callgraph(
    cfg: &Config,
    query: String,
    profile_type: String,
    from: String,
    to: String,
    limit: u32,
) -> Result<()> {
    let mut body = filter_body(&query, &from, &to)?;
    body["profileType"] = json!(profile_type);
    body["limit"] = json!(limit);
    let resp = client::raw_post(cfg, "/api/unstable/profiles/callgraph", body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to load call graph: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn download(cfg: &Config, event_id: &str, output: Option<String>) -> Result<()> {
    use std::io::Write;
    // The path segment is named "profiles/<id>", but the ID is the profile event ID
    // (the `id` field on a `pup profiling list` result), not `attributes.profile-id`.
    let url_path = format!("/api/ui/profiling/profiles/{event_id}/download");
    let resp = client::raw_request(
        cfg,
        "GET",
        &url_path,
        None,
        None,
        "application/octet-stream",
        &[],
    )
    .await
    .map_err(|e| anyhow::anyhow!("failed to download profile: {e:?}"))?;

    match output {
        Some(out_path) => {
            let mut f = std::fs::File::create(&out_path)
                .map_err(|e| anyhow::anyhow!("failed to create {out_path}: {e}"))?;
            f.write_all(&resp.bytes)
                .map_err(|e| anyhow::anyhow!("failed to write {out_path}: {e}"))?;
            f.sync_all()
                .map_err(|e| anyhow::anyhow!("failed to flush {out_path} to disk: {e}"))?;
            eprintln!("Wrote {} bytes to {}", resp.bytes.len(), out_path);
        }
        None => {
            let mut out = std::io::stdout().lock();
            out.write_all(&resp.bytes)
                .map_err(|e| anyhow::anyhow!("failed to write to stdout: {e}"))?;
            out.flush()
                .map_err(|e| anyhow::anyhow!("failed to flush stdout: {e}"))?;
        }
    }
    Ok(())
}

pub async fn fields(
    cfg: &Config,
    field: String,
    query: String,
    from: String,
    to: String,
    limit: u32,
) -> Result<()> {
    let mut body = filter_body(&query, &from, &to)?;
    body["fieldName"] = json!(field);
    body["limit"] = json!(limit);
    let resp = client::raw_post(
        cfg,
        "/api/unstable/profiles/interactive-analytics/field",
        body,
    )
    .await
    .map_err(|e| anyhow::anyhow!("failed to list field values: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn info(cfg: &Config, profile_id: &str, event_id: Option<String>) -> Result<()> {
    let path = format!("/profiling/api/v1/profiles/{profile_id}/info");
    let query: Vec<(&str, &str)> = match event_id.as_deref() {
        Some(eid) => vec![("eventId", eid)],
        None => vec![],
    };
    let resp = client::raw_get(cfg, &path, &query)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get profile info: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn list(
    cfg: &Config,
    query: String,
    from: String,
    to: String,
    sort_field: Option<String>,
    sort_order: String,
    limit: u32,
) -> Result<()> {
    let mut body = filter_body(&query, &from, &to)?;
    body["limit"] = json!(limit);
    if let Some(field) = sort_field {
        body["sort"] = json!({ "field": field, "order": sort_order });
    }
    let resp = client::raw_post(cfg, "/api/unstable/profiles/list", body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list profiles: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn save_favorite(
    cfg: &Config,
    query: String,
    from: String,
    to: String,
    query_id: String,
    limit: u32,
) -> Result<()> {
    let mut body = filter_body(&query, &from, &to)?;
    body["queryId"] = json!(query_id);
    body["limit"] = json!(limit);
    let resp = client::raw_post(cfg, "/api/unstable/profiles/save-favorite", body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to save favorite: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn timeline(cfg: &Config, profile_id: &str, event_id: &str) -> Result<()> {
    // TimelineRequest DTO uses kebab-case JSON keys and requires both profile-ids and event-ids.
    let body = json!({
        "profile-ids": [profile_id],
        "event-ids": [event_id],
        "archivalContext": "",
    });
    let path = format!("/profiling/api/v1/profiles/{profile_id}/timeline");
    let resp = client::raw_post(cfg, &path, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to load profile timeline: {e:?}"))?;
    formatter::output(cfg, &resp)
}
