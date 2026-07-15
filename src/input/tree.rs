use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AppMode, AppState};
use crate::clipboard::copy_to_clipboard;
use crate::git::{build_desc_set, open_diff, open_files, open_report};
use crate::input::show_msg;
use crate::render::build_render_buffer;

// ═══════════════════════════════════════════════════════════════════════════
// Tree
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_tree_key(app: &mut AppState, key: KeyEvent) -> bool {
    match key.code {
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
                let today = std::process::Command::new("date")
                    .args(["+%Y-%m-%d"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let week_ago = std::process::Command::new("date")
                    .args(["-d", &format!("{} - 7 days", today), "+%Y-%m-%d"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .or_else(|| {
                        std::process::Command::new("date")
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
            app.mode = AppMode::Repos;
            app.dirty = true;
        }
        KeyCode::Char('\'') => {
            crate::github::open_gists_view(app);
            app.dirty = true;
        }
        _ => {}
    }
    true
}

// ═══════════════════════════════════════════════════════════════════════════
// Tree Filter
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_tree_filter_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Sidebar Focus
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_sidebar_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Diff
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_diff_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Files
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_files_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

