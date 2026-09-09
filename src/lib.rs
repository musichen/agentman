#![forbid(unsafe_code)]

mod adapters;
pub mod app;
mod model;
pub mod ui;

use std::process::Command;

use anyhow::{Context, Result};

pub use adapters::{CommandSpec, SessionAction, discover_all, launch_command, rename_session};
pub use model::{AgentKind, Capability, Session, ranked_sessions};

/// Fetch the latest published version from crates.io.
///
/// # Errors
/// Returns an error when crates.io is unavailable or returns invalid metadata.
pub fn latest_version() -> Result<semver::Version> {
    let output = Command::new("cargo")
        .args(["search", "agentman", "--limit", "1"])
        .output()
        .context("could not run cargo search")?;
    anyhow::ensure!(output.status.success(), "cargo search failed");
    let line = String::from_utf8_lossy(&output.stdout);
    let version = line
        .split('"')
        .nth(1)
        .context("agentman was not found on crates.io")?;
    Ok(semver::Version::parse(version)?)
}

#[must_use]
pub fn update_message(current: &str, latest: &str) -> String {
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(current), Ok(latest)) if latest > current => {
            format!("Update available: agentman {current} -> {latest}")
        }
        (Ok(current), Ok(_)) => format!("agentman {current} is up to date."),
        _ => format!("Current agentman version: {current}; latest: {latest}"),
    }
}
