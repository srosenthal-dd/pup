use anyhow::{bail, Result};
use std::path::PathBuf;

use crate::config::Config;
use crate::extensions;

/// List all installed extensions.
pub fn list(cfg: &Config) -> Result<()> {
    let exts = extensions::discovery::list_extensions()?;
    if exts.is_empty() {
        match cfg.output_format {
            crate::config::OutputFormat::Table => {
                println!("No extensions installed.");
                println!();
                println!("Install one with: pup extension install <source>");
            }
            _ => {
                crate::formatter::format_and_print(
                    &Vec::<serde_json::Value>::new(),
                    &cfg.output_format,
                    cfg.agent_mode,
                    None,
                )?;
            }
        }
        return Ok(());
    }

    match cfg.output_format {
        crate::config::OutputFormat::Table => {
            for ext in &exts {
                let desc = if ext.description.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", ext.description)
                };
                println!("{} v{}{}", ext.name, ext.version, desc);
            }
        }
        _ => {
            let items: Vec<serde_json::Value> = exts
                .iter()
                .map(|ext| {
                    serde_json::json!({
                        "name": ext.name,
                        "version": ext.version,
                        "source": ext.source,
                        "description": ext.description,
                        "installed_at": ext.installed_at,
                    })
                })
                .collect();
            crate::formatter::format_and_print(&items, &cfg.output_format, cfg.agent_mode, None)?;
        }
    }
    Ok(())
}

/// Install an extension from a source.
#[allow(clippy::too_many_arguments)]
pub fn install(
    _cfg: &Config,
    source: String,
    _tag: Option<String>,
    local: bool,
    link: bool,
    _url: bool,
    name: Option<String>,
    force: bool,
    description: Option<String>,
) -> Result<()> {
    if local {
        let source_path = PathBuf::from(&source);
        // Derive name from filename if not provided.
        let ext_name = match name {
            Some(n) => n,
            None => {
                let file_name = source_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("");
                // Strip pup- prefix and .exe suffix if present.
                let stripped = file_name.strip_prefix("pup-").unwrap_or(file_name);
                let stripped = stripped.strip_suffix(".exe").unwrap_or(stripped);
                if stripped.is_empty() {
                    bail!(
                        "could not derive extension name from '{}', use --name to specify it",
                        source
                    );
                }
                stripped.to_string()
            }
        };

        extensions::install::install_from_local(
            &source_path,
            &ext_name,
            link,
            force,
            description.as_deref(),
        )?;
        if link {
            println!("Linked extension '{ext_name}' from {source}");
        } else {
            println!("Installed extension '{ext_name}' from {source}");
        }
        return Ok(());
    }

    // GitHub install (not yet implemented).
    bail!(
        "GitHub-based installation is not yet implemented. \
         Use --local to install from a local file path."
    );
}

/// Remove an installed extension.
pub fn remove(_cfg: &Config, name: String) -> Result<()> {
    extensions::install::remove_extension(&name)?;
    println!("Removed extension '{name}'");
    Ok(())
}

/// Upgrade an extension (stub for future implementation).
pub fn upgrade(_cfg: &Config, _name: Option<String>, _all: bool) -> Result<()> {
    bail!("extension upgrade is not yet implemented")
}
