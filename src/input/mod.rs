mod gists;
mod issues;
mod report;
mod repos;
mod tree;

pub(crate) use gists::*;
pub(crate) use issues::*;
pub(crate) use report::*;
pub(crate) use repos::*;
pub(crate) use tree::*;

use std::time::Instant;

use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};

use crate::app::{AppMode, AppState};

// ═══════════════════════════════════════════════════════════════════════════
// Message
// ═══════════════════════════════════════════════════════════════════════════

pub fn show_msg(app: &mut AppState, msg: &str) {
    app.msg = msg.to_string();
    app.msg_time = Some(Instant::now());
}

// ═══════════════════════════════════════════════════════════════════════════
// Input Handling - Main Dispatcher
// ═══════════════════════════════════════════════════════════════════════════

pub fn handle_key(app: &mut AppState, key: KeyEvent) -> bool {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return false;
    }

    let should_quit = match &app.mode {
        AppMode::TreeFilter => {
            handle_tree_filter_key(app, key);
            false
        }
        AppMode::Tree | AppMode::Help => !handle_tree_key(app, key),
        AppMode::Diff => {
            handle_diff_key(app, key);
            false
        }
        AppMode::Files => {
            handle_files_key(app, key);
            false
        }
        AppMode::SidebarFocus => {
            handle_sidebar_key(app, key);
            false
        }

        AppMode::ReportFilter => {
            handle_report_filter_key(app, key);
            false
        }
        AppMode::Report => {
            handle_report_key(app, key);
            false
        }

        AppMode::IssuesFilter => {
            handle_issues_filter_key(app, key);
            false
        }
        AppMode::Issues => {
            handle_issues_key(app, key);
            false
        }
        AppMode::IssueDetail => {
            handle_issue_detail_key(app, key);
            false
        }
        AppMode::IssueCreate => {
            handle_issue_create_key(app, key);
            false
        }
        AppMode::IssueEdit => {
            handle_issue_edit_key(app, key);
            false
        }

        AppMode::GistsFilter => {
            handle_gists_filter_key(app, key);
            false
        }
        AppMode::Gists => {
            handle_gists_key(app, key);
            false
        }
        AppMode::GistContent => {
            handle_gist_content_key(app, key);
            false
        }

        AppMode::Repos => {
            handle_repos_key(app, key);
            false
        }
        AppMode::ReposAdd => {
            handle_repos_add_key(app, key);
            false
        }
    };

    if should_quit {
        return false;
    }

    // Update report autocomplete when in ReportFilter mode
    if app.mode == AppMode::ReportFilter && !app.report_email_input.is_empty() {
        let input_lower = app.report_email_input.to_lowercase();
        let output = std::process::Command::new("git")
            .args(["log", "--format=%ae"])
            .output();
        if let Ok(o) = output {
            use std::collections::HashSet;
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

