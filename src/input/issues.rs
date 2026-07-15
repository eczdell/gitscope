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

