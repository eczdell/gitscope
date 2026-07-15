use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use crate::ansi;
use crate::app::AppState;
use crate::ui::ansi_to_line;

// ═══════════════════════════════════════════════════════════════════════════
// Repos View (multi-repo management)
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_repos_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.repos.len();
    let max_s = total.saturating_sub(vis);

    // Auto-scroll to keep cursor visible
    let mut scroll = app.repos_scroll;
    if app.repos_cursor < scroll {
        scroll = app.repos_cursor;
    }
    if app.repos_cursor >= scroll + vis {
        scroll = app.repos_cursor.saturating_sub(vis - 1);
    }
    scroll = scroll.min(max_s);

    let cursor_bg = Style::default().bg(Color::DarkGray);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut drawn = 0;

    // Header
    lines.push(ansi_to_line(&format!(
        "  {}Registered Repositories{}   {}({} total){}",
        ansi::BLD, ansi::RST, ansi::DIM, total, ansi::RST
    )));
    lines.push(ansi_to_line(&format!(
        "  {}──────────────────────────────────────────────{}",
        ansi::DIM, ansi::RST
    )));
    drawn += 2;

    // Scroll up indicator
    if scroll > 0 {
        let indicator = format!("  {}↑ {} more{}", ansi::DIM, scroll, ansi::RST);
        lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    if app.repos.is_empty() {
        lines.push(ansi_to_line(&format!(
            "  {}  No repositories registered.{}",
            ansi::DIM, ansi::RST
        )));
        lines.push(ansi_to_line(&format!(
            "  {}  Press 'a' to add the current directory or 's' to scan a directory.{}",
            ansi::DIM, ansi::RST
        )));
        drawn += 2;
    } else {
        for idx in scroll..total {
            if drawn >= vis {
                break;
            }
            let repo = &app.repos[idx];
            let mut line_str = String::new();

            // Cursor marker
            if idx == app.repos_cursor {
                line_str.push_str(&format!("{}▸{} ", ansi::LYL, ansi::RST));
            } else {
                line_str.push_str("  ");
            }

            // Valid indicator
            if repo.is_valid() {
                line_str.push_str(&format!("{}●{} ", ansi::LGR, ansi::RST));
            } else {
                line_str.push_str(&format!("{}○{} ", ansi::LRE, ansi::RST));
            }

            // Name
            line_str.push_str(&format!("{}{}{}", ansi::BLD, repo.name, ansi::RST));

            // Path
            line_str.push_str(&format!(
                "  {}{}{}",
                ansi::DIM, repo.path, ansi::RST
            ));

            // Commit count or error
            if let Some(ref err) = repo.error {
                line_str.push_str(&format!("  {}[{}]{}", ansi::LRE, err, ansi::RST));
            } else {
                line_str.push_str(&format!(
                    "  {}· {} commits{}",
                    ansi::GRY, repo.commit_count, ansi::RST
                ));
            }

            let mut line = ansi_to_line(&line_str);
            if idx == app.repos_cursor {
                let mut new_spans = vec![Span::styled(
                    "▸ ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )];
                for span in line.spans.iter() {
                    new_spans.push(Span::styled(
                        span.content.clone(),
                        span.style.patch(cursor_bg),
                    ));
                }
                line = Line::from(new_spans);
            }
            lines.push(line);
            drawn += 1;
        }
    }

    // Scroll down indicator
    if scroll + drawn < total {
        let remaining = total - scroll - drawn;
        let indicator = format!("  {}↓ {} more{}", ansi::DIM, remaining, ansi::RST);
        lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    for _ in drawn..vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════
// Repos Add View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_repos_add_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(ansi_to_line(&format!(
        "  {}Add a Repository{}",
        ansi::BLD, ansi::RST
    )));
    lines.push(ansi_to_line(&format!(
        "  {}─────────────────────{}",
        ansi::DIM, ansi::RST
    )));
    lines.push(Line::raw(""));
    lines.push(ansi_to_line(&format!(
        "  {}Path:{} {}{}",
        ansi::BLD, ansi::RST,
        app.repos_add_input,
        "█"
    )));
    lines.push(Line::raw(""));
    lines.push(ansi_to_line(&format!(
        "  {}{}Enter{} = add  {}{}Esc{} = cancel  {}{}Tab{} = autocomplete from cwd",
        ansi::BLD, ansi::LGR, ansi::RST,
        ansi::BLD, ansi::LRE, ansi::RST,
        ansi::BLD, ansi::DIM, ansi::RST
    )));

    let vis = (area.height as usize).max(1);
    while lines.len() < vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

