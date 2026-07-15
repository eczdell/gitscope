use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{AppMode, AppState};
use crate::clipboard::copy_to_clipboard;
use crate::input::show_msg;

// ═══════════════════════════════════════════════════════════════════════════
// Gists Filter
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_gists_filter_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Gists
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_gists_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Gist Content
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn handle_gist_content_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
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
    }
}

