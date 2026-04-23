use anyhow::Result;

use crate::client;
use crate::config::Config;
use crate::formatter;

#[derive(Clone, clap::ValueEnum)]
pub enum SymdbView {
    Full,
    Names,
    #[value(name = "probe-locations")]
    ProbeLocations,
}

impl std::fmt::Display for SymdbView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymdbView::Full => write!(f, "full"),
            SymdbView::Names => write!(f, "names"),
            SymdbView::ProbeLocations => write!(f, "probe-locations"),
        }
    }
}

async fn fetch(cfg: &Config, path: &str, query: &[(&str, &str)]) -> Result<serde_json::Value> {
    client::raw_get(cfg, path, query).await
}

pub async fn search(
    cfg: &Config,
    service: &str,
    query: &str,
    version: Option<&str>,
    view: &SymdbView,
) -> Result<()> {
    let mut params: Vec<(&str, &str)> = vec![("service", service), ("query", query)];
    if let Some(v) = version {
        params.push(("version", v));
    }
    let data = fetch(cfg, "/api/unstable/symdb-api/v2/scopes/search", &params).await?;

    match view {
        SymdbView::Full => formatter::output(cfg, &data),
        SymdbView::Names => {
            let names = collect_names(&data);
            output_lines(cfg, &names)
        }
        SymdbView::ProbeLocations => {
            let locations = collect_probe_locations(cfg, service, &data).await?;
            output_lines(cfg, &locations)
        }
    }
}

