use std::collections::HashSet;
use std::process::Command;
use std::time::Instant;

use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::app::{AppMode, AppState};
use crate::clipboard::copy_to_clipboard;
use crate::git::{build_desc_set, open_diff, open_files, open_report};
use crate::render::build_render_buffer;
use crate::repos;

// ═══════════════════════════════════════════════════════════════════════════
// Message
// ═══════════════════════════════════════════════════════════════════════════

pub fn show_msg(app: &mut AppState, msg: &str) {
    app.msg = msg.to_string();
    app.msg_time = Some(Instant::now());
}

// ═══════════════════════════════════════════════════════════════════════════
// Input Handling
// ═══════════════════════════════════════════════════════════════════════════

pub fn handle_key(app: &mut AppState, key: event::KeyEvent) -> bool {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return false;
    }

    match &app.mode {
        // ─── Tree Filter Mode ───
        AppMode::TreeFilter => match key.code {
            KeyCode::Enter => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            KeyCode::Esc => {
                app.filter_input.clear();
                app.filter_text.clear();
                app.mode = AppMode::Tree;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Backspace => {
                app.filter_input.pop();
                app.filter_text = app.filter_input.clone();
                app.scroll = 0;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                app.filter_input.push(c);
                app.filter_text = app.filter_input.clone();
                app.scroll = 0;
                build_render_buffer(app);
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Issues Filter Mode ───
        AppMode::IssuesFilter => match key.code {
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
        },

        // ─── Diff Mode ───
        AppMode::Diff => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.diff_scroll += 1;
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.diff_scroll = app.diff_scroll.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.diff_scroll += 5;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.diff_scroll = app.diff_scroll.saturating_sub(5);
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            KeyCode::Char('e') => {
                crate::github::start_edit_issue(app);
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Files Mode ───
        AppMode::Files => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.files_scroll += 1;
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.files_scroll = app.files_scroll.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.files_scroll += 5;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.files_scroll = app.files_scroll.saturating_sub(5);
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Report Filter Mode ───
        AppMode::ReportFilter => match key.code {
            KeyCode::Enter => {
                if let Some(first) = app.report_ac_list.first().cloned() {
                    app.report_email_input = first;
                }
                app.report_email_filter = app.report_email_input.clone();
                app.mode = AppMode::Report;
                open_report(app);
                app.dirty = true;
            }
            KeyCode::Tab => {
                if app.report_ac_list.len() == 1 {
                    app.report_email_input = app.report_ac_list[0].clone();
                    app.report_email_filter = app.report_email_input.clone();
                    app.mode = AppMode::Report;
                    open_report(app);
                    app.dirty = true;
                } else if app.report_ac_list.len() > 1 {
                    app.report_ac_idx += 1;
                    if app.report_ac_idx >= app.report_ac_list.len() {
                        app.report_ac_idx = 0;
                    }
                    app.report_email_input = app.report_ac_list[app.report_ac_idx].clone();
                    app.dirty = true;
                }
            }
            KeyCode::Esc => {
                app.report_email_input.clear();
                app.mode = AppMode::Report;
                app.dirty = true;
            }
            KeyCode::Backspace => {
                app.report_email_input.pop();
                app.report_ac_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                app.report_email_input.push(c);
                app.report_ac_idx = 0;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Report Mode ───
        AppMode::Report => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.report_scroll += 1;
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.report_scroll = app.report_scroll.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.report_scroll += 5;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.report_scroll = app.report_scroll.saturating_sub(5);
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.report_scroll = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.report_scroll = usize::MAX;
                app.dirty = true;
            }
            KeyCode::Char('/') => {
                app.report_email_input.clear();
                app.report_ac_idx = 0;
                app.report_ac_list.clear();
                app.mode = AppMode::ReportFilter;
                app.dirty = true;
            }
            KeyCode::Char('t') => {
                if app.report_sort == "name" {
                    app.report_sort = "date".to_string();
                } else {
                    app.report_sort = "name".to_string();
                }
                open_report(app);
                app.dirty = true;
            }
            KeyCode::Char('b') => {
                app.mode = AppMode::SidebarFocus;
                app.report_email_filter.clear();
                app.branch_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.report_email_filter.clear();
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Sidebar Focus ───
        AppMode::SidebarFocus => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.branch_idx += 1;
                if app.branch_idx >= app.branches.len() {
                    app.branch_idx = app.branches.len().saturating_sub(1);
                }
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.branch_idx = app.branch_idx.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.branch_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.branch_idx = app.branches.len().saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Enter => {
                if let Some(br) = app.branches.get(app.branch_idx) {
                    app.branch_filter = br.name.clone();
                }
                app.mode = AppMode::Tree;
                app.refresh = true;
            }
            KeyCode::Char('0') => {
                app.branch_filter.clear();
                app.mode = AppMode::Tree;
                app.refresh = true;
            }
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('q') => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Issues Mode ───
        AppMode::Issues => match key.code {
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
                // Auto-scroll to keep cursor in view
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
                    // Parse issue number for confirmation message
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
                    app.msg_time = Some(std::time::Instant::now());
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
        },

        // ─── Issue Detail Mode ───
        AppMode::IssueDetail => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.issue_detail_cursor += 1;
                let max = app.issue_detail_lines.len().saturating_sub(1);
                if app.issue_detail_cursor > max {
                    app.issue_detail_cursor = max;
                }
                // Auto-scroll: keep cursor in view
                let vis = (app.term_h as usize).max(1).saturating_sub(6);
                if app.issue_detail_cursor >= app.issue_detail_scroll + vis {
                    app.issue_detail_scroll = app.issue_detail_cursor.saturating_sub(vis) + 1;
                }
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.issue_detail_cursor = app.issue_detail_cursor.saturating_sub(1);
                // Auto-scroll: keep cursor in view
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
        },

        // ─── Issue Create Mode ───
        AppMode::IssueCreate => match key.code {
            KeyCode::Enter => {
                crate::github::submit_issue_from_tui(app);
                app.dirty = true;
            }
            KeyCode::Tab => {
                app.issue_create_focus_title = !app.issue_create_focus_title;
                app.dirty = true;
            }
            KeyCode::Esc => {
                app.mode = AppMode::Issues;
                app.dirty = true;
            }
            KeyCode::Backspace => {
                if app.issue_create_focus_title {
                    app.issue_create_title.pop();
                } else {
                    app.issue_create_body.pop();
                }
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                if app.issue_create_focus_title {
                    app.issue_create_title.push(c);
                } else {
                    app.issue_create_body.push(c);
                }
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Issue Edit Mode ───
        AppMode::IssueEdit => match key.code {
            KeyCode::Enter => {
                crate::github::update_issue_from_tui(app);
                app.dirty = true;
            }
            KeyCode::Esc => {
                app.mode = AppMode::Issues;
                app.dirty = true;
            }
            KeyCode::Tab => {
                app.issue_edit_focus_title = !app.issue_edit_focus_title;
                app.dirty = true;
            }
            KeyCode::Backspace => {
                if app.issue_edit_focus_title {
                    app.issue_edit_title.pop();
                } else {
                    app.issue_edit_body.pop();
                }
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                if app.issue_edit_focus_title {
                    app.issue_edit_title.push(c);
                } else {
                    app.issue_edit_body.push(c);
                }
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Gists Filter Mode ───
        AppMode::GistsFilter => match key.code {
            KeyCode::Enter => {
                app.gists_filter_text = app.gists_filter_input.clone();
                app.mode = AppMode::Gists;
                crate::github::apply_gists_filter(app);
                app.dirty = true;
            }
            KeyCode::Esc => {
                app.gists_filter_input.clear();
                app.gists_filter_text.clear();
                app.mode = AppMode::Gists;
                crate::github::apply_gists_filter(app);
                app.dirty = true;
            }
            KeyCode::Backspace => {
                app.gists_filter_input.pop();
                app.gists_filter_text = app.gists_filter_input.clone();
                crate::github::apply_gists_filter(app);
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                app.gists_filter_input.push(c);
                app.gists_filter_text = app.gists_filter_input.clone();
                crate::github::apply_gists_filter(app);
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Gists Mode ───
        AppMode::Gists => match key.code {
            KeyCode::Char('/') => {
                app.gists_filter_input.clear();
                app.mode = AppMode::GistsFilter;
                app.dirty = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.gists_cursor += 1;
                let max = app.gists_lines.len().saturating_sub(1);
                if app.gists_cursor > max {
                    app.gists_cursor = max;
                }
                // Auto-scroll to keep cursor in view
                let vis = (app.term_h as usize).max(1).saturating_sub(6);
                if app.gists_cursor >= app.gists_scroll + vis {
                    app.gists_scroll = app.gists_cursor.saturating_sub(vis) + 1;
                }
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.gists_cursor = app.gists_cursor.saturating_sub(1);
                if app.gists_cursor < app.gists_scroll {
                    app.gists_scroll = app.gists_cursor;
                }
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.gists_cursor += 5;
                let max = app.gists_lines.len().saturating_sub(1);
                if app.gists_cursor > max {
                    app.gists_cursor = max;
                }
                let vis = (app.term_h as usize).max(1).saturating_sub(6);
                if app.gists_cursor >= app.gists_scroll + vis {
                    app.gists_scroll = app.gists_cursor.saturating_sub(vis) + 1;
                }
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.gists_cursor = app.gists_cursor.saturating_sub(5);
                if app.gists_cursor < app.gists_scroll {
                    app.gists_scroll = app.gists_cursor;
                }
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.gists_cursor = 0;
                app.gists_scroll = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.gists_cursor = app.gists_lines.len().saturating_sub(1);
                let vis = (app.term_h as usize).max(1).saturating_sub(6);
                if app.gists_cursor >= app.gists_scroll + vis {
                    app.gists_scroll = app.gists_cursor.saturating_sub(vis) + 1;
                }
                app.dirty = true;
            }
            KeyCode::Char('y') => {
                // Copy gist file content to clipboard
                if let Some(gist_id) = crate::github::extract_gist_id(&app.gists_lines, app.gists_cursor) {
                    match crate::github::get_gist_content(&gist_id) {
                        Ok(content) => {
                            if let Some(ref cmd) = app.clipboard_cmd.clone() {
                                if copy_to_clipboard(cmd, &content) {
                                    show_msg(
                                        app,
                                        &format!("Copied gist {} content to clipboard", &gist_id[..7.min(gist_id.len())]),
                                    );
                                }
                            } else {
                                show_msg(app, "No clipboard tool found");
                            }
                        }
                        Err(e) => {
                            show_msg(app, &format!("Error: {}", e));
                        }
                    }
                } else {
                    show_msg(app, "Could not find gist ID at cursor");
                }
                app.dirty = true;
            }
            KeyCode::Enter => {
                // View gist content
                if let Some(gist_id) = crate::github::extract_gist_id(&app.gists_lines, app.gists_cursor) {
                    crate::github::open_gist_content(app, &gist_id);
                } else {
                    show_msg(app, "Could not find gist ID at cursor");
                }
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── GistContent ───
        AppMode::GistContent => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.gist_content_scroll += 1;
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.gist_content_scroll = app.gist_content_scroll.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.gist_content_scroll += 5;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.gist_content_scroll = app.gist_content_scroll.saturating_sub(5);
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.gist_content_scroll = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.gist_content_scroll = usize::MAX;
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Gists;
                app.gist_content_scroll = 0;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Help ───
        AppMode::Help => match key.code {
            KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Repos Mode ───
        AppMode::Repos => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.repos_cursor += 1;
                let max = app.repos.len().saturating_sub(1);
                if app.repos_cursor > max {
                    app.repos_cursor = max;
                }
                let vis = (app.term_h as usize).max(1).saturating_sub(6);
                if app.repos_cursor >= app.repos_scroll + vis {
                    app.repos_scroll = app.repos_cursor.saturating_sub(vis) + 1;
                }
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.repos_cursor = app.repos_cursor.saturating_sub(1);
                if app.repos_cursor < app.repos_scroll {
                    app.repos_scroll = app.repos_cursor;
                }
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.repos_cursor += 5;
                let max = app.repos.len().saturating_sub(1);
                if app.repos_cursor > max {
                    app.repos_cursor = max;
                }
                let vis = (app.term_h as usize).max(1).saturating_sub(6);
                if app.repos_cursor >= app.repos_scroll + vis {
                    app.repos_scroll = app.repos_cursor.saturating_sub(vis) + 1;
                }
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.repos_cursor = app.repos_cursor.saturating_sub(5);
                if app.repos_cursor < app.repos_scroll {
                    app.repos_scroll = app.repos_cursor;
                }
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.repos_cursor = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.repos_cursor = app.repos.len().saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Enter => {
                if let Some(repo) = app.repos.get(app.repos_cursor) {
                    if repo.is_valid() {
                        match repos::switch_repo(&repo.path) {
                            Ok(()) => {
                                show_msg(app, &format!("Switched to {}", repo.name));
                                app.mode = AppMode::Tree;
                                app.refresh = true;
                            }
                            Err(e) => {
                                show_msg(app, &format!("Error: {}", e));
                            }
                        }
                    } else {
                        show_msg(app, "Cannot switch: repository is invalid");
                    }
                }
                app.dirty = true;
            }
            KeyCode::Char('a') => {
                // Switch to add mode, pre-fill with current directory
                app.repos_add_input = repos::get_cwd();
                app.mode = AppMode::ReposAdd;
                app.dirty = true;
            }
            KeyCode::Char('s') => {
                // Scan current directory for repos
                let cwd = repos::get_cwd();
                let found = repos::scan_directory(&cwd);
                if found.is_empty() {
                    show_msg(app, &format!("No git repos found in {}", cwd));
                } else {
                    // Add any new repos not already in the list
                    let existing_paths: std::collections::HashSet<String> =
                        app.repos.iter().map(|r| r.path.clone()).collect();
                    let mut added = 0;
                    for repo in found {
                        if !existing_paths.contains(&repo.path) {
                            app.repos.push(repo);
                            added += 1;
                        }
                    }
                    show_msg(app, &format!("Found {} new repo(s) in {}", added, cwd));
                }
                app.dirty = true;
            }
            KeyCode::Char('x') => {
                if app.repos_cursor < app.repos.len() {
                    let removed = app.repos.remove(app.repos_cursor);
                    show_msg(app, &format!("Removed {}", removed.name));
                    if app.repos_cursor >= app.repos.len() && !app.repos.is_empty() {
                        app.repos_cursor = app.repos.len() - 1;
                    }
                }
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Repos Add Mode ───
        AppMode::ReposAdd => match key.code {
            KeyCode::Enter => {
                let path = repos::expand_path(&app.repos_add_input);
                match repos::validate_repo_path(&path) {
                    Ok(()) => {
                        // Check if already in list
                        let already_exists = app.repos.iter().any(|r| r.path == path);
                        if already_exists {
                            show_msg(app, "Repository already in list");
                        } else {
                            let entry = repos::RepoEntry::from_path(&path);
                            if entry.is_valid() {
                                show_msg(app, &format!("Added {}", entry.name));
                            } else {
                                let err = entry.error.clone().unwrap_or_default();
                                show_msg(app, &format!("Added {} (error: {})", entry.name, err));
                            }
                            app.repos.push(entry);
                        }
                        app.mode = AppMode::Repos;
                    }
                    Err(e) => {
                        show_msg(app, &format!("Invalid path: {}", e));
                    }
                }
                app.repos_add_input.clear();
                app.dirty = true;
            }
            KeyCode::Esc => {
                app.repos_add_input.clear();
                app.mode = AppMode::Repos;
                app.dirty = true;
            }
            KeyCode::Tab => {
                // Autocomplete from cwd subdirectories
                let dirs = repos::list_subdirs(&repos::get_cwd());
                if !dirs.is_empty() {
                    app.repos_add_input = format!("{}/{}", repos::get_cwd(), dirs[0]);
                }
                app.dirty = true;
            }
            KeyCode::Backspace => {
                app.repos_add_input.pop();
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                app.repos_add_input.push(c);
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Tree Mode (Normal) ───
        AppMode::Tree => match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return false,

            KeyCode::Char('j') | KeyCode::Down => {
                let ci = app.render_lines.get(app.cursor as usize).and_then(|rl| rl.commit_idx);
                let mut n = (app.cursor + 1) as usize;
                while n < app.render_lines.len() {
                    if app.render_lines[n].commit_idx != ci {
                        break;
                    }
                    n += 1;
                }
                if n < app.render_lines.len() {
                    app.cursor = n as isize;
                }
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let ci = app.render_lines.get(app.cursor as usize).and_then(|rl| rl.commit_idx);
                let mut n = (app.cursor as usize).saturating_sub(1);
                loop {
                    if app.render_lines[n].commit_idx != ci {
                        break;
                    }
                    if n == 0 {
                        break;
                    }
                    n -= 1;
                }
                if app.render_lines[n].commit_idx != ci {
                    app.cursor = n as isize;
                }
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                let mut cnt = 0;
                let mut ci = app.render_lines.get(app.cursor as usize).and_then(|rl| rl.commit_idx);
                let mut n = app.cursor as usize;
                while cnt < 5 && n < app.render_lines.len() - 1 {
                    n += 1;
                    if app.render_lines[n].commit_idx != ci {
                        ci = app.render_lines[n].commit_idx;
                        cnt += 1;
                    }
                }
                app.cursor = n as isize;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                let mut cnt = 0;
                let mut ci = app.render_lines.get(app.cursor as usize).and_then(|rl| rl.commit_idx);
                let mut n = app.cursor as usize;
                while cnt > 0 || n == app.cursor as usize {
                    if n == 0 {
                        break;
                    }
                    n -= 1;
                    if app.render_lines[n].commit_idx != ci {
                        ci = app.render_lines[n].commit_idx;
                        cnt += 1;
                    }
                    if cnt >= 5 {
                        break;
                    }
                }
                if n != app.cursor as usize {
                    app.cursor = n as isize;
                }
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.cursor = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.cursor = app.render_lines.len().saturating_sub(1) as isize;
                app.dirty = true;
            }
            KeyCode::PageUp => {
                app.cursor -= (app.term_h as isize) - 4;
                if app.cursor < 0 {
                    app.cursor = 0;
                }
                app.dirty = true;
            }
            KeyCode::PageDown => {
                app.cursor += (app.term_h as isize) - 4;
                let max = app.render_lines.len().saturating_sub(1) as isize;
                if app.cursor > max {
                    app.cursor = max;
                }
                app.dirty = true;
            }
            KeyCode::Home => {
                app.cursor = 0;
                app.dirty = true;
            }
            KeyCode::End => {
                app.cursor = app.render_lines.len().saturating_sub(1) as isize;
                app.dirty = true;
            }

            KeyCode::Char('a') => {
                app.show_all = !app.show_all;
                app.branch_filter.clear();
                app.filter_text.clear();
                app.descendant_filter.clear();
                app.refresh = true;
            }
            KeyCode::Char('m') => {
                app.show_meta = !app.show_meta;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Char('c') => {
                app.compact = !app.compact;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Char('+') => {
                app.count = (app.count + 10).min(200);
                app.refresh = true;
            }
            KeyCode::Char('-') => {
                if app.count > 10 {
                    app.count -= 10;
                    app.refresh = true;
                }
            }
            KeyCode::Char('b') => {
                app.mode = AppMode::SidebarFocus;
                app.branch_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char('0') => {
                app.branch_filter.clear();
                app.filter_text.clear();
                app.descendant_filter.clear();
                app.date_from.clear();
                app.date_to.clear();
                app.refresh = true;
            }
            KeyCode::Char('/') => {
                app.filter_input.clear();
                app.mode = AppMode::TreeFilter;
                app.dirty = true;
            }

            KeyCode::Char('o') | KeyCode::Enter => {
                if app.cursor >= 0 && (app.cursor as usize) < app.render_lines.len() {
                    if let Some(ci) = app.render_lines[app.cursor as usize].commit_idx {
                        app.descendant_filter = app.commits[ci].hash.clone();
                        build_desc_set(app);
                        app.cursor = 0;
                        build_render_buffer(app);
                        app.dirty = true;
                    }
                }
            }

            KeyCode::Esc => {
                if !app.descendant_filter.is_empty() {
                    app.descendant_filter.clear();
                    app.cursor = 0;
                    build_render_buffer(app);
                    app.dirty = true;
                } else if !app.date_from.is_empty() || !app.date_to.is_empty() {
                    app.date_from.clear();
                    app.date_to.clear();
                    app.cursor = 0;
                    build_render_buffer(app);
                    app.dirty = true;
                }
            }

            KeyCode::Char('d') => {
                if app.cursor >= 0 {
                    if let Some(ci) = app
                        .render_lines
                        .get(app.cursor as usize)
                        .and_then(|rl| rl.commit_idx)
                    {
                        open_diff(app, ci);
                        app.dirty = true;
                    }
                }
            }
            KeyCode::Char('f') => {
                if app.cursor >= 0 {
                    if let Some(ci) = app
                        .render_lines
                        .get(app.cursor as usize)
                        .and_then(|rl| rl.commit_idx)
                    {
                        open_files(app, ci);
                        app.dirty = true;
                    }
                }
            }
            KeyCode::Char('y') => {
                if app.cursor >= 0 {
                    if let Some(ci) = app
                        .render_lines
                        .get(app.cursor as usize)
                        .and_then(|rl| rl.commit_idx)
                    {
                        if ci < app.commits.len() {
                            if let Some(ref cmd) = app.clipboard_cmd.clone() {
                                if copy_to_clipboard(cmd, &app.commits[ci].hash) {
                                    show_msg(
                                        app,
                                        &format!("Copied {} to clipboard", &app.commits[ci].hash[..7]),
                                    );
                                }
                            } else {
                                show_msg(app, "No clipboard tool found");
                            }
                        }
                    }
                }
                app.dirty = true;
            }

            KeyCode::Char('D') => {
                if !app.date_from.is_empty() || !app.date_to.is_empty() {
                    app.date_from.clear();
                    app.date_to.clear();
                    show_msg(app, "Date filter cleared");
                } else {
                    let today = Command::new("date")
                        .args(["+%Y-%m-%d"])
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    let week_ago = Command::new("date")
                        .args(["-d", &format!("{} - 7 days", today), "+%Y-%m-%d"])
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                        .or_else(|| {
                            Command::new("date")
                                .args(["-v-7d", "+%Y-%m-%d"])
                                .output()
                                .ok()
                                .and_then(|o| String::from_utf8(o.stdout).ok())
                                .map(|s| s.trim().to_string())
                        })
                        .unwrap_or_default();
                    if !week_ago.is_empty() {
                        app.date_from = week_ago.clone();
                        app.date_to = today;
                        show_msg(app, &format!("Last 7 days: {} → {}", week_ago, app.date_to));
                    }
                }
                app.cursor = 0;
                build_render_buffer(app);
                app.dirty = true;
            }

            KeyCode::Char('?') => {
                app.mode = AppMode::Help;
                app.dirty = true;
            }
            KeyCode::Char('R') => {
                open_report(app);
                app.dirty = true;
            }
            KeyCode::Char('r') => {
                app.refresh = true;
            }
            KeyCode::Char('i') => {
                crate::github::open_issues_view(app);
                app.dirty = true;
            }
            KeyCode::Char('P') => {
                // Switch to repos management view
                app.mode = AppMode::Repos;
                app.dirty = true;
            }
            KeyCode::Char('\'') => {
                // Open gists view
                crate::github::open_gists_view(app);
                app.dirty = true;
            }
            _ => {}
        },
    }

    // Update report autocomplete when in ReportFilter mode
    if app.mode == AppMode::ReportFilter && !app.report_email_input.is_empty() {
        let input_lower = app.report_email_input.to_lowercase();
        let output = Command::new("git")
            .args(["log", "--format=%ae"])
            .output();
        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let mut matches: Vec<String> = stdout
                .lines()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>()
                .into_iter()
                .filter(|e| e.to_lowercase().contains(&input_lower))
                .collect();
            matches.sort();
            matches.truncate(5);
            app.report_ac_list = matches;
        }
    }

    true
}

