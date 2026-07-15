mod issues;
mod gists;
mod labels;

pub use issues::*;
pub use gists::*;
pub(crate) use labels::*;

use std::process::Command;

use octocrab::Octocrab;

// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct GithubRepo {
    pub owner: String,
    pub repo: String,
}

/// Parse owner/repo from a git remote URL.
///
/// Supports formats:
/// - `https://github.com/owner/repo.git`
/// - `git@github.com:owner/repo.git`
/// - `https://github.com/owner/repo`
fn parse_remote_url(url: &str) -> Option<GithubRepo> {
    let url = url.trim().trim_end_matches(".git");

    if let Some(cap) = url.rsplit_once("github.com/") {
        let parts: Vec<&str> = cap.1.split('/').collect();
        if parts.len() >= 2 {
            return Some(GithubRepo {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            });
        }
    }
    if let Some(cap) = url.rsplit_once("github.com:") {
        let parts: Vec<&str> = cap.1.split('/').collect();
        if parts.len() >= 2 {
            return Some(GithubRepo {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            });
        }
    }

    None
}

/// Detect owner and repo from the current git remote origin.
pub fn detect_owner_repo() -> Option<GithubRepo> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&output.stdout);
    parse_remote_url(&url)
}

// ═══════════════════════════════════════════════════════════════════════════
// GitHub Token
// ═══════════════════════════════════════════════════════════════════════════

pub fn get_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN").ok().or_else(|| {
        std::env::var("GH_TOKEN").ok()
    }).or_else(|| {
        // Fallback: try `gh auth token` from GitHub CLI
        Command::new("gh")
            .args(["auth", "token"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper: Build Octocrab instance
// ═══════════════════════════════════════════════════════════════════════════

fn build_octocrab() -> Result<Octocrab, String> {
    let token = get_token().ok_or_else(|| {
        "No GitHub token found. Set GITHUB_TOKEN, GH_TOKEN, or run `gh auth login`.".to_string()
    })?;
    Octocrab::builder()
        .personal_token(token)
        .build()
        .map_err(|e| format!("Failed to build Octocrab client: {}", e))
}

/// Build an Octocrab instance without requiring a token (for read-only public repo access).
fn build_octocrab_unauthed() -> Octocrab {
    if let Some(token) = get_token() {
        Octocrab::builder()
            .personal_token(token)
            .build()
            .unwrap_or_default()
    } else {
        Octocrab::default()
    }
}

