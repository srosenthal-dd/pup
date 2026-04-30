use anyhow::{bail, Context, Result};
use std::path::Path;

use super::discovery::extension_dir;
use super::manifest::Manifest;
use crate::version;

/// GitHub release asset metadata (subset of the GitHub Releases API response).
#[derive(Debug, serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// GitHub release metadata (subset of the GitHub Releases API response).
#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

/// Map `std::env::consts::OS` to the asset name convention.
fn platform_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    }
}

/// Map `std::env::consts::ARCH` to the asset name convention.
fn platform_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => other,
    }
}

/// Build a reqwest client with a User-Agent header (required by GitHub API).
fn github_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(format!("pup/{}", version::VERSION))
        .build()
        .context("building HTTP client for GitHub API")
}

/// Fetch a GitHub release (latest or by tag).
async fn fetch_github_release(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    tag: Option<&str>,
) -> Result<GitHubRelease> {
    let url = match tag {
        Some(t) => format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{t}"),
        None => format!("https://api.github.com/repos/{owner}/{repo}/releases/latest"),
    };

    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("fetching release from {url}"))?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        match tag {
            Some(t) => bail!(
                "release tag '{t}' not found in {owner}/{repo}. \
                 Check that the tag exists at https://github.com/{owner}/{repo}/releases"
            ),
            None => bail!(
                "no releases found for {owner}/{repo}. \
                 Check that the repository exists and has at least one release at \
                 https://github.com/{owner}/{repo}/releases"
            ),
        }
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("GitHub API returned {status} for {url}: {body}");
    }

    resp.json::<GitHubRelease>()
        .await
        .with_context(|| format!("parsing release JSON from {url}"))
}

/// Find the matching asset for the current platform in a release.
fn find_platform_asset<'a>(release: &'a GitHubRelease, ext_name: &str) -> Result<&'a GitHubAsset> {
    let os = platform_os();
    let arch = platform_arch();

    // Expected asset name: pup-<name>-<os>-<arch> (or with .exe on Windows)
    let expected = format!("pup-{ext_name}-{os}-{arch}");
    let expected_exe = format!("{expected}.exe");

    release
        .assets
        .iter()
        .find(|a| a.name == expected || a.name == expected_exe)
        .ok_or_else(|| {
            let available: Vec<&str> = release.assets.iter().map(|a| a.name.as_str()).collect();
            anyhow::anyhow!(
                "no matching asset for platform {os}-{arch} (expected '{expected}'). \
                 Available assets: {}",
                if available.is_empty() {
                    "(none)".to_string()
                } else {
                    available.join(", ")
                }
            )
        })
}

/// Download a binary asset from a URL.
async fn download_asset(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("downloading asset from {url}"))?;

    let status = resp.status();
    if !status.is_success() {
        bail!("download failed with HTTP {status} for {url}");
    }

    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .with_context(|| format!("reading asset bytes from {url}"))
}

/// Validate that a string contains only characters allowed in GitHub usernames/repo names.
/// GitHub allows `[a-zA-Z0-9._-]` for both owners and repos.
fn is_valid_github_name(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// Validate that a GitHub release tag contains only safe characters.
/// Tags generally allow `[a-zA-Z0-9._-/+]`.
fn is_valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '/' || c == '+'
        })
}

/// Parse an "owner/repo" string into (owner, repo).
pub fn parse_owner_repo(source: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = source.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        bail!(
            "invalid GitHub source '{source}': expected format 'owner/repo' \
             (e.g., 'jkirsteins/pup-hello')"
        );
    }
    if !is_valid_github_name(parts[0]) {
        bail!(
            "invalid GitHub owner '{owner}': only alphanumeric characters, hyphens, \
             underscores, and dots are allowed",
            owner = parts[0]
        );
    }
    if !is_valid_github_name(parts[1]) {
        bail!(
            "invalid GitHub repo '{repo}': only alphanumeric characters, hyphens, \
             underscores, and dots are allowed",
            repo = parts[1]
        );
    }
    Ok((parts[0], parts[1]))
}

