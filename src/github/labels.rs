use crate::app::AppState;

use super::detect_owner_repo;

// ═══════════════════════════════════════════════════════════════════════════
// Fetch Available Labels
// ═══════════════════════════════════════════════════════════════════════════

/// Fetch available labels for the current repo using `gh label list`.
pub(crate) fn fetch_available_labels(app: &mut AppState) {
    let (owner, repo) = if let Some(r) = detect_owner_repo() {
        (r.owner, r.repo)
    } else {
        return;
    };

    let cmd = std::process::Command::new("gh")
        .args([
            "label",
            "list",
            "--repo",
            &format!("{}/{}", owner, repo),
            "--json",
            "name",
            "-q",
            ".[].name",
        ])
        .output();

    match cmd {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let labels: Vec<String> = stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            app.available_labels = labels;
        }
        _ => {
            app.available_labels.clear();
        }
    }
}

