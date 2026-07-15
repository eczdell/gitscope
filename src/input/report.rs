use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AppMode, AppState};
use crate::git::open_report;

// ═══════════════════════════════════════════════════════════════════════════
// Report Filter
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_report_filter_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Report
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_report_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