/// Derive the extension name from a GitHub repo name.
/// Strips the "pup-" prefix if present.
pub fn derive_name_from_repo(repo: &str) -> String {
    repo.strip_prefix("pup-").unwrap_or(repo).to_string()
}

/// Prepare (create or recreate) an extension directory.
fn prepare_extension_dir(ext_dir: &Path) -> Result<()> {
    if ext_dir.exists() {
        std::fs::remove_dir_all(ext_dir).with_context(|| {
            format!(
                "removing existing extension directory: {}",
                ext_dir.display()
            )
        })?;
    }
    std::fs::create_dir_all(ext_dir).with_context(|| format!("creating {}", ext_dir.display()))?;
    Ok(())
}

/// Write a binary to the extension directory and set executable permissions.
/// Returns the executable filename (e.g., "pup-hello" or "pup-hello.exe").
fn write_extension_binary(ext_dir: &Path, name: &str, bytes: &[u8]) -> Result<String> {
    let exe_name = if cfg!(target_os = "windows") {
        format!("pup-{name}.exe")
    } else {
        format!("pup-{name}")
    };
    let dest = ext_dir.join(&exe_name);

    std::fs::write(&dest, bytes)
        .with_context(|| format!("writing binary to {}", dest.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&dest, perms)
            .with_context(|| format!("setting permissions on {}", dest.display()))?;
    }

    Ok(exe_name)
}

/// Install an extension from a GitHub repository.
/// Downloads the appropriate platform-specific binary from GitHub Releases.
pub fn install_from_github(
    source: &str,
    tag: Option<&str>,
    name_override: Option<&str>,
    force: bool,
    description: Option<&str>,
) -> Result<()> {
    let (owner, repo) = parse_owner_repo(source)?;
    // The asset name is always derived from the repo (e.g., "hello" from "pup-hello").
    // The ext_name may be overridden by the user via --name for the local directory/manifest.
    let asset_name = derive_name_from_repo(repo);
    let ext_name = match name_override {
        Some(n) => n.to_string(),
        None => asset_name.clone(),
    };

    validate_extension_name(&ext_name)?;

    if let Some(t) = tag {
        if !is_valid_tag(t) {
            bail!(
                "invalid release tag '{t}': only alphanumeric characters, hyphens, \
                 underscores, dots, slashes, and plus signs are allowed"
            );
        }
    }

    let ext_base =
        extension_dir().context("could not determine config directory for extensions")?;
    let ext_dir = ext_base.join(format!("pup-{ext_name}"));

    if ext_dir.exists() && !force {
        bail!("extension '{ext_name}' is already installed (use --force to overwrite)");
    }

    // Run the async download inside the existing tokio runtime.
    let (release, asset_bytes) = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let client = github_client()?;
            let release = fetch_github_release(&client, owner, repo, tag).await?;
            let asset = find_platform_asset(&release, &asset_name)?;
            let bytes = download_asset(&client, &asset.browser_download_url).await?;
            Ok::<_, anyhow::Error>((release, bytes))
        })
    })?;

    let release_version = extract_version(&release.tag_name);

    // Create (or recreate) the extension directory.
    prepare_extension_dir(&ext_dir)?;

    let exe_name = write_extension_binary(&ext_dir, &ext_name, &asset_bytes)?;

    let manifest = Manifest {
        name: ext_name.clone(),
        version: release_version,
        source: format!("github:{source}"),
        installed_at: chrono_now_iso(),
        binary: exe_name,
        description: description.unwrap_or_default().to_string(),
        installed_by_pup: version::VERSION.to_string(),
    };
    manifest.save(&ext_dir.join("manifest.json"))?;

    Ok(())
}

