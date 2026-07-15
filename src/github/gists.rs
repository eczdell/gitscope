use crate::ansi;
use crate::app::AppState;

use super::{build_octocrab, get_token};

// ═══════════════════════════════════════════════════════════════════════════
// Gists
// ═══════════════════════════════════════════════════════════════════════════

/// Open gists view (list the authenticated user's gists).
pub fn open_gists_view(app: &mut AppState) {
    // Check for token
    let token = get_token();
    if token.is_none() {
        app.gists_lines = vec![
            format!(
                "  {}GitHub token required to list gists.{}\n  {}Set GITHUB_TOKEN or GH_TOKEN env var, or run `gh auth login`{}",
                ansi::LRE, ansi::RST,
                ansi::DIM, ansi::RST
            ),
        ];
        app.gists_scroll = 0;
        app.mode = crate::app::AppMode::Gists;
        return;
    }

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let result = rt.block_on(async {
        let octo = build_octocrab()?;
        // Use the raw GET endpoint to list gists
        let gists: serde_json::Value = octo
            .get("/gists", Some(&[("per_page", "50"), ("page", "1")]))
            .await
            .map_err(|e| format!("Failed to list gists: {}", e))?;
        Ok::<_, String>(gists)
    });

    let gists = match result {
        Ok(g) => g,
        Err(e) => {
            app.gists_lines = vec![format!(
                "  {}Error fetching gists: {}{}",
                ansi::LRE, e, ansi::RST
            )];
            app.gists_scroll = 0;
            app.mode = crate::app::AppMode::Gists;
            return;
        }
    };

    format_gists_lines(app, &gists);
}

/// Format gists JSON into display lines
fn format_gists_lines(app: &mut AppState, gists: &serde_json::Value) {
    let gist_array = match gists.as_array() {
        Some(a) => a,
        None => {
            app.gists_lines = vec![format!(
                "  {}Unexpected response format from GitHub API.{}",
                ansi::LRE, ansi::RST
            )];
            app.gists_scroll = 0;
            app.mode = crate::app::AppMode::Gists;
            return;
        }
    };

    if gist_array.is_empty() {
        app.gists_lines = vec![format!(
            "  {}No gists found.{}",
            ansi::DIM, ansi::RST
        )];
        app.gists_scroll = 0;
        app.mode = crate::app::AppMode::Gists;
        return;
    }

    let mut lines: Vec<String> = Vec::new();

    // Header
    lines.push(format!(
        "{}{} Your Gists{}  {}({} total){}{}  {}q{} back{}",
        ansi::BLD, ansi::CYN, ansi::RST,
        ansi::DIM, gist_array.len(), ansi::RST,
        ansi::DIM, ansi::RST, ansi::DIM, ansi::RST
    ));
    lines.push(format!(
        "{}{}{}",
        ansi::DIM,
        "─".repeat(80.min(app.term_w as usize)),
        ansi::RST
    ));

    for gist in gist_array {
        // Extract fields from the JSON value
        let id = gist["id"].as_str().unwrap_or("");
        let description = gist["description"].as_str().unwrap_or("(no description)");
        let public = gist["public"].as_bool().unwrap_or(false);
        let html_url = gist["html_url"].as_str().unwrap_or("");
        let owner_login = gist["owner"]["login"].as_str().unwrap_or("unknown");
        let created_at = gist["created_at"].as_str().unwrap_or("");
        let updated_at = gist["updated_at"].as_str().unwrap_or("");
        let comments = gist["comments"].as_u64().unwrap_or(0);

        // Extract file names
        let files = gist["files"].as_object();
        let file_names: Vec<String> = files
            .map(|f| f.keys().cloned().collect())
            .unwrap_or_default();

        // Public/Private indicator
        let vis_label = if public {
            format!("{}PUBLIC {}", ansi::LGR, ansi::RST)
        } else {
            format!("{}PRIVATE{}", ansi::LRE, ansi::RST)
        };

        // Truncate description to fit
        let max_desc_len = (app.term_w as usize).saturating_sub(50);
        let desc_short = if description.len() > max_desc_len {
            format!("{}…", &description[..max_desc_len.saturating_sub(1)])
        } else {
            description.to_string()
        };

        // Gist line
        lines.push(format!(
            "  {} {} {}{}{}  {}{}",
            vis_label,
            ansi::BLD, ansi::WHT, desc_short, ansi::RST,
            ansi::GRY, ansi::RST
        ));

        // File names on a sub-line
        if !file_names.is_empty() {
            let files_str = file_names.join(", ");
            let max_files_len = (app.term_w as usize).saturating_sub(6);
            let files_display = if files_str.len() > max_files_len {
                format!("{}…", &files_str[..max_files_len.saturating_sub(1)])
            } else {
                files_str
            };
            lines.push(format!(
                "  {}  {}📄{} {}{}",
                ansi::DIM, ansi::RST, ansi::DIM, files_display, ansi::RST
            ));
        }

        // Metadata line (author, dates, comments, id)
        lines.push(format!(
            "  {}    by {}{}  created: {}  updated: {}  {}comments: {}{}  {}id: {}{}",
            ansi::GRY, owner_login, ansi::RST,
            &created_at[..10.min(created_at.len())],
            &updated_at[..10.min(updated_at.len())],
            ansi::LYL, comments, ansi::RST,
            ansi::DIM, id, ansi::RST
        ));

        // URL line (dim, optional)
        if !html_url.is_empty() {
            lines.push(format!(
                "  {}    {}{}",
                ansi::GRY, html_url, ansi::RST
            ));
        }

        // Separator between gists
        lines.push(format!(
            "  {}·{}",
            ansi::DIM, ansi::RST
        ));
    }

    app.gists_lines = lines;
    app.gists_lines_full = app.gists_lines.clone();
    app.gists_scroll = 0;
    app.mode = crate::app::AppMode::Gists;
}