/// In agent mode, emit a structured envelope; otherwise print one line per item.
fn output_lines(cfg: &Config, lines: &[String]) -> Result<()> {
    if cfg.agent_mode {
        let v = lines.to_vec();
        return formatter::output(cfg, &v);
    }
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

fn collect_names(data: &serde_json::Value) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();
    if let Some(items) = data["data"].as_array() {
        for item in items {
            if let Some(scopes) = item["attributes"]["scopes"].as_array() {
                for s in scopes {
                    if let Some(name) = s["scope"]["name"].as_str() {
                        if seen.insert(name.to_string()) {
                            names.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    names
}

#[cfg(feature = "native")]
async fn fetch_children_bulk(
    cfg: &Config,
    service: &str,
    class_names: &[&str],
) -> Vec<Result<serde_json::Value>> {
    let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(20));
    let futs: Vec<_> = class_names
        .iter()
        .map(|name| {
            let service = service.to_string();
            let name = name.to_string();
            let sem = sem.clone();
            async move {
                let _permit = sem.acquire().await;
                let query = [("service", service.as_str()), ("scope_name", name.as_str())];
                fetch(cfg, "/api/unstable/symdb-api/v2/scopes/children", &query).await
            }
        })
        .collect();
    futures::future::join_all(futs).await
}

#[cfg(not(feature = "native"))]
async fn fetch_children_bulk(
    cfg: &Config,
    service: &str,
    class_names: &[&str],
) -> Vec<Result<serde_json::Value>> {
    let mut results = Vec::with_capacity(class_names.len());
    for name in class_names {
        let query = [("service", service), ("scope_name", name)];
        results.push(fetch(cfg, "/api/unstable/symdb-api/v2/scopes/children", &query).await);
    }
    results
}

async fn collect_probe_locations(
    cfg: &Config,
    service: &str,
    data: &serde_json::Value,
) -> Result<Vec<String>> {
    let Some(items) = data["data"].as_array() else {
        return Ok(Vec::new());
    };

    // First pass: collect direct probe locations and class names that need
    // a children lookup.
    let mut seen = std::collections::HashSet::new();
    let mut lines: Vec<String> = Vec::new();
    let mut class_names: Vec<&str> = Vec::new();

    for item in items {
        let Some(scopes) = item["attributes"]["scopes"].as_array() else {
            continue;
        };
        for s in scopes {
            let scope_type = s["scope"]["scope_type"].as_str().unwrap_or("");
            let pl = &s["probe_location"];

            if !pl.is_null() {
                if let (Some(t), Some(m)) = (pl["type_name"].as_str(), pl["method_name"].as_str()) {
                    let loc = format!("{t}:{m}");
                    if seen.insert(loc.clone()) {
                        lines.push(loc);
                    }
                }
            } else if scope_type == "CLASS" {
                if let Some(name) = s["scope"]["name"].as_str() {
                    class_names.push(name);
                }
            }
        }
    }

    // Fetch children for all classes.
    let results = fetch_children_bulk(cfg, service, &class_names).await;

    for result in results {
        let Ok(children) = result else { continue };
        let Some(child_items) = children["data"].as_array() else {
            continue;
        };
        for child_item in child_items {
            let Some(child_scopes) = child_item["attributes"]["scopes"].as_array() else {
                continue;
            };
            for cs in child_scopes {
                let cpl = &cs["probe_location"];
                if cpl.is_null() {
                    continue;
                }
                if let (Some(t), Some(m)) = (cpl["type_name"].as_str(), cpl["method_name"].as_str())
                {
                    let loc = format!("{t}:{m}");
                    if seen.insert(loc.clone()) {
                        lines.push(loc);
                    }
                }
            }
        }
    }

    Ok(lines)
}

/// Fetch the language for a service from symdb metadata.
/// Returns the language string (e.g. "java", "python") or an error if not found.
pub async fn service_language(cfg: &Config, service: &str) -> Result<String> {
    let params = [("service", service)];
    let data = fetch(cfg, "/api/unstable/symdb-api/v2/services/metadata", &params).await?;
    data["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|entry| entry["attributes"]["language"].as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("could not detect language for service {service}"))
}

#[cfg(test)]
mod tests {
    use crate::test_support::*;

    use super::*;

    #[test]
    fn test_collect_names_empty() {
        let data = serde_json::json!({"data": []});
        assert!(collect_names(&data).is_empty());
    }

    #[test]
    fn test_collect_names_extracts_scope_names() {
        let data = serde_json::json!({
            "data": [{
                "attributes": {
                    "scopes": [
                        {"scope": {"name": "com.example.Foo"}},
                        {"scope": {"name": "com.example.Bar"}}
                    ]
                }
            }]
        });
        let names = collect_names(&data);
        assert_eq!(names, vec!["com.example.Foo", "com.example.Bar"]);
    }

    #[test]
    fn test_collect_names_deduplicates_across_versions() {
        let data = serde_json::json!({
            "data": [
                {
                    "attributes": {
                        "scopes": [
                            {"scope": {"name": "com.example.Foo"}},
                            {"scope": {"name": "com.example.Bar"}}
                        ]
                    }
                },
                {
                    "attributes": {
                        "scopes": [
                            {"scope": {"name": "com.example.Bar"}},
                            {"scope": {"name": "com.example.Baz"}}
                        ]
                    }
                }
            ]
        });
        let names = collect_names(&data);
        assert_eq!(
            names,
            vec!["com.example.Foo", "com.example.Bar", "com.example.Baz"]
        );
    }

    #[test]
    fn test_collect_names_missing_data_key() {
        let data = serde_json::json!({"other": "stuff"});
        assert!(collect_names(&data).is_empty());
    }

    #[test]
    fn test_symdb_view_display() {
        assert_eq!(SymdbView::Full.to_string(), "full");
        assert_eq!(SymdbView::Names.to_string(), "names");
        assert_eq!(SymdbView::ProbeLocations.to_string(), "probe-locations");
    }

    #[tokio::test]
    async fn test_symdb_search() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(
            &mut s,
            r#"{"data": [{"attributes": {"scopes": [{"scope": {"name": "MyClass"}}]}}]}"#,
        )
        .await;
        let _ = super::search(&cfg, "my-service", "MyClass", None, &super::SymdbView::Full).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_symdb_search_names_view() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(
            &mut s,
            r#"{"data": [{"attributes": {"scopes": [{"scope": {"name": "MyClass"}}]}}]}"#,
        )
        .await;
        let _ = super::search(
            &cfg,
            "my-service",
            "MyClass",
            None,
            &super::SymdbView::Names,
        )
        .await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_symdb_search_probe_locations_view() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": [{"attributes": {"scopes": [{"scope": {"name": "MyClass", "scope_type": "METHOD"}, "probe_location": {"type_name": "MyClass", "method_name": "doStuff"}}]}}]}"#).await;
        let _ = super::search(
            &cfg,
            "my-service",
            "MyClass",
            None,
            &super::SymdbView::ProbeLocations,
        )
        .await;
        cleanup_env();
    }
}