/// Upgrade a single GitHub-sourced extension. Returns a message describing what happened.
pub fn upgrade_extension(name: &str) -> Result<String> {
    validate_extension_name(name)?;

    let ext_base =
        extension_dir().context("could not determine config directory for extensions")?;
    let ext_dir = ext_base.join(format!("pup-{name}"));

    if !ext_dir.exists() {
        bail!("extension '{name}' is not installed");
    }

    let manifest = Manifest::load(&ext_dir.join("manifest.json"))
        .with_context(|| format!("loading manifest for extension '{name}'"))?;

    if manifest.source.starts_with("local:") || manifest.source.starts_with("local-link:") {
        bail!(
            "extension '{name}' was installed from a local source ({}) and cannot be upgraded \
             automatically. Reinstall it manually with: pup extension install --local <path> --force",
            manifest.source
        );
    }

    if !manifest.source.starts_with("github:") {
        bail!(
            "extension '{name}' has an unrecognized source type '{}' and cannot be upgraded",
            manifest.source
        );
    }

    let gh_source = manifest
        .source
        .strip_prefix("github:")
        .expect("source starts with github:");
    let gh_source = gh_source
        .split_once('@')
        .map_or(gh_source, |(base, _)| base);

    let (owner, repo) = parse_owner_repo(gh_source)?;
    // Asset name is derived from the repo, not the manifest name (which may have been overridden
    // via --name at install time).
    let asset_name = derive_name_from_repo(repo);

    // Build the HTTP client once for both the metadata fetch and the binary download.
    let client = github_client()?;

    // Step 1: Fetch the release metadata (small JSON) and check version BEFORE downloading.
    let release = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { fetch_github_release(&client, owner, repo, None).await })
    })?;

    let new_version = extract_version(&release.tag_name);

    if new_version == manifest.version {
        return Ok(format!("{name}: already at latest version ({new_version})"));
    }

    let old_version = manifest.version.clone();

    // Step 2: Version differs - now download the binary.
    let asset = find_platform_asset(&release, &asset_name)?;
    let asset_url = asset.browser_download_url.clone();

    let asset_bytes = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { download_asset(&client, &asset_url).await })
    })?;

    // Prepare (recreate) the extension directory before writing, so a failed write
    // does not leave a partially-corrupted state.
    prepare_extension_dir(&ext_dir)?;

    let exe_name = write_extension_binary(&ext_dir, name, &asset_bytes)?;

    // Update the manifest
    let updated_manifest = Manifest {
        version: new_version.clone(),
        source: format!("github:{gh_source}"),
        installed_at: chrono_now_iso(),
        binary: exe_name,
        installed_by_pup: version::VERSION.to_string(),
        ..manifest
    };
    updated_manifest.save(&ext_dir.join("manifest.json"))?;

    Ok(format!("{name}: upgraded {old_version} -> {new_version}"))
}

/// Upgrade all installed extensions. Returns a summary of what happened.
pub fn upgrade_all_extensions() -> Result<Vec<String>> {
    let exts = super::discovery::list_extensions()?;
    if exts.is_empty() {
        return Ok(vec!["No extensions installed.".to_string()]);
    }

    let mut results = Vec::new();

    for ext in &exts {
        if ext.source.starts_with("local:") || ext.source.starts_with("local-link:") {
            results.push(format!(
                "{}: skipped (installed from local source)",
                ext.name
            ));
            continue;
        }
        match upgrade_extension(&ext.name) {
            Ok(msg) => results.push(msg),
            Err(e) => results.push(format!("{}: error: {e}", ext.name)),
        }
    }

    Ok(results)
}

/// Validate that an extension name is well-formed and does not conflict with built-in commands.
pub fn validate_extension_name(name: &str) -> Result<()> {
    // Must match ^[a-z][a-z0-9-]*$
    if name.is_empty() {
        bail!("extension name cannot be empty");
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_lowercase() {
        bail!("extension name must start with a lowercase letter, got '{name}'");
    }
    for c in chars {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
            bail!(
                "extension name '{name}' contains invalid character '{c}' \
                 (only lowercase letters, digits, and hyphens allowed)"
            );
        }
    }

    // Reject names that collide with built-in commands.
    if super::is_builtin_command(name) {
        bail!(
            "'{name}' conflicts with a built-in pup command and cannot be used as an extension name"
        );
    }

    Ok(())
}

