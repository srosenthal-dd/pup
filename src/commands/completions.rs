use anyhow::{Context, Result};
use clap::CommandFactory;
use clap_complete::Shell;
use std::fs;
use std::path::PathBuf;

use crate::Cli;

/// Install shell completions to the standard location for the given shell.
pub fn install(shell: Shell) -> Result<()> {
    let (path, post_install_msg) = install_path(shell)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory: {}", parent.display()))?;
    }

    let mut file = fs::File::create(&path)
        .with_context(|| format!("failed to create file: {}", path.display()))?;

    clap_complete::generate(shell, &mut Cli::command(), "pup", &mut file);

    println!("Installed {} completions to: {}", shell, path.display());
    if let Some(msg) = post_install_msg {
        println!("{}", msg);
    }

    Ok(())
}

/// Returns (install_path, optional_post_install_instructions) for the given shell.
fn install_path(shell: Shell) -> Result<(PathBuf, Option<String>)> {
    match shell {
        Shell::Bash => {
            let dir = bash_completion_dir()?;
            let path = dir.join("pup");
            let msg = Some(format!(
                "To enable completions, add the following to your .bashrc (if not already present):\n  source \"{}\"\n  # or restart your shell",
                path.display()
            ));
            Ok((path, msg))
        }
        Shell::Zsh => {
            let dir = zsh_completion_dir();
            let path = dir.join("_pup");
            let msg = Some(format!(
                "To enable completions, add the following to your .zshrc (if not already present):\n  fpath=(\"{}\" $fpath)\n  autoload -Uz compinit && compinit",
                dir.display()
            ));
            Ok((path, msg))
        }
        Shell::Fish => {
            let path = fish_completion_dir()?.join("pup.fish");
            Ok((path, None))
        }
        Shell::PowerShell => {
            let path = powershell_completion_path()?;
            let msg = Some(format!(
                "To enable completions, add the following to your PowerShell profile ($PROFILE):\n  . \"{}\"",
                path.display()
            ));
            Ok((path, msg))
        }
        Shell::Elvish => {
            let path = elvish_completion_dir()?.join("pup.elv");
            let msg = Some(
                "To enable completions, add the following to your ~/.elvish/rc.elv:\n  use ./completions/pup".to_string(),
            );
            Ok((path, msg))
        }
        _ => anyhow::bail!("unsupported shell: {shell}"),
    }
}

fn bash_completion_dir() -> Result<PathBuf> {
    // Prefer ~/.local/share/bash-completion/completions (user-writable, XDG standard)
    if let Some(home) = dirs::home_dir() {
        let xdg_dir = home
            .join(".local")
            .join("share")
            .join("bash-completion")
            .join("completions");
        return Ok(xdg_dir);
    }
    anyhow::bail!("could not determine home directory for bash completions")
}

fn zsh_completion_dir() -> PathBuf {
    // Use ~/.zfunc as the conventional user-local zsh completion directory
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".zfunc")
}

fn fish_completion_dir() -> Result<PathBuf> {
    // XDG_CONFIG_HOME/fish/completions or ~/.config/fish/completions
    let base = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".config")
    });
    Ok(base.join("fish").join("completions"))
}

fn powershell_completion_path() -> Result<PathBuf> {
    // Store alongside the user's Documents/PowerShell directory
    let docs = dirs::document_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("Documents")
    });
    Ok(docs.join("PowerShell").join("pup_completions.ps1"))
}

fn elvish_completion_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        anyhow::anyhow!("could not determine home directory for elvish completions")
    })?;
    Ok(home.join(".elvish").join("completions"))
}
