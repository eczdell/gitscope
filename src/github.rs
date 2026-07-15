use std::process::Command;

use octocrab::params::State;
use octocrab::Octocrab;

use crate::ansi;
use crate::app::AppState;

// ═══════════════════════════════════════════════════════════════════════════
// GitHub Repository Detection
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

// ═══════════════════════════════════════════════════════════════════════════
// List Issues
// ═══════════════════════════════════════════════════════════════════════════

pub async fn list_issues(
    owner: &str,
    repo: &str,
    state: &str,
) -> Result<(), String> {
    let octo = build_octocrab()?;

    let state_filter = match state {
        "closed" => Some(State::Closed),
        "all" => None,
        _ => Some(State::Open),
    };

    let issues_list = octo.issues(owner, repo);
    let mut req = issues_list
        .list()
        .per_page(50)
        .page(1u32);

    if let Some(s) = state_filter {
        req = req.state(s);
    }

    let page = req
        .send()
        .await
        .map_err(|e| format!("Failed to list issues: {}", e))?;

    let issues = &page.items;

    if issues.is_empty() {
        println!(
            "  {}No {} issues found for {}/{}{}",
            ansi::DIM, state, owner, repo, ansi::RST
        );
        return Ok(());
    }

    println!(
        "{}{} {}/{}{}  {}({} issues, showing up to 50){}",
        ansi::BLD, ansi::CYN, owner, repo, ansi::RST,
        ansi::DIM, issues.len(), ansi::RST
    );
    println!(
        "{}{}{}",
        ansi::DIM,
        "─".repeat(80),
        ansi::RST
    );

    for issue in issues {
        let state_label = if issue.state == octocrab::models::IssueState::Open {
            format!("{}{}OPEN  {}", ansi::LGR, ansi::BLD, ansi::RST)
        } else {
            format!("{}{}CLOSED{}", ansi::LRE, ansi::BLD, ansi::RST)
        };

        let num = issue.number;
        let title = &issue.title;

        // Labels
        let _labels_str: String = issue
            .labels
            .iter()
            .map(|l| {
                let name = &l.name;
                format!(" [{}]", name)
            })
            .collect::<Vec<_>>()
            .join("");

        // Author and date
        let user = issue.user.login.as_str();
        let _created = issue
            .created_at
            .format("%Y-%m-%d")
            .to_string();

        // Assignee
        let assignee_str = issue
            .assignees
            .first()
            .map(|a| format!("{}@{}", ansi::LMA, a.login))
            .unwrap_or_else(|| format!("{}-{}", ansi::DIM, ansi::RST));

        // Labels (compact, first 3 max)
        let label_str: String = {
            let labels: Vec<String> = issue
                .labels
                .iter()
                .take(3)
                .map(|l| {
                    let name = &l.name;
                    format!("{}{}[{}]{}", ansi::BLD, ansi::LBL, name, ansi::RST)
                })
                .collect();
            if issue.labels.len() > 3 {
                format!("{} {}…{}", labels.join(" "), ansi::DIM, ansi::RST)
            } else {
                labels.join(" ")
            }
        };

        println!(
            "  {}#{}  {} {}{}{}  {}  {}by {}{}  {}",
            ansi::LYL,
            num,
            state_label,
            ansi::BLD,
            ansi::WHT,
            title,
            ansi::RST,
            assignee_str,
            ansi::GRY,
            user,
            ansi::RST
        );

        if !issue.labels.is_empty() {
            println!(
                "  {}↳ Labels:{} {}",
                ansi::DIM, ansi::RST, label_str
            );
        }
    }

    // Pagination info
    if page.next.is_some() {
        println!(
            "  {}(more results available on next page){}",
            ansi::DIM, ansi::RST
        );
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Create Issue
// ═══════════════════════════════════════════════════════════════════════════

pub async fn create_issue(
    owner: &str,
    repo: &str,
    title: &str,
    body: &str,
) -> Result<(), String> {
    let octo = build_octocrab()?;

    let issue = octo
        .issues(owner, repo)
        .create(title)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Failed to create issue: {}", e))?;

    let url = issue.html_url.to_string();
    let num = issue.number;

    println!(
        "{}✓{} Created issue {}#{} {}",
        ansi::LGR, ansi::RST,
        ansi::BLD, num, ansi::RST
    );
    println!("  Title: {}", title);
    println!("  URL:   {}", url);

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Open Issues View (from TUI)
// ═══════════════════════════════════════════════════════════════════════════

pub fn open_issues_view(app: &mut AppState) {
    // Detect owner/repo
    let (owner, repo) = if let Some(r) = detect_owner_repo() {
        (r.owner, r.repo)
    } else {
        app.issues_lines = vec![format!(
            "  {}No GitHub remote detected. Use `gitscope issues --owner O --repo R` from CLI.{}",
            ansi::DIM, ansi::RST
        )];
        app.issues_scroll = 0;
        app.mode = crate::app::AppMode::Issues;
        return;
    };

    // Check for token — try unauthenticated first for read-only public access
    if crate::github::get_token().is_none() {
        // Try without token for public repos
        let state = app.issues_state.clone();

        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let result = rt.block_on(async {
            let octo = build_octocrab_unauthed();

            let state_filter = match state.as_str() {
                "closed" => Some(State::Closed),
                "all" => None,
                _ => Some(State::Open),
            };

            let issues_list = octo.issues(&owner, &repo);
            let mut req = issues_list
                .list()
                .per_page(50)
                .page(1u32);

            if let Some(s) = state_filter {
                req = req.state(s);
            }

            let page = req
                .send()
                .await
                .map_err(|e| format!("{}", e))?;

            Ok::<_, String>(page)
        });

        match result {
            Ok(page) => {
                format_issues_lines(app, page, &owner, &repo);
                return;
            }
            Err(_) => {
                // Token needed — show helpful message
                app.issues_lines = vec![
                    format!(
                        "  {}This repository may require authentication.{}",
                        ansi::LRE, ansi::RST
                    ),
                    format!(
                        "  {}Set GITHUB_TOKEN or GH_TOKEN env var, or run `gh auth login`{}",
                        ansi::DIM, ansi::RST
                    ),
                ];
                app.issues_scroll = 0;
                app.mode = crate::app::AppMode::Issues;
                return;
            }
        }
    }

    let state = app.issues_state.clone();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let result = rt.block_on(async {
        let octo = build_octocrab()?;

        let state_filter = match state.as_str() {
            "closed" => Some(State::Closed),
            "all" => None,
            _ => Some(State::Open),
        };

        let issues_list = octo.issues(&owner, &repo);
        let mut req = issues_list
            .list()
            .per_page(50)
            .page(1u32);

        if let Some(s) = state_filter {
            req = req.state(s);
        }

        let page = req
            .send()
            .await
            .map_err(|e| format!("{}", e))?;

        Ok::<_, String>(page)
    });

    let page = match result {
        Ok(p) => p,
        Err(e) => {
            app.issues_lines = vec![format!(
                "  {}Error fetching issues: {}{}",
                ansi::LRE, e, ansi::RST
            )];
            app.issues_scroll = 0;
            app.mode = crate::app::AppMode::Issues;
            return;
        }
    };

    let issues = &page.items;

    if issues.is_empty() {
        app.issues_lines = vec![format!(
            "  {}No {} issues found for {}/{}{}",
            ansi::DIM, state, owner, repo, ansi::RST
        )];
        app.issues_scroll = 0;
        app.mode = crate::app::AppMode::Issues;
        return;
    }

    // Format issues into lines
    let mut lines: Vec<String> = Vec::new();

    let state_label = match app.issues_state.as_str() {
        "closed" => "closed",
        "all" => "all",
        _ => "open",
    };

    lines.push(format!(
        "{}{} {}/{}{}  {}({} {}){}{}  {}s{} toggle  {}q{} back{}",
        ansi::BLD, ansi::CYN, owner, repo, ansi::RST,
        ansi::DIM, issues.len(), state_label, ansi::RST,
        ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST
    ));
    lines.push(format!(
        "{}{}{}",
        ansi::DIM,
        "─".repeat(80.min(app.term_w as usize)),
        ansi::RST
    ));

    for issue in issues {
        let state_label = if issue.state == octocrab::models::IssueState::Open {
            format!("{}{}OPEN  {}", ansi::LGR, ansi::BLD, ansi::RST)
        } else {
            format!("{}{}CLOSED{}", ansi::LRE, ansi::BLD, ansi::RST)
        };

        let num = issue.number;
        let title = &issue.title;
        let user = issue.user.login.as_str();
        let _created = issue
            .created_at
            .format("%Y-%m-%d")
            .to_string();

        // Assignee
        let assignee_str = issue
            .assignees
            .first()
            .map(|a| format!("{}@{}", ansi::LMA, a.login))
            .unwrap_or_else(|| format!("{}-{}", ansi::DIM, ansi::RST));

        // Labels (compact, first 2 max)
        let label_str: String = {
            let labels: Vec<String> = issue
                .labels
                .iter()
                .take(2)
                .map(|l| {
                    let name = &l.name;
                    format!("{}{}[{}]{}", ansi::BLD, ansi::LBL, name, ansi::RST)
                })
                .collect();
            if issue.labels.len() > 2 {
                format!("{} {}…{}", labels.join(" "), ansi::DIM, ansi::RST)
            } else {
                labels.join(" ")
            }
        };

        lines.push(format!(
            "  {}#{}  {} {}{}{}  {}  {}by {}{}  {}",
            ansi::LYL, num, state_label,
            ansi::BLD, ansi::WHT, title, ansi::RST,
            assignee_str,
            ansi::GRY, user, ansi::RST
        ));

        // If there are labels, add a second line for them
        if !issue.labels.is_empty() {
            lines.push(format!(
                "  {}↳ Labels:{} {}",
                ansi::DIM, ansi::RST, label_str
            ));
        }
    }

    app.issues_lines = lines;
    app.issues_lines_full = app.issues_lines.clone();
    app.issues_scroll = 0;
    app.mode = crate::app::AppMode::Issues;
}

// ═══════════════════════════════════════════════════════════════════════════
// Issues Filtering
// ═══════════════════════════════════════════════════════════════════════════

/// Filter the issues lines by the given filter text (case-insensitive match).
/// The first two lines (header and separator) are always kept.
pub fn apply_issues_filter(app: &mut AppState) {
    let filter = app.issues_filter_text.to_lowercase();
    if filter.is_empty() {
        app.issues_lines = app.issues_lines_full.clone();
        return;
    }

    let mut filtered: Vec<String> = Vec::new();
    // Always keep the header lines (first 2 lines)
    for (i, line) in app.issues_lines_full.iter().enumerate() {
        if i < 2 {
            filtered.push(line.clone());
            continue;
        }
        if line.to_lowercase().contains(&filter) {
            filtered.push(line.clone());
        }
    }

    // Clamp cursor and scroll to new filtered list
    app.issues_lines = filtered;
    if app.issues_cursor >= app.issues_lines.len() {
        app.issues_cursor = app.issues_lines.len().saturating_sub(1);
    }
    if app.issues_scroll >= app.issues_lines.len() {
        app.issues_scroll = app.issues_lines.len().saturating_sub(1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Issue Detail (from TUI)
// ═══════════════════════════════════════════════════════════════════════════

/// Open issue detail view for the selected issue
pub fn open_issue_detail(app: &mut AppState) {
    // Detect owner/repo
    let (owner, repo) = if let Some(r) = detect_owner_repo() {
        (r.owner, r.repo)
    } else {
        app.msg = "No GitHub remote detected".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    };

    // Parse issue number from the selected line
    let cursor_idx = app.issues_cursor;
    let lines = &app.issues_lines;
    if cursor_idx >= lines.len() {
        return;
    }
    let line = &lines[cursor_idx];

    // Extract issue number from format: "  #2  OPEN   okay..."
    let issue_num = line
        .split('#')
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<u64>().ok());

    let issue_num = match issue_num {
        Some(n) => n,
        None => {
            app.msg = "Could not parse issue number".to_string();
            app.msg_time = Some(std::time::Instant::now());
            return;
        }
    };

    // Reset cursor and scroll for the detail view
    app.issue_detail_cursor = 0;
    app.issue_detail_scroll = 0;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    // Try unauthenticated first
    if get_token().is_none() {
        let result = rt.block_on(async {
            let octo = build_octocrab_unauthed();
            let issue = octo
                .issues(&owner, &repo)
                .get(issue_num)
                .await
                .map_err(|e| format!("{}", e))?;
            Ok::<_, String>(issue)
        });

        match result {
            Ok(issue) => {
                format_issue_detail(app, &issue);
                return;
            }
            Err(_) => {
                app.msg = "Authentication required to view issue details".to_string();
                app.msg_time = Some(std::time::Instant::now());
                return;
            }
        }
    }

    // Authenticated path
    let result = rt.block_on(async {
        let octo = build_octocrab()?;
        let issue = octo
            .issues(&owner, &repo)
            .get(issue_num)
            .await
            .map_err(|e| format!("{}", e))?;
        Ok::<_, String>(issue)
    });

    match result {
        Ok(issue) => {
            format_issue_detail(app, &issue);
        }
        Err(e) => {
            app.msg = format!("Error fetching issue: {}", e);
            app.msg_time = Some(std::time::Instant::now());
        }
    }
}

/// Format a single issue into detail lines
fn format_issue_detail(app: &mut AppState, issue: &octocrab::models::issues::Issue) {
    let mut lines: Vec<String> = Vec::new();

    // Header
    let state_label = if issue.state == octocrab::models::IssueState::Open {
        format!("{}{}OPEN{}", ansi::LGR, ansi::BLD, ansi::RST)
    } else {
        format!("{}{}CLOSED{}", ansi::LRE, ansi::BLD, ansi::RST)
    };

    lines.push(format!(
        "{}{} #{}  {}  {}",
        ansi::BLD, ansi::CYN, issue.number, state_label, issue.title
    ));

    // Author and dates
    let user = issue.user.login.as_str();
    let created = issue
        .created_at
        .format("%Y-%m-%d")
        .to_string();
    let updated = issue
        .updated_at
        .format("%Y-%m-%d")
        .to_string();

    lines.push(format!(
        "  {}by {}{}  created: {}  updated: {}",
        ansi::GRY, user, ansi::RST, created, updated
    ));

    // Assignees
    if !issue.assignees.is_empty() {
        let assignee_list: Vec<String> = issue.assignees
            .iter()
            .map(|a| format!("{}@{}", ansi::LMA, a.login))
            .collect();
        lines.push(format!(
            "  {}Assigned:{} {}",
            ansi::BLD, ansi::RST, assignee_list.join(" ")
        ));
    }

    // Milestone
    if let Some(ref milestone) = issue.milestone {
        lines.push(format!(
            "  {}Milestone:{} {}{}{}",
            ansi::BLD, ansi::RST,
            ansi::LCY, milestone.title, ansi::RST
        ));
    }

    // Comments
    if issue.comments > 0 {
        lines.push(format!(
            "  {}Comments:{} {}{}{}",
            ansi::BLD, ansi::RST,
            ansi::LYL, issue.comments, ansi::RST
        ));
    }

    // PR indicator
    if issue.pull_request.is_some() {
        lines.push(format!(
            "  {}Type:{} {}Pull Request{}",
            ansi::BLD, ansi::RST,
            ansi::LMA, ansi::RST
        ));
    }

    // Labels
    if !issue.labels.is_empty() {
        let labels_str: String = issue
            .labels
            .iter()
            .map(|l| {
                let name = &l.name;
                format!("{}{}[{}]{} ", ansi::BLD, ansi::LBL, name, ansi::RST)
            })
            .collect::<Vec<_>>()
            .join("");
        lines.push(format!("  {}Labels:{} {}", ansi::BLD, ansi::RST, labels_str));
    }

    // URL
    let url = issue.html_url.to_string();
    lines.push(format!("  {}URL:{}  {}", ansi::BLD, ansi::RST, url));

    // Separator
    lines.push(format!("{}", ansi::DIM));
    lines.push(format!("{}", ansi::RST));

    // Body
    if let Some(ref body) = issue.body {
        if !body.is_empty() {
            lines.push(format!("{}{}Body:{}", ansi::BLD, ansi::LGR, ansi::RST));
            for body_line in body.lines() {
                lines.push(format!("  {}", body_line));
            }
        } else {
            lines.push(format!(
                "  {}(no description provided){}",
                ansi::DIM, ansi::RST
            ));
        }
    } else {
        lines.push(format!(
            "  {}(no description provided){}",
            ansi::DIM, ansi::RST
        ));
    }

    // Separator
    lines.push(format!("{}", ansi::DIM));
    lines.push(format!("{}", ansi::RST));

    app.issue_detail_lines = lines;
    app.issue_detail_scroll = 0;
    app.mode = crate::app::AppMode::IssueDetail;
}

// ═══════════════════════════════════════════════════════════════════════════
// Create Issue from TUI
// ═══════════════════════════════════════════════════════════════════════════

/// Switch to issue creation mode (input title)
pub fn start_create_issue(app: &mut AppState) {
    app.issue_create_title.clear();
    app.issue_create_body.clear();
    app.issue_create_focus_title = true;
    app.mode = crate::app::AppMode::IssueCreate;
}

/// Submit the issue from TUI state
pub(crate) fn submit_issue_from_tui(app: &mut AppState) {
    let title = app.issue_create_title.trim().to_string();
    if title.is_empty() {
        app.msg = "Title cannot be empty".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    }

    let (owner, repo) = if let Some(r) = detect_owner_repo() {
        (r.owner, r.repo)
    } else {
        app.msg = "No GitHub remote detected".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    };

    let body = app.issue_create_body.trim().to_string();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let result = rt.block_on(create_issue_impl(&owner, &repo, &title, &body));

    match result {
        Ok(num) => {
            app.msg = format!("Issue #{} created!", num);
            app.msg_time = Some(std::time::Instant::now());
            app.issue_create_title.clear();
            app.issue_create_body.clear();
            app.mode = crate::app::AppMode::Issues;
            // Refresh the issues list
            open_issues_view(app);
        }
        Err(e) => {
            app.msg = format!("Error: {}", e);
            app.msg_time = Some(std::time::Instant::now());
        }
    }
}

/// Internal helper that creates an issue and returns the issue number
async fn create_issue_impl(owner: &str, repo: &str, title: &str, body: &str) -> Result<u64, String> {
    let octo = build_octocrab()?;

    let issue = octo
        .issues(owner, repo)
        .create(title)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Failed to create issue: {}\n\nSet GITHUB_TOKEN or GH_TOKEN env var, or run: gh auth login", e))?;

    let num = issue.number;

    Ok(num)
}

/// Start editing an issue from the TUI (fetches current issue data and switches to edit mode)
pub(crate) fn start_edit_issue(app: &mut AppState) {
    // Parse issue number from the selected line
    let cursor = app.issues_cursor;
    if cursor >= app.issues_lines.len() {
        app.msg = "No issue selected".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    }

    let line = &app.issues_lines[cursor];
    let issue_num: u64 = {
        let mut num = 0u64;
        let mut finding = false;
        let mut digits = String::new();
        for c in line.chars() {
            if c == '#' {
                finding = true;
                continue;
            }
            if finding {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else if !digits.is_empty() {
                    num = digits.parse().unwrap_or(0);
                    break;
                } else {
                    break;
                }
            }
        }
        num
    };

    if issue_num == 0 {
        app.msg = "Could not parse issue number".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    }

    // Resolve owner/repo
    let (owner, repo) = if let Some(r) = detect_owner_repo() {
        (r.owner, r.repo)
    } else {
        app.msg = "No GitHub remote detected".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    };

    // Fetch current issue details
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let result = rt.block_on(async {
        if let Some(token) = get_token() {
            let octo = Octocrab::builder()
                .personal_token(token)
                .build()
                .map_err(|e| format!("{}", e))?;
            let issue = octo
                .issues(&owner, &repo)
                .get(issue_num)
                .await
                .map_err(|e| format!("{}", e))?;
            Ok::<_, String>(issue)
        } else {
            let octo = Octocrab::default();
            let issue = octo
                .issues(&owner, &repo)
                .get(issue_num)
                .await
                .map_err(|e| format!("{}", e))?;
            Ok::<_, String>(issue)
        }
    });

    match result {
        Ok(issue) => {
            app.issue_edit_number = issue_num;
            app.issue_edit_title = issue.title.clone();
            app.issue_edit_body = issue.body.unwrap_or_default();
            app.issue_edit_focus_title = true;
            app.mode = crate::app::AppMode::IssueEdit;
            app.dirty = true;
        }
        Err(e) => {
            app.msg = format!("Error fetching issue: {}", e);
            app.msg_time = Some(std::time::Instant::now());
        }
    }
}

/// Submit the edited issue from TUI state using `gh issue edit`
pub(crate) fn update_issue_from_tui(app: &mut AppState) {
    let title = app.issue_edit_title.trim().to_string();
    if title.is_empty() {
        app.msg = "Title cannot be empty".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    }

    let (owner, repo) = if let Some(r) = detect_owner_repo() {
        (r.owner, r.repo)
    } else {
        app.msg = "No GitHub remote detected".to_string();
        app.msg_time = Some(std::time::Instant::now());
        return;
    };

    let issue_num = app.issue_edit_number;
    let body = app.issue_edit_body.trim().to_string();

    let mut args = vec![
        "issue".to_string(),
        "edit".to_string(),
        issue_num.to_string(),
        "--repo".to_string(),
        format!("{}/{}", owner, repo),
        "--title".to_string(),
        title,
    ];

    if !body.is_empty() {
        args.push("--body".to_string());
        args.push(body);
    }

    let output = std::process::Command::new("gh")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run `gh`: {}", e));

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            app.msg = format!("Error updating issue: {}", e);
            app.msg_time = Some(std::time::Instant::now());
            app.mode = crate::app::AppMode::Issues;
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        app.msg = format!("Error updating issue: {}", detail);
        app.msg_time = Some(std::time::Instant::now());
        app.mode = crate::app::AppMode::Issues;
        return;
    }

    app.msg = format!("Issue #{} updated!", issue_num);
    app.msg_time = Some(std::time::Instant::now());
    app.mode = crate::app::AppMode::Issues;
    // Refresh the issues list
    open_issues_view(app);
}


/// Delete an issue from the TUI
pub(crate) fn delete_issue_from_tui(app: &mut AppState) {
    // Parse issue number from the selected line (format: "  #NUMBER  STATE  ...")
    let cursor = app.issues_cursor;
    if cursor >= app.issues_lines.len() {
        app.msg = "No issue selected".to_string();
        app.msg_time = Some(std::time::Instant::now());
        app.confirm_delete_issue = false;
        return;
    }

    let line = &app.issues_lines[cursor];
    let issue_num: u64 = {
        let mut num = 0u64;
        let mut finding = false;
        let mut digits = String::new();
        for c in line.chars() {
            if c == '#' {
                finding = true;
                continue;
            }
            if finding {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else if !digits.is_empty() {
                    num = digits.parse().unwrap_or(0);
                    break;
                } else {
                    break;
                }
            }
        }
        num
    };

    if issue_num == 0 {
        app.msg = "Could not parse issue number".to_string();
        app.msg_time = Some(std::time::Instant::now());
        app.confirm_delete_issue = false;
        return;
    }

    // Resolve owner/repo
    let (owner, repo) = if let Some(r) = detect_owner_repo() {
        (r.owner, r.repo)
    } else {
        app.msg = "No GitHub remote detected".to_string();
        app.msg_time = Some(std::time::Instant::now());
        app.confirm_delete_issue = false;
        return;
    };

    let result = delete_issue_impl(&owner, &repo, issue_num);

    match result {
        Ok(_) => {
            app.msg = format!("Issue #{} deleted!", issue_num);
            app.msg_time = Some(std::time::Instant::now());
            app.confirm_delete_issue = false;
            if app.issues_cursor > 0 {
                app.issues_cursor -= 1;
            }
            // Refresh the issues list
            open_issues_view(app);
        }
        Err(e) => {
            app.msg = format!("Error deleting issue: {}", e);
            app.msg_time = Some(std::time::Instant::now());
            app.confirm_delete_issue = false;
        }
    }
}

/// Internal helper that closes an issue using the `gh` CLI.
/// Using `gh issue close` is more reliable than the octocrab REST API because
/// the CLI handles authentication natively with the user's existing `gh` session.
fn delete_issue_impl(owner: &str, repo: &str, issue_number: u64) -> Result<(), String> {
    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "close",
            &issue_number.to_string(),
            "--repo",
            &format!("{}/{}", owner, repo),
        ])
        .output()
        .map_err(|e| format!("Failed to run `gh`: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Err(format!(
            "gh issue close failed: {}\nMake sure you are authenticated with `gh auth login` and have write access to the repository.",
            detail
        ));
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Helper: Format issues page into lines (shared between authed and unauthed paths)
// ═══════════════════════════════════════════════════════════════════════════

fn format_issues_lines(
    app: &mut AppState,
    page: octocrab::Page<octocrab::models::issues::Issue>,
    owner: &str,
    repo: &str,
) {
    let issues = &page.items;

    if issues.is_empty() {
        app.issues_lines = vec![format!(
            "  {}No {} issues found for {}/{}{}",
            ansi::DIM, app.issues_state, owner, repo, ansi::RST
        )];
        app.issues_scroll = 0;
        app.mode = crate::app::AppMode::Issues;
        return;
    }

    let mut lines: Vec<String> = Vec::new();

    let state_label = match app.issues_state.as_str() {
        "closed" => "closed",
        "all" => "all",
        _ => "open",
    };

    lines.push(format!(
        "{}{} {}/{}{}  {}({} {}){}{}  {}s{} toggle  {}q{} back{}",
        ansi::BLD, ansi::CYN, owner, repo, ansi::RST,
        ansi::DIM, issues.len(), state_label, ansi::RST,
        ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST
    ));
    lines.push(format!(
        "{}{}{}",
        ansi::DIM,
        "─".repeat(80.min(app.term_w as usize)),
        ansi::RST
    ));

    for issue in issues {
        let state_label = if issue.state == octocrab::models::IssueState::Open {
            format!("{}{}OPEN  {}", ansi::LGR, ansi::BLD, ansi::RST)
        } else {
            format!("{}{}CLOSED{}", ansi::LRE, ansi::BLD, ansi::RST)
        };

        let num = issue.number;
        let title = &issue.title;
        let user = issue.user.login.as_str();
        let _created = issue
            .created_at
            .format("%Y-%m-%d")
            .to_string();

        // Assignee
        let assignee_str = issue
            .assignees
            .first()
            .map(|a| format!("{}@{}", ansi::LMA, a.login))
            .unwrap_or_else(|| format!("{}-{}", ansi::DIM, ansi::RST));

        // Labels (compact, first 2 max)
        let label_str: String = {
            let labels: Vec<String> = issue
                .labels
                .iter()
                .take(2)
                .map(|l| {
                    let name = &l.name;
                    format!("{}{}[{}]{}", ansi::BLD, ansi::LBL, name, ansi::RST)
                })
                .collect();
            if issue.labels.len() > 2 {
                format!("{} {}…{}", labels.join(" "), ansi::DIM, ansi::RST)
            } else {
                labels.join(" ")
            }
        };

        lines.push(format!(
            "  {}#{}  {} {}{}{}  {}  {}by {}{}  {}",
            ansi::LYL, num, state_label,
            ansi::BLD, ansi::WHT, title, ansi::RST,
            assignee_str,
            ansi::GRY, user, ansi::RST
        ));

        // If there are labels, add a second line for them
        if !issue.labels.is_empty() {
            lines.push(format!(
                "  {}↳ Labels:{} {}",
                ansi::DIM, ansi::RST, label_str
            ));
        }
    }

    app.issues_lines = lines;
    app.issues_lines_full = app.issues_lines.clone();
    app.issues_scroll = 0;
    app.mode = crate::app::AppMode::Issues;
}

