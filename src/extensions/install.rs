use anyhow::{bail, Context, Result};
use std::path::Path;

use super::discovery::extension_dir;
use super::manifest::Manifest;
use crate::version;

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

    let ext_base =
        extension_dir().context("could not determine config directory for extensions")?;
    let ext_dir = ext_base.join(format!("pup-{name}"));

    if ext_dir.exists() && !force {
        bail!("extension '{name}' is already installed (use --force to overwrite)");
    }

    // Create the extension directory (remove first if forcing).
    if ext_dir.exists() {
        std::fs::remove_dir_all(&ext_dir).with_context(|| {
            format!(
                "removing existing extension directory: {}",
                ext_dir.display()
            )
        })?;
    }
    std::fs::create_dir_all(&ext_dir).with_context(|| format!("creating {}", ext_dir.display()))?;

    let exe_name = if cfg!(target_os = "windows") {
        format!("pup-{name}.exe")
    } else {
        format!("pup-{name}")
    };
    let dest = ext_dir.join(&exe_name);

    if link {
        #[cfg(unix)]
        std::os::unix::fs::symlink(source, &dest).with_context(|| {
            format!(
                "creating symlink {} -> {}",
                dest.display(),
                source.display()
            )
        })?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(source, &dest).with_context(|| {
            format!(
                "creating symlink {} -> {}",
                dest.display(),
                source.display()
            )
        })?;
    } else {
        std::fs::copy(source, &dest)
            .with_context(|| format!("copying {} -> {}", source.display(), dest.display()))?;

        // Set executable permission on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&dest, perms)
                .with_context(|| format!("setting permissions on {}", dest.display()))?;
        }
    }

    let source_str = if link {
        format!("local-link:{}", source.display())
    } else {
        format!("local:{}", source.display())
    };

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
    let ext_base =
        extension_dir().context("could not determine config directory for extensions")?;
    let ext_dir = ext_base.join(format!("pup-{name}"));

    if !ext_dir.exists() {
        bail!("extension '{name}' is not installed");
    }

    std::fs::remove_dir_all(&ext_dir).with_context(|| format!("removing {}", ext_dir.display()))?;
    Ok(())
}

/// Return the current time as an ISO 8601 string (UTC).
fn chrono_now_iso() -> String {
    // Use a simple approach without pulling in chrono.
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    // Format as a simple timestamp - not perfect ISO 8601 but functional.
    format!("{}s-since-epoch", now.as_secs())
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
    fn test_remove_nonexistent() {
        let dir = std::env::temp_dir().join("pup-test-remove-nonexistent");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("extensions")).unwrap();

        let _guard = crate::test_utils::ENV_LOCK.lock().unwrap();
        std::env::set_var("PUP_CONFIG_DIR", &dir);

        let result = remove_extension("nonexistent");
        assert!(result.is_err());

        std::env::remove_var("PUP_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