/// Install an extension from a local file path.
/// If `link` is true, creates a symlink instead of copying.
pub fn install_from_local(
    source: &Path,
    name: &str,
    link: bool,
    force: bool,
    description: Option<&str>,
) -> Result<()> {
    validate_extension_name(name)?;

    if !source.exists() {
        bail!("source file does not exist: {}", source.display());
    }
    if !source.is_file() {
        bail!(
            "source must be a regular file, not a directory: {}",
            source.display()
        );
    }

    // Canonicalize the source path so that symlinks resolve correctly.
    // Without this, a relative path like ./pup-hello would be resolved
    // relative to the symlink's parent directory, not the user's CWD.
    let source = std::fs::canonicalize(source)
        .with_context(|| format!("resolving absolute path for source: {}", source.display()))?;

    let ext_base =
        extension_dir().context("could not determine config directory for extensions")?;
    let ext_dir = ext_base.join(format!("pup-{name}"));

    if ext_dir.exists() && !force {
        bail!("extension '{name}' is already installed (use --force to overwrite)");
    }

    prepare_extension_dir(&ext_dir)?;

    let exe_name = if link {
        // For symlinks, we need to create the link directly rather than writing bytes.
        let exe_name = if cfg!(target_os = "windows") {
            format!("pup-{name}.exe")
        } else {
            format!("pup-{name}")
        };
        let dest = ext_dir.join(&exe_name);

        #[cfg(unix)]
        std::os::unix::fs::symlink(&source, &dest).with_context(|| {
            format!(
                "creating symlink {} -> {}",
                dest.display(),
                source.display()
            )
        })?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&source, &dest).with_context(|| {
            format!(
                "creating symlink {} -> {}",
                dest.display(),
                source.display()
            )
        })?;

        exe_name
    } else {
        let bytes = std::fs::read(&source)
            .with_context(|| format!("reading source file: {}", source.display()))?;
        write_extension_binary(&ext_dir, name, &bytes)?
    };

    let source_str = if link {
        format!("local-link:{}", source.display())
    } else {
        format!("local:{}", source.display())
    };

    // Local installs have no version source (unlike GitHub releases which provide a tag).
    let manifest = Manifest {
        name: name.to_string(),
        version: "unknown".to_string(),
        source: source_str,
        installed_at: chrono_now_iso(),
        binary: exe_name,
        description: description.unwrap_or_default().to_string(),
        installed_by_pup: version::VERSION.to_string(),
    };
    manifest.save(&ext_dir.join("manifest.json"))?;

    Ok(())
}

/// Remove an installed extension by name.
pub fn remove_extension(name: &str) -> Result<()> {
    validate_extension_name(name)?;

    let ext_base =
        extension_dir().context("could not determine config directory for extensions")?;
    let ext_dir = ext_base.join(format!("pup-{name}"));

    if !ext_dir.exists() {
        bail!("extension '{name}' is not installed");
    }

    std::fs::remove_dir_all(&ext_dir).with_context(|| format!("removing {}", ext_dir.display()))?;
    Ok(())
}

/// Extract a version string from a GitHub release tag name, stripping the 'v' prefix if present.
fn extract_version(tag_name: &str) -> String {
    tag_name.strip_prefix('v').unwrap_or(tag_name).to_string()
}