/// Apply gists filter
pub fn apply_gists_filter(app: &mut AppState) {
    let filter = app.gists_filter_text.to_lowercase();
    if filter.is_empty() {
        app.gists_lines = app.gists_lines_full.clone();
        return;
    }

    let mut filtered: Vec<String> = Vec::new();
    for (i, line) in app.gists_lines_full.iter().enumerate() {
        if i < 2 {
            filtered.push(line.clone());
            continue;
        }
        if line.to_lowercase().contains(&filter) {
            filtered.push(line.clone());
        }
    }

    app.gists_lines = filtered;
    if app.gists_cursor >= app.gists_lines.len() {
        app.gists_cursor = app.gists_lines.len().saturating_sub(1);
    }
    if app.gists_scroll >= app.gists_lines.len() {
        app.gists_scroll = app.gists_lines.len().saturating_sub(1);
    }
}

/// Fetch the raw content of a gist using `gh gist view --raw`.
pub fn get_gist_content(gist_id: &str) -> Result<String, String> {
    let output = std::process::Command::new("gh")
        .args(["gist", "view", gist_id, "--raw"])
        .output()
        .map_err(|e| format!("Failed to run `gh gist view`: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh gist view failed: {}", stderr.trim()));
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(content)
}

/// Open gist content view using `gh gist view` (formatted output).
pub fn open_gist_content(app: &mut AppState, gist_id: &str) {
    let output = std::process::Command::new("gh")
        .args(["gist", "view", gist_id])
        .output()
        .map_err(|e| format!("Failed to run `gh gist view`: {}", e));

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            app.gist_content_lines = vec![format!("  {}Error: {}{}", ansi::LRE, e, ansi::RST)];
            app.gist_content_scroll = 0;
            app.mode = crate::app::AppMode::GistContent;
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        app.gist_content_lines = vec![format!(
            "  {}gh gist view failed: {}{}",
            ansi::LRE,
            stderr.trim(),
            ansi::RST
        )];
        app.gist_content_scroll = 0;
        app.mode = crate::app::AppMode::GistContent;
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<String> = Vec::new();

    // Header with gist ID
    lines.push(format!(
        "{}{} Gist Content{}   {}(id: {}){}  {}q{} back",
        ansi::BLD, ansi::CYN, ansi::RST,
        ansi::DIM, gist_id, ansi::RST,
        ansi::DIM, ansi::RST
    ));
    lines.push(format!(
        "{}{}{}",
        ansi::DIM,
        "─".repeat(80.min(app.term_w as usize)),
        ansi::RST
    ));

    // Format the gist content lines
    for line in stdout.lines() {
        lines.push(line.to_string());
    }

    app.gist_content_lines = lines;
    app.gist_content_scroll = 0;
    app.mode = crate::app::AppMode::GistContent;
}

/// Extract gist ID from the cursor position in gists_lines.
/// Scans backwards from the cursor to find a line containing "id:" and extracts the ID.
/// Strips any ANSI escape codes that may be present in the extracted ID.
pub fn extract_gist_id(lines: &[String], cursor: usize) -> Option<String> {
    let mut idx = cursor;
    loop {
        if let Some(line) = lines.get(idx) {
            if line.contains("id:") {
                if let Some(pos) = line.find("id:") {
                    let id_part = &line[pos + 3..].trim();
                    let id = id_part.split_whitespace().next().unwrap_or("");
                    if !id.is_empty() {
                        return Some(crate::ansi::strip_ansi(id));
                    }
                }
            }
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    None
}

