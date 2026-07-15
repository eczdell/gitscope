use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AppMode, AppState};
use crate::input::show_msg;

// ═══════════════════════════════════════════════════════════════════════════
// Repos
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_repos_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
                    match crate::repos::switch_repo(&repo.path) {
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
            app.repos_add_input = crate::repos::get_cwd();
            app.mode = AppMode::ReposAdd;
            app.dirty = true;
        }
        KeyCode::Char('s') => {
            // Scan current directory for repos
            let cwd = crate::repos::get_cwd();
            let found = crate::repos::scan_directory(&cwd);
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Repos Add
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_repos_add_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            let path = crate::repos::expand_path(&app.repos_add_input);
            match crate::repos::validate_repo_path(&path) {
                Ok(()) => {
                    // Check if already in list
                    let already_exists = app.repos.iter().any(|r| r.path == path);
                    if already_exists {
                        show_msg(app, "Repository already in list");
                    } else {
                        let entry = crate::repos::RepoEntry::from_path(&path);
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
            let dirs = crate::repos::list_subdirs(&crate::repos::get_cwd());
            if !dirs.is_empty() {
                app.repos_add_input = format!("{}/{}", crate::repos::get_cwd(), dirs[0]);
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
    }
}

