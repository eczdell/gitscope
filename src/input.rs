use std::collections::HashSet;
use std::process::Command;
use std::time::Instant;

use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::app::{AppMode, AppState};
use crate::clipboard::copy_to_clipboard;
use crate::git::{build_desc_set, open_diff, open_files, open_report};
use crate::render::build_render_buffer;

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
                // Toggle between title and body
                if app.issue_create_title.is_empty() && app.issue_create_body.is_empty() {
                    // First Tab: skip to body if title is done
                }
                app.dirty = true;
            }
            KeyCode::Esc => {
                app.mode = AppMode::Issues;
                app.dirty = true;
            }
            KeyCode::Backspace => {
                if !app.issue_create_body.is_empty() {
                    app.issue_create_body.pop();
                } else if !app.issue_create_title.is_empty() {
                    app.issue_create_title.pop();
                }
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                if app.issue_create_body.is_empty() && !app.issue_create_title.contains('\n') {
                    if c == '\n' {
                        // do nothing, Enter handled above
                    } else {
                        app.issue_create_title.push(c);
                    }
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

        // ─── Help ───
        AppMode::Help => match key.code {
            KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
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