/// Return the current time as an ISO 8601 / RFC 3339 string (UTC).
fn chrono_now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_extension_name("hello").is_ok());
        assert!(validate_extension_name("my-tool").is_ok());
        assert!(validate_extension_name("tool2").is_ok());
        assert!(validate_extension_name("a").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_extension_name("").is_err());
    }

    #[test]
    fn test_validate_name_starts_with_digit() {
        assert!(validate_extension_name("2tool").is_err());
    }

    #[test]
    fn test_validate_name_uppercase() {
        assert!(validate_extension_name("Hello").is_err());
    }

    #[test]
    fn test_validate_name_special_chars() {
        assert!(validate_extension_name("my_tool").is_err());
        assert!(validate_extension_name("my.tool").is_err());
    }

    #[test]
    fn test_validate_name_builtin_conflict() {
        assert!(validate_extension_name("monitors").is_err());
        assert!(validate_extension_name("extension").is_err());
        assert!(validate_extension_name("help").is_err());
        assert!(validate_extension_name("version").is_err());
    }

    #[test]
    fn test_validate_name_path_traversal() {
        // Names containing path separators or traversal sequences must be rejected
        assert!(validate_extension_name("../etc").is_err());
        assert!(validate_extension_name("foo/bar").is_err());
        assert!(validate_extension_name("..").is_err());
    }

    #[test]
    fn test_chrono_now_iso_format() {
        let ts = chrono_now_iso();
        // Must parse as a valid RFC 3339 / ISO 8601 timestamp
        assert!(
            chrono::DateTime::parse_from_rfc3339(&ts).is_ok(),
            "chrono_now_iso() produced invalid RFC 3339: {}",
            ts
        );
    }

    #[test]
    fn test_remove_rejects_path_traversal() {
        // remove_extension must reject names with path traversal characters
        // before attempting any filesystem operations.
        assert!(remove_extension("../important-data").is_err());
        assert!(remove_extension("foo/bar").is_err());
        assert!(remove_extension("..").is_err());
    }

    #[test]
    fn test_remove_nonexistent() {
        let dir = std::env::temp_dir().join("pup-test-remove-nonexistent");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("extensions")).unwrap();

        let _guard = crate::test_utils::ENV_LOCK.blocking_lock();
        std::env::set_var("PUP_CONFIG_DIR", &dir);

        let result = remove_extension("nonexistent");
        assert!(result.is_err());

        std::env::remove_var("PUP_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_owner_repo_valid() {
        let (owner, repo) = parse_owner_repo("jkirsteins/pup-hello").unwrap();
        assert_eq!(owner, "jkirsteins");
        assert_eq!(repo, "pup-hello");
    }

    #[test]
    fn test_parse_owner_repo_no_slash() {
        assert!(parse_owner_repo("noslash").is_err());
    }

    #[test]
    fn test_parse_owner_repo_empty_parts() {
        assert!(parse_owner_repo("/repo").is_err());
        assert!(parse_owner_repo("owner/").is_err());
        assert!(parse_owner_repo("").is_err());
    }

    #[test]
    fn test_parse_owner_repo_extra_slashes() {
        assert!(parse_owner_repo("a/b/c").is_err());
    }

    #[test]
    fn test_derive_name_from_repo_strips_prefix() {
        assert_eq!(derive_name_from_repo("pup-hello"), "hello");
        assert_eq!(derive_name_from_repo("pup-my-tool"), "my-tool");
    }

    #[test]
    fn test_derive_name_from_repo_no_prefix() {
        assert_eq!(derive_name_from_repo("hello"), "hello");
        assert_eq!(derive_name_from_repo("my-tool"), "my-tool");
    }

    #[test]
    fn test_platform_os_known() {
        let os = platform_os();
        // Should be one of the expected values on any CI/dev machine
        assert!(
            ["darwin", "linux", "windows"].contains(&os),
            "unexpected platform_os: {os}"
        );
    }

    #[test]
    fn test_platform_arch_known() {
        let arch = platform_arch();
        assert!(
            ["x86_64", "aarch64"].contains(&arch),
            "unexpected platform_arch: {arch}"
        );
    }

    #[test]
    fn test_find_platform_asset_found() {
        let os = platform_os();
        let arch = platform_arch();
        let expected_name = format!("pup-hello-{os}-{arch}");
        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            assets: vec![
                GitHubAsset {
                    name: "pup-hello-linux-x86_64".to_string(),
                    browser_download_url: "https://example.com/linux-x86_64".to_string(),
                },
                GitHubAsset {
                    name: "pup-hello-darwin-aarch64".to_string(),
                    browser_download_url: "https://example.com/darwin-aarch64".to_string(),
                },
                GitHubAsset {
                    name: "pup-hello-darwin-x86_64".to_string(),
                    browser_download_url: "https://example.com/darwin-x86_64".to_string(),
                },
                GitHubAsset {
                    name: "pup-hello-windows-x86_64".to_string(),
                    browser_download_url: "https://example.com/windows-x86_64".to_string(),
                },
            ],
        };
        let asset = find_platform_asset(&release, "hello").unwrap();
        assert_eq!(asset.name, expected_name);
    }

    #[test]
    fn test_find_platform_asset_not_found() {
        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            assets: vec![GitHubAsset {
                name: "pup-hello-fakeos-fakearch".to_string(),
                browser_download_url: "https://example.com/fake".to_string(),
            }],
        };
        let result = find_platform_asset(&release, "hello");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("no matching asset"),
            "error should mention 'no matching asset': {err_msg}"
        );
    }

    #[test]
    fn test_find_platform_asset_empty() {
        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            assets: vec![],
        };
        let result = find_platform_asset(&release, "hello");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("(none)"),
            "error for empty assets should mention '(none)': {err_msg}"
        );
    }

    #[test]
    fn test_is_valid_github_name() {
        assert!(is_valid_github_name("jkirsteins"));
        assert!(is_valid_github_name("pup-hello"));
        assert!(is_valid_github_name("my_repo.v2"));
        assert!(is_valid_github_name("A-Z"));
        // GitHub rejects "." and ".." as repo names
        assert!(!is_valid_github_name("."));
        assert!(!is_valid_github_name(".."));
        assert!(!is_valid_github_name(""));
        assert!(!is_valid_github_name("owner name"));
        assert!(!is_valid_github_name("owner%0a"));
        assert!(!is_valid_github_name("foo/bar"));
    }

    #[test]
    fn test_parse_owner_repo_rejects_invalid_chars() {
        assert!(parse_owner_repo("owner%0a/repo").is_err());
        assert!(parse_owner_repo("owner/repo%00").is_err());
        assert!(parse_owner_repo("own er/repo").is_err());
        assert!(parse_owner_repo("owner/re po").is_err());
    }

    #[test]
    fn test_is_valid_tag() {
        assert!(is_valid_tag("v1.0.0"));
        assert!(is_valid_tag("v1.0.0-rc1"));
        assert!(is_valid_tag("release/v2.0"));
        assert!(is_valid_tag("v1.0.0+build.123"));
        assert!(!is_valid_tag(""));
        assert!(!is_valid_tag("v1.0.0 spaces"));
        assert!(!is_valid_tag("v1.0.0%0a"));
        assert!(!is_valid_tag("v1.0.0\nnewline"));
    }

    #[test]
    fn test_find_platform_asset_uses_asset_name_not_ext_name() {
        // Verify that find_platform_asset uses the repo-derived name, not a user override.
        // If installed with --name custom, the asset should still be looked up as "pup-hello-..."
        let os = platform_os();
        let arch = platform_arch();
        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            assets: vec![GitHubAsset {
                name: format!("pup-hello-{os}-{arch}"),
                browser_download_url: "https://example.com/hello".to_string(),
            }],
        };
        // Looking up by the repo-derived name "hello" should succeed.
        assert!(find_platform_asset(&release, "hello").is_ok());
        // Looking up by a user-overridden name "custom" should fail (no such asset).
        assert!(find_platform_asset(&release, "custom").is_err());
    }

    #[test]
    fn test_derive_name_from_repo_used_for_asset_lookup() {
        // Verify that derive_name_from_repo produces the correct asset lookup name,
        // independent of any --name override.
        assert_eq!(derive_name_from_repo("pup-hello"), "hello");
        assert_eq!(derive_name_from_repo("pup-my-extension"), "my-extension");
        assert_eq!(derive_name_from_repo("my-tool"), "my-tool");
    }
}
