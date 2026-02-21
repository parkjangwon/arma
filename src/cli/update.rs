use std::fmt;
use std::io::{self, Write};
use std::process::Command;

use nix::libc;
use serde::Deserialize;

const DEFAULT_REPO: &str = "parkjangwon/arma";
const INSTALL_SCRIPT_URL: &str = "https://raw.githubusercontent.com/{repo}/main/install.sh";

#[derive(Debug)]
pub enum UpdateError {
    PermissionDenied,
    Http(reqwest::Error),
    InvalidReleaseTag(String),
    Io(io::Error),
    CommandFailed(i32),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => write!(f, "arma update requires root privileges (run with sudo)"),
            Self::Http(err) => write!(f, "failed to query latest release: {err}"),
            Self::InvalidReleaseTag(value) => write!(f, "invalid release tag format: {value}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::CommandFailed(code) => write!(f, "install script exited with code {code}"),
        }
    }
}

impl std::error::Error for UpdateError {}

impl From<reqwest::Error> for UpdateError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<io::Error> for UpdateError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
}

pub fn run_update(yes: bool) -> Result<(), UpdateError> {
    // SAFETY: libc call has no preconditions.
    if unsafe { libc::geteuid() } != 0 {
        return Err(UpdateError::PermissionDenied);
    }

    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(UpdateError::Io)?;

    let latest_tag = runtime.block_on(fetch_latest_tag(DEFAULT_REPO))?;
    let latest_version = parse_tag_version(&latest_tag)?;

    let should_overwrite_rules = if yes { true } else { prompt_overwrite_rules()? };

    println!("Current version : v{current_version}");
    println!("Target version  : {latest_tag}");
    println!(
        "Rule update mode: {}",
        if should_overwrite_rules {
            "overwrite"
        } else {
            "keep existing"
        }
    );

    let mut command = Command::new("bash");
    command.arg("-c").arg(format!(
        "curl -fsSL {} | bash -s -- --repo {} --tag {} --update-rules{}",
        INSTALL_SCRIPT_URL.replace("{repo}", DEFAULT_REPO),
        DEFAULT_REPO,
        latest_tag,
        if should_overwrite_rules {
            " --overwrite-rules"
        } else {
            ""
        }
    ));

    let status = command.status()?;
    if !status.success() {
        return Err(UpdateError::CommandFailed(status.code().unwrap_or(-1)));
    }

    println!("Update complete.");
    println!("Current version : v{current_version}");
    println!("Updated version : v{latest_version}");
    Ok(())
}

async fn fetch_latest_tag(repo: &str) -> Result<String, UpdateError> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = reqwest::Client::builder().user_agent("arma-updater").build()?;
    let response = client.get(url).send().await?.error_for_status()?;
    let payload = response.json::<ReleaseResponse>().await?;
    Ok(payload.tag_name)
}

fn parse_tag_version(tag: &str) -> Result<String, UpdateError> {
    let raw = tag.strip_prefix('v').unwrap_or(tag);
    let mut parts = raw.split('.');
    let major = parts.next().and_then(|value| value.parse::<u64>().ok());
    let minor = parts.next().and_then(|value| value.parse::<u64>().ok());
    let patch = parts.next().and_then(|value| value.parse::<u64>().ok());

    if major.is_none() || minor.is_none() || patch.is_none() || parts.next().is_some() {
        return Err(UpdateError::InvalidReleaseTag(tag.to_string()));
    }

    Ok(raw.to_string())
}

fn prompt_overwrite_rules() -> Result<bool, UpdateError> {
    print!("Overwrite local filter packs with latest defaults? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let normalized = input.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}
