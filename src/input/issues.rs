use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AppMode, AppState};

// ═══════════════════════════════════════════════════════════════════════════
// Issues
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_issues_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('/') => {
            app.issues_filter_input.clear();
            app.mode = AppMode::IssuesFilter;
            app.dirty = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.issues_cursor += 1;
            let max = app.issues_lines.len().saturating_sub(1);
            if app.issues_cursor > max {
                app.issues_cursor = max;
            }
            let vis = (app.term_h as usize).max(1).saturating_sub(6);
            if app.issues_cursor >= app.issues_scroll + vis {
                app.issues_scroll = app.issues_cursor.saturating_sub(vis) + 1;
            }
            app.dirty = true;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.issues_cursor = app.issues_cursor.saturating_sub(1);
            if app.issues_cursor < app.issues_scroll {
                app.issues_scroll = app.issues_cursor;
            }
            app.dirty = true;
        }
        KeyCode::Char('J') => {
            app.issues_cursor += 5;
            let max = app.issues_lines.len().saturating_sub(1);
            if app.issues_cursor > max {
                app.issues_cursor = max;
            }
            let vis = (app.term_h as usize).max(1).saturating_sub(6);
            if app.issues_cursor >= app.issues_scroll + vis {
                app.issues_scroll = app.issues_cursor.saturating_sub(vis) + 1;
            }
            app.dirty = true;
        }
        KeyCode::Char('K') => {
            app.issues_cursor = app.issues_cursor.saturating_sub(5);
            if app.issues_cursor < app.issues_scroll {
                app.issues_scroll = app.issues_cursor;
            }
            app.dirty = true;
        }
        KeyCode::Char('s') => {
            app.issues_state = match app.issues_state.as_str() {
                "open" => "closed".to_string(),
                "closed" => "all".to_string(),
                _ => "open".to_string(),
            };
            app.issues_cursor = 0;
            app.issues_scroll = 0;
            crate::github::open_issues_view(app);
            app.dirty = true;
        }
        KeyCode::Char('c') => {
            crate::github::start_create_issue(app);
            app.dirty = true;
        }
        KeyCode::Char('e') => {
            crate::github::start_edit_issue(app);
            app.dirty = true;
        }
        KeyCode::Enter => {
            crate::github::open_issue_detail(app);
            app.dirty = true;
        }
        KeyCode::Char('x') => {
            if app.confirm_delete_issue {
                crate::github::delete_issue_from_tui(app);
            } else {
                app.confirm_delete_issue = true;
                let cursor = app.issues_cursor;
                let line = app.issues_lines.get(cursor).map(|s| s.clone()).unwrap_or_default();
                let mut finding = false;
                let mut digits = String::new();
                for c in line.chars() {
                    if c == '#' { finding = true; continue; }
                    if finding {
                        if c.is_ascii_digit() { digits.push(c); }
                        else if !digits.is_empty() { break; }
                        else { break; }
                    }
                }
                let num = digits.parse::<u64>().unwrap_or(0);
                app.msg = format!("Press x again to delete issue #{}", num);
                app.msg_time = Some(Instant::now());
            }
            app.dirty = true;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.mode = AppMode::Tree;
            app.confirm_delete_issue = false;
            app.dirty = true;
        }
        KeyCode::Char('l') => {
            app.issues_label_filter_input.clear();
            app.mode = AppMode::IssuesLabelFilter;
            app.dirty = true;
        }
        KeyCode::Char('p') => {
            app.issues_project_status_filter_input.clear();
            app.mode = AppMode::IssuesProjectStatusFilter;
            app.dirty = true;
        }
        KeyCode::Char('t') => {
            app.issues_date_filter = match app.issues_date_filter.as_str() {
                "" => "today".to_string(),
                "today" => "week".to_string(),
                "week" => "month".to_string(),
                "month" => "year".to_string(),
                _ => String::new(),
            };
            let msg = if app.issues_date_filter.is_empty() {
                "Date filter: off".to_string()
            } else {
                format!("Date filter: {}", app.issues_date_filter)
            };
            app.msg = msg;
            app.msg_time = Some(Instant::now());
            app.issues_cursor = 0;
            app.issues_scroll = 0;
            app.confirm_delete_issue = false;
            crate::github::open_issues_view(app);
            app.dirty = true;
        }
        KeyCode::Char('g') => {
            app.issues_cursor = 0;
            app.issues_scroll = 0;
            app.confirm_delete_issue = false;
            app.dirty = true;
        }
        KeyCode::Char('G') => {
            app.issues_cursor = app.issues_lines.len().saturating_sub(1);
            app.issues_scroll = app.issues_cursor.saturating_sub(
                ((app.term_h as usize).max(1).saturating_sub(6)).saturating_sub(1),
            );
            app.confirm_delete_issue = false;
            app.dirty = true;
        }
        _ => {
            app.confirm_delete_issue = false;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Issues Filter
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_issues_filter_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.issues_filter_text = app.issues_filter_input.clone();
            app.mode = AppMode::Issues;
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Esc => {
            app.issues_filter_input.clear();
            app.issues_filter_text.clear();
            app.mode = AppMode::Issues;
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Backspace => {
            app.issues_filter_input.pop();
            app.issues_filter_text = app.issues_filter_input.clone();
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Char(c) => {
            app.issues_filter_input.push(c);
            app.issues_filter_text = app.issues_filter_input.clone();
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Issues Label Filter
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_issues_label_filter_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.issues_label_filter = app.issues_label_filter_input.clone();
            app.mode = AppMode::Issues;
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Esc => {
            app.issues_label_filter_input.clear();
            app.issues_label_filter.clear();
            app.mode = AppMode::Issues;
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Backspace => {
            app.issues_label_filter_input.pop();
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Char(c) => {
            app.issues_label_filter_input.push(c);
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Issues Project Status Filter
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_issues_project_status_filter_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.issues_project_status_filter = app.issues_project_status_filter_input.clone();
            app.mode = AppMode::Issues;
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Esc => {
            app.issues_project_status_filter_input.clear();
            app.issues_project_status_filter.clear();
            app.mode = AppMode::Issues;
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Backspace => {
            app.issues_project_status_filter_input.pop();
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        KeyCode::Char(c) => {
            app.issues_project_status_filter_input.push(c);
            crate::github::apply_issues_filter(app);
            app.dirty = true;
        }
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Issue Detail
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_issue_detail_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.issue_detail_cursor += 1;
            let max = app.issue_detail_lines.len().saturating_sub(1);
            if app.issue_detail_cursor > max {
                app.issue_detail_cursor = max;
            }
            let vis = (app.term_h as usize).max(1).saturating_sub(6);
            if app.issue_detail_cursor >= app.issue_detail_scroll + vis {
                app.issue_detail_scroll = app.issue_detail_cursor.saturating_sub(vis) + 1;
            }
            app.dirty = true;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.issue_detail_cursor = app.issue_detail_cursor.saturating_sub(1);
            if app.issue_detail_cursor < app.issue_detail_scroll {
                app.issue_detail_scroll = app.issue_detail_cursor;
            }
            app.dirty = true;
        }
        KeyCode::Char('J') => {
            app.issue_detail_cursor += 5;
            let max = app.issue_detail_lines.len().saturating_sub(1);
            if app.issue_detail_cursor > max {
                app.issue_detail_cursor = max;
            }
            let vis = (app.term_h as usize).max(1).saturating_sub(6);
            if app.issue_detail_cursor >= app.issue_detail_scroll + vis {
                app.issue_detail_scroll = app.issue_detail_cursor.saturating_sub(vis) + 1;
            }
            app.dirty = true;
        }
        KeyCode::Char('K') => {
            app.issue_detail_cursor = app.issue_detail_cursor.saturating_sub(5);
            if app.issue_detail_cursor < app.issue_detail_scroll {
                app.issue_detail_scroll = app.issue_detail_cursor;
            }
            app.dirty = true;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.mode = AppMode::Issues;
            app.dirty = true;
        }
        KeyCode::Char('o') => {
            // Scan issue_detail_lines for image URLs and open with feh
            let urls: Vec<String> = app.issue_detail_lines
                .iter()
                .flat_map(|line| extract_image_urls(line))
                .collect();
            if urls.is_empty() {
                app.msg = "No image URLs found in this issue".to_string();
                app.msg_time = Some(Instant::now());
            } else {
                let count = urls.len();
                for url in &urls {
                    let url = url.clone();
                    std::thread::spawn(move || {
                        std::process::Command::new("feh")
                            .arg(&url)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                            .ok();
                    });
                }
                app.msg = format!("Opening {} image(s) with feh", count);
                app.msg_time = Some(Instant::now());
            }
            app.dirty = true;
        }
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Issue Create
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_issue_create_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            crate::github::submit_issue_from_tui(app);
            app.dirty = true;
        }
        KeyCode::Tab => {
            if app.issue_create_focus == 2 && !app.label_ac_list.is_empty() {
                app.label_ac_idx = (app.label_ac_idx + 1) % app.label_ac_list.len();
                let selected = &app.label_ac_list[app.label_ac_idx];
                app.issue_create_labels_input =
                    replace_label_current_token(&app.issue_create_labels_input, selected);
            } else {
                app.issue_create_focus = (app.issue_create_focus + 1) % 3;
            }
            app.dirty = true;
        }
        KeyCode::Esc => {
            app.mode = AppMode::Issues;
            app.dirty = true;
        }
        KeyCode::Backspace => {
            match app.issue_create_focus {
                0 => { app.issue_create_title.pop(); }
                1 => { app.issue_create_body.pop(); }
                2 => {
                    app.issue_create_labels_input.pop();
                    update_label_ac(app);
                }
                _ => {}
            }
            app.dirty = true;
        }
        KeyCode::Char(c) => {
            match app.issue_create_focus {
                0 => { app.issue_create_title.push(c); }
                1 => { app.issue_create_body.push(c); }
                2 => {
                    app.issue_create_labels_input.push(c);
                    update_label_ac(app);
                }
                _ => {}
            }
            app.dirty = true;
        }
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Issue Edit
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_issue_edit_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            crate::github::update_issue_from_tui(app);
            app.dirty = true;
        }
        KeyCode::Tab => {
            if app.issue_edit_focus == 2 && !app.label_ac_list.is_empty() {
                app.label_ac_idx = (app.label_ac_idx + 1) % app.label_ac_list.len();
                let selected = &app.label_ac_list[app.label_ac_idx];
                app.issue_edit_labels_input =
                    replace_label_current_token(&app.issue_edit_labels_input, selected);
            } else {
                app.issue_edit_focus = (app.issue_edit_focus + 1) % 3;
            }
            app.dirty = true;
        }
        KeyCode::Esc => {
            app.mode = AppMode::Issues;
            app.dirty = true;
        }
        KeyCode::Backspace => {
            match app.issue_edit_focus {
                0 => { app.issue_edit_title.pop(); }
                1 => { app.issue_edit_body.pop(); }
                2 => {
                    app.issue_edit_labels_input.pop();
                    update_label_ac(app);
                }
                _ => {}
            }
            app.dirty = true;
        }
        KeyCode::Char(c) => {
            match app.issue_edit_focus {
                0 => { app.issue_edit_title.push(c); }
                1 => { app.issue_edit_body.push(c); }
                2 => {
                    app.issue_edit_labels_input.push(c);
                    update_label_ac(app);
                }
                _ => {}
            }
            app.dirty = true;
        }
        _ => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Label Autocomplete Helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Get the current token being typed (text after the last comma, trimmed).
fn get_label_current_token(input: &str) -> &str {
    input.rsplit(',').next().map(|s| s.trim()).unwrap_or("")
}

/// Replace the current token (text after the last comma) with a new token.
fn replace_label_current_token(input: &str, new_token: &str) -> String {
    if let Some(last_comma) = input.rfind(',') {
        let prefix = &input[..=last_comma];
        format!("{} {}", prefix, new_token)
    } else {
        new_token.to_string()
    }
}

/// Rebuild the label autocomplete list based on the current label input.
fn update_label_ac(app: &mut AppState) {
    let labels_input = match app.mode {
        AppMode::IssueCreate => &app.issue_create_labels_input,
        AppMode::IssueEdit => &app.issue_edit_labels_input,
        _ => return,
    };

    let token = get_label_current_token(labels_input).to_lowercase();
    if token.is_empty() {
        app.label_ac_list.clear();
        app.label_ac_idx = 0;
        return;
    }

    let matches: Vec<String> = app.available_labels
        .iter()
        .filter(|l| l.to_lowercase().starts_with(&token))
        .cloned()
        .collect();

    app.label_ac_list = matches;
    app.label_ac_idx = 0;
}


// ═══════════════════════════════════════════════════════════════════════════
// Image URL Extraction
// ═══════════════════════════════════════════════════════════════════════════

/// Extract image URLs from a line of text.
/// Matches URLs ending with common image extensions or GitHub user attachment URLs.
fn extract_image_urls(line: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut remaining = line;
    let image_extensions = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico"];

    while let Some(start) = remaining.find("http") {
        let candidate = &remaining[start..];
        // Find end of URL (whitespace, closing bracket, etc.)
        let end = candidate.find(|c: char| c.is_whitespace() || c == ')' || c == '>' || c == '"' || c == '<')
            .unwrap_or(candidate.len());
        let url = &candidate[..end];

        // Check if URL looks like an image
        let lower_url = url.to_lowercase();
        let is_image = image_extensions.iter().any(|ext| {
            lower_url.ends_with(&format!(".{}", ext)) || lower_url.contains(&format!(".{}?", ext))
        });
        let is_github_asset = lower_url.contains("github.com/user-attachments/assets/")
            || lower_url.contains("githubusercontent.com");

        if is_image || is_github_asset {
            // Strip any trailing punctuation
            let clean_url = url.trim_end_matches(&['.', ',', ';', ':', '!', '?'][..]);
            urls.push(clean_url.to_string());
        }

        remaining = &remaining[start + end..];
    }

    urls
}

